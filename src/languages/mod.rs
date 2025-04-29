use std::fmt;

pub trait Language {
    fn is_kind(&self, filepath: &str) -> bool;
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>>;
    fn indent_size(&self) -> usize;
}

#[derive(PartialEq, Clone)]
pub struct StyledChar {
    pub style: String,
    pub ch: char,
}

impl fmt::Display for StyledChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}\x1b[0m", self.style, self.ch)
    }
}

impl StyledChar {
    pub fn from_char(ch: char) -> StyledChar {
        StyledChar {
            style: String::new(),
            ch,
        }
    }
    pub fn from_string(s: &str) -> Vec<StyledChar> {
        let mut ret_vec: Vec<StyledChar> = vec![];
        for ch in s.chars() {
            ret_vec.push(StyledChar::from_char(ch));
        }
        ret_vec
    }
}

pub mod rust;
use rust::*;

pub mod text;
use text::*;

pub mod markdown;
use markdown::*;

pub fn get_lang(path: &str) -> Box<dyn Language> {
    if RUST.is_kind(path) {
        Box::new(RUST)
    } else if MARKDOWN.is_kind(path) {
        Box::new(MARKDOWN)
    } else {
        Box::new(TEXT)
    }
}
