use hashmap_macro::hashmap;

use crate::{cards::*, particle};

pub fn aiming() -> Card {
    Card {
        ty: CardType::Modifier,
        sprite: 0,
        function: CardFunction::ModifyContext(&|ctx| {
            ctx.aim = true;
        }),
        ..Default::default()
    }
}

pub fn magicbolt() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(0),
        speed: 8,
        lifetime: 40,
        damage: hashmap!(DamageType::Magic => 3),
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(true),
        sprite: 1,
        function: CardFunction::SummonProjectile(projectile),
        ..Default::default()
    }
}

pub fn bomb() -> Card {
    let explosion_projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::EXPLOSION.clone()),
        speed: 0,
        lifetime: 0,
        damage: hashmap![DamageType::Explosion => 13],
        ..Default::default()
    };
    let explosion = Card {
        ty: CardType::Projectile(false),
        sprite: 0,
        function: CardFunction::SummonProjectile(explosion_projectile),
        ..Default::default()
    };
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(1),
        speed: 1,
        lifetime: 40,
        death_payload: vec![explosion],
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(false),
        sprite: 3,
        function: CardFunction::SummonProjectile(projectile),
        ..Default::default()
    }
}
