//! Haskell highlighting support.

// This is not finished at all, I just have this so
// I can designate that Haskell uses 2 space indenting.

use crate::languages::{Language, StyledChar};

pub struct Haskell {}
pub const HASKELL: Haskell = Haskell {};
impl Language for Haskell {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".hs")
    }
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        buffer.iter().map(|l| StyledChar::from_string(l)).collect()
    }
    fn indent_size(&self) -> usize {
        2
    }
    fn display_str(&self) -> &'static str {
        "HaskLUL"
    }
}
