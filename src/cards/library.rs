use hashmap_macro::hashmap;

use crate::{cards::*, particle};

pub fn speed() -> Card {
    Card {
        ty: CardType::Modifier,
        sprite: 7,
        modifier_data: CardModifierData {
            speed: 2,
            ..Default::default()
        },

        ..Default::default()
    }
}

pub fn aiming() -> Card {
    Card {
        ty: CardType::Modifier,
        sprite: 0,
        modifier_data: CardModifierData {
            aim: true,
            ..Default::default()
        },

        ..Default::default()
    }
}

pub fn magicbolt() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(0, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 8,
            lifetime: 40,
            damage: hashmap!(DamageType::Magic => 3),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(true),
        sprite: 1,
        projectile: Some(projectile),
        ..Default::default()
    }
}

pub fn bomb() -> Card {
    let explosion_projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::EXPLOSION.clone()),
        modifier_data: CardModifierData {
            speed: 0,
            lifetime: 0,
            damage: hashmap!(DamageType::Explosion => 13),
            ..Default::default()
        },
        ..Default::default()
    };
    let explosion = Card {
        ty: CardType::Projectile(false),
        sprite: 0,
        projectile: Some(explosion_projectile),
        ..Default::default()
    };
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(1, SpriteRotationMode::Spin),
        modifier_data: CardModifierData {
            speed: 1,
            lifetime: 40,
            ..Default::default()
        },
        death_payload: vec![explosion],
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(false),
        sprite: 3,
        projectile: Some(projectile),
        ..Default::default()
    }
}
