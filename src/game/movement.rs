//! Movement system.
//!
//! This system is responsible for moving entities around the world, including melee attacks.

use std::marker::PhantomData;

use specs::prelude::*;

use super::components::{Moving, Player, Position};
use super::map::Map;

pub struct MovementSystem<T: Component> {
    of_type: PhantomData<T>,
}

impl MovementSystem<Player> {
    pub fn new() -> Self {
        Self {
            of_type: PhantomData,
        }
    }
}

impl<'a, T: Component> System<'a> for MovementSystem<T> {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, T>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Moving>,
    );

    fn run(&mut self, data: Self::SystemData) {
        // Unpack the system data.
        let (map, _, mut positions, mut moving) = data;

        // Iterate over all entities that have a position and are moving.
        for (position, direction) in (&mut positions, &mut moving).join() {
            let mut prospective = position.to_point();

            // Move the entity in the direction it is moving.
            match direction {
                Moving::Up => prospective.y -= 1,
                Moving::Down => prospective.y += 1,
                Moving::Left => prospective.x -= 1,
                Moving::Right => prospective.x += 1,
            }

            // If the entity is within the bounds of the map, update its position.
            if map.in_bounds(prospective.x, prospective.y) {
                position.update(&prospective);
            }
        }

        // Remove the moving component from all entities.
        moving.clear();
    }
}
