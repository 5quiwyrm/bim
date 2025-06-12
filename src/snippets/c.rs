//! Support for todo snippets

use crate::snippets::Snippet;

pub struct Clang {}
pub const CLANG: Clang = Clang {};

impl Snippet for Clang {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".c") || filepath.ends_with(".cpp")
    }
    fn query(&self, query: &str) -> Vec<String> {
        match query.trim() {
            "include" | "i" => vec!["#include <>".to_string()],
            "define" | "d" => vec!["#define ".to_string()],
            "std" => vec!["#include <stdio.h>".to_string(), "#include <stdlib.h>".to_string()],
            "struct" => vec!["typedef struct {".to_string(), "    type field;".to_string(), "} name;".to_string()],
            "enum" => vec!["typedef enum {".to_string(), "    member,".to_string(), "} name;".to_string()],
            _ => vec![],
        }
    }
    fn display_str(&self) -> &'static str {
        "PDP-11"
    }
}
