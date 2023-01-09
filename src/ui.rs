use std::cmp::min;

use bracket_lib::prelude::*;

use bracket_lib::terminal::{BTerm, Console, Point, Rect, VirtualConsole};

use crate::game::{logger::LogMessage, DrawEntity, GameStats, Glyph};

pub struct UIProperties {
    pub fg: (u8, u8, u8), // Foreground color
    pub bg: (u8, u8, u8), // Background color
    pub sym: char,
}

/// What we want to draw the screen logically.
///
/// This won't know about game logic, just what to draw.
pub struct UIState {
    pub entities: Vec<DrawEntity>,
    pub stats: GameStats,
    pub mouse_grid: (i32, i32),
    pub logs: Vec<LogMessage>,
}

impl UIState {
    pub fn new(
        entities: Vec<DrawEntity>,
        stats: GameStats,
        mouse_grid: (i32, i32),
        logs: Vec<LogMessage>,
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
            let e_pos_ui = grid2ui((e.x, e.y), self.grid_res);

            for dx in 0..self.grid_res {
                for dy in 0..self.grid_res {
                    self.ctx.print_color(
                        e_pos_ui.x + dx,
                        e_pos_ui.y + dy,
                        ui_properties(&e.glyph).fg,
                        ui_properties(&e.glyph).bg,
                        if dx == self.grid_res / 2 && dy == self.grid_res / 2 {
                            ui_properties(&e.glyph).sym
                        } else {
                            ' '
                        },
                    );
                }
            }
            if e.hp.0 > 1 {
                self.ctx.print(e_pos_ui.x + 1, e_pos_ui.y + 1, e.hp.0)
            }
        }
    }

    fn draw_logger(&mut self, state: &UIState) {
        // self.logger
        //     .set_translation_mode(CharacterTranslationMode::Unicode);

        for (i, log) in state.logs.iter().enumerate() {
            #[allow(clippy::single_match)]
            match log {
                LogMessage::Attacked {
                    attacker,
                    target,
                    position,
                    ..
                } => {
                    self.write_row_logger(
                        i as i32,
                        format!(
                            "{:?} ⚔ {:?} at {:?}",
                            ui_properties(attacker).sym,
                            ui_properties(target).sym,
                            position
                        ),
                    );
                }
                #[allow(unreachable_patterns)]
                _ => {}
            }
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
                44,
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
            GREEN.into(),
            BLACK.into(),
        );

        self.write_row_sidebar(0, format!("Health {:?}", state.stats.health));
        self.write_row_sidebar(1, format!("Round  {}", state.stats.round));
        self.write_row_sidebar(2, format!("Farms  {}", state.stats.farms));
        self.write_row_sidebar(3, format!("Houses {}", state.stats.houses));
        self.write_row_sidebar(4, format!("Money  $ {}", state.stats.money));
        self.write_row_sidebar(5, format!("Mouse (GRID) : {:?}", state.mouse_grid));
        for uie in &state.entities {
            if let Glyph::Player = uie.glyph {
                self.write_row_sidebar(6, format!("Player {:?}", (uie.x, uie.y)));
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
                let grid_point = grid2ui((x_ui, y_ui), self.grid_res);
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
pub fn grid2ui(pos_grid: (i32, i32), grid_res: i32) -> Point {
    Point::new(grid_res * pos_grid.0, grid_res * pos_grid.1)
}

/// Transforms native UI coordinates to grid coordinates
pub fn ui2grid(pos_ui: (i32, i32), grid_res: i32) -> Point {
    Point::new(pos_ui.0 / grid_res, pos_ui.1 / grid_res)
}

/// Create UIProperites struct for all Glyph types
fn ui_properties(g: &Glyph) -> UIProperties {
    match g {
        Glyph::Goblin => UIProperties {
            fg: RED,
            bg: BLACK,
            sym: 'g',
        },
        Glyph::Rat => UIProperties {
            fg: SADDLE_BROWN,
            bg: BLACK,
            sym: 'r',
        },
        Glyph::Player => UIProperties {
            fg: SKY_BLUE,
            bg: DARKBLUE,
            sym: '@',
        },
        Glyph::Wall => UIProperties {
            fg: SILVER,
            bg: BLACK,
            sym: '#',
        },
        Glyph::Farm => UIProperties {
            fg: GOLD,
            bg: LIGHTGREEN,
            sym: 'f',
        },
        Glyph::House => UIProperties {
            fg: PURPLE,
            bg: LIGHTGREEN,
            sym: 'h',
        },
        Glyph::Tree => UIProperties {
            fg: DARKGREEN,
            bg: LIGHTGREEN,
            sym: 't',
        },
        Glyph::Orc => UIProperties {
            fg: ORANGE,
            bg: BLACK,
            sym: 'o',
        },
    }
}
