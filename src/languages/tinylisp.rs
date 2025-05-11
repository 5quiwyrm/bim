//! Support for tinylisp syntax highlighting.

use crate::languages::{Language, StyledChar};

pub struct Tinylisp {}
pub const TINYLISP: Tinylisp = Tinylisp {};
impl Language for Tinylisp {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".tlp")
    }
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        let mut ret_buf: Vec<Vec<StyledChar>> = vec![];
        for line in buffer {
            let mut push_buf = vec![];
            let mut escaping = false;
            let mut wrapped = false;
            let mut tks: Vec<String> = vec![];
            let mut acc = String::new();
            let linechars = line.chars();
            for c in linechars {
                acc.push(c);
                let change = c == '\"' && !escaping;
                if change {
                    wrapped = !wrapped;
                }
                escaping = c == '\\';
                if !escaping && !wrapped && c == ')' {
                    acc.pop();
                    tks.push(acc.clone());
                    acc.clear();
                    tks.push(")".to_string());
                }
                if (change && !wrapped)
                    || (c == '(' && !escaping && !wrapped)
                    || (c.is_whitespace() && !escaping && !wrapped)
                {
                    tks.push(acc.clone());
                    acc.clear();
                }
            }
            tks.push(acc);
            tks.iter().for_each(|s| {
                let style = (match s.trim() {
                    "+" | "-" | "*" | "/" | "=" | ">" | "<" => "\x1b[36m",
                    "str" => "\x1b[31m",
                    "car" | "cdr" | "cons" | "quote" | "eval" => "\x1b[35m",
                    "if" | "and" | "or" | "not" => "\x1b[1;34m",
                    "def" | "defn" | "defmacro" => "\x1b[33m",
                    t if t.chars().nth(0) == Some('\"') => "\x1b[32m",
                    u if u.chars().all(|c| c.is_numeric()) => "\x1b[1;34m",
                    "nil" => "\x1b[31m",
                    _ => "",
                })
                .to_string();
                let styled_str = StyledChar::colour_string(s, style.clone());
                styled_str.iter().for_each(|c| push_buf.push(c.clone()));
            });
            ret_buf.push(push_buf);
        }
        ret_buf
    }
    fn indent_size(&self) -> usize {
        2
    }
    fn display_str(&self) -> &str {
        "Tinylisp"
    }
}
