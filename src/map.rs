use std::fs::{read_dir, read_to_string};

use crate::consts::*;
use macroquad::prelude::*;

pub struct Spritesheet {
    texture: Texture2D,
    width: usize,
    height: usize,
}
impl Spritesheet {
    pub fn new(texture: Texture2D) -> Self {
        texture.set_filter(FilterMode::Nearest);
        let width = texture.width() as usize;
        let height = texture.height() as usize;
        Self {
            texture,
            width,
            height,
        }
    }
    pub fn id_to_pos(&self, id: usize) -> (usize, usize) {
        let x = id % (self.width / SPRITE_SIZE);
        let y = id / (self.height / SPRITE_SIZE);
        (x, y)
    }
    pub fn draw_tile(&self, x: usize, y: usize, id: usize, flipped: bool, rotation: f32) {
        let (texture_x, texture_y) = self.id_to_pos(id);
        let size = SPRITE_SIZE as f32;
        let params = DrawTextureParams {
            dest_size: Some(Vec2 { x: size, y: size }),
            source: Some(Rect {
                x: (texture_x * SPRITE_SIZE) as f32,
                y: (texture_y * SPRITE_SIZE) as f32,
                w: SPRITE_SIZE as f32,
                h: SPRITE_SIZE as f32,
            }),
            rotation,
            flip_x: flipped,
            flip_y: false,
            pivot: None,
        };
        draw_texture_ex(&self.texture, x as f32, y as f32, WHITE, params);
    }
    pub fn draw_tilemap(&self, map: &TileMap) {
        for y in 0..SCREEN_HEIGHT / SPRITE_SIZE {
            for x in 0..SCREEN_WIDTH / SPRITE_SIZE {
                let tile = map[y][x].checked_sub(1);
                if let Some(tile) = tile {
                    self.draw_tile(x * SPRITE_SIZE, y * SPRITE_SIZE, tile, false, 0.0);
                }
            }
        }
    }
}

pub type TileMap = [[usize; SCREEN_WIDTH / SPRITE_SIZE]; SCREEN_HEIGHT / SPRITE_SIZE];

#[derive(Debug)]
#[allow(dead_code)]
pub struct BadMapDataError(&'static str);

#[derive(Clone)]
pub struct Map {
    pub background: TileMap,
    pub obstructions: TileMap,
    pub points: Vec<(usize, usize)>,
    pub tower_spawnpoints: [(usize, usize); 4],
}
impl Map {
    pub fn is_unobstructed(&self, x: usize, y: usize) -> bool {
        // make size slightly smaller than sprite size so you can squeeze towers in slightly tighter spots
        let size = SPRITE_SIZE - 1;

        let top_left = (x, y);
        let top_right = (x + size, y);
        let bottom_left = (x, y + size);
        let bottom_right = (x + size, y + size);
        for (corner_x, corner_y) in [top_left, top_right, bottom_left, bottom_right] {
            if self.obstructions[corner_y / SPRITE_SIZE][corner_x / SPRITE_SIZE] != 0 {
                return false;
            }
            if self
                .points
                .contains(&(corner_x / SPRITE_SIZE, corner_y / SPRITE_SIZE))
            {
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
                65 => points[0] = (x * SPRITE_SIZE, y * SPRITE_SIZE),
                66 => points[1] = (x * SPRITE_SIZE, y * SPRITE_SIZE),
                97 => points[2] = (x * SPRITE_SIZE, y * SPRITE_SIZE),
                98 => points[3] = (x * SPRITE_SIZE, y * SPRITE_SIZE),
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
fn parse_points_from_tilemap(map: &TileMap) -> Vec<(usize, usize)> {
    let mut points = Vec::new();
    // find start
    let mut current_x = 0;
    let mut current_y = 0;
    'master: for y in 0..map.len() {
        for x in 0..map[0].len() {
            let tile = map[y][x];
            if tile == 33 {
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
                .min(map.len() as isize - 1) as usize;
            let x = ((current_x as isize) + dir.0)
                .max(0)
                .min(map[0].len() as isize - 1) as usize;
            if points.contains(&(x, y)) {
                continue;
            }
            if map[y][x] == 34 {
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
    let mut data: TileMap = [[0; SCREEN_WIDTH / SPRITE_SIZE]; SCREEN_HEIGHT / SPRITE_SIZE];
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
