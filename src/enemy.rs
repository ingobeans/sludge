use crate::cards::DamageType;

#[derive(Clone, Copy)]
pub enum DamageResistance {
    None,
    Partial(DamageType),
    Full(DamageType),
}

const DEFAULT_ENEMY_TYPE: EnemyType = EnemyType {
    name: "",
    sprite: 0,
    anim_length: 1,
    size: 1,
    speed: 1.0,
    max_health: 5.0,
    anim_speed: 1.0,
    damage: 1,
    payload: EnemyPayload::None,
    damage_resistance: DamageResistance::None,
    should_flip: true,
};

#[derive(Clone, Copy)]
/// Struct that holds information about an enemy type
pub struct EnemyType {
    pub name: &'static str,
    pub sprite: usize,
    pub anim_length: usize,
    pub size: usize,
    pub speed: f32,
    pub max_health: f32,
    /// How many lives are lost when it finishes path
    pub damage: u8,
    pub anim_speed: f32,
    pub payload: EnemyPayload,
    pub damage_resistance: DamageResistance,
    /// If enemy should flip its sprite when moving to the left
    pub should_flip: bool,
}
impl EnemyType {
    /// Recursively calculates damage sum of self and children
    pub fn calc_damage(&self) -> u8 {
        let mut damage = self.damage;
        if let EnemyPayload::Some(enemy_type, amount) = self.payload {
            let child_damage = enemy_type.calc_damage();
            damage += child_damage * amount;
        }
        damage
    }
}
#[derive(Clone, Default)]
/// Stores the state of an enemy. Is passed to its children on death
pub struct EnemyState {
    /// Tracks how far along the path this enemy has moved
    pub score: f32,
    /// When an enemy is frozen, this value > 0, and ticks down
    pub freeze_frames: u8,
    /// When an enemy is stunned, this value > 0, and ticks down
    pub stun_frames: u8,
}
/// A live instance of an enemy
pub struct Enemy {
    pub ty: &'static EnemyType,
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub state: EnemyState,
    /// Is enemy moving left?
    pub moving_left: bool,
}
impl Enemy {
    pub fn new(ty: &'static EnemyType, x: f32, y: f32, state: EnemyState) -> Self {
        Self {
            ty,
            x,
            y,
            health: ty.max_health,
            state,
            moving_left: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum EnemyPayload {
    None,
    /// Type, amount
    Some(&'static EnemyType, u8),
}

static BABY_SPIDER: EnemyType = EnemyType {
    name: "baby_spider",
    sprite: 2 * 32 + 4,
    anim_length: 2,
    speed: 2.0,
    anim_speed: 1.6,
    max_health: 1.0,
    damage_resistance: DamageResistance::None,
    ..DEFAULT_ENEMY_TYPE
};
static FIRE_MITE: EnemyType = EnemyType {
    name: "fire_mite",
    sprite: 4 * 32 + 12,
    anim_length: 2,
    speed: 2.0,
    anim_speed: 1.6,
    max_health: 5.0,
    damage_resistance: DamageResistance::Full(DamageType::Burn),
    ..DEFAULT_ENEMY_TYPE
};

static HORSEY: EnemyType = EnemyType {
    name: "horsey",
    sprite: 6 * 32 + 8,
    anim_length: 2,
    speed: 1.2,
    anim_speed: 0.25,
    size: 2,
    damage: 1,
    max_health: 8.0,
    damage_resistance: DamageResistance::None,
    ..DEFAULT_ENEMY_TYPE
};
static SLIME: EnemyType = EnemyType {
    name: "slime",
    sprite: 3 * 32,
    anim_length: 3,
    damage: 2,
    speed: 0.75,
    max_health: 6.0,
    damage_resistance: DamageResistance::Partial(DamageType::Pierce),
    ..DEFAULT_ENEMY_TYPE
};
static SPIDER: EnemyType = EnemyType {
    name: "spider",
    sprite: 2 * 32,
    anim_length: 2,
    speed: 1.0,
    max_health: 3.0,
    damage_resistance: DamageResistance::None,
    ..DEFAULT_ENEMY_TYPE
};
pub static ENEMY_TYPES: &[EnemyType] = &[
    HORSEY,
    EnemyType {
        name: "horsey_rider",
        sprite: 6 * 32 + 4,
        speed: 1.0,
        damage: 5,
        max_health: 15.0,
        anim_length: 2,
        payload: EnemyPayload::Some(&HORSEY, 1),
        damage_resistance: DamageResistance::Full(DamageType::Pierce),
        ..HORSEY
    },
    SPIDER,
    EnemyType {
        name: "armored_spider",
        sprite: 2 * 32 + 2,
        anim_length: 2,
        damage: 9,
        speed: 0.5,
        max_health: 34.0,
        damage_resistance: DamageResistance::Full(DamageType::Pierce),
        payload: EnemyPayload::Some(&SPIDER, 1),
        ..DEFAULT_ENEMY_TYPE
    },
    BABY_SPIDER,
    EnemyType {
        name: "spider_warrior",
        sprite: 2 * 32 + 6,
        anim_length: 2,
        damage: 10,
        speed: 1.0,
        max_health: 20.0,
        damage_resistance: DamageResistance::None,
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "big_spider",
        sprite: 6 * 32,
        anim_length: 2,
        speed: 1.0 / 4.0,
        max_health: 30.0,
        damage_resistance: DamageResistance::None,
        payload: EnemyPayload::Some(&BABY_SPIDER, 4),
        size: 2,
        ..DEFAULT_ENEMY_TYPE
    },
    SLIME,
    EnemyType {
        name: "big_slime",
        sprite: 3 * 32 + 3,
        anim_length: 3,
        damage: 2,
        speed: 0.4,
        max_health: 32.0,
        damage_resistance: DamageResistance::Partial(DamageType::Pierce),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "fire_slime",
        sprite: 3 * 32 + 6,
        anim_length: 3,
        damage: 2,
        speed: 0.6,
        max_health: 24.0,
        damage_resistance: DamageResistance::Partial(DamageType::Burn),
        payload: EnemyPayload::Some(&SLIME, 1),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "slime_car",
        sprite: 3 * 32 + 9,
        anim_length: 2,
        damage: 2,
        speed: 2.0,
        anim_speed: 1.6,
        max_health: 6.0,
        damage_resistance: DamageResistance::Partial(DamageType::Pierce),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "fire_cat",
        sprite: 4 * 32,
        anim_length: 2,
        speed: 1.0,
        max_health: 5.0,
        damage_resistance: DamageResistance::Full(DamageType::Burn),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "fire_golem",
        sprite: 4 * 32 + 2,
        anim_length: 5,
        damage: 15,
        speed: 1.0 / 4.0,
        size: 2,
        max_health: 65.0,
        damage_resistance: DamageResistance::Partial(DamageType::Burn),
        payload: EnemyPayload::Some(&FIRE_MITE, 15),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "skeleton",
        sprite: 5 * 32,
        anim_length: 2,
        anim_speed: 0.8,
        speed: 1.7,
        max_health: 6.0,
        damage_resistance: DamageResistance::None,
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "ice_slug",
        sprite: 8 * 32,
        anim_length: 2,
        speed: 1.0,
        max_health: 15.0,
        damage_resistance: DamageResistance::Full(DamageType::Cold),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "snow_ball",
        sprite: 8 * 32 + 2,
        anim_length: 6,
        speed: 2.0,
        anim_speed: 1.5,
        max_health: 13.0,
        damage_resistance: DamageResistance::Full(DamageType::Cold),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "giga_ice_slug",
        sprite: 9 * 32,
        anim_length: 2,
        speed: 0.42,
        damage: 15,
        size: 2,
        max_health: 77.0,
        damage_resistance: DamageResistance::Full(DamageType::Cold),
        ..DEFAULT_ENEMY_TYPE
    },
    EnemyType {
        name: "cultist",
        sprite: 11 * 32,
        anim_length: 2,
        speed: 1.0,
        damage: 2,
        max_health: 20.0,
        damage_resistance: DamageResistance::Full(DamageType::Magic),
        ..DEFAULT_ENEMY_TYPE
    },
];
