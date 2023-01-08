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
        let direction: Option<Direction> = match ctx.key {
            None => None,
            Some(key) => match key {
                VirtualKeyCode::Left | VirtualKeyCode::A => Some(Direction::Left),
                VirtualKeyCode::Right | VirtualKeyCode::D => Some(Direction::Right),
                VirtualKeyCode::Up | VirtualKeyCode::W => Some(Direction::Up),
                VirtualKeyCode::Down | VirtualKeyCode::S => Some(Direction::Down),
                _ => None,
            },
        };
        if let Some(direction) = direction {
            self.game.player_move(direction);
        }

        // Update the game state.
        self.game.tick();

        // Create the UI state.
        let ui_state = UIState::new(
            self.game
                .to_render()
                .into_iter()
                .map(|de| UIEntity {
                    sym: match de.glyph {
                        Glyph::Goblin => 'g',
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
