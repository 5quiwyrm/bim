//! Module for adding snippets.

pub trait Snippet {
    /// Gets the snippet corresponding to a query string.
    fn query(&self, query: &str) -> Vec<String>;
    /// Detects whether a file should use these snippets.
    fn is_kind(&self, filepath: &str) -> bool;
    /// Returns string to display.
    fn display_str(&self) -> &'static str;
}

pub mod text;
use text::*;

pub mod todo;
use todo::*;

pub mod c;
use c::*;

pub mod html;
use html::*;

pub fn get_snippets(path: &str) -> Box<dyn Snippet> {
    if TODO.is_kind(path) {
        Box::new(TODO)
    } else if CLANG.is_kind(path) {
        Box::new(CLANG)
    } else if HTML.is_kind(path) {
        Box::new(HTML)
    } else {
        Box::new(TEXT)
    }
}
