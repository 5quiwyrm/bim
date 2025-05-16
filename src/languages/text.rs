//! Default text support.

use crate::languages::{Language, StyledChar};

pub struct Text {}
pub const TEXT: Text = Text {};
impl Language for Text {
    fn is_kind(&self, _filepath: &str) -> bool {
        true
    }
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        buffer.iter().map(|l| StyledChar::from_string(l)).collect()
    }
    fn indent_size(&self) -> usize {
        2
    }
    fn display_str(&self) -> &'static str {
        "Text"
    }
}
