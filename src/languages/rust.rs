//! Rust syntax highlighting support.

use crate::languages::{Language, StyledChar};

pub struct Rust {}
pub const RUST: Rust = Rust {};
impl Language for Rust {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".rs")
    }

    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        let mut ret_buf: Vec<Vec<StyledChar>> = vec![];
        let mut ml_commented = false;
        let mut ml_comment_ending = 0;
        let mut escaping = false;
        let mut quoted = false;
        let mut quote_ending = false;
        let mut charquoted = 0;
        let mut isnum = false;
        let mut errorsize = 0;
        for line in buffer {
            let mut push_buf: Vec<StyledChar> = vec![];
            let line_chars: Vec<char> = line.chars().collect();
            let mut commented = false;
            let mut idx = 0;
            while let Some(&ch) = line_chars.get(idx) {
                if ch == '*' && line_chars.get(idx + 1) == Some(&'/') {
                    ml_commented = false;
                    ml_comment_ending = 2;
                } else if ch == '/' {
                    match line_chars.get(idx + 1) {
                        Some(&'/') => {
                            commented = true;
                        }
                        Some(&'*') => {
                            ml_commented = true;
                        }
                        _ => {}
                    }
                } else if !escaping && !ml_commented && !commented && ml_comment_ending == 0 {
                    if ch == '\"' {
                        if quoted {
                            quote_ending = true;
                        }
                        quoted = !quoted;
                    } else if charquoted == 0 && !quoted {
                        if ch == '\'' {
                            if line_chars.get(idx + 2) == Some(&'\'')
                                && line_chars.get(idx + 1) != Some(&'\\')
                            {
                                charquoted = 3;
                            } else if line_chars.get(idx + 3) == Some(&'\'') {
                                charquoted = 4;
                            }
                        } else if ch.is_numeric() {
                            isnum = true;
                        }
                    }
                }
                escaping = ch == '\\';
                push_buf.push(StyledChar {
                    style: (if errorsize != 0 {
                        errorsize -= 1;
                        "\x1b[41m\x1b[30m"
                    } else if ml_commented || commented {
                        "\x1b[2m"
                    } else if ml_comment_ending != 0 {
                        ml_comment_ending -= 1;
                        "\x1b[2m"
                    } else if quoted {
                        "\x1b[32m"
                    } else if quote_ending {
                        quote_ending = false;
                        "\x1b[32m"
                    } else if charquoted != 0 {
                        charquoted -= 1;
                        "\x1b[36m"
                    } else if isnum {
                        isnum = false;
                        "\x1b[1;34m"
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
