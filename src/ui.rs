use bracket_lib::terminal::{BTerm, BLACK, GRAY0, WHITE};

/// UI module draws the game to the screen.
pub struct UI<'a> {
    ctx: &'a mut BTerm,
}

pub struct UIEntity {
    pub x: i32,
    pub y: i32,
    pub sym: char,
}

/// What we want to draw the screen logically.
///
/// This won't know about game logic, just what to draw.
pub struct UIState {
    pub entities: Vec<UIEntity>,
    draw_grid: bool,
    width_grid: i32,
    height_grid: i32,
    ui_width: i32,
    grid_res: i32,
}

impl UIState {
    pub fn new(entities: Vec<UIEntity>) -> Self {
        Self {
            entities,
            draw_grid: true, //TODO: Remove when not debugging
            width_grid: 80,
            height_grid: 50,
            ui_width: 30,
            grid_res: 2,
        }
    }
}

impl<'a> UI<'a> {
    /// Create a new UI from an existing terminal.
    pub fn new(ctx: &'a mut BTerm) -> Self {
        Self { ctx }
    }

    /// Draw the game to the screen.
    ///
    /// The UI grid will be 3x the resolution of the game grid.
    /// That is, every third "tile" will be an actual place that a
    /// UIEntity can contain, and the "tiles" between are reserved for a grid.
    pub fn draw(&mut self, state: &UIState) {
        // Draw a grid (debug purposes)
        if state.draw_grid {
            for x_game in 0..((state.width_grid - state.ui_width) / state.grid_res) {
                for y_game in 0..(state.height_grid / state.grid_res) {
                    let (x_ui, y_ui) = game2ui(x_game, y_game, false, state);
                    self.ctx
                        .draw_box(x_ui, y_ui, state.grid_res, state.grid_res, WHITE, GRAY0);
                }
            }
        }

        // Draw UI Box
        let (x_uibox, y_uibox) = game2ui(
            (state.width_grid - state.ui_width) / state.grid_res,
            0,
            true,
            state,
        );
        self.ctx.draw_hollow_box_double(
            x_uibox,
            y_uibox,
            state.ui_width - state.grid_res,
            state.height_grid - state.grid_res,
            BLACK,
            WHITE,
        );

        // Draw game entities
        //TODO: include colors
        for e in &state.entities {
            let (x_ui, y_ui) = game2ui(e.x, e.y, true, state);
            self.ctx.print(x_ui, y_ui, e.sym);
        }
    }
}

/// Transform game coordiantes to UI coordinates
/// Transforms entity coordinates differently so that they
/// land in center of grid. Otherwise coordinates will land in
/// upper-left corner of bounding box.
fn game2ui(x_game: i32, y_game: i32, is_entity: bool, state: &UIState) -> (i32, i32) {
    let x_ui = state.grid_res * x_game;
    let y_ui = state.grid_res * y_game;
    if is_entity {
        (x_ui + state.grid_res / 2, y_ui + state.grid_res / 2)
    } else {
        (x_ui, y_ui)
    }
}
