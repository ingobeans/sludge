use crate::{
    consts::*,
    enemy::{EnemyType, ENEMY_TYPES},
};
use std::fs::read_to_string;

pub fn load_round_data() -> RoundManager {
    let data = read_to_string("data/round_data.txt").expect("data/round_data.txt is missign!!!");
    let mut rounds = Vec::new();
    for line in data.lines() {
        let mut entries = Vec::new();
        let line = line.trim();
        for entry in line.split(' ') {
            let new_entry = if entry.starts_with("delay") {
                RoundEntry::SetDelay(entry.trim_start_matches("delay").parse().unwrap())
            } else if entry.contains("x") {
                let (target, amount) = entry.split_once("x").unwrap();
                RoundEntry::Spawn(target.parse().unwrap(), amount.parse().unwrap())
            } else {
                println!("bad round data at {line} ({entry})");
                continue;
            };
            entries.push(new_entry);
        }
        rounds.push(Round { entries });
    }
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

pub enum RoundEntry {
    SetDelay(u8),
    Spawn(usize, usize),
}

pub struct Round {
    pub entries: Vec<RoundEntry>,
}
