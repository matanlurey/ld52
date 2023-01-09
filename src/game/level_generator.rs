use std::num::NonZeroU8;

use bracket_lib::random::RandomNumberGenerator;
use specs::{Builder, Entity, World, WorldExt};

use super::{components::*, Glyph};

#[derive(Debug)]
pub struct LevelInsert {
    pub position: (u8, u8),
    pub item: LevelItem,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelItem {
    Player { health: NonZeroU8 },
    Farm,
    House,
    Tree,
    Wall,
}

pub struct LevelGenerator {
    width: usize,
    height: usize,
}

impl LevelGenerator {
    /// Creates a new level generator.
    ///
    /// # Panics
    ///
    /// - If width or height is 0.
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        Self { width, height }
    }

    /// Generates a new level with the given width, height, houses, and tree density (0.0 to 1.0).
    ///
    /// # Panics
    ///
    /// - If houses is 0.
    /// - If density is not between 0.0 and 1.0.
    pub fn generate(
        &mut self,
        rng: &mut RandomNumberGenerator,
        houses: u8,
        density: f32,
    ) -> Vec<LevelInsert> {
        assert!(houses > 0);
        assert!((0.0..=1.0).contains(&density));

        // First, create a grid of empty (None) tiles we'll use as a baseline.
        let mut grid = vec![vec![None; self.width]; self.height];

        // First, add houses. Houses should be within 3 tiles of another house, but not within 1.
        {
            let mut houses_added = 0;

            // Add the first house at a random position in the center-ish of the map.
            // Meaning, given a 10x10 map, the first house will be placed between (3,3) and (6,6).
            {
                let x_third = self.width / 3;
                let y_third = self.height / 3;

                let x = rng.range(x_third, x_third * 2);
                let y = rng.range(y_third, y_third * 2);

                grid[y][x] = Some(LevelItem::House);
                houses_added += 1;
            }

            // Add the remaining houses.
            while houses_added < houses {
                // Find a random position within 3 tiles of an existing house.
                let (x, y) =
                    self.find_somewhat_adjacent_position(rng, 2, 4, &LevelItem::House, &grid);

                // Place the house.
                grid[y][x] = Some(LevelItem::House);
                houses_added += 1;
            }
        }

        // Add a farm as close as possible to each house.
        {
            let mut farms_added = 0;

            while farms_added < houses {
                // Find a random position within 1 tile of an existing house.
                let (x, y) =
                    self.find_somewhat_adjacent_position(rng, 1, 2, &LevelItem::House, &grid);

                // Place the farm.
                grid[y][x] = Some(LevelItem::Farm);
                farms_added += 1;
            }
        }

        // Add 2x walls per house. Each wall has a 50% chance of "defending" a house or farm.
        // Walls always try to face "outwards" towards the edge of the board.
        {
            let mut walls_added = 0;

            while walls_added < houses * 2 {
                // Find a random position adjacent to an existing house or farm.
                let item_to_protect = if rng.range(0, 2) == 0 {
                    LevelItem::House
                } else {
                    LevelItem::Farm
                };
                let (x, y) =
                    self.find_adjacent_outwards_facing_position(rng, &item_to_protect, &grid);

                // Place the wall.
                grid[y][x] = Some(LevelItem::Wall);
                walls_added += 1;
            }
        }

        // Next, check density so far and add trees until we hit the expected density.
        {
            // Amount of occupied tiles so far.
            let total_tiles = (self.width * self.height) as f32;
            let total_houses_farms_and_walls = houses as f32 * 4.0;

            let mut total_trees = 0.0;
            let mut total_density = total_houses_farms_and_walls / total_tiles;

            while total_density < density {
                // Find a random position within 1 tile of an existing house or farm.
                let (x, y) = self.find_any_open_position(rng, &grid);

                // Place the tree.
                grid[y][x] = Some(LevelItem::Tree);

                // Recalculate density.
                total_trees += 1.0;
                total_density = (total_houses_farms_and_walls + total_trees) / total_tiles;
            }
        }

        // Next add the player to the closest point in the center that is open.
        {
            // Find the closest open position to the center.
            let (x, y) = self.find_somewhat_adjacent_position(rng, 1, 3, &LevelItem::House, &grid);

            // Place the player.
            grid[y][x] = Some(LevelItem::Player {
                health: NonZeroU8::new(5).unwrap(),
            });
        }

        // Goblins will be added by the spawn system.

        // Finally, convert the grid into a vector of inserts.
        self.convert_to_level_inserts(grid)
    }

    /// Given a vector of items, shuffles them in place.
    pub fn shuffle<T>(&mut self, rng: &mut RandomNumberGenerator, items: &mut Vec<T>) {
        // This is not a great shuffle impl, but self.rng is not a great RNG impl.
        // In practice we should not need something significantly better.
        let starting_len = items.len();
        for i in 0..items.len() {
            let j = rng.range(0, items.len());
            items.swap(i, j);
        }
        debug_assert!(items.len() == starting_len);
    }

    /// Returns all positions of a given item type in a grid, shuffled.
    fn all_items_of_type_shuffled(
        &mut self,
        rng: &mut RandomNumberGenerator,
        of: &LevelItem,
        grid: &[Vec<Option<LevelItem>>],
    ) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();

        for (y, row) in grid.iter().enumerate() {
            for (x, item) in row.iter().enumerate() {
                if let Some(item) = item {
                    if item == of {
                        positions.push((x, y));
                    }
                }
            }
        }

        self.shuffle(rng, &mut positions);

        positions
    }

    /// Returns the adjacent tiles that are the closest board edges to a given position.
    ///
    /// For example, given (2, 3) will return (1, 3), (3, 3), (2, 2), (2, 4) (but in an order where
    /// the first item is closest to a board edge and the last item is furthest from a board edge).
    fn closest_board_edges(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        struct Spot {
            x: usize,
            y: usize,
            distance_to_nearest_edge: usize,
        }

        let mut spots = Vec::new();

        fn add_spot_if_in_bounds(
            spots: &mut Vec<Spot>,
            x: i32,
            y: i32,
            width: usize,
            height: usize,
        ) {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                spots.push(Spot {
                    x: x as usize,
                    y: y as usize,
                    distance_to_nearest_edge: 0,
                });
            }
        }

        // Left.
        add_spot_if_in_bounds(&mut spots, x as i32 - 1, y as i32, self.width, self.height);
        // Right.
        add_spot_if_in_bounds(&mut spots, x as i32 + 1, y as i32, self.width, self.height);
        // Up.
        add_spot_if_in_bounds(&mut spots, x as i32, y as i32 - 1, self.width, self.height);
        // Down.
        add_spot_if_in_bounds(&mut spots, x as i32, y as i32 + 1, self.width, self.height);

        // Sort by distance to nearest edge.
        spots.sort_by_key(|s| s.distance_to_nearest_edge);

        // Convert to a vector of (x, y) tuples.
        spots.into_iter().map(|s| (s.x, s.y)).collect()
    }

    /// Finds an adjacent outward-facing open position.
    ///
    /// - Finds a random item of the provided item type.
    /// - Finds an adjacent position to that that is facing to the closest edge of the grid.
    /// - If no open position is found, tries the next closest edge.
    /// - If still no open position is found, tries the next item of the provided item type.
    ///
    /// As a fallback, picks a completely random open position.
    fn find_adjacent_outwards_facing_position(
        &mut self,
        rng: &mut RandomNumberGenerator,
        _of: &LevelItem,
        grid: &[Vec<Option<LevelItem>>],
    ) -> (usize, usize) {
        // Make a list of the positions of all items of the provided type.
        let mut positions = self.all_items_of_type_shuffled(rng, &LevelItem::House, grid);

        while !positions.is_empty() {
            let next = positions.pop().unwrap();

            // Try to find an open adjacent position that is facing outward.
            let adjacent_positions = self.closest_board_edges(next.0, next.1);
            for (x, y) in adjacent_positions {
                if grid[y][x].is_none() {
                    return (x, y);
                }
            }
        }

        // Fallback
        self.find_any_open_position(rng, grid)
    }

    /// Finds a random position within the grid that is somewhat adjacent to the given position.
    pub fn find_somewhat_adjacent_position(
        &mut self,
        rng: &mut RandomNumberGenerator,
        outside: usize,
        within: usize,
        of: &LevelItem,
        grid: &[Vec<Option<LevelItem>>],
    ) -> (usize, usize) {
        // Make a list of the positions of all items of the provided type.
        let positions = self.all_items_of_type_shuffled(rng, of, grid);

        // Try spots that are exactly `outside` tiles away (x or y offset) from a position.
        let mut try_distance = outside;
        while try_distance < within {
            // Try each position.
            for (x, y) in &positions {
                // Try each direction.
                for (dx, dy) in &[(1, 0), (-1_isize, 0), (0, 1), (0, -1_isize)] {
                    let x = *x as isize + dx * try_distance as isize;
                    let y = *y as isize + dy * try_distance as isize;

                    // Check if the position is in bounds.
                    if x < 0 || x >= self.width as isize || y <= 0 || y >= self.height as isize {
                        continue;
                    }

                    // Check if the position is open.
                    if grid[y as usize][x as usize].is_none() {
                        return (x as usize, y as usize);
                    }
                }
            }

            try_distance += 1;
        }

        // Fallback
        self.find_any_open_position(rng, grid)
    }

    /// Finds a random position within the grid that is empty.
    fn find_any_open_position(
        &mut self,
        rng: &mut RandomNumberGenerator,
        grid: &[Vec<Option<LevelItem>>],
    ) -> (usize, usize) {
        let mut positions = Vec::new();

        for (y, row) in grid.iter().enumerate() {
            for (x, item) in row.iter().enumerate() {
                if item.is_none() {
                    positions.push((x, y));
                }
            }
        }

        positions[rng.range(0, positions.len())]
    }

    fn convert_to_level_inserts(&self, grid: Vec<Vec<Option<LevelItem>>>) -> Vec<LevelInsert> {
        let mut level = Vec::new();

        for (y, row) in grid.into_iter().enumerate() {
            for (x, item) in row.into_iter().enumerate() {
                if let Some(item) = item {
                    level.push(LevelInsert {
                        position: (x as u8, y as u8),
                        item,
                    });
                }
            }
        }

        level
    }

    /// Inserts a level into the world, consuming it.
    ///
    /// An assumption is made that the world is empty.
    ///
    /// Returns the player entity.
    pub fn insert(world: &mut World, level: Vec<LevelInsert>) -> Option<Entity> {
        let mut player = None;
        for insert in level {
            let position: Position = {
                let (x, y) = insert.position;
                Position::new(x as i32, y as i32)
            };

            let entity = world.create_entity().with(position);

            match insert.item {
                LevelItem::Player { health } => {
                    player = Some(
                        entity
                            .with(Renderable::new(Glyph::Player))
                            .with(Health::new(health.get()))
                            .with(Player)
                            .build(),
                    );
                }
                LevelItem::Farm => {
                    entity
                        .with(Renderable::new(Glyph::Farm))
                        .with(Health::new(1))
                        .with(Town)
                        .build();
                }
                LevelItem::House => {
                    entity
                        .with(Renderable::new(Glyph::House))
                        .with(Health::new(2))
                        .with(Town)
                        .build();
                }
                LevelItem::Wall => {
                    entity
                        .with(Renderable::new(Glyph::Wall))
                        .with(Health::new(3))
                        .with(Town)
                        .build();
                }
                LevelItem::Tree => {
                    entity
                        .with(Renderable::new(Glyph::Tree))
                        .with(Health::new(1))
                        .build();
                }
            }
        }
        player
    }
}
