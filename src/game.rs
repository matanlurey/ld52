use std::num::NonZeroU8;

use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

pub use components::Glyph;
pub use components::Moving as Direction;

use map::Map;
use specs::shred::FetchMut;

use self::level_generator::LevelGenerator;
use self::level_generator::LevelInsert;
use self::level_generator::LevelItem;
use self::logger::LogMessage;
use self::logger::Logs;

mod combat;
mod components;
#[allow(dead_code)]
mod demo;
mod level_generator;
mod logger;
mod map;
mod monster;
mod movement;
mod tree_growth;

/// Our external world state, i.e. how it will be drawn to the screen.
#[derive(Debug)]
pub struct DrawEntity {
    pub x: i32,
    pub y: i32,
    pub glyph: Glyph,
    pub hp: (u8, u8),
}

/// Statistics used to draw the player's UI.
#[derive(Debug)]
pub struct GameStats {
    /// Round number, starting at 1.
    pub round: NonZeroU8,

    /// Current and maximum health.
    pub health: (u8, u8),

    /// Amount of $ for the player.
    pub money: u8,

    /// Farms remaining.
    pub farms: u8,

    /// Houses remaining.
    pub houses: u8,

    /// State of the game.
    pub state: GameState,
}

/// Possible states that the game can be in.
#[derive(Debug)]
pub enum GameState {
    /// The player has been defeated.
    GameOver,

    /// The player is actively playing the game.
    DefendingTheRealm,

    /// The player can now build structures (walls and farms, basically).
    WaitingForBuild,
}

/// Possible states that the game can be in and executing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    PreRun,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    BuildingTurn,
}

/// A logical representation of the game world and its state.
pub struct WorldState {
    ecs: World,
    player_entity: Entity,
}

/// Why a player cannot move in a given direction.
#[derive(Debug)]
pub enum MovementDenied {
    /// The player is not allowed to move at this time.
    NotPlayerTurn,

    /// The player has been defeated and/or all houses have been destroyed.
    GameOver,

    /// Either would be out of bounds or moving through impassable terrain.
    Impassable,

    /// The player is trying to move into a friendly unit or building.
    ///
    /// The next time this same directional input is given, the player will attack the unit/terrain.
    Friendly,
}

impl WorldState {
    pub fn new() -> Self {
        let mut ecs = World::new();

        // Register all of our components.
        ecs.register::<components::Position>();
        ecs.register::<components::Renderable>();
        ecs.register::<components::Player>();
        ecs.register::<components::Monster>();
        ecs.register::<components::Moving>();
        ecs.register::<components::Health>();
        ecs.register::<components::Attacking>();
        ecs.register::<components::Defeated>();

        // Start the demo.
        let mut rng = RandomNumberGenerator::new();
        let mut level_generator = LevelGenerator::new(12, 12);
        let level_items = level_generator.generate(&mut rng, 2, 0.15);
        let player_entity = LevelGenerator::insert(&mut ecs, level_items);

        // Insert the map and initial running state.
        ecs.insert(Map::new(12, 12));
        ecs.insert(RunState::PreRun);
        ecs.insert(Logs::new());
        ecs.insert(rng);
        ecs.insert(level_generator);

        let mut it = Self {
            ecs,
            player_entity: player_entity.expect("A player entity must be present"),
        };

        // Spawn the monsters.
        it.spawn_monsters();

        it
    }

    fn rng(&self) -> FetchMut<RandomNumberGenerator> {
        self.ecs.fetch_mut::<RandomNumberGenerator>()
    }

    pub fn tick(&mut self) {
        let mut run;
        {
            // Get the current running state.
            let run_state = self.ecs.fetch::<RunState>();
            run = *run_state;
        }

        // State machine.
        match run {
            RunState::PreRun => {
                // Run the pre-run state.
                self.run_systems();
                run = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                // Wait for input.
                run = RunState::AwaitingInput;
            }
            RunState::PlayerTurn => {
                // Run the player turn.
                self.run_systems();
                run = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                // Run the monster turn.
                self.run_systems();

                // If monsters have been eliminated, switch to building turn.
                let monsters = self
                    .ecs
                    .read_storage::<components::Monster>()
                    .join()
                    .count();
                if monsters == 0 {
                    self.switch_to_building_turn();

                    // Move to turn building phase.
                    run = RunState::BuildingTurn;
                } else {
                    run = RunState::AwaitingInput;
                }
            }
            RunState::BuildingTurn => {
                // Run the building turn.
                self.run_systems();
                run = RunState::BuildingTurn;
            }
        }

        // Store next state.
        {
            let mut run_state = self.ecs.fetch_mut::<RunState>();
            *run_state = run;
        }
    }

    fn switch_to_building_turn(&mut self) {
        // Increment the round.
        {
            let mut map = self.ecs.fetch_mut::<Map>();
            map.next_round();

            // Grow trees.
            tree_growth::TreeGrowthSystem.run_now(&self.ecs);

            // Reset all health.
            let mut health = self.ecs.write_storage::<components::Health>();
            for h in (&mut health).join() {
                h.reset();
            }

            // Give 1 $ for each surviving farm glyph.
            map.money += map.farms;

            // Move to turn building phase.
            let mut run_state = self.ecs.fetch_mut::<RunState>();
            *run_state = RunState::BuildingTurn;
        }

        // Spawn a new house.
        self.spawn_house();
    }

    pub fn player_move(&mut self, direction: Direction) -> Result<(), MovementDenied> {
        // If we're not awaiting input, don't do anything.
        let mut run_state = self.ecs.fetch_mut::<RunState>();
        if *run_state != RunState::AwaitingInput {
            return Err(MovementDenied::NotPlayerTurn);
        }

        // If the move is out of bounds, don't do anything.
        let mut map = self.ecs.fetch_mut::<Map>();
        let position = self.ecs.read_storage::<components::Position>();
        let position = position.get(self.player_entity);
        if position.is_none() {
            return Err(MovementDenied::GameOver);
        }
        let position = position.unwrap();
        let position = position.after(&direction);
        if !map.in_bounds(position.x, position.y) {
            return Err(MovementDenied::Impassable);
        }

        // If the game is over, don't do anything.
        if map.houses == 0 {
            return Err(MovementDenied::GameOver);
        }

        // If the player is trying to move into a non-monster, ignore the first time.
        let entity = map.get_entity(position.x, position.y);
        if let Some(entity) = entity {
            let monster = self.ecs.read_storage::<components::Monster>();
            if monster.get(entity).is_none() && !map.allow_move_into_friendly(direction.clone()) {
                return Err(MovementDenied::Friendly);
            }
        }

        // Get the player entity and update the movement component.
        let mut moving = self.ecs.write_storage::<components::Moving>();
        let _ = moving.insert(self.player_entity, direction);

        // Change state to player turn.
        *run_state = RunState::PlayerTurn;
        Ok(())
    }

    /// Build a structure at the given position.
    ///
    /// Note that only walls and farms can be built.
    ///
    /// Returns true if the build was successful, false otherwise.
    #[allow(dead_code)]
    pub fn player_build(&mut self, position: (i32, i32), what: Glyph) -> bool {
        // If we're not in the building phase, don't do anything.
        let run_state = { *self.ecs.fetch_mut::<RunState>() };
        if run_state != RunState::BuildingTurn {
            return false;
        }

        // Get the map.
        {
            let mut map = self.ecs.fetch_mut::<Map>();

            // Check if the player has enough money.
            let cost = match what {
                Glyph::Wall => 1,
                Glyph::Farm => 2,
                _ => return false,
            };
            if map.money < cost {
                return false;
            }

            // Check if the position is valid.
            let (x, y) = position;
            if map.get_entity(x, y).is_some() {
                return false;
            }

            // Subtract the cost.
            map.money -= cost;
        }

        // Build the structure.
        let (x, y) = position;
        let entity = self.ecs.create_entity();
        match what {
            Glyph::Wall => {
                demo::configure_wall(entity, x, y).build();
            }
            Glyph::Farm => {
                demo::configure_farm(entity, x, y).build();
            }
            _ => return false,
        }

        true
    }

    /// Spawns a new house by finding an open position at least 2 tiles away from other houses.
    fn spawn_house(&mut self) {
        let map = self.ecs.fetch::<Map>();

        // Create a grid (vector) that the level generator can use.
        #[rustfmt::skip]
        let mut grid: Vec<Vec<Option<LevelItem>>> = vec![
            vec![None; map.width()]; map.height()
        ];

        // Add placeholders (e.g. trees) to the grid for every entity.
        let positions = self.ecs.read_storage::<components::Position>();
        for position in (&positions).join() {
            let x = position.x as usize;
            let y = position.y as usize;
            grid[y][x] = Some(LevelItem::Tree);
        }

        // Iterate over all the houses and add them to the grid.
        let renderables = self.ecs.read_storage::<components::Renderable>();
        for (house, position) in (&renderables, &positions).join() {
            if house.glyph() != Glyph::House {
                continue;
            }
            let x = position.x as usize;
            let y = position.y as usize;
            grid[y][x] = Some(LevelItem::House);
        }

        // Re-use the level generator to find a new house position.
        let mut generator = self.ecs.fetch_mut::<LevelGenerator>();
        let position = generator.find_somewhat_adjacent_position(
            &mut self.rng(),
            2,
            5,
            &LevelItem::House,
            &grid,
        );

        // This is hacky but so is this entire function.
        drop(map);
        drop(positions);
        drop(renderables);
        drop(generator);

        // Convert position into a u8, u8.
        let position = (position.0 as u8, position.1 as u8);

        // Spawn the house.
        LevelGenerator::insert(
            &mut self.ecs,
            vec![LevelInsert {
                item: LevelItem::House,
                position,
            }],
        );
    }

    /// Indicates the player is ready, spawning goblins.
    #[allow(dead_code)]
    pub fn player_ready(&mut self) {
        self.spawn_monsters();

        // Change state to start the game again.
        let mut run_state = self.ecs.fetch_mut::<RunState>();
        *run_state = RunState::PreRun;
    }

    /// For the given round number, spawn R+3 goblins at the edge of the map.
    fn spawn_monsters(&mut self) {
        // Get the round number to determine how many goblins to spawn.
        let goblins_to_spawn = { self.ecs.fetch::<Map>().round().get() + 3 } as usize;

        let positions: Vec<(i32, i32)> = {
            // Get the map and level generator.
            let map = self.ecs.fetch::<Map>();
            let mut generator = self.ecs.fetch_mut::<LevelGenerator>();

            // Start at the edge of the map, and move inwards if we can't find a position.
            let mut from_edge = 0;

            // Get the positions.
            let mut positions = Vec::new();
            while positions.len() < goblins_to_spawn {
                // Make a list of all positions from_edge tiles away from the edge.
                let mut positions_to_try = Vec::new();

                // For example in the following grid:
                // x x x x
                // x     x
                // x     x
                // x x x x
                //
                // Try all the "x" spots.
                // iF that doesnt' work, we'll try the inner-x box and so on:
                //
                //   x x
                //   x x

                // Top row.
                for x in from_edge..(map.width() - from_edge) {
                    positions_to_try.push((x, from_edge));
                }

                // Bottom row.
                for x in from_edge..(map.width() - from_edge) {
                    positions_to_try.push((x, map.height() - from_edge - 1));
                }

                // Left column.
                for y in from_edge..(map.height() - from_edge) {
                    positions_to_try.push((from_edge, y));
                }

                // Right column.
                for y in from_edge..(map.height() - from_edge) {
                    positions_to_try.push((map.width() - from_edge - 1, y));
                }

                // Shuffle the list.
                generator.shuffle(&mut self.rng(), &mut positions_to_try);

                // Try to find a position that is not occupied.
                for (x, y) in positions_to_try {
                    if map.get_entity(x as i32, y as i32).is_none() {
                        positions.push((x as i32, y as i32));
                    }
                }

                from_edge += 1;
            }

            generator.shuffle(&mut self.rng(), &mut positions);
            positions
        };

        // Spawn the goblins.
        // After level 2, (e.g. starting at 3) L - 2 goblins are actually orcs.
        let mut orcs = {
            let map = self.ecs.fetch::<Map>();
            let round = map.round().get();
            if round >= 3 {
                round - 2
            } else {
                0
            }
        };
        for (x, y) in positions.into_iter().take(goblins_to_spawn) {
            if orcs > 0 {
                orcs -= 1;
                demo::configure_orc(self.ecs.create_entity(), x, y).build();
            } else {
                demo::configure_goblin(self.ecs.create_entity(), x, y).build();
            }
        }
    }

    fn run_systems(&mut self) {
        // Index the map.
        map::MapIndexingSystem.run_now(&self.ecs);

        // Let the monsters do their thing.
        monster::MonsterAISystem.run_now(&self.ecs);

        // Convert movement into combat if necessary.
        combat::ConvertMovementToMeleeAttackSystem.run_now(&self.ecs);

        // Apply movement.
        movement::MovementSystem.run_now(&self.ecs);

        // Apply combat.
        combat::ApplyAttackSystem.run_now(&self.ecs);

        // Defeat entities.
        combat::DefeatSystem.run_now(&self.ecs);

        // Remove defeated entities.
        combat::RemoveDefeatedSystem.run_now(&self.ecs);

        // Maintain the ECS (i.e. built-in systems).
        self.ecs.maintain();
    }

    /// Convert the world state into a representation that can be drawn to the screen.
    pub fn to_render(&self) -> Vec<DrawEntity> {
        let mut drawables = Vec::new();

        // Get all of the entities that have a position and renderable component.
        let positions = self.ecs.read_storage::<components::Position>();
        let renderables = self.ecs.read_storage::<components::Renderable>();
        let health = self.ecs.read_storage::<components::Health>();

        // Iterate over all of the entities that have a position and renderable component.
        for (pos, render, hp) in (&positions, &renderables, &health).join() {
            drawables.push(DrawEntity {
                x: pos.x,
                y: pos.y,
                glyph: render.glyph(),
                hp: (hp.amount(), hp.maximum()),
            });
        }

        drawables
    }

    /// Returns the current game logs, clearing them in the process.
    pub fn get_logs(&mut self) -> Vec<LogMessage> {
        // Get the logs struct.
        let mut logs = self.ecs.fetch_mut::<Logs>();

        // Get the messages.
        logs.flush()
    }

    /// Get the current game statistics.
    pub fn get_stats(&self) -> GameStats {
        // Get player's HP.
        let health = {
            self.ecs
                .read_storage::<components::Health>()
                .get(self.player_entity)
                .map(|h| (h.amount(), h.maximum()))
                .unwrap_or((0, 0))
        };

        // Get the money and round number.
        let (money, round, farms, houses) = {
            let map = self.ecs.fetch::<Map>();

            (map.money, map.round(), map.farms, map.houses)
        };

        let state = {
            // If the player is dead or there are no houses, the game is over.
            if health.0 == 0 || houses == 0 {
                GameState::GameOver
            } else {
                // Otherwise, get the current running state.
                let run_state = self.ecs.fetch::<RunState>();
                match *run_state {
                    RunState::BuildingTurn => GameState::WaitingForBuild,
                    _ => GameState::DefendingTheRealm,
                }
            }
        };

        GameStats {
            round,
            health,
            money,
            farms,
            houses,
            state,
        }
    }
}
