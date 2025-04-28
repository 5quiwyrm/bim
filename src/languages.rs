use std::fmt;

pub trait Language {
    fn is_kind(&self, filepath: &str) -> bool;
    fn highlight(&self, buffer: Vec<String>) -> Vec<Vec<StyledChar>>;
    fn indent_size(&self) -> usize;
}
struct Text {}
pub const TEXT: Text = Text {};
impl Language for Text {
    fn is_kind(&self, filepath: &str) -> bool {
        true
    }
    fn highlight(&self, buffer: Vec<String>) -> Vec<Vec<StyledChar>> {
        buffer.iter().map(|l| StyledChar::from_string(l)).collect()
    }
    fn indent_size(&self) -> usize {
        2
    }
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

    pub fn from_string(s: &String) -> Vec<StyledChar> {
        let mut ret_vec: Vec<StyledChar> = vec![];
        for ch in s.chars() {
            ret_vec.push(StyledChar::from_char(ch));
        }
        ret_vec
    }
}

struct Rust {}
impl Language for Rust {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".rs")
    }

    fn highlight(&self, buffer: Vec<String>) -> Vec<Vec<StyledChar>> {
        let mut ret_buf: Vec<Vec<StyledChar>> = vec![];
        for line in buffer {
            let mut push_buf: Vec<StyledChar> = vec![];
            let mut line_chars = line.chars().peekable();
            let mut commented = false;
            while let Some(ch) = line_chars.next() {
                push_buf.push(StyledChar {
                    style: (if commented {
                        "\x1b[32m"
                    } else if ch == '/' && line_chars.peek() == Some(&'/') {
                        commented = true;
                        "\x1b[32m"
                    } else {
                        ""
                    })
                    .to_string(),
                    ch,
                })
            }
            ret_buf.push(push_buf);
        }
        ret_buf
    }

    fn indent_size(&self) -> usize {
        4
    }
}
pub const RUST: Rust = Rust {};

pub fn get_lang(path: &str) -> Box<dyn Language> {
    if RUST.is_kind(path) {
        Box::new(RUST)
    } else {
        Box::new(TEXT)
    }
}
