use std::num::NonZeroU8;

use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

pub use components::Glyph;
pub use components::Moving as Direction;

use map::Map;

mod combat;
mod components;
mod demo;
mod map;
mod monster;
mod movement;

/// Our external world state, i.e. how it will be drawn to the screen.
#[derive(Debug)]
pub struct DrawEntity {
    pub x: i32,
    pub y: i32,
    pub glyph: Glyph,
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
    rng: RandomNumberGenerator,
    player_entity: Entity,
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
        let player_entity = demo::spawn_demo(&mut ecs);

        // Insert goblins at the following positions

        ecs.create_entity()
            .with(components::Position::new(11, 2))
            .with(components::Renderable::new(Glyph::Goblin))
            .with(components::Monster)
            .with(components::Health::new(1))
            .build();

        // Insert the map and initial running state.
        ecs.insert(Map::new(12, 12));
        ecs.insert(RunState::PreRun);

        let rng = RandomNumberGenerator::new();

        Self {
            ecs,
            player_entity,
            rng,
        }
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

            // Reset the player's health.
            let mut health = self.ecs.write_storage::<components::Health>();
            health.get_mut(self.player_entity).unwrap().reset();

            // Give 1 $ for each surviving farm glyph.
            map.money += map.farms;

            // Move to turn building phase.
            let mut run_state = self.ecs.fetch_mut::<RunState>();
            *run_state = RunState::BuildingTurn;
        }

        // Spawn a new house.
        self.spawn_house();
    }

    pub fn player_move(&mut self, direction: Direction) {
        // If we're not awaiting input, don't do anything.
        let mut run_state = self.ecs.fetch_mut::<RunState>();
        if *run_state != RunState::AwaitingInput {
            return;
        }

        // If the move is out of bounds, don't do anything.
        // TODO: Implement.

        // Get the player entity and update the movement component.
        let mut moving = self.ecs.write_storage::<components::Moving>();
        let _ = moving.insert(self.player_entity, direction);

        // Change state to player turn.
        *run_state = RunState::PlayerTurn;
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
        let (found, position) = {
            // Get the map.
            let map = self.ecs.fetch::<Map>();

            // Find an open position at least 2 tiles away from other houses.
            let mut position: (i32, i32) = (0, 0);
            let mut found = false;

            for _ in 0..100 {
                position = (
                    self.rng.range(0, map.width() as i32),
                    self.rng.range(0, map.height() as i32),
                );
                if map.get_entity(position.0, position.1).is_none() {
                    let mut valid = true;
                    for x in position.0 - 2..position.0 + 2 {
                        for y in position.1 - 2..position.1 + 2 {
                            if map.get_entity(x, y).is_some() {
                                valid = false;
                                break;
                            }
                        }
                    }
                    if valid {
                        found = true;
                        break;
                    }
                }
            }

            (found, position)
        };

        // If we found a valid position, spawn a house.
        if found {
            let (x, y) = position;
            demo::configure_house(self.ecs.create_entity(), x, y).build();
        }
    }

    /// Indicates the player is ready, spawning goblins.
    #[allow(dead_code)]
    pub fn player_ready(&mut self) {
        self.spawn_goblins();

        // Change state to start the game again.
        let mut run_state = self.ecs.fetch_mut::<RunState>();
        *run_state = RunState::PreRun;
    }

    /// For the given round number, spawn R+1 goblins at the edge of the map.
    fn spawn_goblins(&mut self) {
        let positions: Vec<(i32, i32)> = {
            // Get the map.
            let map = self.ecs.fetch::<Map>();

            // Get the round number.
            let round = map.round().get();

            // Get the positions.
            let mut positions = Vec::new();
            for _ in 0..round + 1 {
                let mut position: (i32, i32) = (0, 0);
                let mut found = false;
                for _ in 0..100 {
                    position = (
                        self.rng.range(0, map.width() as i32),
                        self.rng.range(0, map.height() as i32),
                    );
                    if map.get_entity(position.0, position.1).is_none() {
                        found = true;
                        break;
                    }
                }
                if found {
                    positions.push(position);
                }
            }

            positions
        };

        // Spawn the goblins.
        for (x, y) in positions {
            demo::configure_goblin(self.ecs.create_entity(), x, y).build();
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

        // Iterate over all of the entities that have a position and renderable component.
        for (pos, render) in (&positions, &renderables).join() {
            drawables.push(DrawEntity {
                x: pos.x,
                y: pos.y,
                glyph: render.glyph(),
            });
        }

        drawables
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
