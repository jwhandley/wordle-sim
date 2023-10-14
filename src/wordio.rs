use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufRead, BufReader};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use ndarray::Array2;

#[derive(Serialize, Deserialize)]
pub struct ScoreMap {
    pub map: Array2<u8>,
}

pub fn read_words(path: PathBuf) -> HashMap<String, usize> {
    let mut words = HashMap::new();
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let line = line.expect("Failed to read line").trim().to_lowercase();
        words.insert(line, i);
    }

    words
}

pub fn load_score_map() -> Option<ScoreMap> {
    let mut file = match File::open("data/score_map.bincode") {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)
        .expect("Failed to read score_map.mat");

    let score_map: ScoreMap = bincode::deserialize(&encoded)
        .expect("Failed to deserialize score map");

    Some(score_map)


}

pub fn save_score_map(score_map: &ScoreMap) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("data/score_map.bincode")
        .expect("Failed to open score_map.mat");

    let encoded = bincode::serialize(score_map)
        .expect("Failed to serialize score map");

    file.write_all(&encoded)
        .expect("Failed to write score_map.mat");
}