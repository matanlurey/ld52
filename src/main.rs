// Allow it to build as WASM.
add_wasm_support!();

use bracket_lib::prelude::*;
use game::{Direction, Glyph, WorldState};
use ui::{UIEntity, UIState, UI};

mod game;
mod ui;

fn main() -> BError {
    // TermBuilder offers a number of helps to get up and running quickly.
    let context = BTermBuilder::simple(80, 50)?
        .with_title("Hello, Bracket!")
        .with_tile_dimensions(16, 16)
        .build()?;

    let (w, h) = context.get_char_size();
    println!("Tile width {}", context.width_pixels / w);
    println!("Tile height {}", context.height_pixels / h);
    // Empty state object.
    let state = State::new();

    main_loop(context, state)
}

/// This is the game state.
///
/// We are going to try and have the game state be a representation of the game at a point in time.
struct State {
    #[allow(dead_code)]
    game: WorldState,
}

impl State {
    /// Create a new game state.
    pub fn new() -> Self {
        Self {
            game: WorldState::new(),
        }
    }
}

impl GameState for State {
    /// This is called every time the screen refreshes (a "tick") by the main loop.
    fn tick(&mut self, ctx: &mut BTerm) {
        // Clear the screen.
        ctx.cls();

        // If a player is moving

        match ctx.key {
            None => self.game.stop_player(),
            Some(key) => match key {
                VirtualKeyCode::Left => self.game.move_player(Direction::Left),
                VirtualKeyCode::Right => self.game.move_player(Direction::Right),
                VirtualKeyCode::Up => self.game.move_player(Direction::Up),
                VirtualKeyCode::Down => self.game.move_player(Direction::Down),
                _ => {}
            },
        };

        // Create the UI state.
        let ui_state = UIState::new(
            self.game
                .to_render()
                .into_iter()
                .map(|de| UIEntity {
                    sym: match de.glyph {
                        Glyph::Player => '@',
                    },
                    e: de,
                })
                .collect(),
        );

        // Create a UI renderer.
        let mut ui = UI::new(ctx);

        // Draw the UI.
        ui.draw(&ui_state);
    }
}
