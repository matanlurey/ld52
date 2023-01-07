// Allow it to build as WASM.
add_wasm_support!();

use bracket_lib::prelude::*;
use game::{Direction, WorldState};
use ui::{UIState, UI};

mod game;
mod ui;

fn main() -> BError {
    // TermBuilder offers a number of helps to get up and running quickly.
    let context = BTermBuilder::simple80x50()
        .with_title("Hello, Bracket!")
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

        // Player movement.
        match ctx.key {
            None => {}
            Some(key) => match key {
                VirtualKeyCode::Left => self.game.move_player(Direction::Left),
                VirtualKeyCode::Right => self.game.move_player(Direction::Right),
                VirtualKeyCode::Up => self.game.move_player(Direction::Up),
                VirtualKeyCode::Down => self.game.move_player(Direction::Down),
                _ => {}
            },
        }

        // Iterate the game state.
        for entity in self.game.to_render() {
            // TODO: Replace with drawing logic.
            dbg!(entity);
        }

        // Create a UI renderer.
        let mut ui = UI::new(ctx);

        // Create the UI state.
        let ui_state = UIState;

        // Draw the UI.
        ui.draw(&ui_state);
    }
}
