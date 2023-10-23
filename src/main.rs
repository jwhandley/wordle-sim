use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;

mod io;

#[inline]
fn calculate_pattern(guess: &[u8; 5], secret: &[u8; 5]) -> [u8; 5] {
    assert_eq!(guess.len(), 5);
    assert_eq!(secret.len(), 5);

    let mut score: [u8; 5] = [0; 5];

    // Use a frequency table for characters in `secret`.
    // We use a size of 256 for the array to cover all possible byte values.
    let mut freq: [u8; 256] = [0; 256];
    let mut used_secret_indices = [false; 5];

    for &s in secret.iter() {
        freq[s as usize] += 1;
    }

    // Green pass
    for (i, (g, s)) in guess.iter().zip(secret.iter()).enumerate() {
        if g == s {
            score[i] = 2;
            used_secret_indices[i] = true;
            // Decrement the frequency since we've found a match.
            freq[*g as usize] -= 1;
        }
    }

    // Yellow pass
    for (i, &g) in guess.iter().enumerate() {
        if score[i] != 2 && freq[g as usize] > 0 {
            score[i] = 1;
            freq[g as usize] -= 1;
        }
    }

    score
}

fn entropy(guess: &[u8; 5], allowed_words: &[([u8; 5], u64)]) -> f32 {
    let mut entropy = 0.0;
    let mut freq: [u64; 243] = [0; 243];
    let ternary_powers: [u8; 5] = [1, 3, 9, 27, 81];

    let total_count: u64 = allowed_words.iter().map(|&(_, count)| count).sum();

    for (word, count) in allowed_words.iter() {
        let pattern = calculate_pattern(guess, word);
        // Convert pattern to ternary using lookup
        let index = pattern
            .iter()
            .enumerate()
            .map(|(i, &p)| p * ternary_powers[i])
            .sum::<u8>();
        freq[index as usize] += count;
    }

    for &f in freq.iter() {
        if f > 0 {
            let p = f as f32 / total_count as f32;
            entropy -= p * p.log2();
        }
    }

    entropy
}

fn best_guess(allowed_words: &[([u8; 5], u64)]) -> [u8; 5] {
    if allowed_words.len() == 1 {
        return allowed_words[0].0.clone();
    }


    let mut best_word = None;
    let mut best_entropy = 0.0;
    for (word, _count) in allowed_words.iter() {
        let e = entropy(word, allowed_words);
        if e > best_entropy {
            best_entropy = e;
            best_word = Some(word);
        }
    }
    best_word
        .expect("There should be at least one allowed word")
        .to_owned()
        
}

fn reduce_allowed_words(
    allowed_words: &[([u8; 5], u64)],
    guess: &[u8; 5],
    pattern: [u8; 5],
) -> Vec<([u8; 5], u64)> {
    allowed_words
        .iter()
        .filter_map(|(word, count)| {
            if calculate_pattern(guess, word) == pattern {
                Some((word.clone(), *count))
            } else {
                None
            }
        })
        .collect()
}

fn simulate_game(
    allowed_words: &[([u8; 5], u64)],
    secret: &[u8; 5],
    initial_guess: &[u8; 5],
) -> (bool, i32) {
    let mut is_solved = false;
    let mut final_score = 1;

    assert_eq!(initial_guess.len(), 5);
    let pattern = calculate_pattern(initial_guess, secret);
    let mut allowed_words = reduce_allowed_words(allowed_words, initial_guess, pattern);

    while !is_solved {
        let guess = best_guess(&allowed_words);
        let pattern = calculate_pattern(&guess, secret);

        // dbg!(&guess, secret, pattern);

        allowed_words = reduce_allowed_words(&allowed_words, &guess, pattern);

        final_score += 1;

        if pattern == [2; 5] && final_score <= 6 {
            is_solved = true;
        } else if pattern == [2; 5] {
            break;
        }
    }

    (is_solved, final_score)
}

fn main() {
    let possible_words = io::read_wordlist(PathBuf::from("data/possible_words.txt"));
    let allowed_words = io::read_wordlist_counts(PathBuf::from("data/dictionary.txt"));

    println!("Running simulations!");
    let pb = ProgressBar::new(possible_words.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .expect("Failed to set progress bar style")
            .progress_chars("##-"),
    );

    let mut scores = Vec::new();
    let mut times = Vec::new();
    let mut wins = 0;
    for secret in possible_words.iter() {
        
            let start_time = Instant::now();
            let (solved, score) = simulate_game(&allowed_words, secret, b"tares");

            if solved {
                wins += 1;
            }

            scores.push(score);
            times.push(start_time.elapsed().as_millis());
            pb.inc(1);
        
    }

    pb.finish();

    let average_score =
        scores.iter().sum::<i32>() as f32 / possible_words.len() as f32;
    println!("Average score: {:.02}", average_score);

    let win_rate = wins as f32 / possible_words.len() as f32;
    println!("Win rate: {:.02}", win_rate * 100.0);

    let average_time =
        times.iter().sum::<u128>() as f32 / possible_words.len() as f32;
    println!("Average time: {:.02} ms", average_time);

    let total_time = times.iter().sum::<u128>();
    println!("Total time: {} ms", total_time);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let guess = b"apple";
        let secret = b"apple";
        let expected = [2, 2, 2, 2, 2];
        assert_eq!(calculate_pattern(guess, secret), expected);
    }

    #[test]
    fn test_no_match() {
        let guess = b"apple";
        let secret = b"bravo";
        let expected = [1, 0, 0, 0, 0];
        assert_eq!(calculate_pattern(guess, secret), expected);
    }

    #[test]
    fn test_partial_exact_match() {
        let guess = b"apple";
        let secret = b"apric";
        let expected = [2, 2, 0, 0, 0];
        assert_eq!(calculate_pattern(guess, secret), expected);
    }

    #[test]
    fn test_wrong_locations() {
        let guess = b"apple";
        let secret = b"plape";
        let expected = [1, 1, 1, 1, 2];
        assert_eq!(calculate_pattern(guess, secret), expected);
    }

    #[test]
    fn test_repeated_characters() {
        let guess = b"apple";
        let secret = b"ppppp";
        let expected = [0, 2, 2, 0, 0];
        assert_eq!(calculate_pattern(guess, secret), expected);
    }

    #[test]
    fn test_all_different_characters() {
        let guess = b"abcde";
        let secret = b"fghij";
        let expected = [0, 0, 0, 0, 0];
        assert_eq!(calculate_pattern(guess, secret), expected);
    }

    #[test]
    fn chat_tests() {
        let guess = b"aaabc";
        let secret = b"aabbb";
        let expected = [2, 2, 0, 2, 0];

        assert_eq!(calculate_pattern(guess, secret), expected);
    }

}
