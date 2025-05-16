//! Buffer and cursor handling module.

use crate::languages;
use crate::snippets;
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
    /// Copy mode.
    Copy,
    /// Kill lines mode.
    KillLines,
    /// Snippet mode.
    Snippet,
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
            "copy" | "c" => Mode::Copy,
            "snippet" | "sn" => Mode::Snippet,
            "killlines" | "kl" | "k" => Mode::KillLines,
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
            Mode::Copy => write!(f, "copying (from -> to)"),
            Mode::Snippet => write!(f, "snippet request"),
            Mode::KillLines => write!(f, "Killing lines (from -> to"),
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
            Goto | Switch | OpenFile | Copy | Snippet | KillLines => true,
        }
    }
}

pub fn style_time(t: u128) -> String {
    if t < 16666 {
        format!("\x1b[32m{}\x1b[0m", 1_000_000 / (t + 1)) // > 60
    } else if t < 50000 {
        format!("\x1b[33m{}\x1b[0m", 1_000_000 / (t + 1)) // 60 ~ 20
    } else {
        format!("\x1b[31m{}\x1b[0m", 1_000_000 / (t + 1)) // < 20
    }
}

pub fn style_time_raw(t: u128) -> String {
    if t < 16666 {
        format!("\x1b[32m{}\x1b[0m", t) // > 60
    } else if t < 50000 {
        format!("\x1b[33m{}\x1b[0m", t) // 60 ~ 20
    } else {
        format!("\x1b[31m{}\x1b[0m", t) // < 20
    }
}

pub enum BimVar {
    Bool(bool),
    Str(String),
}

impl fmt::Display for BimVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BimVar::Bool(x) => write!(f, "{x}"),
            BimVar::Str(x) => write!(f, "{x}"),
        }
    }
}

/// Structure for storing the current displayed buffer.
pub struct Buffer {
    /// Contents of the file, as lines. This is what is modified.
    pub contents: Vec<String>,
    /// Syntax highlighted contents. This is what is shown.
    pub highlighted_contents: Vec<Vec<languages::StyledChar>>,
    /// Time taken per iteration in microseconds.
    pub iter_time: u128,
    /// Position of the cursor.
    pub cursor_pos: Cursor,
    /// Line number of the top line shown.
    pub top: usize,
    /// Filepath of the file currently being edited.
    pub filepath: String,
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
    /// Current snippets used.
    pub snippets: Box<dyn snippets::Snippet>,
    /// Local vars.
    pub vars: HashMap<String, BimVar>,
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
        let lang = if contents[0].contains("use-ext:") {
            languages::get_lang(&contents[0])
        } else {
            languages::get_lang(filepath)
        };
        let snippets = if contents[0].contains("use-ext:") {
            snippets::get_snippets(&contents[0])
        } else {
            snippets::get_snippets(filepath)
        };
        let initvars = HashMap::from([
            ("lastact".to_string(), BimVar::Str(String::new())),
            ("showbottombar".to_string(), BimVar::Bool(true)),
            (
                "line-num-type".to_string(),
                BimVar::Str(String::from("absolute")),
            ),
            ("changed".to_string(), BimVar::Bool(true)),
        ]);
        let highlighted_contents = lang.highlight(&contents);
        Buffer {
            contents: contents.clone(),
            highlighted_contents,
            iter_time: 0,
            top: 0,
            vars: initvars,
            cursor_pos: Cursor { line: 0, idx: 0 },
            filepath: filepath.to_string(),
            find_str: String::new(),
            replace_str: String::new(),
            temp_str: String::new(),
            marklist: HashMap::new(),
            indent_lvl: 0,
            lang,
            snippets,
            mode: Mode::Default,
        }
    }

    /// Updates highlighting. Performance cost varies. Call as infrequently as possible.
    #[inline]
    pub fn update_highlighting(&mut self) {
        *self.vars.get_mut("changed").unwrap() = BimVar::Bool(true);
        self.highlighted_contents = self.lang.highlight(&self.contents);
    }

    #[inline]
    pub fn adjust_indent(&mut self) {
        let mut original = String::new();
        for _ in 0..self.indent_lvl * self.lang.indent_size() {
            original.push(' ');
        }
        original.push_str(self.contents[self.cursor_pos.line].trim());
        self.contents[self.cursor_pos.line] = original;
        self.update_highlighting();
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
            if let Some(BimVar::Bool(changed)) = self.vars.get_mut("changed") {
                if *changed {
                    *changed = false;
                    let trimmedlines: Vec<&str> =
                        self.contents.iter().map(|s| s.trim_end()).collect();
                    let mut writecontent = trimmedlines.join("\n");
                    writecontent.push('\n');
                    _ = fs::write(self.filepath.clone(), writecontent);
                    _ = self
                        .vars
                        .insert(String::from("lastact"), BimVar::Str(String::from("save")));
                }
            }
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

    // This doesn't reload the highlighting, so use at your own risk.
    pub fn fast_backspace(&mut self) -> Option<char> {
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
                Some(self.contents[0].remove(0))
            } else {
                None
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
        self.lang = if self.contents[0].contains("use-ext:") {
            languages::get_lang(&self.contents[0])
        } else {
            languages::get_lang(&self.filepath)
        };
        self.snippets = if self.contents[0].contains("use-ext:") {
            snippets::get_snippets(&self.contents[0])
        } else {
            snippets::get_snippets(&self.filepath)
        };
        self.update_highlighting();
        self.save();
    }

    pub fn print(&mut self, event: &event::Event) {
        print!("\x1b[J\x1b[H");
        let (widthu, heightu) = terminal::size().unwrap();
        let width = widthu as usize;
        // the width calculations are wrong on posix
        // uncomment the next line on posix:
        // width -= 1;

        let showbottombar = matches!(self.vars.get("showbottombar"), Some(BimVar::Bool(true)));

        let height = heightu as usize;
        let mut bottom_pad = if showbottombar { 2 } else { 0 };
        if cfg!(feature = "profile") {
            bottom_pad += 1;
        }
        if self.cursor_pos.line > self.top + height - bottom_pad - 3 {
            self.top = self.cursor_pos.line + bottom_pad + 3 - height;
        }

        let top_pad = 3;
        if self.cursor_pos.line < self.top + top_pad && self.cursor_pos.line >= top_pad {
            self.top = self.cursor_pos.line - 3;
        }
        if self.cursor_pos.line <= top_pad {
            self.top = 0;
        }
        let mut tb_printed = String::new();

        #[derive(PartialEq, Debug)]
        enum LineNumType {
            None,
            Absolute,
            Relative,
        }

        let content = &self.highlighted_contents;
        let indent_size = self.lang.indent_size();
        let spaces = 2;
        let mut sidesize = spaces;
        let mut lenfile = content.len();
        while lenfile != 0 {
            sidesize += 1;
            lenfile /= 10;
        }
        let numsize = sidesize - spaces;

        let linetype = match self.vars.get("line-num-type") {
            Some(BimVar::Str(s)) => match s.as_str() {
                "absolute" => LineNumType::Absolute,
                "relative" => LineNumType::Relative,
                _ => LineNumType::None,
            },
            _ => LineNumType::None,
        };
        let truewidth = if linetype != LineNumType::None {
            width - sidesize
        } else {
            width
        };

        let mut linectr = self.top;
        let mut linesprinted = 0;
        while linectr < self.top + height - bottom_pad && linectr < content.len() {
            linesprinted += 1;
            if linetype != LineNumType::None {
                if linectr == self.cursor_pos.line {
                    _ = write!(
                        &mut tb_printed,
                        "\x1b[36m{: >numsize$}  \x1b[0m",
                        linectr + 1
                    );
                } else if linetype == LineNumType::Relative {
                    if linectr > self.cursor_pos.line {
                        _ = write!(
                            &mut tb_printed,
                            "\x1b[2m\x1b[36m{: >numsize$}  \x1b[0m",
                            linectr - self.cursor_pos.line
                        );
                    } else {
                        _ = write!(
                            &mut tb_printed,
                            "\x1b[2m\x1b[36m{: >numsize$}  \x1b[0m",
                            self.cursor_pos.line - linectr
                        );
                    }
                } else {
                    _ = write!(
                        &mut tb_printed,
                        "\x1b[2m\x1b[36m{: >numsize$}  \x1b[0m",
                        linectr + 1
                    );
                }
            }
            if linectr == self.cursor_pos.line {
                let mut i = 0;
                for content in content[self.cursor_pos.line].iter().take(truewidth) {
                    let id = self.indent_lvl * indent_size;
                    match i {
                        a if a == self.cursor_pos.idx => {
                            _ = write!(&mut tb_printed, "\x1b[47m\x1b[30m{content}\x1b[0m");
                        }
                        b if b == id => {
                            if content.ch == ' ' {
                                _ = write!(&mut tb_printed, "\x1b[2;33m|\x1b[0m");
                            } else {
                                _ = write!(&mut tb_printed, "\x1b[33m{content}\x1b[0m");
                            }
                        }
                        _ => {
                            _ = write!(&mut tb_printed, "{content}");
                        }
                    }
                    i += 1;
                }
                while i < truewidth {
                    let id = self.indent_lvl * indent_size;
                    match i {
                        a if a == self.cursor_pos.idx => {
                            tb_printed.push_str("\x1b[47m\x1b[30m \x1b[0m");
                        }
                        b if b == id => {
                            tb_printed.push_str("\x1b[2;33m|\x1b[0m");
                        }
                        _ => {
                            tb_printed.push(' ');
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
                for c in content[linectr].iter() {
                    _ = write!(&mut tb_printed, "{c}");
                    i += 1;
                }
                while i < truewidth {
                    tb_printed.push(' ');
                    i += 1;
                }
                tb_printed.push('\n');
            }
            linectr += 1;
        }

        while linesprinted + bottom_pad < height {
            for _ in 0..width {
                tb_printed.push(' ');
            }
            tb_printed.push('\n');
            linesprinted += 1;
        }

        if showbottombar {
            let mut bottom_bar = format!(
                "[{}] {}{}(act: {}) [{}; {}] ({: <12} fps) {}",
                self.filepath,
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
                self.vars.get("lastact").unwrap(),
                self.lang.display_str(),
                self.snippets.display_str(),
                style_time(self.iter_time),
                pretty_str_event(event),
            );
            let escape_code_size = 5;
            if bottom_bar.len() > width + escape_code_size {
                bottom_bar.truncate(width + escape_code_size);
            }
            _ = write!(&mut tb_printed, "{bottom_bar: <width$}\x1b[0m");
            tb_printed.push('\n');
            _ = write!(
                &mut tb_printed,
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
            );
        } else {
            tb_printed.pop();
        }
        print!("{tb_printed}");
    }
}
