//! Buffer and cursor handling module.

use crate::languages;
use crossterm::{event, terminal};
use std::{
    collections::HashMap,
    fmt::{self, Write},
    fs,
};

/// Prettifies events to string for printing.
pub fn pretty_str_event(event: &event::Event) -> String {
    if let event::Event::Key(key) = event {
        if key.modifiers == event::KeyModifiers::NONE {
            format!("{}", key.code)
        } else {
            format!("{} {}", key.modifiers, key.code)
        }
    } else {
        String::new()
    }
}

/// Structure for storing cursor position.
#[derive(Copy, Clone)]
pub struct Cursor {
    /// Line number (subtracted by 1).
    pub line: usize,
    /// Index in line (subtracted by 1);
    pub idx: usize,
}

/// Structure for actions. Will be modified in the future.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Action {
    /// Default state.
    None,
    /// Successfully saved file.
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

/// Enum for the mode of the program. Directly affects the behaviour of the program.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    /// Default mode.
    Default,
    /// Paste mode.
    Paste,
    /// Replacement mode.
    Replace,
    /// Find mode.
    Find,
    /// Replace string mode.
    ReplaceStr,
    /// Goto mode.
    Goto,
    /// Openfile mode.
    OpenFile,
    /// Mode switching mode.
    Switch,
}

impl Mode {
    /// Primarily used for switching modes.
    /// To add more aliases for modes, the match statement below should be changed.
    pub fn from_string(s: &str) -> Mode {
        match s.trim() {
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
    /// Determines whether the `temp_str` buffer should be shown.
    pub fn show_temp(self) -> bool {
        use Mode::*;
        match self {
            Default | Paste | Replace | Find | ReplaceStr => false,
            Goto | Switch | OpenFile => true,
        }
    }
}

/// Structure for storing the current displayed buffer.
pub struct Buffer {
    /// Contents of the file, as lines. This is what is modified.
    pub contents: Vec<String>,
    /// Syntax highlighted contents. This is what is shown.
    pub highlighted_contents: Vec<Vec<languages::StyledChar>>,
    /// Position of the cursor.
    pub cursor_pos: Cursor,
    /// Line number of the top line shown.
    pub top: usize,
    /// Filepath of the file currently being edited.
    pub filepath: String,
    /// Last action taken.
    pub lastact: Action,
    /// String to be found when using `M-n` and `M-p`.
    pub find_str: String,
    /// String to replace by when using `M-h`.
    pub replace_str: String,
    /// Temporary buffer for all purposes.
    pub temp_str: String,
    /// Hashmap containing the locations of marks.
    pub marklist: HashMap<char, Cursor>,
    /// Current indent level. This is language agnostic.
    pub indent_lvl: usize,
    /// Current language. Used for determining indent size and highlighting.
    pub lang: Box<dyn languages::Language>,
    /// Current mode.
    pub mode: Mode,
}

impl Buffer {
    /// Constructs a new instance of `Buffer` from a filepath.
    pub fn new(filepath: &str) -> Self {
        let contents: Vec<String> = fs::read_to_string(filepath)
            .unwrap_or({
                if filepath != "scratch" {
                    _ = fs::File::create(filepath).map_err(|_| {
                        println!("Illegal filepath, proceeding to scratch buffer...");
                    });
                }
                "\n".to_string()
            })
            .lines()
            .map(|s| s.to_string())
            .collect();
        let lang = languages::get_lang(filepath);
        Buffer {
            contents: contents.clone(),
            highlighted_contents: lang.highlight(&contents),
            top: 0,
            cursor_pos: Cursor { line: 0, idx: 0 },
            filepath: filepath.to_string(),
            lastact: Action::None,
            find_str: String::new(),
            replace_str: String::new(),
            temp_str: String::new(),
            marklist: HashMap::new(),
            indent_lvl: 0,
            lang: languages::get_lang(filepath),
            mode: Mode::Default,
        }
    }

    /// Updates highlighting. Performance cost varies. Call as infrequently as possible.
    #[inline]
    pub fn update_highlighting(&mut self) {
        self.highlighted_contents = self.lang.highlight(&self.contents);
    }

    /// Moves the cursor left, wrapping around lines.
    /// Return value signifies whether the cursor actually moved.
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

    /// Moves the cursor right, wrapping around lines.
    /// Return value signifies whether the cursor actually moved.
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

    /// Moves the cursor up, moving to the end of a line if the index is larger than the length of the line.
    /// Return value signifies whether the cursor actually moved.
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

    /// Moves the cursor down, moving to the end of a line if the index is larger than the length of a line.
    /// Return value signfies whether the cursor actually moved.
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
        if self.filepath != *"scratch" {
            let trimmedlines: Vec<&str> = self.contents.iter().map(|s| s.trim_end()).collect();
            let mut writecontent = trimmedlines.join("\n");
            writecontent.push('\n');
            _ = fs::write(self.filepath.clone(), writecontent);
            self.lastact = Action::Save;
        }
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
                self.update_highlighting();
                Some('\n')
            } else if !self.contents[0].is_empty() {
                let ret = Some(self.contents[0].remove(0));
                self.update_highlighting();
                ret
            } else {
                return None;
            }
        } else {
            self.cursor_pos.idx -= 1;
            let ret = Some(self.contents[self.cursor_pos.line].remove(t - 1));
            self.update_highlighting();
            ret
        }
    }

    #[inline]
    pub fn type_char(&mut self, ch: char) {
        self.contents[self.cursor_pos.line].insert(self.cursor_pos.idx, ch);
        self.cursor_pos.idx += 1;
        self.update_highlighting();
    }

    pub fn newline_below(&mut self, linect: &str) {
        let mut newline = String::new();
        for _ in 0..(self.indent_lvl * self.lang.indent_size()) {
            newline.push(' ');
        }
        newline.push_str(linect);
        self.cursor_pos.line += 1;
        self.cursor_pos.idx = self.indent_lvl * self.lang.indent_size();
        self.contents.insert(self.cursor_pos.line, newline);
        self.update_highlighting();
    }

    pub fn reload_file(&mut self) {
        self.contents = fs::read_to_string(&self.filepath)
            .unwrap_or("\n".to_string())
            .lines()
            .map(|s| s.to_string())
            .collect();
        self.cursor_pos.idx = 0;
        self.cursor_pos.line = 0;
        self.save();
        self.lang = languages::get_lang(&self.filepath);
        self.update_highlighting();
    }

    pub fn print(&mut self, event: &event::Event) {
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

        let content = &self.highlighted_contents;
        let indent_size = self.lang.indent_size();
        let spaces = 2;
        let mut sidesize = spaces;
        let mut lenfile = content.len() + 1;
        while lenfile != 0 {
            sidesize += 1;
            lenfile /= 10;
        }
        let numsize = sidesize - spaces;

        let truewidth = width - sidesize;

        let mut linectr = self.top;
        while linectr < self.top + height - bottom_pad && linectr < content.len() {
            tb_printed.push_str(format!("\x1b[36m{: >numsize$}  \x1b[0m", linectr + 1).as_str());
            if linectr == self.cursor_pos.line {
                let mut i = 0;
                let mut line_content = content[self.cursor_pos.line].iter();
                while i < truewidth {
                    let empty = languages::StyledChar {
                        style: String::new(),
                        ch: ' ',
                    };
                    let content = line_content.next().unwrap_or(&empty);
                    let id = self.indent_lvl * indent_size;
                    match i {
                        a if a == self.cursor_pos.idx => {
                            tb_printed
                                .push_str(format!("\x1b[47m\x1b[30m{content}\x1b[0m").as_str());
                        }
                        b if b == id => {
                            if content.ch == ' ' {
                                tb_printed.push_str("\x1b[33m|\x1b[0m");
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
            } else if content[linectr].len() > truewidth {
                content[linectr].iter().take(truewidth).for_each(|c| {
                    _ = write!(&mut tb_printed, "{c}");
                });
                tb_printed.push('\n');
            } else {
                let mut i = 0;
                let mut linechs = content[linectr].iter();
                while i < truewidth {
                    let empty = languages::StyledChar {
                        style: String::new(),
                        ch: ' ',
                    };
                    tb_printed.push_str(format!("{}", linechs.next().unwrap_or(&empty)).as_str());
                    i += 1;
                }
                tb_printed.push('\n');
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
                String::new()
            } else {
                format!("(?: {:?}) ", self.find_str)
            },
            if self.replace_str.is_empty() {
                String::new()
            } else {
                format!("(-> {:?}) ", self.replace_str)
            },
            self.lastact,
            pretty_str_event(event),
        );
        bottom_bar = format!("{bottom_bar: <width$}");
        tb_printed.push_str(format!("{bottom_bar: <width$}").as_str());
        tb_printed.push('\n');
        tb_printed.push_str(
            format!(
                "{: <width$}",
                format!(
                    "{}{}",
                    self.mode,
                    if self.mode.show_temp() {
                        format!(": {}", self.temp_str)
                    } else {
                        String::new()
                    }
                )
            )
            .as_str(),
        );

        print!("{tb_printed}");
    }
}
