use std::collections::HashMap;
#[cfg(not(feature = "bundled"))]
use std::fs::{read_dir, read_to_string};

use macroquad::{
    audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound},
    prelude::*,
    rand,
};

use crate::{
    map::{
        parse_points_from_tilemap, parse_spawnpoints_from_tilemap, parse_tilemap_layer, Map,
        Spritesheet,
    },
    rounds::{decode_rounds, Round, RoundManager},
};

async fn load_sounds(path: &str) -> Vec<Sound> {
    let mut sounds: Vec<Option<Sound>> = vec![None];
    for (mut index, sound) in sounds.iter_mut().enumerate() {
        index += 1;
        let bytes;
        let path = path.to_string() + &(index.to_string() + ".wav");
        #[cfg(feature = "bundled")]
        {
            bytes = DATA.get_file(&path[5..]).unwrap().contents();
        }
        #[cfg(not(feature = "bundled"))]
        {
            let error = format!("{:?} is missing!!", path);
            use std::fs::read;

            bytes = read(path).expect(&error);
        }
        *sound = Some(load_sound_from_bytes(&bytes).await.unwrap());
    }
    sounds.into_iter().flatten().collect()
}
#[derive(Clone)]
pub enum ProjectileSound {
    Hit,
    None,
    Explosion,
}
impl ProjectileSound {
    pub fn play(&self, sfx_manager: &SFXManager) {
        match self {
            ProjectileSound::None => {}
            ProjectileSound::Hit => SFXManager::play_sound(&sfx_manager.hit),
            ProjectileSound::Explosion => SFXManager::play_sound(&sfx_manager.explosion),
        }
    }
}
impl Default for ProjectileSound {
    fn default() -> Self {
        Self::None
    }
}
type Sfx = (Vec<Sound>, f32);
pub struct SFXManager {
    pub explosion: Sfx,
    pub hit: Sfx,
}
impl SFXManager {
    pub async fn new() -> Self {
        SFXManager {
            explosion: (load_sounds("data/sfx/explosion/").await, 0.3),
            hit: (load_sounds("data/sfx/hit/").await, 0.2),
        }
    }
    pub fn play_sound(sounds: &Sfx) {
        let sound = &sounds.0[rand::gen_range(0, sounds.0.len())];

        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: sounds.1,
            },
        );
    }
}

#[cfg(feature = "bundled")]
use include_directory::{include_directory, Dir};

#[cfg(feature = "bundled")]
static DATA: Dir<'_> = include_directory!("data");

pub fn load_texture(path: &str) -> Texture2D {
    #[cfg(feature = "bundled")]
    {
        let bytes = DATA.get_file(&path[5..]).unwrap().contents();
        Texture2D::from_file_with_format(bytes, None)
    }
    #[cfg(not(feature = "bundled"))]
    {
        let error = format!("{} is missing!!", path);
        use std::fs::read;

        let bytes = read(path).expect(&error);
        Texture2D::from_file_with_format(&bytes, None)
    }
}

pub type SublevelHashmap = HashMap<String, Vec<Vec<Round>>>;

pub fn load_maps() -> Vec<Map> {
    let mut maps = Vec::new();
    let contents;

    #[cfg(feature = "bundled")]
    {
        contents = DATA.get_dir("maps").unwrap().entries()
    }
    #[cfg(not(feature = "bundled"))]
    {
        contents = read_dir("data/maps")
            .expect("data/maps is missing!!")
            .flatten();
    }
    for item in contents {
        let data;
        let name;
        #[cfg(feature = "bundled")]
        {
            data = DATA.get_file(item.path()).unwrap().contents_utf8().unwrap();
            name = item.path().file_name().unwrap().to_string_lossy()[1..]
                .split_once('.')
                .unwrap()
                .0
                .to_string();
        }
        #[cfg(not(feature = "bundled"))]
        {
            name = item.file_name().to_string_lossy()[1..]
                .split_once('.')
                .unwrap()
                .0
                .to_string();
            data = read_to_string(item.path()).expect("failed to read map data :(");
        }
        let background = parse_tilemap_layer(&data, "Background").expect("bad map data");
        let obstructions = parse_tilemap_layer(&data, "Obstructions").expect("bad map data");
        let out_of_bounds = parse_tilemap_layer(&data, "Out of Bounds").expect("bad map data");
        let path = parse_tilemap_layer(&data, "Path").expect("bad map data");

        let map = Map {
            name,
            points: parse_points_from_tilemap(&path),
            background,
            out_of_bounds,
            obstructions,
            tower_spawnpoints: parse_spawnpoints_from_tilemap(&path),
            path,
        };
        maps.push(map);
    }

    maps
}

pub fn load_round_data() -> RoundManager {
    let sublevels = get_sublevels_hashmap();

    let data;
    #[cfg(feature = "bundled")]
    {
        data = DATA
            .get_file("round_data.txt")
            .unwrap()
            .contents_utf8()
            .unwrap();
    }
    #[cfg(not(feature = "bundled"))]
    {
        data = read_to_string("data/round_data.txt").expect("data/round_data.txt is missign!!!");
    }
    let rounds = decode_rounds(&data, Some(sublevels));
    RoundManager {
        in_progress: false,
        round: 0,
        rounds,
        delay_counter: 0,
        spawn_counter: 0,
    }
}

pub fn get_sublevels_hashmap() -> SublevelHashmap {
    let mut sublevels: SublevelHashmap = HashMap::new();

    #[cfg(feature = "bundled")]
    {
        for sublevel in DATA.get_dir("sublevels").unwrap().entries() {
            let name = sublevel
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let mut entries = Vec::new();
            for entry in sublevel.as_dir().unwrap().entries() {
                let data = entry.as_file().unwrap().contents_utf8().unwrap();
                entries.push(decode_rounds(&data, None));
            }
            sublevels.insert(name, entries);
        }
    }
    #[cfg(not(feature = "bundled"))]
    {
        for sublevel in read_dir("data/sublevels")
            .expect("no data/sublevels!!!")
            .flatten()
        {
            let name = sublevel.file_name().to_string_lossy().to_string();
            let mut entries = Vec::new();
            for entry in read_dir(sublevel.path()).unwrap().flatten() {
                let data = read_to_string(entry.path()).unwrap();
                entries.push(decode_rounds(&data, None));
            }
            sublevels.insert(name, entries);
        }
    }
    sublevels
}

pub fn load_spritesheet(path: &str, size: usize) -> Spritesheet {
    let texture = load_texture(path);
    Spritesheet::new(texture, size)
}
