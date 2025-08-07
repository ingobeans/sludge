use macroquad::prelude::*;

pub const SCREEN_WIDTH: i16 = 192;
pub const SCREEN_HEIGHT: i16 = 144;
pub const SPRITE_SIZE: i16 = 8;
pub const MENU_WIDTH: i16 = CARD_SIZE * 2 + 4;
pub const CARD_SIZE: i16 = SPRITE_SIZE + 4;

pub const DEFAULT_SPAWN_DELAY: u8 = 30;
pub const STARTING_LIVES: u8 = 100;

pub const UI_BG_COLOR: Color = Color::from_hex(0xb86f50);
pub const UI_INNER_BORDER_COLOR: Color = Color::from_hex(0xffe762);
pub const UI_OUTER_BORDER_COLOR: Color = Color::from_hex(0x743f39);

// define usize variants
pub const SCREEN_WIDTH_USIZE: usize = SCREEN_WIDTH as usize;
pub const SCREEN_HEIGHT_USIZE: usize = SCREEN_HEIGHT as usize;
pub const SPRITE_SIZE_USIZE: usize = SPRITE_SIZE as usize;
pub const MENU_WIDTH_USIZE: usize = MENU_WIDTH as usize;
pub const CARD_SIZE_USIZE: usize = CARD_SIZE as usize;
