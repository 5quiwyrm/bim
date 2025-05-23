use crate::Mods;
use crate::buffer::*;
use crossterm::event::{self, KeyCode};
pub const VIM_ITER_LIMIT: usize = 10000;

macro_rules! repeat_action {
    ($buf: ident, $action: block) => {
        if $buf.temp_str.is_empty() {
            $action
        } else {
            if let Ok(times) = $buf.temp_str.parse::<usize>() {
                let mut i = 0;
                while i < times && i < VIM_ITER_LIMIT {
                    $action
                    i += 1;
                }
            }
            $buf.temp_str.clear();
        }
    };
}

pub fn handle_nav(
    buf: &mut Buffer,
    key: event::KeyEvent,
    modifiers: &Mods,
    height: usize,
    _width: usize,
) -> bool {
    match modifiers {
        Mods::None => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                buf.mode = Mode::Default;
                buf.vars
                    .insert(String::from("ret-to-nav"), BimVar::Bool(false));
                buf.temp_str.clear();
            }
            KeyCode::Home | KeyCode::Char('0') if buf.temp_str.is_empty() => {
                buf.cursor_pos.idx = 0;
            }
            KeyCode::Char(n) if &buf.temp_str == "f" => {
                buf.temp_str.clear();
                if buf.move_right() {
                    'findfwd: loop {
                        let prevpos = buf.cursor_pos;
                        if let Some(p) =
                            buf.contents[buf.cursor_pos.line][buf.cursor_pos.idx..].find(n)
                        {
                            buf.cursor_pos.idx += p;
                            buf.cursor_pos.idx += 1;
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
            KeyCode::Char(n) if n.is_numeric() => {
                buf.temp_str.push(n);
            }
            KeyCode::Char('c') => {
                repeat_action!(buf, {
                    buf.move_left();
                });
            }
            KeyCode::Char('i') => {
                repeat_action!(buf, {
                    buf.move_right();
                });
            }
            KeyCode::Char('e') => {
                repeat_action!(buf, {
                    buf.move_up();
                });
            }
            KeyCode::Char('a') => {
                repeat_action!(buf, {
                    buf.move_down();
                });
            }
            KeyCode::Char('f') => {
                buf.temp_str.push('f');
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
            KeyCode::Char('o') => {
                repeat_action!(buf, {
                    buf.newline_below("");
                });
                buf.mode = Mode::Default;
            }
            KeyCode::Char('O') => {
                repeat_action!(buf, {
                    if buf.move_up() {
                        buf.newline_below("");
                    } else {
                        buf.contents.insert(0, String::new());
                        buf.cursor_pos.idx = 0;
                        buf.update_highlighting();
                    }
                });
                buf.mode = Mode::Default;
            }
            KeyCode::Char('w') => {
                repeat_action!(buf, {
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
                })
            }
            KeyCode::Char('W') => {
                repeat_action!(buf, {
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
                });
            }
            KeyCode::Char('g') => {
                buf.mode = Mode::Goto;
                buf.temp_str.clear();
            }
            KeyCode::Char('/') => {
                buf.mode = Mode::Find;
                buf.find_str.clear();
            }
            KeyCode::Char('r') => {
                buf.mode = Mode::ReplaceStr;
                buf.replace_str.clear();
            }
            KeyCode::Char('R') => {
                buf.mode = Mode::Replace;
                buf.temp_str.clear();
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
            KeyCode::Char('b') => {
                let linect = buf.contents.len();
                buf.cursor_pos.line = if linect == 0 { 0 } else { linect - 1 };
                buf.cursor_pos.idx = 0;
            }
            KeyCode::End | KeyCode::Char('$') => {
                buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
            }
            KeyCode::Char('t') => {
                buf.cursor_pos.line = 0;
                buf.cursor_pos.idx = 0;
            }
            KeyCode::Char('u') => {
                repeat_action!(buf, {
                    if buf.cursor_pos.line >= height {
                        buf.cursor_pos.line -= height;
                    } else {
                        buf.cursor_pos.line = 0;
                    }
                    let linelen = buf.contents[buf.cursor_pos.line].len();
                    if buf.cursor_pos.idx > linelen {
                        buf.cursor_pos.idx = linelen;
                    }
                });
            }
            KeyCode::Char('n') => {
                repeat_action!(buf, {
                    if buf.move_right() {
                        'findfwd: loop {
                            let prevpos = buf.cursor_pos;
                            if let Some(p) = buf.contents[buf.cursor_pos.line][buf.cursor_pos.idx..]
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
                });
            }
            KeyCode::Char('p') => {
                repeat_action!(buf, {
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
                            buf.cursor_pos.idx = buf.contents[buf.cursor_pos.line].len();
                            if !status {
                                buf.cursor_pos = prevpos;
                                break 'findfwd;
                            }
                        }
                    }
                });
            }
            KeyCode::Char('d') => {
                repeat_action!(buf, {
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
                });
            }
            KeyCode::Char('k') => {
                repeat_action!(buf, {
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
                });
                buf.update_highlighting();
            }
            KeyCode::Char('K') => {
                repeat_action!(buf, {
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
                });
                buf.mode = Mode::Default;
                buf.move_up();
                buf.newline_below("");
                buf.vars
                    .insert(String::from("ret-to-nav"), BimVar::Bool(false));
                buf.temp_str.clear();
                buf.update_highlighting();
            }
            KeyCode::Char('y') => {
                repeat_action!(buf, {
                    buf.contents.insert(
                        buf.cursor_pos.line,
                        buf.contents[buf.cursor_pos.line].clone(),
                    );
                    buf.move_down();
                });
                buf.update_highlighting();
            }
            KeyCode::Char('A') => {
                repeat_action!(buf, {
                    if buf.cursor_pos.line + 1 < buf.contents.len() {
                        buf.contents
                            .swap(buf.cursor_pos.line, buf.cursor_pos.line + 1);
                    }
                    buf.move_down();
                });
                buf.update_highlighting();
            }
            KeyCode::Char('E') => {
                repeat_action!(buf, {
                    if buf.cursor_pos.line != 0 {
                        buf.contents
                            .swap(buf.cursor_pos.line, buf.cursor_pos.line - 1);
                        buf.move_up();
                    }
                });
                buf.update_highlighting();
            }
            KeyCode::Char('<') => {
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
            }
            KeyCode::Char('>') => {
                buf.indent_lvl += 1;
                buf.cursor_pos.idx += buf.lang.indent_size();
                buf.adjust_indent();
                let linelen = buf.contents[buf.cursor_pos.line].len();
                if buf.cursor_pos.idx > linelen {
                    buf.cursor_pos.idx = linelen;
                }
            }
            KeyCode::Char('s') => {
                buf.save();
            }
            _ => {}
        },
        Mods::Alt => match key.code {
            KeyCode::Char('q') => {
                return true;
            }
            KeyCode::Char('o') => {
                buf.mode = Mode::OpenFile;
                buf.temp_str.clear();
            }
            KeyCode::Char('a') => {
                repeat_action!(buf, {
                    if buf.top < buf.contents.len() {
                        buf.top += 1;
                    }
                    buf.move_down();
                });
            }
            KeyCode::Char('s') => {
                buf.mode = Mode::Snippet;
                buf.temp_str.clear();
            }
            KeyCode::Char('y') => {
                if buf.mode != Mode::Copy {
                    buf.mode = Mode::Copy;
                    buf.temp_str.clear();
                }
                let numbuf = format!("{} ", buf.cursor_pos.line + 1);
                for c in numbuf.chars() {
                    buf.temp_str.push(c);
                }
            }
            KeyCode::Char('e') => {
                repeat_action!(buf, {
                    if buf.top > 0 {
                        buf.top -= 1;
                    }
                    buf.move_up();
                });
            }
            _ => {}
        },
        Mods::Ctrl => {}
        Mods::CtrlAlt => {}
    }
    false
}
