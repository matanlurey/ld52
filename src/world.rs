/// This is the (right now, 10x10) logical world.
///
/// It has all of the terrain, entities, and physical movement.
pub struct World {
    cells: Vec<Cell>,
    width: usize,
}

impl World {
    /// Create a new demo world with a 10x10 grid.
    pub fn demo() -> Self {
        let mut demo = Self::new(10, 10);

        // Add Golbins: (6, 0), (9, 2), (9, 3), (0, 5), (4, 9).
        // Have them logically move to the other side of the map.
        demo.at(6, 0).add(Entity::Goblin {
            moving: Direction::Down,
        });

        demo.at(9, 2).add(Entity::Goblin {
            moving: Direction::Left,
        });

        demo.at(9, 3).add(Entity::Goblin {
            moving: Direction::Left,
        });

        demo.at(0, 5).add(Entity::Goblin {
            moving: Direction::Right,
        });

        demo.at(4, 9).add(Entity::Goblin {
            moving: Direction::Up,
        });

        // Add Houses: (4, 5), (6, 6)
        // Add Walls: (3, 3), (4, 3), (8, 5), (8, 6)
        // Add Farms: (4, 4), (7, 6)
        demo.at(4, 5).add(Entity::House);
        demo.at(6, 6).add(Entity::House);
        demo.at(3, 3).add(Entity::Wall);
        demo.at(4, 3).add(Entity::Wall);
        demo.at(8, 5).add(Entity::Wall);
        demo.at(8, 6).add(Entity::Wall);
        demo.at(4, 4).add(Entity::Farm);
        demo.at(7, 6).add(Entity::Farm);

        // Add Player: (5, 4)
        demo.at(5, 4).add(Entity::Player);

        demo
    }

    /// Create a new world with a given width and height.
    ///
    /// # Panics
    ///
    /// Panics if the width or height is 0.
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        Self {
            cells: vec![Cell::empty(); width * height],
            width,
        }
    }

    /// Returns the cell at the given coordinates.
    fn at(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[y * self.width + x]
    }

    /// Tick the world, moving all the goblins.
    ///
    /// If a goblin moves into another goblin, nothing happens.
    /// If a goblin moves into anything else, it is destroyed.
    fn tick(&mut self) {}
}

/// An entity in the game world.
#[derive(Clone)]
enum Entity {
    Farm,
    Goblin { moving: Direction },
    House,
    Player,
    Wall,
}

#[derive(Clone)]
enum Direction {
    Down,
    Left,
    Right,
    Up,
}

/// A cell in the game world.
#[derive(Clone)]
struct Cell {
    entities: Vec<Entity>,
}

impl Cell {
    /// Create a new empty cell.
    fn empty() -> Self {
        Self { entities: vec![] }
    }

    /// Add an entity to the cell.
    fn add(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Returns an iterator over the entities in the cell.
    fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    /// Returns the number of entities in the cell.
    fn len(&self) -> usize {
        self.entities.len()
    }

    /// Removes and returns the entity at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn remove(&mut self, index: usize) -> Entity {
        self.entities.remove(index)
    }
}
