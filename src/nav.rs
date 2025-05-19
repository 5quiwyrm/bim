use crate::Mods;
use crate::buffer::*;
use crossterm::event::{self, KeyCode};

pub const VIM_ITER_LIMIT: usize = 10000;

macro_rules! vim_repeat {
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
            KeyCode::Char(n) if n.is_numeric() => {
                buf.temp_str.push(n);
            }
            KeyCode::Char('c') => {
                vim_repeat!(buf, {
                    buf.move_left();
                });
            }
            KeyCode::Char('i') => {
                vim_repeat!(buf, {
                    buf.move_right();
                });
            }
            KeyCode::Char('e') => {
                vim_repeat!(buf, {
                    buf.move_up();
                });
            }
            KeyCode::Char('a') => {
                vim_repeat!(buf, {
                    buf.move_down();
                });
            }
            KeyCode::Char('w') => {
                vim_repeat!(buf, {
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
                vim_repeat!(buf, {
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
                vim_repeat!(buf, {
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
                vim_repeat!(buf, {
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
                vim_repeat!(buf, {
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
                vim_repeat!(buf, {
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
            _ => {}
        },
        Mods::Alt => match key.code {
            KeyCode::Char('q') => {
                return true;
            }
            _ => {}
        },
        Mods::Ctrl => {}
        Mods::CtrlAlt => {}
    }
    false
}
