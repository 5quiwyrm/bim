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

    pub fn from_string(s: &str) -> Vec<StyledChar> {
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
        let mut multiline_commented = false;
        let mut multiline_comment_ending = 0;
        let mut escaping = false;
        let mut quoted = false;
        let mut quote_ending = false;
        let mut charquoted = 0;
        let mut errorsize = 0;
        for line in buffer {
            let mut push_buf: Vec<StyledChar> = vec![];
            let line_chars: Vec<char> = line.chars().collect();
            let mut commented = false;
            let mut idx = 0;
            while let Some(&ch) = line_chars.get(idx) {
                if ch == '*' && line_chars.get(idx + 1) == Some(&'/') {
                    multiline_commented = false;
                    multiline_comment_ending = 2;
                }
                if ch == '/' {
                    match line_chars.get(idx + 1) {
                        Some(&'/') => {
                            commented = true;
                        }
                        Some(&'*') => {
                            multiline_commented = true;
                        }
                        _ => {}
                    }
                }
                if ch == '\"'
                    && !escaping
                    && !(multiline_commented || commented || multiline_comment_ending != 0)
                {
                    if quoted {
                        quote_ending = true;
                    }
                    quoted = !quoted;
                }
                if ch == '\''
                    && !escaping
                    && charquoted == 0
                    && !(quoted
                        || multiline_commented
                        || commented
                        || multiline_comment_ending != 0)
                {
                    if line_chars.get(idx + 2) == Some(&'\'') && line_chars.get(idx + 1) != Some(&'\\') {
                        charquoted = 3;
                    } else if line_chars.get(idx + 3) == Some(&'\'') {
                        charquoted = 4;
                    }
                }
                escaping = ch == '\\';
                push_buf.push(StyledChar {
                    style: (if errorsize != 0 {
                        errorsize -= 1;
                        "\x1b[41m\x1b[30m"
                    } else if multiline_commented || commented {
                        "\x1b[2m"
                    } else if multiline_comment_ending != 0 {
                        multiline_comment_ending -= 1;
                        "\x1b[2m"
                    } else if quoted {
                        "\x1b[32m"
                    } else if quote_ending {
                        quote_ending = false;
                        "\x1b[32m"
                    } else if charquoted != 0 {
                        charquoted -= 1;
                        "\x1b[36m"
                    } else {
                        ""
                    })
                    .to_string(),
                    ch,
                });
                idx += 1;
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
