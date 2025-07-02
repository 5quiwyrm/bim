//! Golang highlighting support.

// This is not finished at all, I just have this so
// I can designate that Golang uses 4 space indenting.
// Also if you're not aware this was copied from c.rs

use crate::languages::{Language, StyledChar};

pub struct Golang {}
pub const GOLANG: Golang = Golang {};
impl Language for Golang {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".go")
    }
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        buffer.iter().map(|l| StyledChar::from_string(l)).collect()
    }
    fn indent_size(&self) -> usize {
        4
    }
    fn display_str(&self) -> &'static str {
        "Go"
    }
}
