use crate::consts::*;

pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Clone, Copy)]
/// A card used by towers
pub struct Card {
    pub sprite: usize,
}

/// A user placed tower
pub struct Tower {
    pub x: usize,
    pub y: usize,
    pub sprite: usize,
    pub card_slots: Vec<Option<Card>>,
    pub card_index: usize,
    pub shoot_delay: f32,
    pub delay_counter: f32,
    pub direction: Direction,
}
