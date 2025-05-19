//! Support for todo snippets

use crate::snippets::Snippet;

pub struct Clang {}
pub const CLANG: Clang = Clang {};

impl Snippet for Clang {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".c") || filepath.ends_with(".cpp")
    }
    fn query(&self, query: &str) -> &[&str] {
        match query.trim() {
            "include" | "i" => &["#include <>"],
            "define" | "d" => &["#define "],
            "stds" | "s" => &["#include <stdio.h>", "#include <stdlib.h>"],
            _ => &[],
        }
    }
    fn display_str(&self) -> &'static str {
        "PDP-11"
    }
}
