use specs::prelude::*;
use specs_derive::Component;

/// Our external world, i.e. how it will be represented in the game.
#[derive(Debug)]
pub struct DrawEntity {
    pub x: usize,
    pub y: usize,
    pub glyph: Glyph,
}

/// What kind of glyph is meant to be drawn.
#[derive(Clone, Debug)]
pub enum Glyph {
    Player,
}

/// A component that represents a tile position in the world.
#[derive(Component, Clone)]
struct Position {
    x: usize,
    y: usize,
}

/// A component that represents a renderable entity.
#[derive(Component, Clone)]
struct Renderable {
    glyph: Glyph,
}

/// A component that represents a player entity.
#[derive(Component)]
struct Player;

/// Directions that the player can move in.
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct WorldState {
    ecs: World,
}

impl WorldState {
    /// Create a new game state.
    pub fn new() -> Self {
        let mut ecs = World::new();

        ecs.register::<Position>();
        ecs.register::<Renderable>();
        ecs.register::<Player>();

        // Creaet a player entity with ECS.
        ecs.create_entity()
            .with(Position { x: 5, y: 4 })
            .with(Renderable {
                glyph: Glyph::Player,
            })
            .with(Player)
            .build();

        Self { ecs }
    }

    /// Move the player in a given direction.
    pub fn move_player(&mut self, direction: Direction) {
        let mut positions = self.ecs.write_storage::<Position>();
        let players = self.ecs.read_storage::<Player>();

        for (_player, pos) in (&players, &mut positions).join() {
            let (x, y) = match direction {
                Direction::Up => (0, -1),
                Direction::Down => (0, 1),
                Direction::Left => (-1, 0),
                Direction::Right => (1, 0),
            };
            pos.x = (pos.x as i32 + x) as usize;
            pos.y += (pos.y as i32 + y) as usize;
        }
    }

    /// Return a collection of all renderable entities with their positions and glyphs.
    pub fn to_render(&self) -> Vec<DrawEntity> {
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        (&positions, &renderables)
            .join()
            .map(|(pos, render)| DrawEntity {
                x: pos.x,
                y: pos.y,
                glyph: render.glyph.clone(),
            })
            .collect()
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_game_state() {
        let state = WorldState::new();

        let entities = state.to_render();

        assert_eq!(entities.len(), 1);
    }
}
