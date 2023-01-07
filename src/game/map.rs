//! Game map.

#[derive(Default)]
pub struct Map;

impl Map {
    /// Check if a coordinate is within the bounds of the map.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        (0..10).contains(&x) && (0..10).contains(&y)
    }
}
