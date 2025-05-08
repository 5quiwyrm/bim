pub mod buffer;
use buffer::*;
pub mod languages;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
};

use std::{
    env::args,
    io::{self, Write},
    time::Instant,
};

/// Returns matching brace for given open brace.
pub fn get_matching_brace(ch: char) -> char {
    match ch {
        '[' => ']',
        '{' => '}',
        '(' => ')',
        c => c,
    }
}

/** Generates match statement to support autopairs.
The macro takes the form of:
```
autopair!(
    buffer,
    character,
    open, close;
    open, close;
    ...
)
```
Note that you should not put a semicolon after the last pair. */
#[macro_export]
macro_rules! autopair {
    ($buffer: ident, $char: expr, $($open: expr, $close: expr);*) => {
        match $char { $(
            $open => {
                $buffer.type_char($close);
                $buffer.move_left();
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

/// Modifiers. Ignores shiftedness.
pub enum Mods {
    /// No modifiers.
    None,
    /// Control
    Ctrl,
    /// Alt
    Alt,
    /// Control + Alt
    CtrlAlt,
}

impl Mods {
    /// Parses modifier data from crossterm to `Mods`.
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

pub fn main() {
    let mut stdout = io::stdout();
    let mut args = args();
    _ = args.next();
    let path = args.next().unwrap_or("scratch".to_string());
    let mut buf = Buffer::new(&path);
    print!("\x1bc");
    _ = terminal::enable_raw_mode();
    buf.save();
    'ed: loop {
        let (widthu, heightu) = terminal::size().unwrap();
        let _width = widthu as usize;
        let height = heightu as usize;
        let event = event::read().unwrap();
        if let Event::Key(key) = event {
            if key.kind != event::KeyEventKind::Release {
                let start = Instant::now();
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
                                buf.mode = Mode::from_string(&buf.temp_str);
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
                                } else {
                                    _ = buf.vars.insert(
                                        String::from("lastact"),
                                        BimVar::Str(String::from("Inval line num")),
                                    );
                                }
                                buf.mode = Mode::Default;
                            }
                            Mode::Copy => {
                                'cpy: {
                                    let linenums: Vec<&str> =
                                        buf.temp_str.split_whitespace().collect();
                                    if linenums.len() == 2 {
                                        let from = linenums[0].parse::<usize>();
                                        if from.is_err() {
                                            _ = buf.vars.insert(
                                                String::from("lastact"),
                                                BimVar::Str(String::from("Inval from")),
                                            );
                                            break 'cpy;
                                        }
                                        let f = from.unwrap();
                                        if f > buf.contents.len() {
                                            _ = buf.vars.insert(
                                                String::from("lastact"),
                                                BimVar::Str(String::from("Inval from")),
                                            );
                                            break 'cpy;
                                        }
                                        let to = linenums[1].parse::<usize>();
                                        if to.is_err() {
                                            _ = buf.vars.insert(
                                                String::from("lastact"),
                                                BimVar::Str(String::from("Inval to")),
                                            );
                                            break 'cpy;
                                        }
                                        let t = to.unwrap();
                                        if t > buf.contents.len() {
                                            _ = buf.vars.insert(
                                                String::from("lastact"),
                                                BimVar::Str(String::from("Inval to")),
                                            );
                                            break 'cpy;
                                        }
                                        if f >= t || f == 0 {
                                            _ = buf.vars.insert(
                                                String::from("lastact"),
                                                BimVar::Str(String::from("assert: f < t")),
                                            );
                                            break 'cpy;
                                        }

                                        let mut i = 0;
                                        let paste_contents = &buf.contents.clone()[f - 1..t];
                                        paste_contents.iter().for_each(|l| {
                                            buf.contents
                                                .insert(buf.cursor_pos.line + i, l.to_string());
                                            i += 1;
                                        });
                                        buf.update_highlighting();
                                    } else {
                                        _ = buf.vars.insert(
                                            String::from("lastact"),
                                            BimVar::Str(format!(
                                                "{} args given, 2 expected",
                                                linenums.len()
                                            )),
                                        );
                                    }
                                }
                                buf.temp_str.clear();
                                buf.mode = Mode::Default;
                            }
                            Mode::OpenFile => {
                                buf.save();
                                buf.filepath = buf.temp_str.clone();
                                buf.reload_file();
                                buf.mode = Mode::Default;
                                if buf.cursor_pos.line >= buf.contents.len() {
                                    buf.cursor_pos.line = buf.contents.len() - 1;
                                }
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
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
                                        buf.newline_below(&linect);
                                    }
                                    buf.move_up();
                                    buf.indent_lvl += 1;
                                    buf.newline_below("");
                                } else {
                                    let linect: String = buf.contents[buf.cursor_pos.line]
                                        .drain(buf.cursor_pos.idx..)
                                        .collect();
                                    buf.newline_below(&linect);
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
                                '{' | '}' | '[' | ']' | '(' | ')' if buf.mode != Mode::Paste => {
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
                            (0..buf.lang.indent_size()).for_each(|_| {
                                buf.contents[buf.cursor_pos.line].insert(buf.cursor_pos.idx, ' ');
                            });
                            buf.cursor_pos.idx += buf.lang.indent_size();
                            buf.indent_lvl += 1;
                            buf.update_highlighting();
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
                        // My custom keybinds
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
                            let linelen = buf.contents[buf.cursor_pos.line].len();
                            if buf.cursor_pos.idx > linelen {
                                buf.cursor_pos.idx = linelen;
                            };
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
                            let linelen = buf.contents[buf.cursor_pos.line].len();
                            if buf.cursor_pos.idx > linelen {
                                buf.cursor_pos.idx = linelen;
                            };
                        }
                        KeyCode::Char('l') => {
                            if buf.contents.len() != 1 {
                                buf.contents.remove(buf.cursor_pos.line);
                                buf.cursor_pos.idx = 0;
                            } else {
                                buf.contents[0].clear();
                                buf.cursor_pos.idx = 0;
                            }
                            buf.update_highlighting();
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
                            buf.indent_lvl = spaces / buf.lang.indent_size();
                            buf.cursor_pos.idx = spaces;
                        }
                        KeyCode::Char(':') => {
                            let mut currline = buf.contents[buf.cursor_pos.line].chars();
                            let mut spaces = 0;
                            while currline.next() == Some(' ') {
                                spaces += 1;
                            }
                            buf.indent_lvl = spaces / buf.lang.indent_size();
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
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                    if !stat {
                                        buf.cursor_pos = prevpos;
                                        break 'findfwd;
                                    }
                                }
                            }
                        }
                        KeyCode::Char('o') => {
                            buf.newline_below("");
                        }
                        KeyCode::Char('O') => {
                            if buf.move_up() {
                                buf.newline_below("");
                            } else {
                                buf.contents.insert(0, String::new());
                                buf.cursor_pos.idx = 0;
                                buf.update_highlighting();
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
                            buf.update_highlighting();
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
                            buf.update_highlighting();
                        }
                        KeyCode::Char('0') => {
                            buf.indent_lvl = 0;
                        }
                        KeyCode::Char('k') => {
                            let indent_size = buf.lang.indent_size();
                            if buf.indent_lvl * indent_size != buf.cursor_pos.idx {
                                buf.contents[buf.cursor_pos.line].replace_range(
                                    (buf.indent_lvl * indent_size)..buf.cursor_pos.idx,
                                    "",
                                );
                                buf.cursor_pos.idx = buf.indent_lvl * indent_size;
                                buf.update_highlighting();
                            }
                        }
                        KeyCode::Char('K') => {
                            buf.contents[buf.cursor_pos.line].truncate(buf.cursor_pos.idx);
                            buf.update_highlighting();
                        }
                        KeyCode::Char('w') => {
                            while buf.contents[buf.cursor_pos.line]
                                .chars()
                                .nth(buf.cursor_pos.idx)
                                .unwrap_or(' ')
                                .is_whitespace()
                            {
                                if !buf.move_right() {
                                    break;
                                }
                            }
                            while buf.contents[buf.cursor_pos.line]
                                .chars()
                                .nth(buf.cursor_pos.idx)
                                .unwrap_or(' ')
                                .is_alphanumeric()
                            {
                                if !buf.move_right() {
                                    break;
                                }
                            }
                            while buf.contents[buf.cursor_pos.line]
                                .chars()
                                .nth(buf.cursor_pos.idx)
                                .unwrap_or(' ')
                                .is_whitespace()
                            {
                                if !buf.move_right() {
                                    break;
                                }
                            }
                        }
                        KeyCode::Char('W') => {
                            while buf.contents[buf.cursor_pos.line]
                                .chars()
                                .nth(buf.cursor_pos.idx)
                                .unwrap_or(' ')
                                .is_whitespace()
                            {
                                if !buf.move_left() {
                                    break;
                                }
                            }
                            while buf.contents[buf.cursor_pos.line]
                                .chars()
                                .nth(buf.cursor_pos.idx)
                                .unwrap_or(' ')
                                .is_alphanumeric()
                            {
                                if !buf.move_left() {
                                    break;
                                }
                            }
                            while buf.contents[buf.cursor_pos.line]
                                .chars()
                                .nth(buf.cursor_pos.idx)
                                .unwrap_or(' ')
                                .is_whitespace()
                            {
                                if !buf.move_left() {
                                    break;
                                }
                            }
                        }
                        KeyCode::Char('y') => {
                            buf.contents.insert(
                                buf.cursor_pos.line,
                                buf.contents[buf.cursor_pos.line].clone(),
                            );
                            buf.update_highlighting();
                            buf.move_down();
                        }
                        KeyCode::Char('Y') => {
                            if buf.mode != Mode::Copy {
                                buf.mode = Mode::Copy;
                                buf.temp_str.clear();
                            }
                            let numbuf = format!("{} ", buf.cursor_pos.line + 1);
                            for c in numbuf.chars() {
                                buf.temp_str.push(c);
                            }
                        }
                        KeyCode::Char('A') => {
                            if buf.cursor_pos.line + 1 < buf.contents.len() {
                                buf.contents
                                    .swap(buf.cursor_pos.line, buf.cursor_pos.line + 1);
                                buf.update_highlighting();
                                buf.move_down();
                            }
                        }
                        KeyCode::Char('E') => {
                            if buf.cursor_pos.line != 0 {
                                buf.contents
                                    .swap(buf.cursor_pos.line, buf.cursor_pos.line - 1);
                                buf.move_up();
                                buf.update_highlighting();
                            }
                        }
                        KeyCode::Char('G') => {
                            buf.mode = Mode::Goto;
                            buf.temp_str.clear();
                        }
                        _ => {}
                    },
                    Mods::Ctrl => match key.code {
                        KeyCode::Char('r') => {
                            buf.reload_file();
                            buf.cursor_pos.line = 0;
                            buf.cursor_pos.idx = 0;
                            print!("\x1bc");
                        }
                        KeyCode::Char('z') => {
                            buf.reload_file();
                            if buf.cursor_pos.line >= buf.contents.len() {
                                buf.cursor_pos.line = buf.contents.len() - 1;
                            }
                            if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                            }
                        }
                        KeyCode::Backspace => {
                            while buf.fast_backspace().unwrap_or('a').is_whitespace() {}
                            let mut last;
                            'killword: loop {
                                last = buf.fast_backspace().unwrap_or(' ');
                                if !last.is_alphanumeric() {
                                    break 'killword;
                                }
                            }
                            if last != '\n' {
                                buf.type_char(last);
                            } else {
                                buf.update_highlighting();
                            }
                        }
                        _ => {}
                    },
                    Mods::CtrlAlt => match key.code {
                        KeyCode::Char('L') => {
                            if let Some(showlinenos) = buf.vars.get_mut("showlinenos") {
                                if let BimVar::Bool(x) = *showlinenos {
                                    *showlinenos = BimVar::Bool(!x);
                                }
                            }
                        }
                        _ => {}
                    },
                }
                let e = start.elapsed().as_micros();
                buf.iter_time += e;
                buf.iter_time >>= 1;
            }
        }
        buf.print(&event);
        stdout.flush().unwrap();
    }
    print!("\x1bc");
    buf.save();
    _ = terminal::disable_raw_mode();
}
