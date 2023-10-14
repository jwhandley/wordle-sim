use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::PathBuf;

pub fn read_words(path: PathBuf) -> Vec<String> {
    let mut words = Vec::new();
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line").trim().to_lowercase();
        words.push(line);
    }
    

    words
}