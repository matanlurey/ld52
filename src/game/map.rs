//! Game map.

use specs::prelude::*;

use super::components::Position;

pub struct Map {
    /// A 2D vector of entities with positions on the map.
    entities: Vec<Option<Entity>>,

    /// The width of the map.
    width: usize,
}

impl Map {
    /// Create a new empty map of the given dimensions.
    ///
    /// # Panics
    ///
    /// If the width or height is 0.
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0);

        Self {
            entities: vec![None; width * height],
            width,
        }
    }

    /// Clear the map.
    pub fn clear(&mut self) {
        for entity in self.entities.iter_mut() {
            *entity = None;
        }
    }

    /// Check if a coordinate is within the bounds of the map.
    #[must_use]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width() as i32 && y >= 0 && y < self.height() as i32
    }

    /// Set the entity at the given coordinate.
    pub fn set_entity(&mut self, x: i32, y: i32, entity: Entity) {
        // If out of bounds, panic.
        assert!(self.in_bounds(x, y));
        self.entities[(y as usize * self.width) + x as usize] = Some(entity);
    }

    /// Get the entity at the given coordinate.
    #[must_use]
    pub fn get_entity(&self, x: i32, y: i32) -> Option<Entity> {
        // If out of bounds, return None.
        if !self.in_bounds(x, y) {
            return None;
        }
        self.entities[(y as usize * self.width) + x as usize]
    }

    /// Return the width of the map.
    fn width(&self) -> usize {
        self.width
    }

    /// Return the height of the map.
    fn height(&self) -> usize {
        self.entities.len() / self.width
    }
}

/// A system that indexes entities with positions on the map.
pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, positions) = data;

        // Clear the map.
        map.clear();

        // Iterate over all entities with positions and index them on the map.
        for (entity, position) in (&entities, &positions).join() {
            let position = position.to_point();
            map.set_entity(position.x, position.y, entity);
        }
    }
}
