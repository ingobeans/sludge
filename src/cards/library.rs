use hashmap_macro::hashmap;

use crate::{cards::*, particle};

pub fn acidify() -> Card {
    Card {
        ty: CardType::Modifier(CardModifierData {
            damage: hashmap!(DamageType::Acid => 10),
            shoot_delay: 0.2,
            ..Default::default()
        }),
        sprite: 17,
        ..Default::default()
    }
}

pub fn bubble() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::BUBBLE),
        modifier_data: CardModifierData {
            speed: 0,
            lifetime: 10,
            piercing: true,
            shoot_delay: 0.05,
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(projectile, false),
        sprite: 9,
        ..Default::default()
    }
}

pub fn rocket() -> Card {
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
        ty: CardType::Projectile(explosion_projectile, false),
        sprite: 0,
        ..Default::default()
    };
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(2, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 3,
            lifetime: 40,
            shoot_delay: 0.85,
            ..Default::default()
        },
        inate_payload: vec![explosion],
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(projectile, false),
        sprite: 15,
        ..Default::default()
    }
}

pub fn double() -> Card {
    Card {
        ty: CardType::Multidraw(2),
        sprite: 5,
        ..Default::default()
    }
}

pub fn triple() -> Card {
    Card {
        ty: CardType::Multidraw(3),
        sprite: 6,
        ..Default::default()
    }
}

pub fn speed() -> Card {
    Card {
        ty: CardType::Modifier(CardModifierData {
            speed: 2,
            ..Default::default()
        }),
        sprite: 7,

        ..Default::default()
    }
}

pub fn aiming() -> Card {
    Card {
        ty: CardType::Modifier(CardModifierData {
            aim: true,
            ..Default::default()
        }),
        sprite: 0,
        ..Default::default()
    }
}

pub fn magicbolt() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(0, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 7,
            lifetime: 40,
            shoot_delay: 0.1,
            damage: hashmap!(DamageType::Magic => 3),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(projectile, true),
        sprite: 1,
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
        ty: CardType::Projectile(explosion_projectile, false),
        sprite: 0,
        ..Default::default()
    };
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(1, SpriteRotationMode::Spin),
        modifier_data: CardModifierData {
            speed: 1,
            lifetime: 40,
            shoot_delay: 0.85,
            ..Default::default()
        },
        death_payload: vec![explosion],
        ..Default::default()
    };

    Card {
        ty: CardType::Projectile(projectile, false),
        sprite: 3,
        ..Default::default()
    }
}
