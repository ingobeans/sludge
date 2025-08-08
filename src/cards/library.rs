use hashmap_macro::hashmap;

use crate::{cards::*, particle};

pub fn acidify() -> Card {
    Card {
        name: "acidify",
        desc: "adds acid damage",
        ty: CardType::Modifier(CardModifierData {
            damage: hashmap!(DamageType::Acid => 6.0),
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
            speed: 0.0,
            lifetime: 10.0,
            piercing: true,
            shoot_delay: 0.05,
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "bubble",
        desc: "harmless bubble",
        ty: CardType::Projectile(projectile, false),
        sprite: 9,
        ..Default::default()
    }
}
pub fn double() -> Card {
    Card {
        name: "double draw",
        desc: "fires next two\nprojectiles",
        ty: CardType::Multidraw(2),
        sprite: 5,
        ..Default::default()
    }
}

pub fn triple() -> Card {
    Card {
        name: "triple draw",
        desc: "fires next three\nprojectiles",
        ty: CardType::Multidraw(3),
        sprite: 6,
        ..Default::default()
    }
}

pub fn speed() -> Card {
    Card {
        name: "speedify",
        desc: "speeds a proj up",
        ty: CardType::Modifier(CardModifierData {
            speed: 2.0,
            ..Default::default()
        }),
        sprite: 7,

        ..Default::default()
    }
}

pub fn aiming() -> Card {
    Card {
        name: "aiming",
        desc: "aim towards nearest",
        ty: CardType::Modifier(CardModifierData {
            aim: true,
            ..Default::default()
        }),
        sprite: 0,
        ..Default::default()
    }
}

pub fn homing() -> Card {
    Card {
        name: "homing",
        desc: "home towards nearest",
        ty: CardType::Modifier(CardModifierData {
            homing: true,
            ..Default::default()
        }),
        sprite: 8,
        ..Default::default()
    }
}

pub fn magicbolt() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(0, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 7.0,
            lifetime: 60.0,
            shoot_delay: 0.1,
            damage: hashmap!(DamageType::Magic => 3.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "magicbolt",
        desc: "basic projectile",
        ty: CardType::Projectile(projectile, true),
        sprite: 1,
        ..Default::default()
    }
}

pub fn thorn_dart() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(7, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 7.0,
            lifetime: 60.0,
            shoot_delay: 0.5,
            damage: hashmap!(DamageType::Pierce => 3.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let main = Card {
        name: "thorn dart",
        desc: "pierces first enemy",
        ty: CardType::Projectile(projectile, false),
        sprite: 4,
        ..Default::default()
    };
    // make card cast one of itself as a payload.
    // this is what makes it able to cut through two enemies
    let mut card = main.clone();
    if let CardType::Projectile(proj, _) = &mut card.ty {
        proj.payload = vec![main];
    }
    card
}
pub fn dart() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(9, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 5.0,
            lifetime: 40.0,
            shoot_delay: 0.16,
            damage: hashmap!(DamageType::Pierce => 4.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "dart",
        desc: "regular dart",
        ty: CardType::Projectile(projectile, true),
        sprite: 10,
        ..Default::default()
    }
}
pub fn razor() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(10, SpriteRotationMode::Spin),
        modifier_data: CardModifierData {
            speed: 3.0,
            lifetime: 70.0,
            shoot_delay: 0.66,
            damage: hashmap!(DamageType::Pierce => 12.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "razor",
        desc: "sharp razor disc",
        ty: CardType::Projectile(projectile, false),
        sprite: 16,
        ..Default::default()
    }
}
fn fire_explosion() -> Card {
    let explosion_projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::FIRE_EXPLOSION),
        extra_size: SPRITE_SIZE,
        modifier_data: CardModifierData {
            speed: 0.0,
            lifetime: 1.0,
            piercing: true,
            damage: hashmap!(DamageType::Burn => 6.0),
            ..Default::default()
        },
        ..Default::default()
    };
    Card {
        ty: CardType::Projectile(explosion_projectile, false),
        sprite: 1,
        ..Default::default()
    }
}
pub fn fireball() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::FIREBALL),
        modifier_data: CardModifierData {
            speed: 3.0,
            lifetime: 120.0,
            shoot_delay: 1.1,
            damage: hashmap!(DamageType::Burn => 1.0),
            ..Default::default()
        },
        payload: vec![fire_explosion()],
        ..Default::default()
    };

    Card {
        name: "fireball",
        desc: "burning fire",
        ty: CardType::Projectile(projectile, false),
        sprite: 11,
        ..Default::default()
    }
}

pub fn explosion() -> Card {
    let explosion_projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::EXPLOSION),
        extra_size: SPRITE_SIZE,
        modifier_data: CardModifierData {
            speed: 0.0,
            lifetime: 0.0,
            shoot_delay: 1.0,
            piercing: true,
            damage: hashmap!(DamageType::Burn => 13.0),
            ..Default::default()
        },
        ..Default::default()
    };
    Card {
        name: "explosion",
        desc: "instant explosion",
        ty: CardType::Projectile(explosion_projectile, false),
        sprite: 13,
        ..Default::default()
    }
}

pub fn bomb() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(1, SpriteRotationMode::Spin),
        drag: 0.07,
        modifier_data: CardModifierData {
            speed: 2.0,
            lifetime: 40.0,
            piercing: true,
            shoot_delay: 0.85,
            ..Default::default()
        },
        death_payload: vec![explosion()],
        ..Default::default()
    };

    Card {
        name: "bomb",
        desc: "goes boom",
        ty: CardType::Projectile(projectile, false),
        sprite: 3,
        ..Default::default()
    }
}

pub fn rocket() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(2, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 3.0,
            lifetime: 40.0,
            shoot_delay: 0.85,
            ..Default::default()
        },
        payload: vec![explosion()],
        ..Default::default()
    };

    Card {
        name: "rocket",
        desc: "boom on impact",
        ty: CardType::Projectile(projectile, false),
        sprite: 15,
        ..Default::default()
    }
}

fn acid_puddle() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::ACID_PUDDLE),
        extra_size: SPRITE_SIZE,
        modifier_data: CardModifierData {
            speed: 0.0,
            lifetime: 30.0,
            piercing: true,
            damage: hashmap!(DamageType::Acid => 0.5),
            ..Default::default()
        },
        ..Default::default()
    };
    Card {
        ty: CardType::Projectile(projectile, false),
        sprite: 1,
        ..Default::default()
    }
}

pub fn acid_flask() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(8, SpriteRotationMode::Spin),
        modifier_data: CardModifierData {
            speed: 3.5,
            lifetime: 60.0,
            shoot_delay: 0.85,
            ..Default::default()
        },
        payload: vec![acid_puddle()],
        ..Default::default()
    };

    Card {
        name: "acid flask",
        desc: "hurl at your foes",
        ty: CardType::Projectile(projectile, false),
        sprite: 12,
        ..Default::default()
    }
}
