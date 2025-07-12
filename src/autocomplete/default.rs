//! Default autocomplete.
//! This is just a naive scraping of all punctuation or whitespace delimited
//! words and a levenshtein distance search.

use crate::Buffer;
use crate::autocomplete::AutoComplete;

pub struct Default {}
pub const DEFAULT: Default = Default {};

fn isnt_token_char(c: char) -> bool {
    !c.is_alphanumeric() && c != '_' && c != '-' && c != '\''
}

impl AutoComplete for Default {
    fn get_candidates(&self, buf: &Buffer) -> Vec<String> {
        let mut query = String::new();
        let line: Vec<char> = buf.contents[buf.cursor_pos.line].chars().collect();
        let mut idx = buf.cursor_pos.idx;
        while idx > 0 {
            idx -= 1;
            if !isnt_token_char(line[idx]) {
                query.push(line[idx])
            } else {
                break;
            }
        }
        query = query.chars().rev().collect();
        let contents = buf.contents.join(" ");
        let mut tokens: Vec<&str> = contents
            .split(isnt_token_char)
            .filter(|t| t.len() >= query.len())
            .collect();
        tokens.sort();
        tokens.dedup();
        let mut candidates: Vec<(String, usize)> = vec![];
        for tk in &tokens {
            candidates.push((tk.to_string(), optimized_levenshtein_distance(&query, tk)))
        }
        candidates.sort_by(|a, b| a.1.cmp(&b.1));
        candidates.iter().map(|a| a.0.clone()).collect()
    }
    fn is_kind(&self, _path: &str) -> bool {
        true
    }
    fn display_str(&self) -> &str {
        "Text"
    }
}

// Stolen from https://github.com/TheAlgorithms/Rust/blob/master/src/string/levenshtein_distance.rs
pub fn optimized_levenshtein_distance(string1: &str, string2: &str) -> usize {
    if string1.is_empty() {
        return string2.len();
    }
    let l1 = string1.len();
    let mut prev_dist: Vec<usize> = (0..=l1).collect();

    for (row, c2) in string2.chars().enumerate() {
        let mut prev_substitution_cost = prev_dist[0];
        prev_dist[0] = row + 1;

        for (col, c1) in string1.chars().enumerate() {
            let deletion_cost = prev_dist[col] + 1;
            let insertion_cost = prev_dist[col + 1];
            let substitution_cost = if c1 == c2 {
                prev_substitution_cost
            } else {
                prev_substitution_cost + 1
            };
            prev_substitution_cost = prev_dist[col + 1];
            prev_dist[col + 1] = _min3(deletion_cost, insertion_cost, substitution_cost);
        }
    }
    prev_dist[l1]
}

#[inline]
fn _min3<T: Ord>(a: T, b: T, c: T) -> T {
    use std::cmp::min;
    min(a, min(b, c))
}
