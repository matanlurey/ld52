// Allow it to build as WASM.
add_wasm_support!();

use bracket_lib::prelude::*;
use ui::{UIEntity, UIState, UI};
use world::World;

mod ui;
mod world;

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
    world: World,
}

impl State {
    /// Create a new game state.
    pub fn new() -> Self {
        Self { world: World }
    }
}

impl GameState for State {
    /// This is called every time the screen refreshes (a "tick") by the main loop.
    fn tick(&mut self, ctx: &mut BTerm) {
        // Clear the screen.
        ctx.cls();

        // Create a UI renderer.
        let mut ui = UI::new(ctx);

        let mut uie = Vec::new();

        // Player
        uie.push(UIEntity {
            x: 0,
            y: 0,
            sym: '@',
        });

        // Farm(s)
        uie.push(UIEntity {
            x: 2,
            y: 2,
            sym: 'F',
        });

        // Home(s)
        uie.push(UIEntity {
            x: 3,
            y: 3,
            sym: 'H',
        });

        // Goblin(s)
        uie.push(UIEntity {
            x: 4,
            y: 4,
            sym: 'G',
        });

        // Create the UI state.
        let ui_state = UIState::new(uie);

        // Draw the UI.
        ui.draw(&ui_state);
    }
}
