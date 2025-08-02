use macroquad::{miniquad::window::screen_size, prelude::*};

const SCREEN_WIDTH: u32 = 150;
const SCREEN_HEIGHT: u32 = 200;

struct Spritesheet {
    texture: Texture2D,
    width: usize,
    height: usize,
}
impl Spritesheet {
    fn new(texture: Texture2D) -> Self {
        texture.set_filter(FilterMode::Nearest);
        let width = texture.width() as usize;
        let height = texture.height() as usize;
        Self {
            texture,
            width,
            height,
        }
    }
    fn id_to_pos(&self, id: usize) -> (usize, usize) {
        let x = id % (self.width / SPRITE_SIZE);
        let y = id / (self.height / SPRITE_SIZE);
        (x, y)
    }
    fn draw_tile(&self, scale_factor: u32, x: usize, y: usize, id: usize, rotation: f32) {
        let (texture_x, texture_y) = self.id_to_pos(id);
        let size = SPRITE_SIZE as f32 * scale_factor as f32;
        let params = DrawTextureParams {
            dest_size: Some(Vec2 { x: size, y: size }),
            source: Some(Rect {
                x: (texture_x * SPRITE_SIZE) as f32,
                y: (texture_y * SPRITE_SIZE) as f32,
                w: SPRITE_SIZE as f32,
                h: SPRITE_SIZE as f32,
            }),
            rotation,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };
        draw_texture_ex(
            &self.texture,
            x as f32 * scale_factor as f32,
            y as f32 * scale_factor as f32,
            WHITE,
            params,
        );
    }
}

const SPRITE_SIZE: usize = 8;

#[macroquad::main("sludge")]
async fn main() {
    let mut scale_factor;
    let spritesheet = Spritesheet::new(
        load_texture("spritesheet.png")
            .await
            .expect("spritesheet.png is missing!!"),
    );
    loop {
        // update scale factor
        let (screen_width, screen_height) = screen_size();
        scale_factor =
            (screen_width as u32 / SCREEN_WIDTH).min(screen_height as u32 / SCREEN_HEIGHT);
        clear_background(BLACK);

        for y in 0..32 {
            for x in 0..32 {
                spritesheet.draw_tile(
                    scale_factor,
                    x * SPRITE_SIZE,
                    y * SPRITE_SIZE,
                    x + y * 32,
                    0.0,
                );
            }
        }

        next_frame().await;
    }
}
