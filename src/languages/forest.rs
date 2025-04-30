//! Forest syntax highlighting support.

use crate::languages::{Language, StyledChar};
use std::time::Instant;

pub struct Forest {}
pub const FOREST: Forest = Forest {};
impl Language for Forest {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".frt")
    }

    fn highlight(&self, buffer: &[String]) -> (Vec<Vec<StyledChar>>, u128) {
        let start = Instant::now();
        let mut ret_buf: Vec<Vec<StyledChar>> = vec![];
        let mut wrapped = false;
        let mut escaping = false;
        let tokens = buffer.iter().map(|l| {
            l.trim_end().split_inclusive(move |c: char| {
                if c == '\"' && !escaping {
                    wrapped = !wrapped;
                }
                if escaping {
                    escaping = false;
                }
                if c == '\\' {
                    escaping = true;
                }
                c.is_whitespace() && !wrapped
            })
        });
        for l in tokens {
            let mut push_buf: Vec<StyledChar> = vec![];
            let line = l;
            for tk in line {
                StyledChar::colour_string(
                    tk,
                    (match tk.trim() {
                        "dup" | "drop" | "swap" | "rot" => "\x1b[35m",
                        "+" | "-" | "*" | "/" | "=" | ">" | "<" => "\x1b[36m",
                        "str" | "<>" | "." => "\x1b[31m",
                        "{}" | "assoc" | "keys" | "vals" | "splat"
                            => "\x1b[35m",
                        "if" | "ifend" | "&" | "|" | "!" | "[" | "]"
                            | "break" | "exit" => "\x1b[1;34m",
                        "::" | ":" | "=>" | "->" | ";"
                            | "include" => "\x1b[33m",
                        t if t.chars().nth(0) == Some('\"') => "\x1b[32m",
                        u if u.chars().all(|c| c.is_numeric()) => "\x1b[36m",
                        "nil" => "\x1b[31m",
                        _ => "",
                    })
                    .to_string(),
                ).iter().for_each(|c| push_buf.push(c.clone()));
            }
            //println!("{}", push_buf.last().unwrap());
            ret_buf.push(push_buf);
        }
        (ret_buf, start.elapsed().as_micros())
    }

    fn indent_size(&self) -> usize {
        4
    }

    fn display_str(&self) -> &str {
        "Forest"
    }
}
