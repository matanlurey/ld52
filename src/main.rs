// Allow it to build as WASM.
add_wasm_support!();

use bracket_lib::prelude::*;

fn main() -> BError {
    // TermBuilder offers a number of helps to get up and running quickly.
    let context = BTermBuilder::simple80x50()
        .with_title("Hello, Bracket!")
        .build()?;

    // Empty state object.
    let state = State {
        y: 1,
        going_down: true,
    };

    main_loop(context, state)
}

/// Stores game state, typically a state machine pointing to other structures.
///
/// This is a simple demo.
struct State {
    y: i32,
    going_down: bool,
}

impl GameState for State {
    /// This is called every time the screen refreshes (a "tick") by the main loop.
    fn tick(&mut self, ctx: &mut BTerm) {
        let col1 = RGB::named(CYAN);
        let col2 = RGB::named(YELLOW);
        let percent = self.y as f32 / 50.0;
        let fg = col1.lerp(col2, percent);

        ctx.cls();

        ctx.print_color(1, self.y, fg, RGB::named(BLACK), "♫ ♪ Hello, Bracket! ☺");

        // Make the text move up and down.
        if self.going_down {
            self.y += 1;
            if self.y > 48 {
                self.going_down = false;
            }
        } else {
            self.y -= 1;
            if self.y < 2 {
                self.going_down = true;
            }
        }

        // Show the frame rate.
        ctx.draw_box(39, 0, 20, 3, RGB::named(WHITE), RGB::named(BLACK));
        ctx.print_color(
            40,
            1,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            &format!("FPS: {}", ctx.fps),
        );
        ctx.print_color(
            40,
            2,
            RGB::named(CYAN),
            RGB::named(BLACK),
            &format!("Frame Time: {}ms", ctx.frame_time_ms),
        );
    }
}
