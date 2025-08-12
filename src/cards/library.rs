use hashmap_macro::hashmap;

use crate::{cards::*, particle};

pub fn as_trigger(mut card: Card) -> Card {
    card.is_trigger = true;
    card.tier += 1;
    card
}

pub fn shotgun() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::SHOTGUN),
        hit_sound: ProjectileSound::Hit,
        clones_amount: 2,
        drag: 0.01,
        modifier_data: CardModifierData {
            speed: 8.0,
            lifetime: 60.0,
            recharge_speed: 0.65,
            damage: hashmap!(DamageType::Pierce => 4.0),
            spread: 5.0_f32.to_radians(),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "shotgun",
        desc: "triple barrel",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 33,
        ..Default::default()
    }
}
pub fn shock() -> Card {
    Card {
        name: "shock",
        desc: "makes projectile\nbriefly stun enemies",
        tier: 1,
        ty: CardType::Modifier(CardModifierData {
            stuns: 7,
            ..Default::default()
        }),
        sprite: 32,
        ..Default::default()
    }
}
pub fn lightning() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::LIGHTNING),
        only_enemy_triggers: true,
        modifier_data: CardModifierData {
            speed: 8.0,
            lifetime: 3.0,
            shoot_delay: 0.70,
            spread: -100.0,
            aim: true,
            damage: hashmap!(DamageType::Magic => 4.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let card = Card {
        name: "lightning",
        desc: "zaps that chain",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 31,
        ..Default::default()
    };
    // basically make 3 inner copies of the spell, such that the lightning chains thrice.
    let mut last = card;
    for _ in 0..3 {
        let old = last;
        last = old.clone();

        if let CardType::Projectile(proj, _) = &mut last.ty {
            proj.payload = vec![old];
        }
    }
    // make the very last projectile NOT have aiming or negative spread
    if let CardType::Projectile(proj, _) = &mut last.ty {
        proj.modifier_data.aim = false;
        proj.modifier_data.spread = 0.0;
        proj.modifier_data.recharge_speed = 0.6;
    }
    last
}
pub fn hammer() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(16, SpriteRotationMode::Spin),
        hit_sound: ProjectileSound::Hit,
        drag: 0.02,
        modifier_data: CardModifierData {
            speed: 3.0,
            lifetime: 60.0,
            shoot_delay: 0.85,
            recharge_speed: -0.25,
            damage: hashmap!(DamageType::Pierce => 8.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "hammer",
        desc: "throws a hammer",
        tier: 1,
        ty: CardType::Projectile(projectile, false),
        sprite: 30,
        ..Default::default()
    }
}
pub fn ghost_shot() -> Card {
    Card {
        name: "ghost shot",
        desc: "lets proj go\nthrough walls",
        ty: CardType::Modifier(CardModifierData {
            ghost: true,
            ..Default::default()
        }),
        sprite: 29,
        tier: 2,
        ..Default::default()
    }
}
pub fn scatter() -> Card {
    Card {
        name: "scatter",
        desc: "fast but inaccurate",
        ty: CardType::Modifier(CardModifierData {
            spread: 40.0_f32.to_radians(),
            shoot_delay: -0.2,
            recharge_speed: -0.3,
            ..Default::default()
        }),
        tier: 0,
        sprite: 28,
        ..Default::default()
    }
}
pub fn high_precision() -> Card {
    Card {
        name: "high precision",
        desc: "reduces spread",
        ty: CardType::Modifier(CardModifierData {
            spread: -40.0_f32.to_radians(),
            ..Default::default()
        }),
        tier: 1,
        sprite: 27,
        ..Default::default()
    }
}
pub fn playing_card() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(15, SpriteRotationMode::Spin),
        random_damage: Some((0, 10)),
        hit_sound: ProjectileSound::Hit,
        modifier_data: CardModifierData {
            speed: 4.0,
            lifetime: 30.0,
            shoot_delay: 0.15,
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "playing card",
        desc: "random dmg 0-10",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 26,
        ..Default::default()
    }
}
pub fn sunbeam() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::SUNBEAM),
        straight: true,
        modifier_data: CardModifierData {
            speed: 8.0,
            lifetime: 3.0,
            shoot_delay: -0.15,
            recharge_speed: -0.25,
            piercing: true,
            damage: hashmap!(DamageType::Burn => 0.5),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "sunbeam",
        desc: "a bright beam",
        tier: 1,
        ty: CardType::Projectile(projectile, false),
        sprite: 2,
        ..Default::default()
    }
}
pub fn death_ray() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::DEATH_RAY),
        straight: true,
        modifier_data: CardModifierData {
            speed: 8.0,
            lifetime: 3.0,
            shoot_delay: 0.85,
            recharge_speed: 0.65,
            piercing: true,
            damage: hashmap!(DamageType::Magic => 25.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "death ray",
        desc: "magic beam of death",
        tier: 1,
        ty: CardType::Projectile(projectile, false),
        sprite: 21,
        ..Default::default()
    }
}
pub fn freeze_ray() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::FREEZE_RAY),
        straight: true,
        modifier_data: CardModifierData {
            speed: 8.0,
            lifetime: 3.0,
            shoot_delay: 0.45,
            recharge_speed: 0.2,
            piercing: true,
            damage: hashmap!(DamageType::Cold => 8.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "freeze ray",
        desc: "a really cold ray",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 20,
        ..Default::default()
    }
}
pub fn supercharge() -> Card {
    Card {
        name: "supercharge",
        desc: "makes tower faster",
        tier: 1,
        ty: CardType::Modifier(CardModifierData {
            shoot_delay: -0.3,
            recharge_speed: -0.11,
            ..Default::default()
        }),
        sprite: 25,
        ..Default::default()
    }
}
pub fn road_thorns() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(13, SpriteRotationMode::None),
        drag: 0.15,
        hit_sound: ProjectileSound::Hit,
        modifier_data: CardModifierData {
            speed: 1.5,
            lifetime: -1.0,
            shoot_delay: 1.0,
            damage: hashmap!(DamageType::Pierce => 12.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "road thorns",
        desc: "put thorns on path",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 22,
        ..Default::default()
    }
}
pub fn icecicle() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(12, SpriteRotationMode::Direction),
        hit_sound: ProjectileSound::Hit,
        modifier_data: CardModifierData {
            speed: 5.0,
            lifetime: 60.0,
            shoot_delay: 0.5,
            damage: hashmap!(DamageType::Cold => 5.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "icecicle",
        desc: "shoot an icecicle",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 19,
        ..Default::default()
    }
}

pub fn freezeify() -> Card {
    Card {
        name: "freezeify",
        desc: "adds extra cold dmg",
        tier: 2,
        ty: CardType::Modifier(CardModifierData {
            damage: hashmap!(DamageType::Cold => 2.0),
            shoot_delay: 0.4,
            ..Default::default()
        }),
        sprite: 18,
        ..Default::default()
    }
}

pub fn acidify() -> Card {
    Card {
        name: "acidify",
        desc: "adds extra acid dmg",
        tier: 3,
        ty: CardType::Modifier(CardModifierData {
            damage: hashmap!(DamageType::Acid => 2.0),
            shoot_delay: 0.4,
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
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 9,
        ..Default::default()
    }
}
pub fn double() -> Card {
    Card {
        name: "double draw",
        desc: "fires next two\nprojectiles",
        tier: 0,
        ty: CardType::Multidraw(2),
        sprite: 5,
        ..Default::default()
    }
}

pub fn triple() -> Card {
    Card {
        name: "triple draw",
        desc: "fires next three\nprojectiles",
        tier: 0,
        ty: CardType::Multidraw(3),
        sprite: 6,
        ..Default::default()
    }
}

pub fn speed() -> Card {
    Card {
        name: "speedify",
        desc: "speeds a proj up",
        tier: 0,
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
        desc: "makes projectile\naim towards nearest\nenemy",
        tier: 0,
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
        desc: "makes projectile\nhome towards nearest\nenemy",
        tier: 1,
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
        hit_sound: ProjectileSound::Hit,
        modifier_data: CardModifierData {
            speed: 7.0,
            lifetime: 30.0,
            shoot_delay: 0.25,
            damage: hashmap!(DamageType::Magic => 3.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "magicbolt",
        desc: "basic projectile",
        tier: 0,
        ty: CardType::Projectile(projectile, true),
        sprite: 1,
        ..Default::default()
    }
}

pub fn thorn_dart() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(7, SpriteRotationMode::Direction),
        hit_sound: ProjectileSound::Hit,
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
        tier: 0,
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
        hit_sound: ProjectileSound::Hit,
        modifier_data: CardModifierData {
            speed: 5.0,
            lifetime: 40.0,
            shoot_delay: 0.34,
            damage: hashmap!(DamageType::Pierce => 4.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "dart",
        desc: "regular dart",
        tier: 0,
        ty: CardType::Projectile(projectile, true),
        sprite: 10,
        ..Default::default()
    }
}
pub fn razor() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(10, SpriteRotationMode::Spin),
        hit_sound: ProjectileSound::Hit,
        drag: 0.01,
        modifier_data: CardModifierData {
            speed: 3.0,
            lifetime: 70.0,
            shoot_delay: 0.85,
            damage: hashmap!(DamageType::Pierce => 12.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "razor",
        desc: "sharp razor disc",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 16,
        ..Default::default()
    }
}
fn fire_explosion() -> Card {
    let explosion_projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::FIRE_EXPLOSION),
        extra_size: SPRITE_SIZE,
        fire_sound: ProjectileSound::Explosion,
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
            shoot_delay: 1.15,
            damage: hashmap!(DamageType::Burn => 1.0),
            ..Default::default()
        },
        payload: vec![fire_explosion()],
        ..Default::default()
    };

    Card {
        name: "fireball",
        desc: "burning fire",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 11,
        ..Default::default()
    }
}

pub fn explosion() -> Card {
    let explosion_projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::EXPLOSION),
        extra_size: SPRITE_SIZE,
        fire_sound: ProjectileSound::Explosion,
        modifier_data: CardModifierData {
            speed: 0.0,
            lifetime: 0.0,
            shoot_delay: 1.15,
            piercing: true,
            damage: hashmap!(DamageType::Burn => 13.0),
            ..Default::default()
        },
        ..Default::default()
    };
    Card {
        name: "explosion",
        desc: "instant explosion",
        tier: 1,
        ty: CardType::Projectile(explosion_projectile, false),
        sprite: 13,
        ..Default::default()
    }
}

pub fn stun_explosion() -> Card {
    let explosion_projectile = Projectile {
        draw_type: ProjectileDrawType::Particle(particle::STUN_EXPLOSION),
        extra_size: SPRITE_SIZE,
        fire_sound: ProjectileSound::Explosion,
        modifier_data: CardModifierData {
            stuns: 20,
            speed: 0.0,
            lifetime: 0.0,
            shoot_delay: 0.85,
            piercing: true,
            damage: hashmap!(DamageType::Burn => 5.0),
            ..Default::default()
        },
        ..Default::default()
    };
    Card {
        name: "stun explosion",
        desc: "akin to a flashbang",
        tier: 1,
        ty: CardType::Projectile(explosion_projectile, false),
        sprite: 14,
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
            anti_piercing: true,
            shoot_delay: 0.85,
            ..Default::default()
        },
        death_payload: vec![explosion()],
        ..Default::default()
    };

    Card {
        name: "bomb",
        desc: "goes boom",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 3,
        ..Default::default()
    }
}
pub fn banana() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(14, SpriteRotationMode::Spin),
        drag: 0.07,
        modifier_data: CardModifierData {
            stuns: 20,
            speed: 2.0,
            lifetime: 40.0,
            shoot_delay: 0.85,
            damage: hashmap!(DamageType::Pierce => 3.0),
            ..Default::default()
        },
        ..Default::default()
    };

    Card {
        name: "banana peel",
        desc: "stuns enemies",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 23,
        ..Default::default()
    }
}

pub fn rocket() -> Card {
    let projectile = Projectile {
        draw_type: ProjectileDrawType::Sprite(2, SpriteRotationMode::Direction),
        modifier_data: CardModifierData {
            speed: 3.0,
            lifetime: 40.0,
            shoot_delay: 0.9,
            ..Default::default()
        },
        payload: vec![explosion()],
        ..Default::default()
    };

    Card {
        name: "rocket",
        desc: "boom on impact",
        tier: 0,
        ty: CardType::Projectile(projectile, false),
        sprite: 15,
        ..Default::default()
    }
}

pub fn piercing() -> Card {
    Card {
        name: "piercing",
        desc: "proj pierces enemies",
        tier: 3,
        sprite: 24,
        ty: CardType::Modifier(CardModifierData {
            piercing: true,
            shoot_delay: 0.25,
            ..Default::default()
        }),
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
            damage: hashmap!(DamageType::Acid => 0.9),
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
        drag: 0.05,
        hit_sound: ProjectileSound::Hit,
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
        tier: 1,
        ty: CardType::Projectile(projectile, false),
        sprite: 12,
        ..Default::default()
    }
}
