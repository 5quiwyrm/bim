//! C highlighting support.

// This is not finished at all, I just have this so
// I can designate that C uses 4 space indenting.
// Also if you're not aware this was copied from haskell.rs

use crate::languages::{Language, StyledChar};

pub struct Clang {}
pub const CLANG: Clang = Clang {};
impl Language for Clang {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".c") || filepath.ends_with(".cpp") || filepath.ends_with(".h")
    }
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        buffer.iter().map(|l| StyledChar::from_string(l)).collect()
    }
    fn indent_size(&self) -> usize {
        4
    }
    fn display_str(&self) -> &'static str {
        "C(had)"
    }
}
