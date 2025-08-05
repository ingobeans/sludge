use std::fs::{read_dir, read_to_string};

use macroquad::{miniquad::window::screen_size, prelude::*};

const SCREEN_WIDTH: usize = 192;
const SCREEN_HEIGHT: usize = 144;
const SPRITE_SIZE: usize = 8;

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
    fn draw_tile(&self, scale_factor: usize, x: usize, y: usize, id: usize, rotation: f32) {
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
    fn draw_map(&self, scale_factor: usize, map: &Map) {
        for y in 0..SCREEN_HEIGHT / SPRITE_SIZE {
            for x in 0..SCREEN_WIDTH / SPRITE_SIZE {
                let tile = map.data[y][x].checked_sub(1);
                if let Some(tile) = tile {
                    self.draw_tile(scale_factor, x * SPRITE_SIZE, y * SPRITE_SIZE, tile, 0.0);
                }
            }
        }
    }
}

struct Map {
    data: [[usize; SCREEN_WIDTH / SPRITE_SIZE]; SCREEN_HEIGHT / SPRITE_SIZE],
    points: Vec<(usize, usize)>,
}

fn parse_xml_to_map(
    xml: &str,
) -> [[usize; SCREEN_WIDTH / SPRITE_SIZE]; SCREEN_HEIGHT / SPRITE_SIZE] {
    let mut split = xml.split(',');
    let data = std::array::from_fn(|_| {
        std::array::from_fn(|_| split.next().unwrap().trim().parse().unwrap())
    });
    data
}

fn load_maps() -> Vec<Map> {
    let mut maps = Vec::new();
    for item in read_dir("tiled/maps")
        .expect("tiled/maps is missing!!")
        .flatten()
    {
        let data = read_to_string(item.path()).expect("failed to read map data :(");
        let data = data
            .split_once("<data encoding=\"csv\">")
            .expect("bad map data")
            .1
            .split_once("</data>")
            .expect("bad map data")
            .0;
        let data = parse_xml_to_map(data);
        let map = Map {
            data: data,
            points: Vec::new(),
        };
        maps.push(map);
    }

    maps
}

#[macroquad::main("sludge")]
async fn main() {
    let mut scale_factor;
    let spritesheet = Spritesheet::new(
        load_texture("spritesheet.png")
            .await
            .expect("spritesheet.png is missing!!"),
    );
    let maps = load_maps();

    loop {
        // update scale factor
        let (screen_width, screen_height) = screen_size();
        scale_factor =
            (screen_width as usize / SCREEN_WIDTH).min(screen_height as usize / SCREEN_HEIGHT);
        clear_background(BLACK);
        spritesheet.draw_map(scale_factor, &maps[0]);

        next_frame().await;
    }
}
