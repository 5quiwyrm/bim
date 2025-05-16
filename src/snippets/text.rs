//! Default snippets

use crate::snippets::Snippet;

pub struct Text {}
pub const TEXT: Text = Text {};
impl Snippet for Text {
    fn is_kind(&self, _filepath: &str) -> bool {
        true
    }
    fn query(&self, _query: &str) -> &[&str] {
        &[]
    }
    fn display_str(&self) -> &'static str {
        "Text"
    }
}
