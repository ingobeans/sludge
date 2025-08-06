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
    fn draw_tilemap(&self, scale_factor: usize, map: &TileMap) {
        for y in 0..SCREEN_HEIGHT / SPRITE_SIZE {
            for x in 0..SCREEN_WIDTH / SPRITE_SIZE {
                let tile = map[y][x].checked_sub(1);
                if let Some(tile) = tile {
                    self.draw_tile(scale_factor, x * SPRITE_SIZE, y * SPRITE_SIZE, tile, 0.0);
                }
            }
        }
    }
}

type TileMap = [[usize; SCREEN_WIDTH / SPRITE_SIZE]; SCREEN_HEIGHT / SPRITE_SIZE];

#[derive(Debug)]
#[allow(dead_code)]
struct BadMapDataError(&'static str);

struct Map {
    background: TileMap,
    obstructions: TileMap,
    points: Vec<(usize, usize)>,
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

fn parse_tilemap_layer(xml: &str, layer_name: &str) -> Result<TileMap, BadMapDataError> {
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

fn load_maps() -> Vec<Map> {
    let mut maps = Vec::new();
    for item in read_dir("tiled/maps")
        .expect("tiled/maps is missing!!")
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
        };
        maps.push(map);
    }

    maps
}

enum DamageType {
    Magic,
    Pierce,
    Explosion,
    Cold,
    Acid,
}
/// Struct that holds information about an enemy type
struct EnemyType {
    sprite: usize,
    anim_length: usize,
    speed: usize,
    damage_resistance: Vec<DamageType>,
}
/// A live instance of an enemy
struct Enemy {
    ty: &'static EnemyType,
    x: usize,
    y: usize,
    next_path_point: usize,
    score: usize,
}

const ENEMY_TYPES: &[EnemyType] = &[
    // spider
    EnemyType {
        sprite: 1 + 1 * 32,
        anim_length: 2,
        speed: 2,
        damage_resistance: Vec::new(),
    },
];

/// Move source x and y towards target x and y with speed. Returns if the target was reached/hit.
fn move_towards(
    speed: usize,
    source_x: &mut usize,
    source_y: &mut usize,
    target_x: usize,
    target_y: usize,
) -> bool {
    if *source_x < target_x {
        *source_x += speed;
    } else if *source_x > target_x {
        *source_x -= speed;
    }
    if *source_y < target_y {
        *source_y += speed;
    } else if *source_y > target_y {
        *source_y -= speed;
    }
    return *source_x == target_x && *source_y == target_y;
}

#[macroquad::main("sludge")]
async fn main() {
    let mut scale_factor;
    let spritesheet = Spritesheet::new(
        load_texture("spritesheet.png")
            .await
            .expect("spritesheet.png is missing!!"),
    );
    let icons_spritesheet = Spritesheet::new(
        load_texture("icons.png")
            .await
            .expect("icons.png is missing!!"),
    );
    let maps = load_maps();
    let mut enemies: Vec<Enemy> = Vec::new();
    enemies.push(Enemy {
        ty: &ENEMY_TYPES[0],
        x: maps[0].points[0].0 * SPRITE_SIZE,
        y: maps[0].points[0].1 * SPRITE_SIZE,
        next_path_point: 1,
        score: 0,
    });

    loop {
        // update scale factor
        let (screen_width, screen_height) = screen_size();
        scale_factor =
            (screen_width as usize / SCREEN_WIDTH).min(screen_height as usize / SCREEN_HEIGHT);
        clear_background(BLACK);
        spritesheet.draw_tilemap(scale_factor, &maps[0].background);
        spritesheet.draw_tilemap(scale_factor, &maps[0].obstructions);

        let mut death_queue = Vec::new();

        for (index, enemy) in enemies.iter_mut().enumerate() {
            let anim_frame = enemy.score / enemy.ty.speed % enemy.ty.anim_length;
            icons_spritesheet.draw_tile(
                scale_factor,
                enemy.x,
                enemy.y,
                enemy.ty.sprite + anim_frame,
                0.0,
            );
            let next_x = maps[0].points[enemy.next_path_point].0 * SPRITE_SIZE;
            let next_y = maps[0].points[enemy.next_path_point].1 * SPRITE_SIZE;
            // move enemy towards next path point. if point is reached, increment next path point index
            if move_towards(enemy.ty.speed, &mut enemy.x, &mut enemy.y, next_x, next_y) {
                enemy.next_path_point += 1;
                // if at last path point, kill this enemy
                if enemy.next_path_point >= maps[0].points.len() {
                    death_queue.push(index);
                }
            }
            enemy.score += enemy.ty.speed;
        }

        let mut remove_offset = 0;
        for index in death_queue {
            enemies.remove(index - remove_offset);
            remove_offset += 1;
        }

        next_frame().await;
    }
}
