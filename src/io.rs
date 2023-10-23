use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn read_wordlist(path: PathBuf) -> Vec<[u8; 5]> {
    let mut words = Vec::new();
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line").trim().to_lowercase();
        assert_eq!(line.len(), 5);
        let line: [u8; 5] = line.as_bytes().try_into().expect("Failed to convert word to bytes");
        words.push(line);
    }

    words
}

pub fn read_wordlist_counts(path: PathBuf) -> Vec<([u8; 5], u64)> {
    let mut words: Vec<([u8; 5], u64)> = Vec::new();
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line").trim().to_lowercase();
        let mut split = line.split_whitespace();
        let word = split.next().expect("Failed to read word");
        let count = split.next().expect("Failed to read count");
        let count = count.parse::<u64>().expect("Failed to parse count");
        assert_eq!(word.len(), 5);
        let word: [u8; 5] = word.as_bytes().try_into().expect("Failed to convert word to bytes");
        words.push((word, count));
    }

    words
}
