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

pub struct LevelGenerator<'a> {
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> LevelGenerator<'a> {
    pub fn new(rng: &'a mut RandomNumberGenerator) -> Self {
        Self { rng }
    }

    /// Generates a new level with the given width, height, houses, and tree density (0.0 to 1.0).
    ///
    /// # Panics
    ///
    /// - If width or height is 0.
    /// - If houses is 0.
    /// - If density is not between 0.0 and 1.0.
    pub fn generate(
        &mut self,
        width: usize,
        height: usize,
        houses: u8,
        density: f32,
    ) -> Vec<LevelInsert> {
        assert!(width > 0);
        assert!(height > 0);
        assert!(houses > 0);
        assert!((0.0..=1.0).contains(&density));

        // First, create a grid of empty (None) tiles we'll use as a baseline.
        let mut grid = vec![vec![None; width]; height];

        // First, add houses. Houses should be within 3 tiles of another house, but not within 1.
        {
            let mut houses_added = 0;

            // Add the first house at a random position in the center-ish of the map.
            // Meaning, given a 10x10 map, the first house will be placed between (3,3) and (6,6).
            {
                let x_third = width / 3;
                let y_third = height / 3;

                let x = self.rng.range(x_third, x_third * 2);
                let y = self.rng.range(y_third, y_third * 2);

                grid[y][x] = Some(LevelItem::House);
                houses_added += 1;
            }

            // Add the remaining houses.
            while houses_added < houses {
                // Find a random position within 3 tiles of an existing house.
                let (x, y) = self.find_somewhat_adjacent_position(3, 1, &LevelItem::House, &grid);

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
                let (x, y) = self.find_somewhat_adjacent_position(1, 0, &LevelItem::House, &grid);

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
                let item_to_protect = if self.rng.range(0, 2) == 0 {
                    LevelItem::House
                } else {
                    LevelItem::Farm
                };
                let (x, y) = self.find_adjacent_outwards_facing_position(&item_to_protect, &grid);

                // Place the wall.
                grid[y][x] = Some(LevelItem::Wall);
                walls_added += 1;
            }
        }

        // Next, check density so far and add trees until we hit the expected density.
        {
            // Amount of occupied tiles so far.
            let total_tiles = (width * height) as f32;
            let total_houses_farms_and_walls = houses as f32 * 4.0;

            let mut total_trees = 0.0;
            let mut total_density = total_houses_farms_and_walls / total_tiles;

            while total_density < density {
                // Find a random position within 1 tile of an existing house or farm.
                let (x, y) = self.find_any_open_position(&grid);

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
            let (x, y) = self.find_somewhat_adjacent_position(1, 3, &LevelItem::House, &grid);

            // Place the player.
            grid[y][x] = Some(LevelItem::Player {
                health: NonZeroU8::new(5).unwrap(),
            });
        }

        // Goblins will be added by the spawn system.

        // Finally, convert the grid into a vector of inserts.
        self.convert_to_level_inserts(grid)
    }

    fn find_adjacent_outwards_facing_position(
        &mut self,
        _of: &LevelItem,
        grid: &[Vec<Option<LevelItem>>],
    ) -> (usize, usize) {
        // TODO: Implement.

        // Fallback
        self.find_any_open_position(grid)
    }

    /// Finds a random position within the grid that is somewhat adjacent to the given position.
    fn find_somewhat_adjacent_position(
        &mut self,
        _outside: usize,
        _within: usize,
        _of: &LevelItem,
        grid: &[Vec<Option<LevelItem>>],
    ) -> (usize, usize) {
        // TODO: Implement.

        // Fallback
        self.find_any_open_position(grid)
    }

    /// Finds a random position within the grid that is empty.
    fn find_any_open_position(&mut self, grid: &[Vec<Option<LevelItem>>]) -> (usize, usize) {
        let mut positions = Vec::new();

        for (y, row) in grid.iter().enumerate() {
            for (x, item) in row.iter().enumerate() {
                if item.is_none() {
                    positions.push((x, y));
                }
            }
        }

        positions[self.rng.range(0, positions.len())]
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
    pub fn insert(world: &mut World, level: Vec<LevelInsert>) -> Entity {
        dbg!(&level);
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
                        .build();
                }
                LevelItem::House => {
                    entity
                        .with(Renderable::new(Glyph::House))
                        .with(Health::new(1))
                        .build();
                }
                LevelItem::Wall => {
                    entity
                        .with(Renderable::new(Glyph::Wall))
                        .with(Health::new(3))
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
        player.expect("A player is required to insert a level")
    }
}
