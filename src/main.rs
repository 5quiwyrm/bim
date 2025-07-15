#![allow(clippy::must_use_candidate)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::new_without_default)]

pub mod buffer;
use buffer::*;
pub mod nav;
use nav::*;
pub mod autocomplete;
pub mod languages;
pub mod snippets;

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

pub fn is_close_bracket(ch: char) -> bool {
    matches!(ch, ')' | ']' | '}')
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
                let c = $buffer
                        .contents[$buffer.cursor_pos.line]
                        .chars()
                        .nth($buffer.cursor_pos.idx)
                        .unwrap_or(' ');
                $buffer.indent_lvl += 1;
                if c.is_whitespace() || is_close_bracket(c) {
                    $buffer.type_char($close);
                    $buffer.move_left();
                }
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

impl Buffer {
    fn decode_from_to(&self) -> Result<(usize, usize), String> {
        let linenums: Vec<&str> = self.temp_str.split_whitespace().collect();
        if linenums.len() == 2 {
            let f = match linenums[0].parse::<usize>() {
                Ok(o) if o <= self.contents.len() => o,
                _ => {
                    return Err("Inval from".to_string());
                }
            };
            let t = match linenums[1].parse::<usize>() {
                Ok(o) if o <= self.contents.len() => o,
                _ => {
                    return Err("Inval to".to_string());
                }
            };
            if f > t || f == 0 {
                return Err("assert: f <= t".to_string());
            }
            Ok((f, t))
        } else {
            Err(format!("{} args given, 2 expected", linenums.len()))
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
    let mut args = args().peekable();
    let mut processing_time: u128 = 0;
    let mut printing_time: u128 = 0;
    _ = args.next();
    let path = if args.peek().is_some() {
        args.collect::<Vec<String>>().join(" ")
    } else {
        String::from("scratch")
    };
    if path.ends_with(".exe") {
        println!("You shouldn't do this");
        return;
    }
    let mut buf = Buffer::new(&path);
    print!("\x1bc\x1b[?25l");
    _ = terminal::enable_raw_mode();
    buf.save();
    buf.add_tokens();
    print!("Press any key (ideally esc)...");
    'ed: loop {
        let (widthu, heightu) = terminal::size().expect("terminal should have size");
        let width = widthu as usize;
        let height = heightu as usize;
        let event = event::read().expect("there should be an event upon reading");
        let start = Instant::now();
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
                let return_mode = if let Some(BimVar::Bool(true)) = buf.vars.get("ret-to-nav") {
                    Mode::Nav
                } else {
                    Mode::Default
                };
                if buf.mode == Mode::Nav {
                    if handle_nav(&mut buf, key, &modifiers, height, width) {
                        break 'ed;
                    }
                } else {
                    match modifiers {
                        Mods::None => match key.code {
                            KeyCode::Esc => {
                                if cfg!(feature = "nav-pro") {
                                    buf.mode = Mode::Nav;
                                } else {
                                    buf.mode = Mode::Default;
                                }
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
                            KeyCode::Enter => {
                                match buf.mode {
                                    Mode::Find | Mode::ReplaceStr => {
                                        buf.mode = return_mode;
                                    }
                                    Mode::Indent => {
                                        buf.mode = return_mode;
                                        buf.temp_str.clear();
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
                                            buf.alert = Alert::new(
                                                &[String::from("Inval line num")],
                                                1_000_000,
                                            );
                                        }
                                        buf.temp_str.clear();
                                        buf.mode = return_mode;
                                    }
                                    Mode::Copy => {
                                        match buf.decode_from_to() {
                                            Ok((f, t)) => {
                                                let paste_contents =
                                                    &buf.contents.clone()[f - 1..t];
                                                for (i, l) in paste_contents.iter().enumerate() {
                                                    buf.contents.insert(
                                                        buf.cursor_pos.line + i,
                                                        l.to_string(),
                                                    );
                                                }
                                                buf.update_highlighting();
                                            }
                                            Err(e) => {
                                                buf.alert = Alert::new(&[e], 1_000_000);
                                            }
                                        }
                                        buf.temp_str.clear();
                                        buf.mode = return_mode;
                                    }
                                    Mode::KillLines => {
                                        match buf.decode_from_to() {
                                            Ok((f, t)) => {
                                                let mut i = f;
                                                while i <= t {
                                                    buf.contents.remove(f - 1);
                                                    i += 1;
                                                }
                                                if buf.contents.is_empty() {
                                                    buf.contents.push(String::from("\n"));
                                                }
                                                if buf.cursor_pos.line >= buf.contents.len()
                                                    && !buf.contents.is_empty()
                                                {
                                                    buf.cursor_pos.line = buf.contents.len() - 1;
                                                }
                                                buf.update_highlighting();
                                            }
                                            Err(e) => {
                                                buf.alert = Alert::new(&[e], 1_000_000);
                                            }
                                        }
                                        buf.temp_str.clear();
                                        buf.mode = return_mode;
                                    }
                                    Mode::OpenFile => {
                                        buf.save();
                                        buf.buffer_history.hist.push(buf.temp_str.clone());
                                        buf.buffer_history.head = buf.buffer_history.hist.len() - 1;
                                        buf.filepath = buf.temp_str.clone();
                                        buf.reload_file();
                                        buf.mode = return_mode;
                                        if buf.cursor_pos.line >= buf.contents.len() {
                                            buf.cursor_pos.line = buf.contents.len() - 1;
                                        }
                                        if buf.cursor_pos.idx
                                            > buf.contents[buf.cursor_pos.line].len()
                                        {
                                            buf.cursor_pos.idx =
                                                buf.contents[buf.cursor_pos.line].len();
                                        }
                                        buf.add_tokens();
                                    }
                                    Mode::Snippet => {
                                        let sniplines = buf.snippets.query(&buf.temp_str);
                                        if sniplines.is_empty() {
                                            buf.alert = Alert::new(
                                                &["Invalid request".to_string()],
                                                1_000_000,
                                            );
                                        }
                                        for (i, l) in sniplines.iter().enumerate() {
                                            let mut ins_line = String::new();
                                            for _ in 0..buf.lang.indent_size() * buf.indent_lvl {
                                                ins_line.push(' ');
                                            }
                                            ins_line.push_str(l);
                                            buf.contents
                                                .insert(buf.cursor_pos.line + i + 1, ins_line);
                                        }
                                        buf.update_highlighting();
                                        buf.mode = return_mode;
                                    }
                                    Mode::Tee => {
                                        buf.mode = return_mode;
                                    }
                                    _ => {
                                        if let Some('}' | ']' | ')') = buf.contents
                                            [buf.cursor_pos.line]
                                            .chars()
                                            .nth(buf.cursor_pos.idx)
                                        {
                                            if buf.indent_lvl != 0 {
                                                buf.indent_lvl -= 1;
                                            }
                                            if buf.cursor_pos.idx != 0 {
                                                let linect: String = buf.contents
                                                    [buf.cursor_pos.line]
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
                                };
                            }
                            KeyCode::BackTab => {
                                if buf.indent_lvl > 0 {
                                    buf.indent_lvl -= 1;
                                    for _ in 0..buf.lang.indent_size() {
                                        buf.fast_backspace();
                                    }
                                    buf.update_highlighting();
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
                                (0..buf.lang.indent_size()).for_each(|_| {
                                    buf.contents[buf.cursor_pos.line]
                                        .insert(buf.cursor_pos.idx, ' ');
                                });
                                buf.cursor_pos.idx += buf.lang.indent_size();
                                buf.indent_lvl += 1;
                                buf.update_highlighting();
                            }
                            _ => {}
                        },
                        Mods::Alt => match key.code {
                            KeyCode::Char('q') => {
                                if buf.buffer_history.hist.len() < 2 {
                                    break 'ed;
                                } else {
                                    buf.buffer_history.hist.remove(buf.buffer_history.head);
                                    if buf.buffer_history.head > 0 {
                                        buf.buffer_history.head -= 1;
                                    }
                                    buf.filepath =
                                        buf.buffer_history.hist[buf.buffer_history.head].clone();
                                    buf.reload_file();
                                    buf.mode = return_mode;
                                    if buf.cursor_pos.line >= buf.contents.len() {
                                        buf.cursor_pos.line = buf.contents.len() - 1;
                                    }
                                    if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len()
                                    {
                                        buf.cursor_pos.idx =
                                            buf.contents[buf.cursor_pos.line].len();
                                    }
                                    let mut ret: Vec<String> = vec![];
                                    for (idx, s) in buf.buffer_history.hist.iter().enumerate() {
                                        if idx == buf.buffer_history.head {
                                            ret.push(format!("> {s}"));
                                        } else {
                                            ret.push(format!("  {s}"));
                                        }
                                    }
                                    buf.alert = Alert::new(&ret, 5_000_000);
                                }
                            }
                            KeyCode::Char('Q') => {
                                break 'ed;
                            }
                            KeyCode::Char('s') => {
                                buf.save();
                            }
                            KeyCode::Char('b') | KeyCode::End => {
                                let linect = buf.contents.len();
                                buf.cursor_pos.line = if linect == 0 { 0 } else { linect - 1 };
                                buf.cursor_pos.idx = 0;
                            }
                            KeyCode::Char('t') | KeyCode::Home => {
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
                                }
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
                                }
                            }
                            KeyCode::Char('l') => {
                                if buf.contents.len() == 1 {
                                    buf.contents[0].clear();
                                    buf.cursor_pos.idx = 0;
                                } else {
                                    buf.contents.remove(buf.cursor_pos.line);
                                    buf.cursor_pos.idx = 0;
                                }
                                let contentlen = buf.contents.len();
                                if buf.cursor_pos.line >= contentlen && !buf.contents.is_empty() {
                                    buf.cursor_pos.line = contentlen - 1;
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
                            KeyCode::Char('I') => {
                                buf.adjust_indent();
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
                            }
                            KeyCode::Char('<') => {
                                if buf.mode != Mode::Indent {
                                    if buf.indent_lvl != 0 {
                                        buf.indent_lvl -= 1;
                                        let indent_size = buf.lang.indent_size();
                                        if buf.cursor_pos.idx >= indent_size {
                                            buf.cursor_pos.idx -= indent_size;
                                        }
                                        let linelen = buf.contents[buf.cursor_pos.line].len();
                                        if buf.cursor_pos.idx > linelen {
                                            buf.cursor_pos.idx = linelen;
                                        }
                                    }
                                    buf.adjust_indent();
                                } else {
                                    match buf.decode_from_to() {
                                        Ok((f, t)) => {
                                            let mut prefix = String::new();
                                            for _ in 0..buf.lang.indent_size() {
                                                prefix.push(' ');
                                            }
                                            for i in (f - 1)..t {
                                                buf.contents[i] = buf.contents[i]
                                                    .strip_prefix(&prefix)
                                                    .unwrap_or(&buf.contents[i])
                                                    .to_string();
                                            }
                                            buf.update_highlighting();
                                        }
                                        Err(e) => {
                                            buf.alert = Alert::new(&[e], 1_000_000);
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('>') => {
                                if buf.mode != Mode::Indent {
                                    buf.indent_lvl += 1;
                                    buf.adjust_indent();
                                    buf.cursor_pos.idx += buf.lang.indent_size();
                                } else {
                                    match buf.decode_from_to() {
                                        Ok((f, t)) => {
                                            let mut prefix = String::new();
                                            for _ in 0..buf.lang.indent_size() {
                                                prefix.push(' ');
                                            }
                                            for i in (f - 1)..t {
                                                buf.contents[i] =
                                                    format!("{}{}", prefix, buf.contents[i]);
                                            }
                                            buf.update_highlighting();
                                        }
                                        Err(e) => {
                                            buf.alert = Alert::new(&[e], 1_000_000);
                                        }
                                    }
                                }
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
                                    buf.mode = return_mode;
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
                                        let status = buf.move_up();
                                        buf.cursor_pos.idx =
                                            buf.contents[buf.cursor_pos.line].len();
                                        if !status {
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
                            KeyCode::Char('x' | 'M') => {
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
                                    buf.mode = return_mode;
                                } else {
                                    buf.mode = Mode::ReplaceStr;
                                    buf.replace_str.clear();
                                }
                            }
                            KeyCode::Char('j') => {
                                if buf.contents.len() > buf.cursor_pos.line + 1 {
                                    let l = buf.contents.remove(buf.cursor_pos.line + 1);
                                    let o = buf.contents[buf.cursor_pos.line].clone();
                                    buf.contents[buf.cursor_pos.line].clear();
                                    buf.contents[buf.cursor_pos.line].push_str(o.trim_end());
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
                                while !buf.contents[buf.cursor_pos.line]
                                    .chars()
                                    .nth(buf.cursor_pos.idx)
                                    .unwrap_or(' ')
                                    .is_whitespace()
                                {
                                    if !buf.move_right() {
                                        break;
                                    }
                                }
                                buf.move_right();
                                if buf.cursor_pos.idx == 0 {
                                    buf.move_left();
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
                                while !buf.contents[buf.cursor_pos.line]
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
                            KeyCode::Char('C') => {
                                if buf.mode != Mode::KillLines {
                                    buf.mode = Mode::KillLines;
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
                            KeyCode::Char('S') => {
                                buf.mode = Mode::Snippet;
                                buf.temp_str.clear();
                            }
                            KeyCode::Char('[') => {
                                while buf.cursor_pos.line != 0
                                    && buf.contents[buf.cursor_pos.line].is_empty()
                                {
                                    buf.cursor_pos.line -= 1;
                                }
                                while buf.cursor_pos.line != 0
                                    && !buf.contents[buf.cursor_pos.line].is_empty()
                                {
                                    buf.cursor_pos.line -= 1;
                                }
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
                            }
                            KeyCode::Char(']') => {
                                let content_len = buf.contents.len();
                                while buf.cursor_pos.line + 1 < content_len
                                    && !buf.contents[buf.cursor_pos.line].is_empty()
                                {
                                    buf.cursor_pos.line += 1;
                                }
                                while buf.cursor_pos.line + 1 < content_len
                                    && buf.contents[buf.cursor_pos.line].is_empty()
                                {
                                    buf.cursor_pos.line += 1;
                                }
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
                            }
                            KeyCode::Char('N') | KeyCode::Char('v') => {
                                buf.mode = Mode::Nav;
                                buf.vars
                                    .insert(String::from("ret-to-nav"), BimVar::Bool(true));
                                buf.temp_str.clear();
                            }
                            KeyCode::Char('-') => {
                                buf.alert = Alert::new(&[], 1_000_000);
                            }
                            KeyCode::Char('T') => {
                                buf.mode = Mode::Tee;
                                buf.replace_str.clear();
                            }
                            KeyCode::Char(x) if x.is_numeric() => {
                                let (all_matches, mut querylen) =
                                    buf.autocomplete.get_candidates(&buf);
                                let matches: Vec<String> =
                                    all_matches.iter().take(5).map(|s| s.to_string()).collect();
                                match x {
                                    '1' | '2' | '3' | '4' | '5' => {
                                        let replace_str = match x {
                                            '1' => matches[0].clone(),
                                            '2' => matches[1].clone(),
                                            '3' => matches[2].clone(),
                                            '4' => matches[3].clone(),
                                            '5' => matches[4].clone(),
                                            _ => {
                                                querylen = 0;
                                                String::new()
                                            }
                                        };
                                        buf.contents[buf.cursor_pos.line].replace_range(
                                            (if buf.cursor_pos.idx >= querylen {
                                                buf.cursor_pos.idx - querylen
                                            } else {
                                                0
                                            })
                                                ..buf.cursor_pos.idx,
                                            &replace_str,
                                        );
                                        buf.cursor_pos.idx += replace_str.len();
                                        buf.cursor_pos.idx -= querylen;
                                        if buf.cursor_pos.idx
                                            > buf.contents[buf.cursor_pos.line].len()
                                        {
                                            buf.cursor_pos.idx =
                                                buf.contents[buf.cursor_pos.line].len();
                                        }
                                        buf.update_highlighting();
                                    }
                                    '6' => {
                                        let display_matches: Vec<String> = matches
                                            .iter()
                                            .enumerate()
                                            .map(|(i, v)| format!("{}: `{v}`", i + 1))
                                            .collect();
                                        buf.alert = Alert::new(&display_matches, 500_000);
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Char('U') => {
                                if buf.mode != Mode::Indent {
                                    buf.mode = Mode::Indent;
                                    buf.temp_str.clear();
                                }
                                let numbuf = format!("{} ", buf.cursor_pos.line + 1);
                                for c in numbuf.chars() {
                                    buf.temp_str.push(c);
                                }
                            }
                            _ => {}
                        },
                        Mods::Ctrl => match key.code {
                            KeyCode::Char('N') => {
                                buf.mode = Mode::Nav;
                                buf.vars
                                    .insert(String::from("ret-to-nav"), BimVar::Bool(true));
                                buf.temp_str.clear();
                            }
                            KeyCode::Char('r') => {
                                buf.reload_file();
                                buf.cursor_pos.line = 0;
                                buf.cursor_pos.idx = 0;
                                print!("\x1bc\x1b[?25l");
                            }
                            KeyCode::Char('z') => {
                                buf.reload_file();
                                if buf.cursor_pos.line >= buf.contents.len() {
                                    buf.cursor_pos.line = buf.contents.len() - 1;
                                }
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
                                print!("\x1bc\x1b[?25l");
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
                                if last == '\n' {
                                    buf.update_highlighting();
                                } else {
                                    buf.type_char(last);
                                }
                            }
                            KeyCode::Char('o') => {
                                buf.mode = Mode::OpenFile;
                                buf.temp_str.clear();
                            }
                            KeyCode::Char('y') => {
                                if buf.top < buf.contents.len() {
                                    buf.top += 1;
                                }
                                buf.move_down();
                            }
                            KeyCode::Char('e') => {
                                if buf.top > 0 {
                                    buf.top -= 1;
                                }
                                buf.move_up();
                            }
                            KeyCode::Char('b') => {
                                let mut ret: Vec<String> = vec![];
                                for (idx, s) in buf.buffer_history.hist.iter().enumerate() {
                                    if idx == buf.buffer_history.head {
                                        ret.push(format!("> {s}"));
                                    } else {
                                        ret.push(format!("  {s}"));
                                    }
                                }
                                buf.alert = Alert::new(&ret, 5_000_000);
                            }
                            KeyCode::Char('p') => {
                                if buf.buffer_history.head == 0 {
                                    buf.buffer_history.head = buf.buffer_history.hist.len();
                                }
                                buf.buffer_history.head -= 1;
                                buf.save();
                                buf.filepath =
                                    buf.buffer_history.hist[buf.buffer_history.head].clone();
                                buf.reload_file();
                                if buf.cursor_pos.line >= buf.contents.len() {
                                    buf.cursor_pos.line = buf.contents.len() - 1;
                                }
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
                                let mut ret = vec![];
                                for (idx, s) in buf.buffer_history.hist.iter().enumerate() {
                                    if idx == buf.buffer_history.head {
                                        ret.push(format!("> {s}"));
                                    } else {
                                        ret.push(format!("  {s}"));
                                    }
                                }
                                buf.alert = Alert::new(&ret, 500_000);
                            }
                            KeyCode::Char('n') => {
                                buf.buffer_history.head += 1;
                                if buf.buffer_history.head >= buf.buffer_history.hist.len() {
                                    buf.buffer_history.head = 0;
                                }
                                buf.save();
                                buf.filepath =
                                    buf.buffer_history.hist[buf.buffer_history.head].clone();
                                buf.reload_file();
                                if buf.cursor_pos.line >= buf.contents.len() {
                                    buf.cursor_pos.line = buf.contents.len() - 1;
                                }
                                if buf.cursor_pos.idx > buf.contents[buf.cursor_pos.line].len() {
                                    buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                                }
                                let mut ret = vec![];
                                for (idx, s) in buf.buffer_history.hist.iter().enumerate() {
                                    if idx == buf.buffer_history.head {
                                        ret.push(format!("> {s}"));
                                    } else {
                                        ret.push(format!("  {s}"));
                                    }
                                }
                                buf.alert = Alert::new(&ret, 500_000);
                            }
                            _ => {}
                        },
                        Mods::CtrlAlt => match key.code {
                            KeyCode::Char('L') => {
                                if let Some(showlinenos) = buf.vars.get_mut("line-num-type") {
                                    if let BimVar::Str(x) = showlinenos {
                                        *showlinenos = match x.as_str() {
                                            "absolute" => BimVar::Str("relative".to_string()),
                                            "relative" => BimVar::Str("none".to_string()),
                                            _ => BimVar::Str("absolute".to_string()),
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('B') => {
                                if let Some(BimVar::Bool(showbottombar)) =
                                    buf.vars.get_mut("showbottombar")
                                {
                                    *showbottombar = !*showbottombar;
                                }
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
        if cfg!(feature = "profile") {
            processing_time += start.elapsed().as_micros();
            processing_time >>= 1;
        }
        buf.print(&event);
        if cfg!(feature = "profile") {
            printing_time += start.elapsed().as_micros();
            printing_time >>= 1;
        }
        if cfg!(feature = "profile") {
            print!(
                "{: <width$}",
                format!(
                    "Printing: {: <14} us | Processing input: {: <14} us | Total: {: <14} us",
                    style_time_raw(processing_time),
                    style_time_raw(printing_time),
                    style_time_raw(processing_time + printing_time),
                )
            );
        }
        stdout.flush().unwrap();
        buf.iter_time = start.elapsed().as_micros();
        buf.iter_time >>= 1;
    }
    print!("\x1bc\x1b[?25h");
    buf.save();
    _ = terminal::disable_raw_mode();
}
