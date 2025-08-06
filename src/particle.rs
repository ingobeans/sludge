use macroquad::prelude::*;

use crate::{consts::SPRITE_SIZE, map::Spritesheet, ui};

#[derive(Clone)]
pub struct Particle {
    pub life: u8,
    pub lifetime: u8,
    pub function: &'static dyn Fn(&Particle, usize, usize, &Spritesheet, usize),
}

pub const EXPLOSION: Particle = Particle {
    life: 0,
    lifetime: 10,
    function: &|this, x, y, particles, scale_factor| {
        let frame_amt = 3;
        let anim_frame_offset = (this.life / (this.lifetime / frame_amt)) as usize * 2;
        for i in 0..2 {
            for j in 0..2 {
                particles.draw_tile(
                    scale_factor,
                    (x + j * SPRITE_SIZE).saturating_sub(SPRITE_SIZE),
                    (y + i * SPRITE_SIZE).saturating_sub(SPRITE_SIZE),
                    32 + anim_frame_offset + i * 32 + j,
                    false,
                );
            }
        }
        ui::draw_square(scale_factor, x, y, 1, 1);
    },
};
