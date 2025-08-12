use crate::consts::*;
use macroquad::prelude::*;

#[derive(Clone)]
pub struct Spritesheet {
    texture: Texture2D,
    pub width: usize,
    pub sprite_size: usize,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: usize) -> Self {
        texture.set_filter(FilterMode::Nearest);
        let width = texture.width() as usize;
        Self {
            texture,
            width,
            sprite_size,
        }
    }
    fn id_to_pos(&self, id: usize) -> (usize, usize) {
        let x = id % (self.width / self.sprite_size);
        let y = id / (self.width / self.sprite_size);
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
        draw_texture_ex(&self.texture, x, y, WHITE, params);
    }
    pub fn draw_tilemap(&self, map: &TileMap) {
        for (y, row) in map.iter().enumerate() {
            for (x, value) in row.iter().enumerate() {
                let tile = value.checked_sub(1);
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

pub type TileMap = Vec<[usize; SCREEN_WIDTH_USIZE / SPRITE_SIZE_USIZE]>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct BadMapDataError(&'static str);

#[derive(Clone)]
pub struct Map {
    pub name: String,
    pub background: TileMap,
    pub obstructions: TileMap,
    pub out_of_bounds: TileMap,
    pub path: TileMap,
    pub points: Vec<(f32, f32)>,
    pub tower_spawnpoints: [(usize, usize); 4],
}
impl Map {
    pub fn draw_preview(&self, x: f32, y: f32, old_camera: &Camera2D, tileset: &Spritesheet) {
        let render_target = render_target(PREVIEW_WIDTH as u32, PREVIEW_HEIGHT as u32);
        render_target.texture.set_filter(FilterMode::Nearest);
        let preview_camera = Camera2D {
            render_target: Some(render_target),
            zoom: Vec2::new(1.0 / SCREEN_WIDTH * 2.0, 1.0 / SCREEN_HEIGHT * 2.0),
            target: Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),

            ..Default::default()
        };
        set_camera(&preview_camera);
        tileset.draw_tilemap(&self.background);
        tileset.draw_tilemap(&self.out_of_bounds);
        tileset.draw_tilemap(&self.obstructions);
        set_camera(old_camera);
        draw_texture(&preview_camera.render_target.unwrap().texture, x, y, WHITE);
    }
    pub fn is_unobstructed(&self, x: usize, y: usize) -> bool {
        let x = x + SPRITE_SIZE_USIZE / 2;
        let y = y + SPRITE_SIZE_USIZE / 2 + 2;
        if self.obstructions[y / SPRITE_SIZE_USIZE][x / SPRITE_SIZE_USIZE] != 0 {
            return false;
        }
        if self.out_of_bounds[y / SPRITE_SIZE_USIZE][x / SPRITE_SIZE_USIZE] != 0 {
            return false;
        }
        if self.path[y / SPRITE_SIZE_USIZE][x / SPRITE_SIZE_USIZE] != 0 {
            return false;
        }

        true
    }
    pub fn get_pos_along_path(&self, score: f32) -> Option<(f32, f32)> {
        let tiled = score / SPRITE_SIZE;
        let lower = (score / SPRITE_SIZE).floor();
        let max = self.points.len() as f32;
        if lower >= max {
            return None;
        }

        let lower_pos = self.points[lower as usize];
        if lower == tiled {
            return Some(lower_pos);
        }
        let upper = lower + 1.0;
        if upper >= max {
            return None;
        }
        let factor = tiled - lower;
        let upper_pos = self.points[upper as usize];
        let mut new_pos = lower_pos;
        let move_amount = factor;
        if new_pos.0 < upper_pos.0 {
            new_pos.0 += move_amount;
        } else if new_pos.0 > upper_pos.0 {
            new_pos.0 -= move_amount;
        }
        if new_pos.1 < upper_pos.1 {
            new_pos.1 += move_amount;
        } else if new_pos.1 > upper_pos.1 {
            new_pos.1 -= move_amount;
        }

        Some(new_pos)
    }
}

pub fn parse_spawnpoints_from_tilemap(map: &TileMap) -> [(usize, usize); 4] {
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
pub fn parse_points_from_tilemap(map: &TileMap) -> Vec<(f32, f32)> {
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
                break 'master;
            }
        }
    }
    let start_x = current_x;
    let start_y = current_y;

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
                // if the first point after the start, then calc the direction to start,
                // to shift the actual spawn point one tile offscreen
                if points.is_empty() {
                    let dir_x = start_x - x;
                    let dir_y = start_y - y;
                    let new_x = start_x + dir_x;
                    let new_y = start_y + dir_y;
                    points.push((new_x, new_y));
                    points.push((start_x, start_y));
                }
                current_x = x;
                current_y = y;
                points.push((x, y));
                continue 'master;
            }
        }
        // calc direction between last and second to last point
        // to 'extrapolate' a new final point, thats one tile offscreen
        let (prev_x, prev_y) = points[points.len() - 2];
        let dir_x = current_x - prev_x;
        let dir_y = current_y - prev_y;
        let new_x = current_x + dir_x;
        let new_y = current_y + dir_y;

        points.push((new_x, new_y));

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
        vec![[0; SCREEN_WIDTH_USIZE / SPRITE_SIZE_USIZE]; SCREEN_HEIGHT_USIZE / SPRITE_SIZE_USIZE];
    for row in &mut data {
        for element in row {
            *element = split
                .next()
                .ok_or(BadMapDataError("layer data too short!"))?
                .trim()
                .parse()
                .map_err(|_| BadMapDataError("layer data has invalid digit"))?
        }
    }
    Ok(data)
}
