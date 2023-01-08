use bracket_lib::terminal::{BTerm, Point, BLACK, GRAY0, WHITE};

use crate::game::{DrawEntity, GameStats, Glyph};

/// UI module draws the game to the screen.
pub struct UI<'a> {
    ctx: &'a mut BTerm,
    grid_res: i32,
    uibox_pos: Point,
    grid_size: (i32, i32),
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
    pub mouse_grid: (i32, i32),
    draw_grid: bool,
}

impl UIState {
    pub fn new(entities: Vec<UIEntity>, stats: GameStats, mouse_grid: (i32, i32)) -> Self {
        Self {
            entities,
            stats,
            mouse_grid,
            draw_grid: true, //TODO: Remove when not debugging
        }
    }
}

impl<'a> UI<'a> {
    /// Create a new UI from an existing terminal.
    pub fn new(ctx: &'a mut BTerm, grid_res: i32) -> Self {
        let grid_size = (ctx.get_char_size().0 as i32, ctx.get_char_size().1 as i32);
        Self {
            ctx,
            grid_res,
            uibox_pos: Point::new(grid_size.0 / 4, 0),
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
            let e_pos_ui = grid2ui((e.e.x, e.e.y), self.grid_res, true);
            self.ctx.print(e_pos_ui.x, e_pos_ui.y, e.sym);
        }
    }

    /// Draw UI Box
    fn draw_ui_box(&mut self, state: &UIState) {
        self.uibox_pos = grid2ui(
            (self.grid_size.1 as i32 / self.grid_res, 0),
            self.grid_res,
            false,
        );

        self.ctx.draw_hollow_box_double(
            self.uibox_pos.x + 1,
            self.uibox_pos.y + 1,
            self.grid_size.0 - self.grid_size.1 - 1,
            self.grid_size.1 - self.grid_res,
            BLACK,
            WHITE,
        );

        self.write_ui_box_row(1, format!("Health {:?}", state.stats.health));
        self.write_ui_box_row(2, format!("Round  {}", state.stats.round));
        self.write_ui_box_row(3, format!("Farms  {}", state.stats.farms));
        self.write_ui_box_row(4, format!("Houses {}", state.stats.houses));
        self.write_ui_box_row(5, format!("Money  $ {}", state.stats.money));
        self.write_ui_box_row(6, format!("Mouse (GRID) : {:?}", state.mouse_grid));
        self.write_ui_box_row(
            7,
            format!(
                "Mouse (UI)   : {:?}",
                grid2ui(state.mouse_grid, self.grid_res, false).to_tuple()
            ),
        );
        for uie in &state.entities {
            if let Glyph::Player = uie.e.glyph {
                self.write_ui_box_row(8, format!("Player {:?}", (uie.e.x, uie.e.y)));
            }
        }
    }

    fn write_ui_box_row(&mut self, row: i32, value: String) {
        self.ctx.print(
            self.uibox_pos.x + self.grid_res,
            self.uibox_pos.y + self.grid_res * row,
            value,
        );
    }

    /// Draw a grid (debug purposes)
    /// note that main game field needs to be square, so use height_grid not width_grid for X dimension
    fn draw_grid(&mut self, state: &UIState) {
        if state.draw_grid {
            for x_ui in 0..(self.grid_size.1 / self.grid_res) {
                for y_ui in 0..(self.grid_size.1 / self.grid_res) {
                    let grid_point = grid2ui((x_ui, y_ui), self.grid_res, false);
                    self.ctx.draw_box(
                        grid_point.x,
                        grid_point.y,
                        self.grid_res,
                        self.grid_res,
                        WHITE,
                        GRAY0,
                    );
                }
            }
        }
    }
}

/// Transform grid coordiantes to native UI coordinates)
/// Transforms entity coordinates differently so that they
/// land in center of grid. Otherwise coordinates will land in
/// upper-left corner of bounding box.
pub fn grid2ui(pos_grid: (i32, i32), grid_res: i32, is_entity: bool) -> Point {
    if is_entity {
        Point::new(
            grid_res * pos_grid.0 + grid_res / 2,
            grid_res * pos_grid.1 + grid_res / 2,
        )
    } else {
        Point::new(grid_res * pos_grid.0, grid_res * pos_grid.1)
    }
}

/// Transforms native UI coordinates to grid coordinates
pub fn ui2grid(pos_ui: (i32, i32), grid_res: i32) -> Point {
    Point::new(pos_ui.0 / grid_res, pos_ui.1 / grid_res)
}
