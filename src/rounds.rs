use macroquad::rand::{self, RandomRange};

use crate::{
    consts::*,
    enemy::{EnemyType, ENEMY_TYPES},
};
use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string},
};

fn get_index_of_enemy(name: &str) -> usize {
    ENEMY_TYPES.iter().position(|f| f.name == name).unwrap()
}

fn decode_rounds(
    data: &str,
    mut sublevels: Option<HashMap<String, Vec<Vec<Round>>>>,
) -> Vec<Round> {
    let mut rounds = Vec::new();
    for line in data.lines() {
        let mut entries = Vec::new();
        let line = line.trim();
        if line.starts_with(":sublevel ") {
            if let Some(sublevels) = &mut sublevels {
                let key = line.trim_start_matches(":sublevel ");
                let mut sublevels = sublevels.remove(key).expect("sublevel not found!");
                rand::srand(macroquad::miniquad::date::now() as _);
                let mut random = sublevels.remove(rand::gen_range(0, sublevels.len()));
                rounds.append(&mut random);
            }
            continue;
        }

        for entry in line.split(' ') {
            let new_entry = if entry.starts_with("delay-") {
                RoundEntry::SetDelay(entry.trim_start_matches("delay-").parse().unwrap())
            } else if entry.contains("-") {
                let (target, amount) = entry.split_once("-").unwrap();
                RoundEntry::Spawn(get_index_of_enemy(target), amount.parse().unwrap())
            } else {
                println!("bad round entry at '{line}' ({entry})");
                continue;
            };
            entries.push(new_entry);
        }
        rounds.push(Round { entries });
    }
    rounds
}

pub fn load_round_data() -> RoundManager {
    let mut sublevels: HashMap<String, Vec<Vec<Round>>> = HashMap::new();
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

    let data = read_to_string("data/round_data.txt").expect("data/round_data.txt is missign!!!");
    let rounds = decode_rounds(&data, Some(sublevels));
    RoundManager {
        in_progress: false,
        round: 0,
        rounds,
        delay_counter: 0,
        spawn_counter: 0,
    }
}

pub enum RoundUpdate {
    /// Nothing can be spawned, still on cooldown
    Cooldown,
    /// Spawn new enemy
    Spawn(&'static EnemyType),
    /// All enemies of this round have been spawned
    Finished,
}
pub struct RoundManager {
    pub in_progress: bool,
    pub round: usize,
    pub rounds: Vec<Round>,
    pub delay_counter: u8,
    pub spawn_counter: usize,
}
impl RoundManager {
    pub fn finish_round(&mut self) {
        self.in_progress = false;
        self.round += 1;
        self.delay_counter = 0;
        self.spawn_counter = 0;
    }
    fn next_ready(&self) -> bool {
        self.delay_counter == 0
    }
    /// Gets next enemy to spawn.
    pub fn update(&mut self) -> RoundUpdate {
        if !self.next_ready() {
            self.delay_counter -= 1;
            return RoundUpdate::Cooldown;
        }
        let mut counter = self.spawn_counter;
        self.delay_counter = DEFAULT_SPAWN_DELAY;
        self.spawn_counter += 1;
        for entry in &self.rounds[self.round].entries {
            match entry {
                RoundEntry::SetDelay(delay) => {
                    self.delay_counter = *delay;
                }
                RoundEntry::Spawn(target, amount) => {
                    if counter < *amount {
                        return RoundUpdate::Spawn(&ENEMY_TYPES[*target]);
                    } else {
                        counter -= amount;
                    }
                }
                _ => {}
            }
        }
        RoundUpdate::Finished
    }
}

#[derive(Clone)]
pub enum RoundEntry {
    SetDelay(u8),
    Spawn(usize, usize),
}

#[derive(Clone)]
pub struct Round {
    pub entries: Vec<RoundEntry>,
}
