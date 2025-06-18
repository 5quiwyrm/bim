//! Support for for my personal todo doc format.

use crate::languages::{Language, StyledChar};

pub struct Todo {}
pub const TODO: Todo = Todo {};
impl Language for Todo {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".todo")
    }
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        buffer
            .iter()
            .map(|l| {
                StyledChar::colour_string(
                    l,
                    if l.trim().len() > 4 {
                        match &(l.trim())[0..4] {
                            "[ ] " => "\x1b[33m",
                            "[ ]!" => "\x1b[31m",
                            "[ ]~" => "\x1b[36m",
                            "[ ]P" => "\x1b[35m",
                            x if (x.starts_with("[V]")) => "\x1b[32m",
                            x if (x.starts_with('>')) => "",
                            x if (x.starts_with('$')) => "\x1b[3m",
                            _ => "\x1b[2m",
                        }
                    } else {
                        "\x1b[2m"
                    },
                )
            })
            .collect()
    }
    fn indent_size(&self) -> usize {
        2
    }
    fn display_str(&self) -> &'static str {
        "Ssorgn't"
    }
}
