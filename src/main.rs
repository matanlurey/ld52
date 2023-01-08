// Allow it to build as WASM.
add_wasm_support!();

use bracket_lib::prelude::*;
use game::{Direction, Glyph, WorldState};
use ui::{ui2grid, UIEntity, UIState, UI};

mod game;
mod ui;

fn main() -> BError {
    // TermBuilder offers a number of helps to get up and running quickly.
    let context = BTermBuilder::simple(80, 50)?
        .with_title("Harvest Captain")
        .with_tile_dimensions(16, 16)
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
    #[allow(dead_code)]
    game: WorldState,
    grid_res: i32,
}

impl State {
    /// Create a new game state.
    pub fn new() -> Self {
        Self {
            game: WorldState::new(),
            grid_res: 4,
        }
    }
}

impl GameState for State {
    /// This is called every time the screen refreshes (a "tick") by the main loop.
    fn tick(&mut self, ctx: &mut BTerm) {
        // Clear the screen.
        ctx.cls();

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
        if ctx.left_click {
            if ctx.shift {
                self.game.player_build(mouse_pos, Glyph::Farm);
            } else {
                self.game.player_build(mouse_pos, Glyph::Wall);
            }
        }

        // Update the game state.
        self.game.tick();

        // Create a UI renderer.
        let mut ui = UI::new(ctx, self.grid_res);

        // Create the UI state.
        let ui_state = UIState::new(
            self.game
                .to_render()
                .into_iter()
                .map(|de| UIEntity {
                    sym: match de.glyph {
                        Glyph::Goblin => 'g',
                        Glyph::Player => '@',
                        Glyph::Wall => '#',
                        Glyph::Farm => 'f',
                        Glyph::House => 'h',
                    },
                    e: de,
                })
                .collect(),
            self.game.get_stats(),
            mouse_pos,
        );

        // Draw the UI.
        ui.draw(&ui_state);
    }
}
