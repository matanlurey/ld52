use bracket_lib::terminal::{BTerm, BLACK, GRAY0, WHITE};

use crate::game::{DrawEntity, Glyph};

/// UI module draws the game to the screen.
pub struct UI<'a> {
    ctx: &'a mut BTerm,
    grid_resolution: u32,
}

pub struct UIEntity {
    pub e: DrawEntity,
    pub sym: char,
}

/// What we want to draw the screen logically.
///
/// This won't know about game logic, just what to draw.
pub struct UIState {
    pub entities: Vec<UIEntity>,
    draw_grid: bool,
}

impl UIState {
    pub fn new(entities: Vec<UIEntity>) -> Self {
        Self {
            entities,
            draw_grid: true, //TODO: Remove when not debugging
        }
    }
}

impl<'a> UI<'a> {
    /// Create a new UI from an existing terminal.
    pub fn new(ctx: &'a mut BTerm) -> Self {
        Self {
            ctx,
            grid_resolution: 4,
        }
    }

    /// Draw the game to the screen.
    ///
    /// The UI grid will be 3x the resolution of the game grid.
    /// That is, every third "tile" will be an actual place that a
    /// UIEntity can contain, and the "tiles" between are reserved for a grid.
    pub fn draw(&mut self, state: &UIState) {
        let (width_grid, height_grid) = self.ctx.get_char_size();
        // let tile_width = self.ctx.width_pixels / width_grid;
        // let tile_height = self.ctx.height_pixels / height_grid;

        // Draw a grid (debug purposes)
        // note that main field needs to be square, so use height_grid not width_grid for X dimension
        if state.draw_grid {
            for x_game in 0..(height_grid / self.grid_resolution) {
                for y_game in 0..(height_grid / self.grid_resolution) {
                    let (x_ui, y_ui) = self.game2ui(x_game, y_game, false);
                    self.ctx.draw_box(
                        x_ui,
                        y_ui,
                        self.grid_resolution,
                        self.grid_resolution,
                        WHITE,
                        GRAY0,
                    );
                }
            }
        }

        // Draw UI Box
        let (x_uibox, y_uibox) = self.game2ui(height_grid / self.grid_resolution, 0, false);
        self.ctx.draw_hollow_box_double(
            x_uibox + 1,
            y_uibox + 1,
            width_grid - height_grid - 1,
            height_grid - self.grid_resolution,
            BLACK,
            WHITE,
        );

        // Draw game entities
        //TODO: include colors
        for e in &state.entities {
            let (x_ui, y_ui) = self.game2ui(e.e.x as u32, e.e.y as u32, true);
            self.ctx.print(x_ui, y_ui, e.sym);

            match e.e.glyph {
                Glyph::Player => {
                    self.ctx.print(
                        x_uibox + self.grid_resolution,
                        y_uibox + self.grid_resolution,
                        format!("X: {}, Y: {}", e.e.x, e.e.y),
                    );
                }
                Glyph::Goblin => {}
            }
        }
    }

    /// Transform game coordiantes to UI coordinates
    /// Transforms entity coordinates differently so that they
    /// land in center of grid. Otherwise coordinates will land in
    /// upper-left corner of bounding box.
    fn game2ui(&self, x_game: u32, y_game: u32, is_entity: bool) -> (u32, u32) {
        let x_ui = self.grid_resolution * x_game;
        let y_ui = self.grid_resolution * y_game;
        if is_entity {
            (
                x_ui + self.grid_resolution / 2,
                y_ui + self.grid_resolution / 2,
            )
        } else {
            (x_ui, y_ui)
        }
    }
}
