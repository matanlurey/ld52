use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use super::{
    components::{Monster, Moving, Player, Position, Town, AI},
    RunState,
};

pub struct AISystem;

impl<'a> System<'a> for AISystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Town>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, AI>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Moving>,
        WriteExpect<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        // Unpack the system data.
        let (entities, state, monsters, towns, players, ai, positions, mut moving, mut rng) = data;

        // If this is not the monster's turn, do nothing.
        if *state != RunState::MonsterTurn {
            return;
        }

        let player = (&players, &positions).join().next();

        // If there are no players, do nothing.
        if player.is_none() {
            return;
        }

        // Find all town entities on the map.
        let town_positions: Vec<Position> = {
            let mut town_positions: Vec<Position> = Vec::new();
            for (position, _) in (&positions, &towns).join() {
                town_positions.push(position.clone());
            }
            town_positions
        };

        // Find the player entity on the map.
        let player_position = player.unwrap().1;

        // Iterate through AI.
        for (entity, ai, position) in (&entities, &ai, &positions).join() {
            println!("AI: {:?}", ai);

            // If this a monster, and the player is adjacent, attack.
            if monsters.get(entity).is_some() && player_position.distance(position) == 1.0 {
                moving
                    .insert(entity, best_direction(position, player_position))
                    .unwrap();
                continue;
            }

            // Move based on the AIs type.
            moving
                .insert(
                    entity,
                    match ai {
                        AI::Wander => {
                            // Pick a random direction.
                            rng.random_slice_entry(&[
                                Moving::Up,
                                Moving::Down,
                                Moving::Left,
                                Moving::Right,
                            ])
                            .unwrap()
                            .clone()
                        }
                        AI::PrioritizeTown => {
                            // Find the closest town and move towards it.
                            let mut closest_distance = i32::MAX.into();
                            let mut closest_position = Position::new(0, 0);

                            for town_position in town_positions.iter() {
                                let distance = position.distance(town_position);
                                if distance < closest_distance {
                                    closest_distance = distance;
                                    closest_position = town_position.clone();
                                }
                            }

                            best_direction(position, &closest_position)
                        }
                        AI::PrioritizePlayer => {
                            // Move towards the player.
                            best_direction(position, player_position)
                        }
                    },
                )
                .unwrap();
        }
    }
}

/// Returns a direction to move towards a target.
fn best_direction(from: &Position, to: &Position) -> Moving {
    let x_diff = to.x - from.x;
    let y_diff = to.y - from.y;

    if x_diff.abs() > y_diff.abs() {
        if x_diff > 0 {
            Moving::Right
        } else {
            Moving::Left
        }
    } else if y_diff > 0 {
        Moving::Down
    } else {
        Moving::Up
    }
}
