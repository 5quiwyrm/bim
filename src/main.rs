use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyEventState, KeyModifiers},
    terminal,
};

use std::{
    env::args,
    fs,
    io::{self, Write},
    path::Path,
    time::Duration,
};

struct Cursor {
    line: usize,
    idx: usize,
}

enum Action {
    None,
    Save,
}

struct Buffer {
    contents: Vec<String>,
    cursor_pos: Cursor,
    top: usize,
    filepath: String,
    lastact: Action,
    indent_lvl: usize,
}

impl Buffer {
    fn new(filepath: String) -> Self {
        let contents: Vec<String> = fs::read_to_string(&filepath)
            .unwrap_or({
                fs::File::create(&filepath).unwrap();
                "".to_string()
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
            indent_lvl: 0,
        }
    }

    fn move_left(&mut self) {
        if self.cursor_pos.idx == 0 {
            if self.cursor_pos.line != 0 {
                self.cursor_pos.line -= 1;
                self.cursor_pos.idx = self.contents[self.cursor_pos.line].len();
            }
        } else {
            self.cursor_pos.idx -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_pos.idx == self.contents[self.cursor_pos.line].len()
            || self.contents[self.cursor_pos.line].is_empty()
        {
            if self.cursor_pos.line + 1 != self.contents.len() && !self.contents.is_empty() {
                self.cursor_pos.line += 1;
                self.cursor_pos.idx = 0;
            }
        } else {
            self.cursor_pos.idx += 1;
        }
    }

    fn save(&mut self) {
        let trimmedlines: Vec<&str> = self.contents.iter().map(|s| s.trim_end()).collect();
        fs::write(self.filepath.clone(), trimmedlines.join("\n")).unwrap();
        self.lastact = Action::Save;
    }

    fn print(&mut self) {
        let (widthu, heightu) = terminal::size().unwrap();
        let width = widthu as usize;
        let height = heightu as usize;
        if self.cursor_pos.line > self.top + height - 5 {
            self.top = self.cursor_pos.line - height + 5;
        }
        if self.cursor_pos.line < self.top {
            self.top = self.cursor_pos.line;
        }

        let mut linectr = self.top;
        while linectr < self.top + height - 3 && linectr < self.contents.len() {
            if linectr == self.cursor_pos.line {
                let mut i = 0;
                let mut line_content = self.contents[self.cursor_pos.line].chars();
                while i < width {
                    let content = line_content.next().unwrap_or(' ');
                    if i == self.cursor_pos.idx {
                        print!("\x1b[47m\x1b[30m{content}\x1b[0m");
                    } else {
                        print!("{content}");
                    }
                    i += 1;
                }
                println!();
            } else {
                println!("{: <width$}", self.contents[linectr]);
            }
            linectr += 1;
        }
        println!(
            "{: <width$}",
            format!(
                "({}, {}) [{}] (> x {})",
                self.cursor_pos.line, self.cursor_pos.idx, self.filepath, self.indent_lvl,
            )
        );
        println!(
            "{: <width$}",
            match self.lastact {
                Action::Save => "saved!",
                _ => "",
            }
        );
    }
}

fn main() {
    let mut stdout = io::stdout();
    let mut args = args();
    _ = args.next();
    let path = args.next().unwrap_or("./src/main.rs".to_string());
    let mut buf = Buffer::new(path);
    print!("\x1bc");
    _ = terminal::enable_raw_mode();
    buf.save();
    'ed: loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            let (widthu, heightu) = terminal::size().unwrap();
            let width = widthu as usize;
            let height = heightu as usize;
            let event = event::read().unwrap();
            match event {
                Event::Key(key) => {
                    if key.kind != event::KeyEventKind::Release {
                        match key.modifiers {
                            KeyModifiers::NONE | KeyModifiers::SHIFT => match key.code {
                                KeyCode::Backspace => {
                                    let t = buf.cursor_pos.idx;
                                    if t == 0 {
                                        if buf.cursor_pos.line != 0 {
                                            let currline =
                                                buf.contents[buf.cursor_pos.line].clone();
                                            let oldlen =
                                                buf.contents[buf.cursor_pos.line - 1].len();
                                            buf.contents[buf.cursor_pos.line - 1]
                                                .push_str(&currline);
                                            buf.contents.remove(buf.cursor_pos.line);
                                            buf.cursor_pos.line -= 1;
                                            buf.cursor_pos.idx = oldlen;
                                        } else if !buf.contents[0].is_empty() {
                                            buf.contents[0].remove(0);
                                        }
                                    } else {
                                        buf.contents[buf.cursor_pos.line].remove(t - 1);
                                        buf.cursor_pos.idx -= 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    let mut newline = String::new();
                                    for _ in 0..(buf.indent_lvl * 4) {
                                        newline.push(' ');
                                    }
                                    let linect: String = buf.contents[buf.cursor_pos.line]
                                        .drain(buf.cursor_pos.idx..)
                                        .collect();
                                    newline.push_str(&linect);
                                    buf.cursor_pos.line += 1;
                                    buf.cursor_pos.idx = buf.indent_lvl * 4;
                                    buf.contents.insert(buf.cursor_pos.line, newline);
                                }
                                KeyCode::Char(c) => {
                                    if c == '{' || c == '[' {
                                        buf.indent_lvl += 1;
                                    }
                                    if (c == '}' || c == ']') && buf.indent_lvl != 0 {
                                        buf.indent_lvl -= 1;
                                    }
                                    buf.contents[buf.cursor_pos.line].insert(buf.cursor_pos.idx, c);
                                    buf.cursor_pos.idx += 1;
                                }
                                KeyCode::Left => {
                                    buf.move_left();
                                }
                                KeyCode::Right => {
                                    buf.move_right();
                                }
                                KeyCode::Up => {
                                    if buf.cursor_pos.line != 0 {
                                        buf.cursor_pos.line -= 1;
                                        if buf.cursor_pos.idx
                                            >= buf.contents[buf.cursor_pos.line].len()
                                        {
                                            let l = buf.contents[buf.cursor_pos.line].len();
                                            buf.cursor_pos.idx = if l != 0 { l - 1 } else { 0 };
                                        }
                                    } else {
                                        buf.cursor_pos.idx = 0;
                                    }
                                }
                                KeyCode::Down => {
                                    if buf.cursor_pos.line + 1 != buf.contents.len()
                                        && !buf.contents.is_empty()
                                    {
                                        buf.cursor_pos.line += 1;
                                        if buf.cursor_pos.idx
                                            >= buf.contents[buf.cursor_pos.line].len()
                                        {
                                            let l = buf.contents[buf.cursor_pos.line].len();
                                            buf.cursor_pos.idx = if l != 0 { l - 1 } else { 0 };
                                        }
                                    } else {
                                        let l = buf.contents[0].len();
                                        buf.cursor_pos.idx = if l != 0 { l - 1 } else { 0 };
                                    }
                                }
                                KeyCode::Home => {
                                    buf.cursor_pos.idx = 0;
                                }
                                KeyCode::End => {
                                    if !buf.contents[buf.cursor_pos.line].is_empty() {
                                        buf.cursor_pos.idx =
                                            buf.contents[buf.cursor_pos.line].len();
                                    }
                                }
                                KeyCode::Tab => {
                                    buf.contents[buf.cursor_pos.line]
                                        .insert_str(buf.cursor_pos.idx, "    ");
                                    buf.cursor_pos.idx += 4;
                                    buf.indent_lvl += 1;
                                }
                                _ => {}
                            },
                            KeyModifiers::ALT => match key.code {
                                KeyCode::Char('q') => {
                                    break 'ed;
                                }
                                KeyCode::Char('s') => {
                                    let trimmedlines: Vec<&str> =
                                        buf.contents.iter().map(|s| s.trim_end()).collect();
                                    fs::write(buf.filepath.clone(), trimmedlines.join("\n"))
                                        .unwrap();
                                    buf.lastact = Action::Save;
                                }
                                KeyCode::Char('b') => {
                                    let linect = buf.contents.len();
                                    buf.cursor_pos.line = if linect == 0 { 0 } else { linect - 1 };
                                    buf.cursor_pos.idx = 0;
                                }
                                KeyCode::Char('t') => {
                                    buf.cursor_pos.line = 0;
                                    buf.cursor_pos.idx = 0;
                                }
                                KeyCode::Char('c') => {
                                    buf.move_left();
                                }
                                KeyCode::Char('i') => {
                                    buf.move_right();
                                }
                                KeyCode::Char('e') => {
                                    if buf.cursor_pos.line != 0 {
                                        buf.cursor_pos.line -= 1;
                                        if buf.cursor_pos.idx
                                            >= buf.contents[buf.cursor_pos.line].len()
                                        {
                                            let l = buf.contents[buf.cursor_pos.line].len();
                                            buf.cursor_pos.idx = if l != 0 { l - 1 } else { 0 };
                                        }
                                    } else {
                                        buf.cursor_pos.idx = 0;
                                    }
                                }
                                KeyCode::Char('a') => {
                                    if buf.cursor_pos.line + 1 < buf.contents.len()
                                        && !buf.contents.is_empty()
                                    {
                                        buf.cursor_pos.line += 1;
                                        if buf.cursor_pos.idx
                                            >= buf.contents[buf.cursor_pos.line].len()
                                        {
                                            let l = buf.contents[buf.cursor_pos.line].len();
                                            buf.cursor_pos.idx = if l != 0 { l - 1 } else { 0 };
                                        }
                                    } else {
                                        let l = buf.contents[0].len();
                                        buf.cursor_pos.idx = if l != 0 { l - 1 } else { 0 };
                                    }
                                }
                                KeyCode::Char('u') => {
                                    if buf.cursor_pos.line >= height {
                                        buf.cursor_pos.line -= height;
                                    } else {
                                        buf.cursor_pos.line = 0;
                                    }
                                }
                                KeyCode::Char('d') => {
                                    if buf.cursor_pos.line + height >= buf.contents.len() {
                                        buf.cursor_pos.line = buf.contents.len();
                                    } else {
                                        buf.cursor_pos.line += height;
                                    }
                                }
                                KeyCode::Char('l') => {
                                    buf.contents.remove(buf.cursor_pos.line);
                                    buf.cursor_pos.idx = 0;
                                    if buf.cursor_pos.line != 0 {
                                        buf.cursor_pos.line -= 1;
                                    }
                                }
                                KeyCode::Char(',') => {
                                    if buf.indent_lvl != 0 {
                                        buf.indent_lvl -= 1;
                                    }
                                }
                                KeyCode::Char('.') => {
                                    buf.indent_lvl += 1;
                                }
                                KeyCode::Char(';') => {
                                    let mut currline = buf.contents[buf.cursor_pos.line].chars();
                                    let mut spaces = 0;
                                    while currline.next() == Some(' ') {
                                        spaces += 1;
                                    }
                                    buf.indent_lvl = spaces / 4;
                                    buf.cursor_pos.idx = spaces;
                                }
                                KeyCode::Char('f') => {}
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
                Event::Paste(s) => {
                    buf.contents[buf.cursor_pos.line].insert_str(buf.cursor_pos.idx, &s);
                }
                _ => {}
            }
            print!("\x1b[J\x1b[H");
            buf.print();
            stdout.flush().unwrap();
            buf.lastact = Action::None;
        }
    }
    print!("\x1bc");
    buf.save();

    _ = terminal::disable_raw_mode();
}