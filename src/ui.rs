use bracket_lib::terminal::BTerm;

/// UI module draws the game to the screen.
pub struct UI<'a> {
    ctx: &'a mut BTerm,
}

/// What we want to draw the screen logically.
///
/// This won't know about game logic, just what to draw.
pub struct UIState;

impl<'a> UI<'a> {
    /// Create a new UI from an existing terminal.
    pub fn new(ctx: &'a mut BTerm) -> Self {
        Self { ctx }
    }

    /// Draw the game to the screen.
    pub fn draw(&mut self, _state: &UIState) {
        self.ctx.print(0, 0, "@");

        // TODO: Draw a grid.
        // TODO: Given a 2-dimensional grid of cells, draw the individual cells (W/G/F/H/@).
    }
}
