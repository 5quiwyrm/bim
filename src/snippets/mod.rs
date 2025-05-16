//! Module for adding snippets.

pub trait Snippet {
    /// Gets the snippet corresponding to a query string.
    fn query(&self, query: &str) -> &[&str];
    /// Detects whether a file should use these snippets.
    fn is_kind(&self, filepath: &str) -> bool;
    /// Returns string to display.
    fn display_str(&self) -> &'static str;
}

pub mod text;
use text::*;

pub mod todo;
use todo::*;

pub fn get_snippets(path: &str) -> Box<dyn Snippet> {
    if TODO.is_kind(path) {
        Box::new(TODO)
    } else {
        Box::new(TEXT)
    }
}
