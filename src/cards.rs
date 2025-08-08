use std::{collections::HashMap, fmt::Debug};

use macroquad::{color::Color, math::Vec2, shapes::draw_rectangle};

use crate::{consts::*, map::Spritesheet, particle::Particle};

mod library;

/// Returns all player-achievable cards
pub fn get_cards() -> Vec<Card> {
    let mut cards = vec![
        // modifiers
        library::aiming(),
        library::homing(),
        library::speed(),
        library::acidify(),
        // multidraw
        library::double(),
        library::triple(),
        // projectiles
        library::rocket(),
        library::bomb(),
        library::magicbolt(),
        library::bubble(),
        library::explosion(),
        library::fireball(),
        library::thorn_dart(),
        library::dart(),
        library::acid_flask(),
        library::razor(),
    ];

    let mut triggers = Vec::new();
    // generate trigger variants of projectile cards that allow it
    for card in &cards {
        if let CardType::Projectile(_, trigger_allowed) = card.ty {
            if trigger_allowed {
                let mut trigger = card.clone();
                trigger.is_trigger = true;
                triggers.push(trigger);
            }
        }
    }

    cards.append(&mut triggers);
    cards
}

#[derive(Clone)]
pub enum CardType {
    /// bool is whether projectile is allowed to be a trigger
    Projectile(Projectile, bool),
    Modifier(CardModifierData),
    /// usize is how many cards to draw.
    Multidraw(usize),
}
impl CardType {
    fn get_border_color(&self) -> Color {
        match self {
            Self::Projectile(_, _) => Color::from_hex(0x9e2835),
            Self::Modifier(_) => Color::from_hex(0x4f6781),
            Self::Multidraw(_) => Color::from_hex(0xafbfd2),
        }
    }
}
impl Default for CardType {
    fn default() -> Self {
        Self::Modifier(CardModifierData::default())
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum SpriteRotationMode {
    /// Fixed/no rotation
    None,
    /// Faces direction it is traveling
    Direction,
    /// It goes round and round
    Spin,
}

#[derive(Clone)]
pub enum ProjectileDrawType {
    Sprite(usize, SpriteRotationMode),
    Particle(Particle),
    None,
}
impl Default for ProjectileDrawType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Clone)]
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    /// How much larger hitbox the projectile has than usual
    pub extra_size: f32,
    /// Direction projectile is traveling
    pub direction: Vec2,
    /// Drag makes projectile slow down as it travels. 0.0 - 1.0, 1.0 makes it stop immediately after first frame
    pub drag: f32,
    pub draw_type: ProjectileDrawType,
    pub life: f32,
    /// Payload released on hit. Only used on trigger projectiles.
    pub payload: Vec<Card>,
    /// Released on death. Used by ex. the bomb exploding when its lifetime runs out.
    pub death_payload: Vec<Card>,
    pub modifier_data: CardModifierData,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DamageType {
    Magic,
    Pierce,
    Burn,
    Cold,
    Acid,
}
impl Default for DamageType {
    fn default() -> Self {
        Self::Magic
    }
}
#[derive(Clone, Default)]
pub struct FiringContext {
    pub spawn_list: Vec<Projectile>,
    pub modifier_data: CardModifierData,
}

#[derive(Clone, Default)]
pub struct CardModifierData {
    pub shoot_delay: f32,
    pub recharge_speed: f32,
    pub aim: bool,
    pub homing: bool,
    pub lifetime: f32,
    pub piercing: bool,
    pub speed: f32,
    pub damage: HashMap<DamageType, f32>,
}
impl CardModifierData {
    pub fn iter(&self) -> Vec<(&'static str, String)> {
        let mut fields = Vec::new();
        for (k, v) in &self.damage {
            let k = match k {
                DamageType::Acid => "damage acid",
                DamageType::Cold => "damage cold",
                DamageType::Burn => "damage burn",
                DamageType::Magic => "damage magic",
                DamageType::Pierce => "damage pierce",
            };
            fields.push((k, v.to_string()));
        }
        for (k, field) in [
            ("shoot delay", self.shoot_delay),
            ("reload time", self.recharge_speed),
            ("speed", self.speed),
        ] {
            if field != 0.0 {
                fields.push((k, field.to_string()));
            }
        }
        fields
    }
    /// Like [CardModifierData::merge] but only merges shoot_delay and recharge_speed fields, which are the only fields that
    /// projectile type cards modify
    pub fn merge_projectile(&mut self, other: &CardModifierData) {
        self.shoot_delay += other.shoot_delay;
        self.recharge_speed += other.recharge_speed;
    }
    pub fn merge(&mut self, other: &CardModifierData) {
        self.shoot_delay += other.shoot_delay;
        self.recharge_speed += other.recharge_speed;
        self.aim |= other.aim;
        self.homing |= other.homing;
        self.lifetime += other.lifetime;
        self.piercing |= other.piercing;
        self.speed += other.speed;
        for (k, v) in &other.damage {
            if let Some(amt) = self.damage.get_mut(k) {
                *amt += v;
            } else {
                self.damage.insert(k.clone(), *v);
            }
        }
    }
}

#[derive(Clone, Default)]
/// A card used by towers
pub struct Card {
    pub name: &'static str,
    pub desc: &'static str,
    pub ty: CardType,
    pub sprite: usize,
    pub is_trigger: bool,
}
impl Card {
    pub fn draw(&self, card_sheet: &Spritesheet, x: f32, y: f32) {
        draw_rectangle(
            x - 2.0,
            y - 2.0,
            SPRITE_SIZE + 4.0,
            SPRITE_SIZE + 4.0,
            self.ty.get_border_color(),
        );
        draw_rectangle(
            x - 1.0,
            y - 1.0,
            SPRITE_SIZE + 2.0,
            SPRITE_SIZE + 2.0,
            UI_BG_COLOR,
        );
        //draw_rectangle(
        //    x as f32,
        //    y as f32,
        //    SPRITE_SIZE as f32,
        //    SPRITE_SIZE as f32,
        //    UI_BG_COLOR,
        //);
        card_sheet.draw_tile(x, y, self.sprite, false, 0.0);
        if self.is_trigger {
            card_sheet.draw_tile(x, y, 32, false, 0.0);
        }
    }
}
