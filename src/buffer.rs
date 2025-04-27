use crossterm::{event, terminal};
use std::{collections::HashMap, fmt, fs};

fn pretty_str_event(event: &event::Event) -> String {
    if let event::Event::Key(key) = event {
        if key.modifiers != event::KeyModifiers::NONE {
            format!("{} {}", key.modifiers, key.code)
        } else {
            format!("{}", key.code)
        }
    } else {
        "".to_string()
    }
}

#[derive(Copy, Clone)]
pub struct Cursor {
    pub line: usize,
    pub idx: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Action {
    None,
    Save,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::None => write!(f, ""),
            Action::Save => write!(f, "save"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Default,
    Paste,
    Replace,
    Find,
    ReplaceStr,
    Goto,
    OpenFile,
    Switch,
}

impl Mode {
    pub fn from_str(s: &str) -> Mode {
        match s {
            "paste" | "p" => Mode::Paste,
            "replace" | "r" => Mode::Replace,
            "find" | "f" => Mode::Find,
            "replacestr" | "rs" => Mode::ReplaceStr,
            "goto" | "g" => Mode::Goto,
            "switch" | "s" => Mode::Switch,
            "open" | "o" | "openfile" => Mode::OpenFile,
            _ => Mode::Default,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Default => write!(f, "default"),
            Mode::Paste => write!(f, "paste"),
            Mode::Replace => write!(f, "replace"),
            Mode::Find => write!(f, "find"),
            Mode::ReplaceStr => write!(f, "replace str"),
            Mode::Goto => write!(f, "goto"),
            Mode::OpenFile => write!(f, "open file"),
            Mode::Switch => write!(f, "switch to mode"),
        }
    }
}

impl Mode {
    pub fn show_temp(&self) -> bool {
        use Mode::*;
        match self {
            Default | Paste | Replace | Find | ReplaceStr => false,
            Goto | Switch | OpenFile => true,
        }
    }
}

pub struct Buffer {
    pub contents: Vec<String>,
    pub cursor_pos: Cursor,
    pub top: usize,
    pub filepath: String,
    pub lastact: Action,
    pub find_str: String,
    pub replace_str: String,
    pub temp_str: String,
    pub marklist: HashMap<char, Cursor>,
    pub indent_lvl: usize,
    pub mode: Mode,
}

impl Buffer {
    pub fn new(filepath: String) -> Self {
        let contents: Vec<String> = fs::read_to_string(&filepath)
            .unwrap_or({
                fs::File::create(&filepath).unwrap();
                "\n".to_string()
            })
            .lines()
            .map(|s| s.to_string())
            .collect();
        Buffer {
            contents,
            top: 0,
            cursor_pos: Cursor { line: 0, idx: 0 },
            filepath,
            lastact: Action::None,
            find_str: String::new(),
            replace_str: String::new(),
            temp_str: String::new(),
            marklist: HashMap::new(),
            indent_lvl: 0,
            mode: Mode::Default,
        }
    }

    #[inline]
    pub fn move_left(&mut self) -> bool {
        if self.cursor_pos.idx == 0 {
            if self.cursor_pos.line != 0 {
                self.cursor_pos.line -= 1;
                self.cursor_pos.idx = self.contents[self.cursor_pos.line].len();
            } else {
                return false;
            }
        } else {
            self.cursor_pos.idx -= 1;
        }
        true
    }

    #[inline]
    pub fn move_right(&mut self) -> bool {
        if self.cursor_pos.idx == self.contents[self.cursor_pos.line].len()
            || self.contents[self.cursor_pos.line].is_empty()
        {
            if self.cursor_pos.line + 1 != self.contents.len() && !self.contents.is_empty() {
                self.cursor_pos.line += 1;
                self.cursor_pos.idx = 0;
            } else {
                return false;
            }
        } else {
            self.cursor_pos.idx += 1;
        }
        true
    }

    #[inline]
    pub fn move_up(&mut self) -> bool {
        if self.cursor_pos.line != 0 {
            self.cursor_pos.line -= 1;
            if self.cursor_pos.idx >= self.contents[self.cursor_pos.line].len() {
                self.cursor_pos.idx = self.contents[self.cursor_pos.line].len();
            }
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn move_down(&mut self) -> bool {
        if self.cursor_pos.line + 1 != self.contents.len() && !self.contents.is_empty() {
            self.cursor_pos.line += 1;
            if self.cursor_pos.idx > self.contents[self.cursor_pos.line].len() {
                self.cursor_pos.idx = self.contents[self.cursor_pos.line].len();
            }
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn save(&mut self) {
        let trimmedlines: Vec<&str> = self.contents.iter().map(|s| s.trim_end()).collect();
        fs::write(self.filepath.clone(), trimmedlines.join("\n")).unwrap();
        self.lastact = Action::Save;
    }

    pub fn backspace(&mut self) -> Option<char> {
        let t = self.cursor_pos.idx;
        if t == 0 {
            if self.cursor_pos.line != 0 {
                let currline = self.contents[self.cursor_pos.line].clone();
                let oldlen = self.contents[self.cursor_pos.line - 1].len();
                self.contents[self.cursor_pos.line - 1].push_str(&currline);
                self.contents.remove(self.cursor_pos.line);
                self.cursor_pos.line -= 1;
                self.cursor_pos.idx = oldlen;
                Some('\n')
            } else if !self.contents[0].is_empty() {
                return Some(self.contents[0].remove(0));
            } else {
                return None;
            }
        } else {
            self.cursor_pos.idx -= 1;
            Some(self.contents[self.cursor_pos.line].remove(t - 1))
        }
    }

    #[inline]
    pub fn type_char(&mut self, ch: char) {
        self.contents[self.cursor_pos.line].insert(self.cursor_pos.idx, ch);
        self.cursor_pos.idx += 1;
    }

    pub fn newline_below(&mut self, linect: String) {
        let mut newline = String::new();
        for _ in 0..(self.indent_lvl * 4) {
            newline.push(' ');
        }
        newline.push_str(&linect);
        self.cursor_pos.line += 1;
        self.cursor_pos.idx = self.indent_lvl * 4;
        self.contents.insert(self.cursor_pos.line, newline);
    }

    pub fn reload_file(&mut self) {
        self.contents = fs::read_to_string(&self.filepath)
            .unwrap_or("\n".to_string())
            .lines()
            .map(|s| s.to_string())
            .collect();
        if self.contents.len() < self.cursor_pos.line {
            self.cursor_pos.line = if self.contents.is_empty() {
                0
            } else {
                self.contents.len() - 1
            };
        }
        if self.contents[self.cursor_pos.line].len() < self.cursor_pos.idx {
            self.cursor_pos.idx = self.contents[self.cursor_pos.line].len();
        }
        self.save();
    }

    pub fn print(&mut self, event: event::Event) {
        print!("\x1b[J\x1b[H");
        let (widthu, heightu) = terminal::size().unwrap();
        let width = widthu as usize;
        let height = heightu as usize;
        let bottom_pad = 2;
        if self.cursor_pos.line > self.top + height - bottom_pad - 3 {
            self.top = self.cursor_pos.line + bottom_pad + 3 - height;
        }
        if self.cursor_pos.line < self.top {
            self.top = self.cursor_pos.line;
        }
        let mut tb_printed = String::new();

        let mut linectr = self.top;
        while linectr < self.top + height - bottom_pad && linectr < self.contents.len() {
            if linectr == self.cursor_pos.line {
                let mut i = 0;
                let mut line_content = self.contents[self.cursor_pos.line].chars();
                while i < width {
                    let content = line_content.next().unwrap_or(' ');
                    let id = self.indent_lvl * 4;
                    match i {
                        a if a == self.cursor_pos.idx => {
                            tb_printed
                                .push_str(format!("\x1b[47m\x1b[30m{content}\x1b[0m").as_str());
                        }
                        b if b == id => {
                            if content == ' ' {
                                tb_printed.push_str("\x1b[33m\x1b[2m|\x1b[0m");
                            } else {
                                tb_printed.push_str(format!("\x1b[33m{content}\x1b[0m").as_str());
                            }
                        }
                        _ => {
                            tb_printed.push_str(format!("{content}").as_str());
                        }
                    }
                    i += 1;
                }
                tb_printed.push('\n');
            } else if self.contents[linectr].len() > width {
                tb_printed.push_str(&self.contents[linectr][0..width]);
                tb_printed.push('\n');
            } else {
                tb_printed.push_str(format!("{: <width$}", self.contents[linectr]).as_str());
            }
            linectr += 1;
        }
        let mut bottom_bar = format!(
            "({}, {}) [{}] (>: {:?}) {}{}(act: {}) {}",
            self.cursor_pos.line + 1,
            self.cursor_pos.idx + 1,
            self.filepath,
            self.indent_lvl,
            if self.find_str.is_empty() {
                "".to_string()
            } else {
                format!("(?: {:?}) ", self.find_str)
            },
            if self.replace_str.is_empty() {
                "".to_string()
            } else {
                format!("(-> {:?}) ", self.replace_str)
            },
            self.lastact,
            pretty_str_event(&event),
        );
        bottom_bar.truncate(width);
        tb_printed.push_str(format!("{: <width$}", bottom_bar).as_str());
        tb_printed.push_str(
            format!(
                "{: <width$}",
                format!(
                    "{}{}",
                    self.mode,
                    if self.mode.show_temp() {
                        format!(": {}", self.temp_str)
                    } else {
                        "".to_string()
                    }
                )
            )
            .as_str(),
        );
        print!("{}", tb_printed);
    }
}
