//! Movement system.
//!
//! This system is responsible for moving entities around the world, including melee attacks.

use specs::prelude::*;

use super::components::{Moving, Position};
use super::map::Map;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Moving>,
    );

    fn run(&mut self, data: Self::SystemData) {
        // Unpack the system data.
        let (map, mut positions, mut moving) = data;

        // Iterate over all entities that have a position and are moving.
        for (position, direction) in (&mut positions, &mut moving).join() {
            let prospective = position.after(direction);

            // If there would be an overlap with another entity, do not move.
            if map.get_entity(prospective.x, prospective.y).is_some() {
                continue;
            }

            // If the entity is within the bounds of the map, update its position.
            if map.in_bounds(prospective.x, prospective.y) {
                *position = prospective;
            }
        }

        // Remove the moving component from all entities.
        moving.clear();
    }
}
