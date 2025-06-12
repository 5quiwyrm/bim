//! Module for adding syntax highlighting and autoindenting support.

use std::fmt;

/// Trait for supporting indenting and highlighting.
pub trait Language {
    /// Detects whether a file should use this type of highlighting based on file path.
    fn is_kind(&self, filepath: &str) -> bool;
    /// Highlights text from buffer. u128 represents microseconds spent highlighting.
    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>>;
    /// Returns indent size used.
    fn indent_size(&self) -> usize;
    /// Converts to display string.
    fn display_str(&self) -> &'static str;
}

/// Struct for styling chars.
#[derive(PartialEq, Clone)]
pub struct StyledChar {
    pub style: &'static str,
    pub ch: char,
}

impl fmt::Display for StyledChar {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}\x1b[0m", self.style, self.ch)
    }
}

impl StyledChar {
    /// Generates `StyledChar` from raw character, with no styling.
    pub fn from_char(ch: char) -> StyledChar {
        StyledChar { style: "", ch }
    }
    /// Generates `Vec<StyledChar>` from string, with no styling.
    pub fn from_string(s: &str) -> Vec<StyledChar> {
        let mut ret_vec: Vec<StyledChar> = vec![];
        for ch in s.chars() {
            ret_vec.push(StyledChar::from_char(ch));
        }
        ret_vec
    }

    pub fn colour_string(s: &str, style: &'static str) -> Vec<StyledChar> {
        s.chars().map(|ch: char| StyledChar { style, ch }).collect()
    }
}

// Add language modules

pub mod rust;
use rust::*;

pub mod text;
use text::*;

pub mod markdown;
use markdown::*;

pub mod forest;
use forest::*;

pub mod tinylisp;
use tinylisp::*;

pub mod haskell;
use haskell::*;

pub mod todo;
use todo::*;

pub mod c;
use c::*;

pub mod zig;
use zig::*;

// Update get_lang if you added a new language

pub fn get_lang(path: &str) -> Box<dyn Language> {
    if RUST.is_kind(path) {
        Box::new(RUST)
    } else if MARKDOWN.is_kind(path) {
        Box::new(MARKDOWN)
    } else if FOREST.is_kind(path) {
        Box::new(FOREST)
    } else if TINYLISP.is_kind(path) {
        Box::new(TINYLISP)
    } else if HASKELL.is_kind(path) {
        Box::new(HASKELL)
    } else if TODO.is_kind(path) {
        Box::new(TODO)
    } else if CLANG.is_kind(path) {
        Box::new(CLANG)
    } else if ZIG.is_kind(path) {
        Box::new(ZIG)
    } else {
        Box::new(TEXT)
    }
}
