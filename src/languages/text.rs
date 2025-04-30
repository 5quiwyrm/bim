//! Default text support.

use crate::languages::{Language, StyledChar};
use std::time::Instant;

pub struct Text {}
pub const TEXT: Text = Text {};
impl Language for Text {
    fn is_kind(&self, _filepath: &str) -> bool {
        true
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
        "Text"
    }
}
