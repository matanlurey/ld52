use specs::prelude::*;

use super::{
    components::{Monster, Moving, Player, Position},
    map::Map,
    RunState,
};

/// Determines how monsters will react, such as moving around the map.
pub struct MonsterAISystem;

impl<'a> System<'a> for MonsterAISystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Moving>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, state, players, positions, monsters, mut direction) = data;

        // If this is not the monster's turn, do nothing.
        if *state != RunState::MonsterTurn {
            return;
        }

        let mut nearest_player_position: &Position = (&players, &positions)
            .join()
            .next()
            .expect("No player exists!")
            .1;

        // Iterate over all monsters.
        for (_, monster_position) in (&monsters, &positions).join() {
            let smallest_distance: f64 = -1.0;

            // Find nearest player
            for (_player, player_position) in (&players, &positions).join() {
                // let path = a_star_search(monster_position, player_position, map.into().as_ref());

                let smallest_distance_candidate = monster_position.distance(player_position);

                if (smallest_distance < 0.0) || (smallest_distance_candidate < smallest_distance) {
                    nearest_player_position = player_position;
                }
            }

            direction
                .insert(
                    map.get_entity(monster_position.to_point().x, monster_position.to_point().y)
                        .unwrap(),
                    goblin_direction(monster_position, nearest_player_position),
                )
                .expect("Unable to move");
        }
    }
}

/// Goblin AI
/// Basically move in a stupid way towards the physically closest player
fn goblin_direction(goblin_position: &Position, player_position: &Position) -> Moving {
    let player_position_relative_monster_position = player_position.relative(goblin_position);

    if player_position_relative_monster_position.x.abs()
        > player_position_relative_monster_position.y.abs()
    {
        if player_position_relative_monster_position.x > 0 {
            Moving::Right
        } else {
            Moving::Left
        }
    } else if player_position_relative_monster_position.y > 0 {
        Moving::Down
    } else {
        Moving::Up
    }
}
