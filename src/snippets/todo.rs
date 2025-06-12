//! Support for todo snippets

use crate::snippets::Snippet;

pub struct Todo {}
pub const TODO: Todo = Todo {};

impl Snippet for Todo {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".todo")
    }
    fn query(&self, query: &str) -> Vec<String> {
        match query.trim() {
            "newtask" | "n" => vec!["[ ] ".to_string()],
            "asap" | "a" => vec!["[ ]!".to_string()],
            _ => vec![],
        }
    }
    fn display_str(&self) -> &'static str {
        "Ssorgn't"
    }
}
