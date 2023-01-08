use std::num::NonZeroU8;

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

        Self { ecs, player_entity }
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
                let monsters = self.ecs.read_storage::<components::Monster>();
                if monsters.join().count() == 0 {
                    // Increment the round.
                    let mut map = self.ecs.fetch_mut::<Map>();
                    map.next_round();

                    // Reset the player's health.
                    let mut health = self.ecs.write_storage::<components::Health>();
                    health.get_mut(self.player_entity).unwrap().reset();

                    // Give 1 $ for each surviving farm glyph.
                    map.money += map.farms;

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

    pub fn player_move(&mut self, direction: Direction) {
        // If we're not awaiting input, don't do anything.
        let mut run_state = self.ecs.fetch_mut::<RunState>();
        if *run_state != RunState::AwaitingInput {
            return;
        }

        // Get the player entity and update the movement component.
        let mut moving = self.ecs.write_storage::<components::Moving>();
        let _ = moving.insert(self.player_entity, direction);

        // Change state to player turn.
        *run_state = RunState::PlayerTurn;
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
            let pos = pos.to_point();
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
            // If the player is dead, the game is over.
            if health.0 == 0 {
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
