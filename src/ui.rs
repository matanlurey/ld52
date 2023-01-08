use bracket_lib::terminal::{BTerm, Point, BLACK, GRAY0, WHITE};

use crate::game::{DrawEntity, GameStats};

/// UI module draws the game to the screen.
pub struct UI<'a> {
    ctx: &'a mut BTerm,
    grid_resolution: u32,
    uibox_position: Point,
    grid_size: (u32, u32),
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
    pub stats: GameStats,
    pub mouse_xy: (i32, i32),
    draw_grid: bool,
}

impl UIState {
    pub fn new(entities: Vec<UIEntity>, stats: GameStats, mouse_xy: (i32, i32)) -> Self {
        Self {
            entities,
            stats,
            mouse_xy,
            draw_grid: true, //TODO: Remove when not debugging
        }
    }
}

impl<'a> UI<'a> {
    /// Create a new UI from an existing terminal.
    pub fn new(ctx: &'a mut BTerm) -> Self {
        let grid_size = ctx.get_char_size();
        Self {
            ctx,
            grid_resolution: 4,
            uibox_position: Point::new(0, 0),
            grid_size,
        }
    }

    /// Draw the game to the screen.
    pub fn draw(&mut self, state: &UIState) {
        self.draw_grid(state);

        self.draw_ui_box(state);

        self.draw_entities(state);
    }

    /// Draw game entities
    fn draw_entities(&mut self, state: &UIState) {
        for e in &state.entities {
            let ui_point = self.game2ui((e.e.x as u32, e.e.y as u32), true);
            self.ctx.print(ui_point.x, ui_point.y, e.sym);
        }
    }

    /// Draw UI Box
    fn draw_ui_box(&mut self, state: &UIState) {
        self.uibox_position = self.game2ui((self.grid_size.1 / self.grid_resolution, 0), false);

        self.ctx.draw_hollow_box_double(
            self.uibox_position.x + 1,
            self.uibox_position.y + 1,
            self.grid_size.0 - self.grid_size.1 - 1,
            self.grid_size.1 - self.grid_resolution,
            BLACK,
            WHITE,
        );

        self.write_ui_box_row(
            1,
            format!("Health {} / {}", state.stats.health.0, state.stats.health.1),
        );
        self.write_ui_box_row(2, format!("Round  {}", state.stats.round));
        self.write_ui_box_row(3, format!("Farms  {}", state.stats.farms));
        self.write_ui_box_row(4, format!("Houses {}", state.stats.houses));
        self.write_ui_box_row(5, format!("Money  $ {}", state.stats.money));
        self.write_ui_box_row(
            6,
            format!(
                "Mouse {:?}",
                self.ui2game((state.mouse_xy.0 as u32, state.mouse_xy.1 as u32))
            ),
        );
    }

    fn write_ui_box_row(&mut self, row: u32, value: String) {
        self.ctx.print(
            self.uibox_position.x + (self.grid_resolution as i32),
            self.uibox_position.y + (self.grid_resolution * row) as i32,
            value,
        );
    }

    /// Draw a grid (debug purposes)
    /// note that main game field needs to be square, so use height_grid not width_grid for X dimension
    fn draw_grid(&mut self, state: &UIState) {
        if state.draw_grid {
            for x_game in 0..(self.grid_size.1 / self.grid_resolution) {
                for y_game in 0..(self.grid_size.1 / self.grid_resolution) {
                    let ui_point = self.game2ui((x_game, y_game), false);
                    self.ctx.draw_box(
                        ui_point.x,
                        ui_point.y,
                        self.grid_resolution,
                        self.grid_resolution,
                        WHITE,
                        GRAY0,
                    );
                }
            }
        }
    }

    /// Transform game coordiantes to UI coordinates
    /// Transforms entity coordinates differently so that they
    /// land in center of grid. Otherwise coordinates will land in
    /// upper-left corner of bounding box.
    fn game2ui(&self, pos_game: (u32, u32), is_entity: bool) -> Point {
        if is_entity {
            Point::new(
                self.grid_resolution * pos_game.0 + self.grid_resolution / 2,
                self.grid_resolution * pos_game.1 + self.grid_resolution / 2,
            )
        } else {
            Point::new(
                self.grid_resolution * pos_game.0,
                self.grid_resolution * pos_game.1,
            )
        }
    }

    fn ui2game(&self, pos_ui: (u32, u32)) -> Point {
        Point::new(
            pos_ui.0 / self.grid_resolution,
            pos_ui.1 / self.grid_resolution,
        )
    }
}
