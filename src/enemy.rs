use std::sync::LazyLock;

use crate::cards::DamageType;

pub enum DamageResistance {
    None,
    Partial(DamageType),
    Full(DamageType),
}

/// Struct that holds information about an enemy type
pub struct EnemyType {
    pub name: &'static str,
    pub sprite: usize,
    pub anim_length: usize,
    pub size: usize,
    pub speed: f32,
    pub max_health: f32,
    pub damage_resistance: DamageResistance,
    /// If enemy should flip its sprite when moving to the left
    pub should_flip: bool,
}
impl Default for EnemyType {
    fn default() -> Self {
        Self {
            name: "",
            sprite: 0,
            anim_length: 1,
            size: 1,
            speed: 1.0,
            max_health: 5.0,
            damage_resistance: DamageResistance::None,
            should_flip: true,
        }
    }
}
/// A live instance of an enemy
pub struct Enemy {
    pub ty: &'static EnemyType,
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub next_path_point: usize,
    /// Tracks how far along the path this enemy has moved
    pub score: f32,
    /// Is enemy moving left?
    pub moving_left: bool,
}

pub static ENEMY_TYPES: LazyLock<Vec<EnemyType>> = LazyLock::new(|| {
    vec![
        EnemyType {
            name: "spider",
            sprite: 2 * 32,
            anim_length: 2,
            speed: 1.0,
            max_health: 3.0,
            damage_resistance: DamageResistance::None,
            ..Default::default()
        },
        EnemyType {
            name: "armored_spider",
            sprite: 2 * 32 + 2,
            anim_length: 2,
            speed: 0.5,
            max_health: 15.0,
            damage_resistance: DamageResistance::Full(DamageType::Pierce),
            ..Default::default()
        },
        EnemyType {
            name: "baby_spider",
            sprite: 2 * 32 + 4,
            anim_length: 2,
            speed: 2.0,
            max_health: 1.0,
            damage_resistance: DamageResistance::None,
            ..Default::default()
        },
        EnemyType {
            name: "spider_warrior",
            sprite: 2 * 32 + 6,
            anim_length: 2,
            speed: 1.0,
            max_health: 15.0,
            damage_resistance: DamageResistance::None,
            ..Default::default()
        },
        EnemyType {
            name: "big_spider",
            sprite: 6 * 32,
            anim_length: 2,
            speed: 1.0 / 4.0,
            max_health: 30.0,
            damage_resistance: DamageResistance::None,
            size: 2,
            ..Default::default()
        },
        EnemyType {
            name: "slime",
            sprite: 3 * 32,
            anim_length: 3,
            speed: 0.5,
            max_health: 6.0,
            damage_resistance: DamageResistance::Partial(DamageType::Pierce),
            ..Default::default()
        },
        EnemyType {
            name: "big_slime",
            sprite: 3 * 32 + 3,
            anim_length: 3,
            speed: 1.0 / 4.0,
            max_health: 32.0,
            damage_resistance: DamageResistance::Partial(DamageType::Pierce),
            ..Default::default()
        },
        EnemyType {
            name: "fire_slime",
            sprite: 3 * 32 + 6,
            anim_length: 3,
            speed: 0.5,
            max_health: 20.0,
            damage_resistance: DamageResistance::Partial(DamageType::Burn),
            ..Default::default()
        },
        EnemyType {
            name: "slime_car",
            sprite: 3 * 32 + 9,
            anim_length: 2,
            speed: 2.0,
            max_health: 6.0,
            damage_resistance: DamageResistance::Partial(DamageType::Pierce),
            ..Default::default()
        },
        EnemyType {
            name: "fire_cat",
            sprite: 4 * 32,
            anim_length: 2,
            speed: 1.0,
            max_health: 6.0,
            damage_resistance: DamageResistance::Full(DamageType::Burn),
            ..Default::default()
        },
        EnemyType {
            name: "fire_golem",
            sprite: 4 * 32 + 2,
            anim_length: 5,
            speed: 1.0 / 4.0,
            size: 2,
            max_health: 40.0,
            damage_resistance: DamageResistance::Partial(DamageType::Burn),
            ..Default::default()
        },
        EnemyType {
            name: "skeleton",
            sprite: 5 * 32,
            anim_length: 2,
            speed: 1.0,
            max_health: 6.0,
            damage_resistance: DamageResistance::None,
            ..Default::default()
        },
        EnemyType {
            name: "ice_slug",
            sprite: 8 * 32,
            anim_length: 2,
            speed: 1.0,
            max_health: 6.0,
            damage_resistance: DamageResistance::Full(DamageType::Cold),
            ..Default::default()
        },
        EnemyType {
            name: "snow_ball",
            sprite: 8 * 32 + 2,
            anim_length: 6,
            speed: 2.0,
            max_health: 6.0,
            damage_resistance: DamageResistance::Full(DamageType::Cold),
            ..Default::default()
        },
        EnemyType {
            name: "giga_ice_slug",
            sprite: 9 * 32,
            anim_length: 2,
            speed: 0.5,
            size: 2,
            max_health: 30.0,
            damage_resistance: DamageResistance::Full(DamageType::Cold),
            ..Default::default()
        },
        EnemyType {
            name: "cultist",
            sprite: 11 * 32,
            anim_length: 2,
            speed: 1.0,
            max_health: 15.0,
            damage_resistance: DamageResistance::Full(DamageType::Magic),
            ..Default::default()
        },
    ]
});
