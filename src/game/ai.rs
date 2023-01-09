use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use super::components::{Monster, Moving, Player, Position, Town, AI};

pub struct AISystem;

impl<'a> System<'a> for AISystem {
    type SystemData = (
        Entities<'a>,
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
        let (entities, monsters, towns, players, ai, positions, mut moving, mut rng) = data;

        // Find all town entities on the map.
        let town_positions: Vec<Position> = {
            let mut town_positions: Vec<Position> = Vec::new();
            for (position, _) in (&positions, &towns).join() {
                town_positions.push(position.clone());
            }
            town_positions
        };

        // Find the player entity on the map.
        let player_position = {
            let mut player_position: Option<Position> = None;
            for (position, _) in (&positions, &players).join() {
                player_position = Some(position.clone());
            }
            player_position.expect("A player must exist for AI to function?")
        };

        // Iterate through AI.
        for (entity, ai, position, moving) in (&entities, &ai, &positions, &mut moving).join() {
            // If this a monster, and the player is adjacent, attack.
            if monsters.get(entity).is_some() && player_position.distance(position) == 1.0 {
                *moving = best_direction(position, &player_position);
                continue;
            }

            // Move based on the AIs type.
            *moving = match ai {
                AI::Wander => {
                    // Pick a random direction.
                    rng.random_slice_entry(&[Moving::Up, Moving::Down, Moving::Left, Moving::Right])
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
                    best_direction(position, &player_position)
                }
            }
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
