// Allow it to build as WASM.
add_wasm_support!();

use std::cmp::min;

use bracket_lib::prelude::*;
use game::{Direction, Glyph, WorldState};
use ui::{ui2grid, UIState, UI};

mod game;
mod ui;

fn main() -> BError {
    // TermBuilder offers a number of helps to get up and running quickly.
    let context = BTermBuilder::simple(96, 66)?
        .with_title("Harvest Captain")
        .with_tile_dimensions(16, 16)
        .with_fullscreen(false)
        .build()?;

    // Empty state object.

    let state = State::new();

    main_loop(context, state)
}

/// This is the game state.
///
/// We are going to try and have the game state be a representation of the game at a point in time.
/// grid_res is the resolution of each grid square, i.e., a value of 4 means we have 4 titles per grid square
struct State {
    game: WorldState,
    grid_res: i32,
    sidebar: VirtualConsole,
    logger: VirtualConsole,
}

impl State {
    /// Create a new game state.
    pub fn new() -> Self {
        Self {
            game: WorldState::new(),
            grid_res: 5,
            sidebar: VirtualConsole::new(Point::new(30, 44)),
            logger: VirtualConsole::new(Point::new(30, 22)),
        }
    }
}

impl GameState for State {
    /// This is called every time the screen refreshes (a "tick") by the main loop.
    fn tick(&mut self, ctx: &mut BTerm) {
        // Clear the screen.
        ctx.cls();

        // Quit the game
        if ctx.key == Some(VirtualKeyCode::Escape) {
            ctx.quit();
        }

        // Direction player is moving
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

        // Handle Player Movement
        if let Some(direction) = direction {
            let result = self.game.player_move(direction);
            match result {
                Ok(_) => {}
                Err(e) => {
                    // TODO: Show this error to the player.
                    eprintln!("Error: {:?}", e);
                }
            }
        }

        // Get Mouse Position
        let mouse_pos = ui2grid(ctx.mouse_pos(), self.grid_res).to_tuple();

        // Build a Wall if the left mouse button is clicked.
        // Build a House if the SHIFT key is held down and the left mouse button is clicked.
        if in_bounds(mouse_pos, self.game.map_size()) && ctx.left_click {
            if ctx.shift {
                self.game.player_build(mouse_pos, Glyph::Farm);
            } else {
                self.game.player_build(mouse_pos, Glyph::Wall);
            }
        }

        // Update the game state.
        self.game.tick();

        // Create a UI renderer.
        let mut ui = UI::new(
            ctx,
            &mut self.sidebar,
            &mut self.logger,
            self.grid_res,
            BLACK,
        );

        // Create the UI state.
        let ui_state = UIState::new(
            self.game.to_render().into_iter().collect(),
            self.game.get_stats(),
            mouse_pos,
            self.game.get_logs(),
        );

        // Draw the UI.
        ui.draw(&ui_state);
    }
}

fn in_bounds(pos: (i32, i32), grid_size: (i32, i32)) -> bool {
    pos.0 >= 0 && pos.0 < grid_size.0 && pos.1 >= 0 && pos.1 < grid_size.1
}
