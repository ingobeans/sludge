use crate::cards::DamageType;

pub enum DamageResistance {
    None,
    Partial(DamageType),
    Full(DamageType),
}

/// Struct that holds information about an enemy type
pub struct EnemyType {
    pub sprite: usize,
    pub anim_length: usize,
    pub speed: f32,
    pub health: f32,
    pub damage_resistance: DamageResistance,
    /// If enemy should flip its sprite when moving to the left
    pub should_flip: bool,
}
/// A live instance of an enemy
pub struct Enemy {
    pub ty: &'static EnemyType,
    pub x: f32,
    pub y: f32,
    pub next_path_point: usize,
    /// Tracks how far along the path this enemy has moved
    pub score: f32,
    /// Is enemy moving left?
    pub moving_left: bool,
}

pub const ENEMY_TYPES: &[EnemyType] = &[
    // spider
    EnemyType {
        sprite: 2 * 32,
        anim_length: 2,
        speed: 1.0,
        should_flip: true,
        health: 3.0,
        damage_resistance: DamageResistance::None,
    },
    // slime
    EnemyType {
        sprite: 3 * 32,
        anim_length: 3,
        speed: 1.0,
        should_flip: true,
        health: 6.0,
        damage_resistance: DamageResistance::Partial(DamageType::Pierce),
    },
];
