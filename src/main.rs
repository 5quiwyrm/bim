mod buffer;
use buffer::*;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
};

use std::{
    env::args,
    fs,
    io::{self, Write},
    time::Duration,
};

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
            match event {
                Event::Key(key) => {
                    if key.kind != event::KeyEventKind::Release {
                        match key.modifiers {
                            KeyModifiers::NONE | KeyModifiers::SHIFT => match key.code {
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
                                    _ => {
                                        buf.backspace();
                                    }
                                },
                                KeyCode::Delete => {
                                    buf.move_right();
                                    buf.backspace();
                                }
                                KeyCode::Enter => {
                                    if buf.mode == Mode::Find || buf.mode == Mode::ReplaceStr {
                                        buf.mode = Mode::Default;
                                    } else {
                                        match buf.contents[buf.cursor_pos.line]
                                            .chars()
                                            .nth(buf.cursor_pos.idx)
                                        {
                                            Some('}') | Some(']') | Some(')') => {
                                                if buf.indent_lvl != 0 {
                                                    buf.indent_lvl -= 1;
                                                }
                                                if buf.cursor_pos.idx != 0 {
                                                    let linect: String = buf.contents
                                                        [buf.cursor_pos.line]
                                                        .drain(buf.cursor_pos.idx..)
                                                        .collect();
                                                    buf.newline_below(linect);
                                                }
                                                buf.move_up();
                                                buf.indent_lvl += 1;
                                                buf.newline_below("".to_string());
                                            }
                                            _ => {
                                                let linect: String = buf.contents
                                                    [buf.cursor_pos.line]
                                                    .drain(buf.cursor_pos.idx..)
                                                    .collect();
                                                buf.newline_below(linect);
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char(c) => {
                                    match buf.mode {
                                        Mode::Find => {
                                            buf.find_str.push(c);
                                        }
                                        Mode::ReplaceStr => {
                                            buf.replace_str.push(c);
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
                                            )
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
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len()
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
                                            } else {
                                                buf.cursor_pos.idx = 0;
                                                if !buf.move_down() {
                                                    buf.cursor_pos = prevpos;
                                                    break 'findfwd;
                                                }
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
                                            } else {
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
                                }
                                KeyCode::Char('o') => {
                                    buf.newline_below("".to_string());
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
                                    buf.move_left();
                                    if let Some(markchar) = buf.contents[buf.cursor_pos.line]
                                        .chars()
                                        .nth(buf.cursor_pos.idx)
                                    {
                                        buf.move_right();
                                        buf.backspace();
                                        buf.mode = match markchar {
                                            'p' => Mode::Paste,
                                            'r' => Mode::Replace,
                                            'f' => Mode::Find,
                                            _ => Mode::Default,
                                        }
                                    }
                                }
                                KeyCode::Char('h') => {
                                    buf.contents[buf.cursor_pos.line].replace_range(
                                        (if buf.cursor_pos.idx >= buf.find_str.len() {
                                            buf.cursor_pos.idx - buf.find_str.len()
                                        } else {
                                            0
                                        })
                                            ..buf.cursor_pos.idx,
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

                                _ => {}
                            },
                            KeyModifiers::CONTROL => match key.code {
                                KeyCode::Char('r') => {
                                    buf.contents = fs::read_to_string(&buf.filepath)
                                        .unwrap()
                                        .lines()
                                        .map(|s| s.to_string())
                                        .collect();
                                    if buf.contents.len() < buf.cursor_pos.line {
                                        buf.cursor_pos.line = if buf.contents.is_empty() {
                                            0
                                        } else {
                                            buf.contents.len() - 1
                                        };
                                    }
                                    if buf.contents[buf.cursor_pos.line].len() < buf.cursor_pos.idx
                                    {
                                        buf.cursor_pos.idx =
                                            buf.contents[buf.cursor_pos.line].len();
                                    }
                                    buf.save();
                                }
                                KeyCode::Char('t') => {
                                    buf.top = buf.cursor_pos.line;
                                }
                                KeyCode::Char('g') => {}
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
            buf.print();
            stdout.flush().unwrap();
            buf.lastact = Action::None;
        }
    }
    print!("\x1bc");
    buf.save();

    _ = terminal::disable_raw_mode();
}