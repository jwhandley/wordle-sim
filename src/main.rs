use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use rand::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use ndarray::Array2;

mod wordio;
use wordio::ScoreMap;


fn calculate_score(guess: &str, secret: &str) -> u8 {
    let mut score: [u8; 5] = [0; 5];
    let guess_chars: Vec<char> = guess.chars().collect();
    let secret_chars: Vec<char> = secret.chars().collect();
    let mut used_secret_indices = [false; 5];

    // Green pass
    for (i, (g, s)) in guess_chars.iter().zip(secret_chars.iter()).enumerate() {
        if g == s {
            score[i] = 2;
            used_secret_indices[i] = true;
        }
    }

    // Yellow pass
    for (i, g) in guess_chars.iter().enumerate() {
        if score[i] != 2 {
            if let Some(index) = secret_chars.iter().enumerate().find(|&(j, &s)| s == *g && !used_secret_indices[j]) {
                score[i] = 1;
                used_secret_indices[index.0] = true;
            }
        }
    }

    // Convert to base 3
    let mut result = 0;
    for (i, v) in score.iter().enumerate() {
        result += u8::pow(3, i as u32) * *v;
    }

    result
}

#[allow(dead_code)]
fn score_to_array(score: &i32) -> [i32; 5] {
    let mut result = [0; 5];
    let mut score = *score;
    for item in &mut result {
        *item = score % 3;
        score /= 3;
    }
    result
}

fn precalculate_scores(allowed_words: &HashMap<String, usize>) -> ScoreMap {
    let pb = ProgressBar::new(allowed_words.len() as u64 * allowed_words.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to set progress bar style")
        .progress_chars("##-"));

    let mut score_map = Array2::<u8>::zeros((allowed_words.len(), allowed_words.len()));

    for (guess, guess_idx) in allowed_words.iter() {
        for (secret, secret_idx) in allowed_words.iter() {
            score_map[(*guess_idx, *secret_idx)] = calculate_score(guess, secret);
            pb.inc(1);
        }
    }
    pb.finish();

    ScoreMap { map: score_map }
}

fn best_guess(allowed_words: &HashMap<String, usize>, _score_map: &ScoreMap) -> (String, usize) {
    // Choose the guess that reduces the option set the most
    let mut rng = rand::thread_rng();

    let (guess, guess_idx) = allowed_words.iter().choose(&mut rng).unwrap();

    (guess.clone(), *guess_idx)
}

fn reduce_allowed_words(allowed_words: &HashMap<String, usize>, guess_idx: usize, score: u8, score_map: &ScoreMap) -> HashMap<String, usize> {
    let mut new_allowed_words = allowed_words.clone();

    for possible_secret in allowed_words.clone() {
        if score_map.map[(guess_idx,possible_secret.1)] != score {
            new_allowed_words.remove(&possible_secret.0);
        }
    }

    new_allowed_words
    
}

fn simulate_game(allowed_words: &HashMap<String, usize>, secret: &str, score_map: &ScoreMap) -> (bool, i32) {    
    let mut is_solved = false;
    let mut allowed_words = allowed_words.clone();
    let mut final_score = 0;
    let secret_idx = *allowed_words.get(secret).unwrap();

    while !is_solved {
        let (_guess, guess_idx) = best_guess(&allowed_words, score_map);
        let score = score_map.map[(guess_idx,secret_idx)];



        allowed_words = reduce_allowed_words(&allowed_words, guess_idx, score, score_map);

        
        final_score += 1;

        if score == 242 {
            is_solved = true;
        } else if final_score > 5 {
            break;
        }
    }

    (is_solved, final_score)
}



fn main() {
    let possible_words: HashMap<String, usize> = wordio::read_words(PathBuf::from("data/possible_words.txt"));
    let allowed_words: HashMap<String, usize> = wordio::read_words(PathBuf::from("data/allowed_words.txt"));
    
    println!("Loading score map...");
    let score_map = match wordio::load_score_map() {
        Some(score_map) => score_map,
        None => {
            println!("No score map found, precalculating...");
            let score_map = precalculate_scores(&allowed_words);
            println!("Saving score map...");
            wordio::save_score_map(&score_map);
            
            score_map
        }
    };
    println!("Done!");

    println!("Running simulations!");
    let n_simulations = 1;

    let pb = ProgressBar::new(n_simulations*possible_words.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to set progress bar style")
        .progress_chars("##-"));

    let mut scores = Vec::new();
    let mut times = Vec::new();
    let mut wins = 0;
    for secret in possible_words.keys() {
        for _ in 0..n_simulations {
            let start_time = Instant::now();
            let (solved, score)= simulate_game(&allowed_words, secret, &score_map);

            if solved {
                wins += 1;
            }

            scores.push(score);
            times.push(start_time.elapsed().as_millis());
            pb.inc(1);
        }
    }
    
    pb.finish();

    let average_score = scores.iter().sum::<i32>() as f32 / n_simulations as f32 / possible_words.len() as f32;
    println!("Average score: {}", average_score);

    let win_rate = wins as f32 / n_simulations as f32 / possible_words.len() as f32;
    println!("Win rate: {}", win_rate);

    let average_time = times.iter().sum::<u128>() as f32 / n_simulations as f32 / possible_words.len() as f32;
    println!("Average time: {} ms", average_time);

    let total_time = times.iter().sum::<u128>() / 1000;
    println!("Total time: {} s", total_time);
}
