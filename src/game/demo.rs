//! Demo system.
use super::components::*;
use specs::prelude::*;

/// Spawns the demo level, returning the player entity.
#[must_use]
pub(super) fn spawn_demo(ecs: &mut World) -> Entity {
    const LEVEL: [[char; 12]; 12] = [
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', 'G', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', 'G'],
        [' ', ' ', ' ', ' ', ' ', 'F', '@', ' ', ' ', ' ', ' ', ' '],
        ['G', ' ', ' ', ' ', 'W', 'H', ' ', ' ', 'W', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', 'W', ' ', 'H', 'F', 'W', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', 'G', ' ', ' ', ' ', ' ', ' ', ' '],
    ];

    let mut player_entity: Option<Entity> = None;

    for (y, row) in LEVEL.iter().enumerate() {
        for (x, glyph) in row.iter().enumerate() {
            let (x, y) = (x as i32, y as i32);
            let entity = ecs.create_entity();

            match glyph {
                '@' => {
                    let entity = configure_player(entity, x, y).build();
                    player_entity = Some(entity);
                }
                'G' => {
                    configure_goblin(entity, x, y).build();
                }
                'F' => {
                    configure_farm(entity, x, y).build();
                }
                'W' => {
                    configure_wall(entity, x, y).build();
                }
                'H' => {
                    configure_house(entity, x, y).build();
                }
                _ => {}
            };
        }
    }

    player_entity.expect("No player entity found in demo level")
}

fn configure_player(entity: EntityBuilder, x: i32, y: i32) -> EntityBuilder {
    entity
        .with(Position::new(x, y))
        .with(Renderable::new(Glyph::Player))
        .with(Health::new(32))
        .with(Health::new(32))
        .with(Player)
}

pub fn configure_goblin(entity: EntityBuilder, x: i32, y: i32) -> EntityBuilder {
    entity
        .with(Position::new(x, y))
        .with(Renderable::new(Glyph::Goblin))
        .with(Health::new(2))
        .with(Monster)
}

pub fn configure_farm(entity: EntityBuilder, x: i32, y: i32) -> EntityBuilder {
    entity
        .with(Position::new(x, y))
        .with(Renderable::new(Glyph::Farm))
        .with(Health::new(1))
}

pub fn configure_wall(entity: EntityBuilder, x: i32, y: i32) -> EntityBuilder {
    entity
        .with(Position::new(x, y))
        .with(Renderable::new(Glyph::Wall))
        .with(Health::new(3))
}

pub fn configure_house(entity: EntityBuilder, x: i32, y: i32) -> EntityBuilder {
    entity
        .with(Position::new(x, y))
        .with(Renderable::new(Glyph::House))
        .with(Health::new(2))
}
