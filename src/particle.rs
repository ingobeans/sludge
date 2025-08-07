use crate::{consts::SPRITE_SIZE, map::Spritesheet, ui};

#[derive(Clone)]
pub struct Particle {
    pub life: u8,
    pub lifetime: u8,
    pub function: &'static dyn Fn(&Particle, usize, usize, &Spritesheet),
}

pub const HIT_MARKER: Particle = Particle {
    life: 0,
    lifetime: 2,
    function: &|_, x, y, particles| {
        particles.draw_tile(x, y, 6, false, 0.0);
    },
};

pub const BUBBLE: Particle = Particle {
    life: 0,
    lifetime: 20,
    function: &|this, x, y, particles| {
        let stage_1_end = 10;
        let frame_amt = 3;
        let anim_frame_offset =
            (this.life.saturating_sub(stage_1_end) / (10 / frame_amt)) as usize * 2;

        let move_y = this.life.min(stage_1_end) * 2 / stage_1_end;

        particles.draw_tile(x, y - move_y as usize, 3 + anim_frame_offset, false, 0.0);
    },
};

pub const EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, x, y, particles| {
        let frame_amt = 3;
        let anim_frame_offset = (this.life / (this.lifetime / frame_amt)) as usize * 2;
        for i in 0..2 {
            for j in 0..2 {
                particles.draw_tile(
                    (x + j * SPRITE_SIZE).saturating_sub(SPRITE_SIZE),
                    (y + i * SPRITE_SIZE).saturating_sub(SPRITE_SIZE),
                    32 + anim_frame_offset + i * 32 + j,
                    false,
                    0.0,
                );
            }
        }
    },
};
