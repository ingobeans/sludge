use std::f32::consts::PI;

use macroquad::prelude::*;

pub const SCREEN_WIDTH: f32 = 192.0;
pub const SCREEN_HEIGHT: f32 = 144.0;
pub const SPRITE_SIZE: f32 = 8.0;
pub const MENU_WIDTH: f32 = CARD_SIZE * 2.0 + 4.0;
pub const CARD_SIZE: f32 = SPRITE_SIZE + 4.0;

pub const INV_SLOTS_HORIZONTAL: usize = (MENU_WIDTH_USIZE - 4) / CARD_SIZE_USIZE;
pub const INV_SLOTS_VERTICAL: usize = (SCREEN_HEIGHT_USIZE - 4) / CARD_SIZE_USIZE;

pub const LEFT: Vec2 = Vec2::new(-1.0, 0.0);
pub const SPREAD: f32 = 4.0_f32.to_radians();
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
