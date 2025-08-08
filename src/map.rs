use std::fs::{read_dir, read_to_string};

use crate::consts::*;
use macroquad::prelude::*;

pub struct Spritesheet {
    texture: Texture2D,
    pub width: usize,
    pub height: usize,
    pub sprite_size: usize,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: usize) -> Self {
        texture.set_filter(FilterMode::Nearest);
        let width = texture.width() as usize;
        let height = texture.height() as usize;
        Self {
            texture,
            width,
            height,
            sprite_size,
        }
    }
    fn id_to_pos(&self, id: usize) -> (usize, usize) {
        let x = id % (self.width / self.sprite_size);
        let y = id / (self.height / self.sprite_size);
        (x, y)
    }
    pub fn draw_tile(&self, x: f32, y: f32, id: usize, flipped: bool, rotation: f32) {
        let (texture_x, texture_y) = self.id_to_pos(id);
        let size = self.sprite_size as f32;
        let params = DrawTextureParams {
            dest_size: Some(Vec2 { x: size, y: size }),
            source: Some(Rect {
                x: (texture_x * self.sprite_size) as f32,
                y: (texture_y * self.sprite_size) as f32,
                w: self.sprite_size as f32,
                h: self.sprite_size as f32,
            }),
            rotation,
            flip_x: flipped,
            flip_y: false,
            pivot: None,
        };
        draw_texture_ex(&self.texture, x as f32, y as f32, WHITE, params);
    }
    pub fn draw_tilemap(&self, map: &TileMap) {
        for y in 0..SCREEN_HEIGHT_USIZE / self.sprite_size {
            for x in 0..SCREEN_WIDTH_USIZE / self.sprite_size {
                let tile = map[y][x].checked_sub(1);
                let x = x as f32;
                let y = y as f32;
                if let Some(tile) = tile {
                    self.draw_tile(
                        x * self.sprite_size as f32,
                        y * self.sprite_size as f32,
                        tile,
                        false,
                        0.0,
                    );
                }
            }
        }
    }
}

pub type TileMap =
    [[usize; SCREEN_WIDTH_USIZE / SPRITE_SIZE_USIZE]; SCREEN_HEIGHT_USIZE / SPRITE_SIZE_USIZE];

#[derive(Debug)]
#[allow(dead_code)]
pub struct BadMapDataError(&'static str);

#[derive(Clone)]
pub struct Map {
    pub background: TileMap,
    pub obstructions: TileMap,
    pub points: Vec<(f32, f32)>,
    pub tower_spawnpoints: [(usize, usize); 4],
}
impl Map {
    pub fn is_unobstructed(&self, x: usize, y: usize) -> bool {
        // make size slightly smaller than sprite size so you can squeeze towers in slightly tighter spots
        let size = SPRITE_SIZE_USIZE - 1;

        let top_left = (x, y);
        let top_right = (x + size, y);
        let bottom_left = (x, y + size);
        let bottom_right = (x + size, y + size);
        for (corner_x, corner_y) in [top_left, top_right, bottom_left, bottom_right] {
            if self.obstructions[corner_y / SPRITE_SIZE_USIZE][corner_x / SPRITE_SIZE_USIZE] != 0 {
                return false;
            }
            if self.points.contains(&(
                (corner_x / SPRITE_SIZE_USIZE) as f32,
                (corner_y / SPRITE_SIZE_USIZE) as f32,
            )) {
                return false;
            }
        }

        true
    }
}

fn parse_spawnpoints_from_tilemap(map: &TileMap) -> [(usize, usize); 4] {
    let mut points = [(0, 0); 4];
    let mut found = 0;
    for y in 0..map.len() {
        for x in 0..map[0].len() {
            let tile = map[y][x];
            found += 1;
            match tile {
                65 => points[0] = (x * SPRITE_SIZE_USIZE, y * SPRITE_SIZE_USIZE),
                66 => points[1] = (x * SPRITE_SIZE_USIZE, y * SPRITE_SIZE_USIZE),
                97 => points[2] = (x * SPRITE_SIZE_USIZE, y * SPRITE_SIZE_USIZE),
                98 => points[3] = (x * SPRITE_SIZE_USIZE, y * SPRITE_SIZE_USIZE),
                _ => {
                    found -= 1;
                }
            }
            if found >= 4 {
                return points;
            }
        }
    }
    points
}

/// Parses an enemy path from a tilemap. Starts at tile with ID=33, and follows neighbouring ID=34 until stop.
fn parse_points_from_tilemap(map: &TileMap) -> Vec<(f32, f32)> {
    let mut points = Vec::new();
    // find start
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    'master: for y in 0..map.len() {
        for x in 0..map[0].len() {
            let tile = map[y][x];
            if tile == 33 {
                let x = x as f32;
                let y = y as f32;
                current_x = x;
                current_y = y;
                points.push((x, y));
                break 'master;
            }
        }
    }

    let neighbour_directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
    'master: loop {
        for dir in neighbour_directions {
            let y = ((current_y as isize) + dir.1)
                .max(0)
                .min(map.len() as isize - 1) as f32;
            let x = ((current_x as isize) + dir.0)
                .max(0)
                .min(map[0].len() as isize - 1) as f32;
            if points.contains(&(x, y)) {
                continue;
            }
            if map[y as usize][x as usize] == 34 {
                current_x = x;
                current_y = y;
                points.push((x, y));
                continue 'master;
            }
        }
        return points;
    }
}

pub fn parse_tilemap_layer(xml: &str, layer_name: &str) -> Result<TileMap, BadMapDataError> {
    let pattern = format!("name=\"{layer_name}\" ");
    let xml = xml
        .split_once(&pattern)
        .ok_or(BadMapDataError("layer not found"))?
        .1
        .split_once("<data encoding=\"csv\">")
        .ok_or(BadMapDataError("layer's data not found"))?
        .1
        .split_once("</data>")
        .ok_or(BadMapDataError("layer data corrupted"))?
        .0;
    let mut split = xml.split(',');
    let mut data: TileMap =
        [[0; SCREEN_WIDTH_USIZE / SPRITE_SIZE_USIZE]; SCREEN_HEIGHT_USIZE / SPRITE_SIZE_USIZE];
    for y in 0..data.len() {
        for x in 0..data[0].len() {
            data[y][x] = split
                .next()
                .ok_or(BadMapDataError("layer data too short!"))?
                .trim()
                .parse()
                .ok()
                .ok_or(BadMapDataError("layer data has invalid digit"))?
        }
    }
    Ok(data)
}

pub fn load_maps() -> Vec<Map> {
    let mut maps = Vec::new();
    for item in read_dir("data/maps")
        .expect("data/maps is missing!!")
        .flatten()
    {
        let data = read_to_string(item.path()).expect("failed to read map data :(");
        let background = parse_tilemap_layer(&data, "Background").expect("bad map data");
        let obstructions = parse_tilemap_layer(&data, "Obstructions").expect("bad map data");
        let path = parse_tilemap_layer(&data, "Path").expect("bad map data");

        let map = Map {
            background,
            obstructions,
            points: parse_points_from_tilemap(&path),
            tower_spawnpoints: parse_spawnpoints_from_tilemap(&path),
        };
        maps.push(map);
    }

    maps
}
