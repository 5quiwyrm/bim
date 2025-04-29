use crate::languages::{
    StyledChar,
    Language,
};

pub struct Markdown {}
pub const MARKDOWN: Markdown = Markdown {};
impl Language for Markdown {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".md")
    }

    fn highlight(&self, buffer: &[String]) -> Vec<Vec<StyledChar>> {
        let mut ret_buf: Vec<Vec<StyledChar>> = vec![];
        let mut header_lvl;
        for line in buffer {
            header_lvl = 0;
            let mut push_buf: Vec<StyledChar> = vec![];
            let mut line_chars = line.chars().peekable();
            while line_chars.peek() == Some(&'#') {
                header_lvl += 1;
                let pound = StyledChar {
                    style: "\x1b[2m".to_string(),
                    ch: '#',
                };
                push_buf.push(pound);
                _ = line_chars.next();
            }
            let style = (match header_lvl {
                1 => "\x1b[1;34m",
                2 => "\x1b[35m",
                3 => "\x1b[32m",
                4 => "\x1b[33m",
                5 => "\x1b[31m",
                6 => "\x1b[36m",
                _ => "",
            }).to_string();
            for ch in line_chars {
                let c = StyledChar {
                    style: style.clone(),
                    ch,
                };
                push_buf.push(c);
            }
            ret_buf.push(push_buf);
        }
        ret_buf
    }
    fn indent_size(&self) -> usize {
        4
    }
}
