//! Haskell highlighting support.

// This is not finished at all, I just have this so
// I can designate that Haskell uses 2 space indenting.

use crate::languages::{Language, StyledChar};
use std::time::Instant;

pub struct Haskell {}
pub const HASKELL: Haskell = Haskell {};
impl Language for Haskell {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".hs")
    }
    fn highlight(&self, buffer: &[String]) -> (Vec<Vec<StyledChar>>, u128) {
        let start = Instant::now();
        (
            buffer.iter().map(|l| StyledChar::from_string(l)).collect(),
            start.elapsed().as_micros(),
        )
    }
    fn indent_size(&self) -> usize {
        2
    }
    fn display_str(&self) -> &str {
        "HaskLUL"
    }
}
