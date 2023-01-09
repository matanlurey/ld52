use std::cmp::min;

use bracket_lib::terminal::{BTerm, Console, Point, Rect, VirtualConsole, BLACK, RED, WHITE};

use crate::game::{DrawEntity, GameStats, Glyph};

pub struct UIEntity {
    pub e: DrawEntity,
    pub color: (u8, u8, u8),
    pub sym: char,
}

/// What we want to draw the screen logically.
///
/// This won't know about game logic, just what to draw.
pub struct UIState {
    pub entities: Vec<UIEntity>,
    pub stats: GameStats,
    pub mouse_grid: (i32, i32),
    pub logs: Vec<String>,
}

impl UIState {
    pub fn new(
        entities: Vec<UIEntity>,
        stats: GameStats,
        mouse_grid: (i32, i32),
        logs: Vec<String>,
    ) -> Self {
        Self {
            entities,
            stats,
            mouse_grid,
            logs,
        }
    }
}

/// UI module draws the game to the screen.
pub struct UI<'a> {
    ctx: &'a mut BTerm,
    sidebar: &'a mut VirtualConsole,
    logger: &'a mut VirtualConsole,
    grid_res: i32,
    grid_color: (u8, u8, u8),
    field_size: i32,
    _logs: Vec<String>,
}

impl<'a> UI<'a> {
    /// Create a new UI from an existing terminal.
    pub fn new(
        ctx: &'a mut BTerm,
        sidebar: &'a mut VirtualConsole,
        logger: &'a mut VirtualConsole,
        grid_res: i32,
        grid_color: (u8, u8, u8),
    ) -> Self {
        let grid_size = (ctx.get_char_size().0 as i32, ctx.get_char_size().1 as i32);
        let field_size = min(grid_size.0, grid_size.1);
        Self {
            ctx,
            sidebar,
            logger,
            grid_res,
            grid_color,
            field_size,
            _logs: Vec::new(),
        }
    }

    /// Draw the game to the screen.
    pub fn draw(&mut self, state: &UIState) {
        self.draw_grid();

        self.draw_sidebar(state);

        self.draw_logger(state);

        self.draw_entities(state);
    }

    /// Draw game entities
    fn draw_entities(&mut self, state: &UIState) {
        for e in &state.entities {
            let e_pos_ui = grid2ui((e.e.x, e.e.y), self.grid_res, true);
            self.ctx
                .print_color(e_pos_ui.x, e_pos_ui.y, e.color, BLACK, e.sym);
            if e.e.hp.0 > 1 {
                self.ctx.print(
                    e_pos_ui.x - self.grid_res / 2 + 1,
                    e_pos_ui.y - self.grid_res / 2 + 1,
                    e.e.hp.0,
                )
            }
        }
    }

    fn draw_logger(&mut self, state: &UIState) {
        for (i, log) in state.logs.iter().enumerate() {
            self.write_row_logger(i as i32, format!("{:?}", log));
        }

        self.logger.draw_hollow_box_double(
            0,
            0,
            self.logger.width as i32 - 2,
            self.logger.height as i32 - 2,
            RED.into(),
            BLACK.into(),
        );

        self.logger.print_sub_rect(
            Rect::with_size(0, 0, self.logger.width, self.logger.height),
            Rect::with_size(
                (self.field_size + 1).try_into().unwrap(),
                30,
                self.logger.width,
                self.logger.height,
            ),
            self.ctx,
        );
    }

    /// Draw Sidebar
    fn draw_sidebar(&mut self, state: &UIState) {
        self.sidebar.draw_hollow_box_double(
            0,
            0,
            self.sidebar.width as i32 - 2,
            self.sidebar.height as i32 - 2,
            WHITE.into(),
            BLACK.into(),
        );

        self.write_row_sidebar(0, format!("Health {:?}", state.stats.health));
        self.write_row_sidebar(1, format!("Round  {}", state.stats.round));
        self.write_row_sidebar(2, format!("Farms  {}", state.stats.farms));
        self.write_row_sidebar(3, format!("Houses {}", state.stats.houses));
        self.write_row_sidebar(4, format!("Money  $ {}", state.stats.money));
        self.write_row_sidebar(5, format!("Mouse (GRID) : {:?}", state.mouse_grid));
        for uie in &state.entities {
            if let Glyph::Player = uie.e.glyph {
                self.write_row_sidebar(6, format!("Player {:?}", (uie.e.x, uie.e.y)));
            }
        }
        self.sidebar.print_sub_rect(
            Rect::with_size(0, 0, self.sidebar.width, self.sidebar.height),
            Rect::with_size(
                (self.field_size + 1).try_into().unwrap(),
                0,
                self.sidebar.width,
                self.sidebar.height,
            ),
            self.ctx,
        );
    }
    fn write_row_logger(&mut self, row: i32, value: String) {
        self.logger.print(
            self.grid_res / 2,
            (self.grid_res as f64 * (row as f64 + 0.5)) as i32,
            &value,
        );
    }
    fn write_row_sidebar(&mut self, row: i32, value: String) {
        self.sidebar.print(
            self.grid_res / 2,
            (self.grid_res as f64 * (row as f64 + 0.5)) as i32,
            &value,
        );
    }

    /// Draw a grid (debug purposes)
    fn draw_grid(&mut self) {
        for x_ui in 0..(self.field_size / self.grid_res) {
            for y_ui in 0..(self.field_size / self.grid_res) {
                let grid_point = grid2ui((x_ui, y_ui), self.grid_res, false);
                self.ctx.draw_box(
                    grid_point.x,
                    grid_point.y,
                    self.grid_res,
                    self.grid_res,
                    self.grid_color,
                    BLACK,
                );
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
