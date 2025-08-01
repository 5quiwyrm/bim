//! Module for adding autocomplete for languages

use crate::Buffer;

pub trait AutoComplete {
    // Given contents, get candidates and return (in order of likelihood)
    // as a Vec<String>.
    // The usize also returned is the length of the query.
    fn get_candidates(&self, buf: &Buffer) -> (Vec<String>, usize);
    // Add tokens to the memory. This may be left empty.
    fn add_tokens(&mut self, update_request: UpdateRequest);
    fn is_kind(&self, path: &str) -> bool;
    fn display_str(&self) -> &str;
}

pub enum UpdateRequest<'a> {
    Whole {
        filepath: &'a str,
        new_contents: &'a [String],
    },
}

pub mod default;
use default::*;

pub mod markdown;
use markdown::*;

pub fn get_autocomplete_engine(path: &str) -> Box<dyn AutoComplete> {
    if MARKDOWN.is_kind(path) {
        Box::new(MARKDOWN)
    } else {
        Box::new(Default::new())
    }
}
