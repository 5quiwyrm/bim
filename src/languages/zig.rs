//! Zig highlighting support.

// This is not finished at all, I just have this so
// I can designate that Zig uses 4 space indenting.
// Also if you're not aware this was copied from haskell.rs

use crate::languages::{Language, StyledChar};

pub struct Zig {}
pub const ZIG: Zig = Zig {};
impl Language for Zig {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".zig")
    }
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        buffer.iter().map(|l| StyledChar::from_string(l)).collect()
    }
    fn indent_size(&self) -> usize {
        4
    }
    fn display_str(&self) -> &'static str {
        "C3n't"
    }
}
