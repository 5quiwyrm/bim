//! Support for todo snippets

use crate::snippets::Snippet;

pub struct Todo {}
pub const TODO: Todo = Todo {};

impl Snippet for Todo {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".todo")
    }
    fn query(&self, query: &str) -> &[&str] {
        match query.trim() {
            "newtask" | "n" => &["[ ] "],
            "asap" | "a" => &["[ ]!"],
            _ => &[],
        }
    }
    fn display_str(&self) -> &str {
        "Ssorgn't"
    }
}
