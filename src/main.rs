mod buffer;
use buffer::*;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
};

use std::{
    env::args,
    io::{self, Write},
    time::Duration,
};

#[must_use]
pub fn get_matching_brace(ch: char) -> char {
    match ch {
        '[' => ']',
        '{' => '}',
        '(' => ')',
        c => c,
    }
}

macro_rules! autopair {
    ($buffer: ident, $char: expr, $($open: expr, $close: expr);*) => {
        match $char { $(
            $open => {
                if $buffer.contents[$buffer.cursor_pos.line]
                    .chars()
                    .nth($buffer.cursor_pos.idx)
                    .unwrap_or(' ')
                    .is_whitespace()
                {
                    $buffer.type_char($close);
                    $buffer.move_left();
                }
                $buffer.indent_lvl += 1;
            }
            $close => {
                if $buffer.contents[$buffer.cursor_pos.line].chars().nth($buffer.cursor_pos.idx) == Some($close) {
                    $buffer.backspace();
                    $buffer.move_right();
                }
                if $buffer.indent_lvl != 0 {
                   $buffer.indent_lvl -= 1
                }
            }
        ),*
            _ => {}
        }
    }
}

pub enum Mods {
    None,
    Ctrl,
    Alt,
    CtrlAlt,
}

impl Mods {
    fn to_mods(has_alt: bool, has_ctrl: bool) -> Mods {
        if has_alt {
            if has_ctrl { Mods::CtrlAlt } else { Mods::Alt }
        } else if has_ctrl {
            Mods::Ctrl
        } else {
            Mods::None
        }
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
            let _width = widthu as usize;
            let height = heightu as usize;
            let event = event::read().unwrap();
            if let Event::Key(key) = event {
                if key.kind != event::KeyEventKind::Release {
                    let mods = key.modifiers.iter();
                    let mut has_alt = false;
                    let mut has_ctrl = false;
                    mods.for_each(|m| {
                        if m == KeyModifiers::ALT {
                            has_alt = true;
                        }
                        if m == KeyModifiers::CONTROL {
                            has_ctrl = true;
                        }
                    });
                    let modifiers = Mods::to_mods(has_alt, has_ctrl);
                    match modifiers {
                        Mods::None => match key.code {
                            KeyCode::Esc => {
                                buf.mode = Mode::Default;
                            }
                            KeyCode::Backspace => match buf.mode {
                                Mode::Find => {
                                    buf.find_str.pop();
                                }
                                Mode::ReplaceStr => {
                                    buf.replace_str.pop();
                                }
                                m if m.show_temp() => {
                                    buf.temp_str.pop();
                                }
                                _ => {
                                    buf.backspace();
                                }
                            },
                            KeyCode::Delete => {
                                buf.move_right();
                                buf.backspace();
                            }
                            KeyCode::Enter => match buf.mode {
                                Mode::Find | Mode::ReplaceStr => {
                                    buf.mode = Mode::Default;
                                }
                                Mode::Switch => {
                                    buf.mode = Mode::from_str(&buf.temp_str);
                                    buf.temp_str.clear();
                                }
                                Mode::Goto => {
                                    if let Ok(lineno) = buf.temp_str.parse::<usize>() {
                                        if lineno < buf.contents.len() {
                                            if lineno != 0 {
                                                buf.cursor_pos.line = lineno - 1;
                                            } else {
                                                buf.cursor_pos.line = 0;
                                            }
                                            if buf.contents[buf.cursor_pos.line].len()
                                                < buf.cursor_pos.idx
                                            {
                                                buf.cursor_pos.idx =
                                                    buf.contents[buf.cursor_pos.line].len();
                                            }
                                        }
                                    }
                                    buf.mode = Mode::Default;
                                }
                                Mode::OpenFile => {
                                    buf.save();
                                    buf.filepath = buf.temp_str.clone();
                                    buf.reload_file();
                                    buf.mode = Mode::Default;
                                }
                                _ => {
                                    if let Some('}' | ']' | ')') = buf.contents[buf.cursor_pos.line]
                                        .chars()
                                        .nth(buf.cursor_pos.idx)
                                    {
                                        if buf.indent_lvl != 0 {
                                            buf.indent_lvl -= 1;
                                        }
                                        if buf.cursor_pos.idx != 0 {
                                            let linect: String = buf.contents[buf.cursor_pos.line]
                                                .drain(buf.cursor_pos.idx..)
                                                .collect();
                                            buf.newline_below(linect);
                                        }
                                        buf.move_up();
                                        buf.indent_lvl += 1;
                                        buf.newline_below(String::new());
                                    } else {
                                        let linect: String = buf.contents[buf.cursor_pos.line]
                                            .drain(buf.cursor_pos.idx..)
                                            .collect();
                                        buf.newline_below(linect);
                                    }
                                }
                            },
                            KeyCode::Char(c) => {
                                match buf.mode {
                                    Mode::Find => {
                                        buf.find_str.push(c);
                                    }
                                    Mode::ReplaceStr => {
                                        buf.replace_str.push(c);
                                    }
                                    m if m.show_temp() => {
                                        buf.temp_str.push(c);
                                    }
                                    _ => {
                                        buf.type_char(c);
                                    }
                                }
                                match c {
                                    '{' | '}' | '[' | ']' | '(' | ')'
                                        if buf.mode != Mode::Paste =>
                                    {
                                        autopair!(
                                            buf, c,
                                            '{', '}';
                                            '[', ']';
                                            '(', ')'
                                        );
                                    }
                                    _ => {}
                                }
                                if buf.mode == Mode::Replace {
                                    buf.move_right();
                                    buf.backspace();
                                }
                            }
                            KeyCode::Left => {
                                buf.move_left();
                            }
                            KeyCode::Right => {
                                buf.move_right();
                            }
                            KeyCode::Up => {
                                buf.move_up();
                            }
                            KeyCode::Down => {
                                buf.move_down();
                            }
                            KeyCode::Home => {
                                buf.cursor_pos.idx = 0;
                            }
                            KeyCode::End => {
                                if !buf.contents[buf.cursor_pos.line].is_empty() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
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
                        Mods::Alt => match key.code {
                            KeyCode::Char('q') => {
                                break 'ed;
                            }
                            KeyCode::Char('s') => {
                                buf.save();
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
                                buf.move_up();
                            }
                            KeyCode::Char('a') => {
                                buf.move_down();
                            }
                            KeyCode::Char('u') => {
                                if buf.cursor_pos.line >= height {
                                    buf.cursor_pos.line -= height;
                                } else {
                                    buf.cursor_pos.line = 0;
                                }
                                buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                            }
                            KeyCode::Char('d') => {
                                if buf.cursor_pos.line + height >= buf.contents.len() {
                                    buf.cursor_pos.line = buf.contents.len();
                                    if buf.cursor_pos.line != 0 {
                                        buf.cursor_pos.line -= 1;
                                    }
                                } else {
                                    buf.cursor_pos.line += height;
                                }
                                buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
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
                            KeyCode::Char(':') => {
                                let mut currline = buf.contents[buf.cursor_pos.line].chars();
                                let mut spaces = 0;
                                while currline.next() == Some(' ') {
                                    spaces += 1;
                                }
                                buf.indent_lvl = spaces / 4;
                            }
                            KeyCode::Char('/') => {
                                if buf.mode == Mode::Find {
                                    buf.mode = Mode::Default;
                                } else {
                                    buf.mode = Mode::Find;
                                    buf.find_str.clear();
                                }
                            }
                            KeyCode::Char('n') => {
                                if buf.move_right() {
                                    'findfwd: loop {
                                        let prevpos = buf.cursor_pos;
                                        if let Some(p) = buf.contents[buf.cursor_pos.line]
                                            [buf.cursor_pos.idx..]
                                            .find(buf.find_str.as_str())
                                        {
                                            buf.cursor_pos.idx += p;
                                            buf.cursor_pos.idx += buf.find_str.len();
                                            break 'findfwd;
                                        }
                                        buf.cursor_pos.idx = 0;
                                        if !buf.move_down() {
                                            buf.cursor_pos = prevpos;
                                            break 'findfwd;
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('p') => {
                                if buf.move_left() {
                                    'findfwd: loop {
                                        let prevpos = buf.cursor_pos;
                                        if let Some(p) = buf.contents[buf.cursor_pos.line]
                                            [0..buf.cursor_pos.idx]
                                            .rfind(buf.find_str.as_str())
                                        {
                                            buf.cursor_pos.idx = p;
                                            buf.cursor_pos.idx += buf.find_str.len();
                                            break 'findfwd;
                                        }
                                        let stat = buf.move_up();
                                        buf.cursor_pos.idx =
                                            buf.contents[buf.cursor_pos.line].len();
                                        if !stat {
                                            buf.cursor_pos = prevpos;
                                            break 'findfwd;
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('o') => {
                                buf.newline_below(String::new());
                            }
                            KeyCode::Char('O') => {
                                if buf.move_up() {
                                    buf.newline_below(String::new());
                                } else {
                                    buf.contents.insert(0, String::new());
                                }
                            }
                            KeyCode::Char('m') => {
                                buf.move_left();
                                if let Some(markchar) = buf.contents[buf.cursor_pos.line]
                                    .chars()
                                    .nth(buf.cursor_pos.idx)
                                {
                                    _ = buf.marklist.insert(markchar, buf.cursor_pos);
                                }
                                buf.move_right();
                                buf.backspace();
                            }
                            KeyCode::Char('g') => {
                                buf.move_left();
                                if let Some(markchar) = buf.contents[buf.cursor_pos.line]
                                    .chars()
                                    .nth(buf.cursor_pos.idx)
                                {
                                    buf.move_right();
                                    buf.backspace();
                                    if let Some(&loc) = buf.marklist.get(&markchar) {
                                        _ = buf.marklist.insert('_', buf.cursor_pos);
                                        buf.cursor_pos = loc;
                                    }
                                }
                            }
                            KeyCode::Char('x') => {
                                buf.mode = Mode::Switch;
                                buf.temp_str.clear();
                            }
                            KeyCode::Char('h') => {
                                buf.contents[buf.cursor_pos.line].replace_range(
                                    (if buf.cursor_pos.idx >= buf.find_str.len() {
                                        buf.cursor_pos.idx - buf.find_str.len()
                                    } else {
                                        0
                                    })..buf.cursor_pos.idx,
                                    &buf.replace_str,
                                );
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
                            }
                            KeyCode::Char('r') => {
                                if buf.mode == Mode::ReplaceStr {
                                    buf.mode = Mode::Default;
                                } else {
                                    buf.mode = Mode::ReplaceStr;
                                    buf.replace_str.clear();
                                }
                            }
                            KeyCode::Char('j') => {
                                if buf.contents.len() > buf.cursor_pos.line + 1 {
                                    let l = buf.contents.remove(buf.cursor_pos.line + 1);
                                    buf.contents[buf.cursor_pos.line].push(' ');
                                    buf.contents[buf.cursor_pos.line].push_str(l.trim());
                                }
                            }
                            KeyCode::Char('0') => {
                                buf.indent_lvl = 0;
                            }
                            KeyCode::Char('k') => {
                                buf.contents[buf.cursor_pos.line]
                                    .replace_range((buf.indent_lvl * 4)..buf.cursor_pos.idx, "");
                                buf.cursor_pos.idx = buf.indent_lvl * 4;
                            }
                            KeyCode::Char('K') => {
                                buf.contents[buf.cursor_pos.line].truncate(buf.cursor_pos.idx);
                            }
                            _ => {}
                        },
                        Mods::Ctrl => match key.code {
                            KeyCode::Char('r') => {
                                buf.reload_file();
                            }
                            KeyCode::Backspace => {
                                while buf.backspace().unwrap_or('a').is_whitespace() {}
                                let mut last = ' ';
                                'killword: loop {
                                    last = buf.backspace().unwrap_or(' ');
                                    if !last.is_alphanumeric() {
                                        break 'killword;
                                    }
                                }
                                if last != '\n' {
                                    buf.type_char(last);
                                }
                            }
                            _ => {}
                        },
                        Mods::CtrlAlt => {}
                    }
                }
            }
            buf.print(event);
            stdout.flush().unwrap();
            buf.lastact = Action::None;
        }
    }
    print!("\x1bc");
    buf.save();
    _ = terminal::disable_raw_mode();
}