use std::path::PathBuf;
use rand::prelude::*;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};

mod wordio;

fn calculate_score(guess: &str, secret: &str) -> i32 {
    let mut score = [0; 5];
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
        result += i32::pow(3, i as u32) * v;
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

fn reduce_allowed_words(allowed_words: &Vec<String>, guess: &str, score: &i32) -> Vec<String> {
    let mut result = Vec::new();
    for word in allowed_words {
        if calculate_score(guess, word) == *score {
            result.push(word.to_owned());
        }
    }
    result
}

fn simulate_game(allowed_words: &Vec<String>, secret: &str) -> i32 {
    let mut rng = rand::thread_rng();
    let mut guess = allowed_words.choose(&mut rng).unwrap().to_owned();
    let mut score = calculate_score(&guess, secret);

    let mut allowed_words = allowed_words.clone();
    let mut final_score = 1;

    while score != 242 {
        final_score += 1;
        allowed_words = reduce_allowed_words(&allowed_words, &guess, &score);
        guess = allowed_words.choose(&mut rng).unwrap().to_owned();
        score = calculate_score(&guess, secret);

        if final_score > 5 {
            break;
        }

    }

    final_score
}



fn main() {
    let possible_words: Vec<String> = wordio::read_words(PathBuf::from("data/possible_words.txt"));
    let allowed_words: Vec<String> = wordio::read_words(PathBuf::from("data/allowed_words.txt"));

    // let secret = possible_words.choose(&mut rand::thread_rng()).unwrap().to_owned();
    // println!("The secret word is: {}", secret);
    let n_simulations = 1;

    let pb = ProgressBar::new(n_simulations*possible_words.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to set progress bar style")
        .progress_chars("##-"));

    let mut scores = Vec::new();
    let mut times = Vec::new();
    for secret in &possible_words {
        for _ in 0..n_simulations {
            let start_time = Instant::now();
            let score = simulate_game(&allowed_words, &secret);
            scores.push(score);
            times.push(start_time.elapsed().as_millis());
            pb.inc(1);
        }
    }
    
    pb.finish();

    let average_score = scores.iter().sum::<i32>() as f32 / n_simulations as f32;
    println!("Average score: {}", average_score);

    let average_time = times.iter().sum::<u128>() as f32 / n_simulations as f32;
    println!("Average time: {} ms", average_time);

    let total_time = times.iter().sum::<u128>() / 1000;
    println!("Total time: {} s", total_time);
}
