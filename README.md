# Bim

Like Vim but bad.
Also Bim sounds like bin, and this reflects the shittiness of this editor.

This will be a keyboard focused editor, but it will not be much like Vim.
This will have modes that change what keybinds do.

This editor was initially built in Notepad, but once it achieved a usable
state it started to be built in itself!

Please don't use this for your own sanity's sake.

## Usage

To run the project:

`cargo run --release` to only run the project.
`cargo build --release` to build the project as an executable to a directory.
Once the executable is built you can run it by invoking it.
You are able to pass a filepath to it as an argument for the file to be opened.

## Keybinds

```
Esc -> Switch to default mode (no exceptions)
Backspace -> Delete char backwards, in all modes, in the buffer being used.
Delete -> Delete char forwards, in Default and Paste mode.
Enter -> Insert newline if in Default/Paste mode, executes prompt according to
  mode otherwise.
Char(c) -> Type the character c, with autopairs supported. If in a prompt that
  uses a different buffer, that buffer will be used instead.
Left | Down | Up | Right -> Does exactly what they are meant to do.
Home -> Move to start of line.
End -> Move to end of line.
Tab -> Types n spaces, increasing indent lvl by 1

M-q -> Quit bim.
M-s -> Saves file.
M-b -> Goto bottom of file.
M-t -> Goto top of file.

These are personal keybinds, feel free to remove these:
  M-c -> move left.
  M-a -> move down.
  M-e -> move up.
  M-i -> move right.

M-u -> Go up a screen.
M-d -> Go down a screen.
M-l -> Deletes line, moving up a line to the start.
M-, -> Decreases indent lvl by 1 (given that indent lvl > 0).
M-. -> Increases indent lvl by 1.
M-; -> Moves to the first non-whitespace character, detecting the indent level
  as it scans across the row.
M-: -> Auto-set indent level.
M-/ -> Quick switch to Find mode, clears the find buffer. In find mode, typed
  characters are typed into the find_str buffer, which is used in finding.
M-n -> Finds next string matching find_str. Note that this is case-sensitive.
  If the string is not found, then it will go to the end of the file.
M-p -> Finds previous string matching find_str. Note that this is
  case-sensitive.
  If the string is not found, then it will go to the start of the file.
M-o -> Open new line below, respecting the indent level.
M-O -> Open new line above, respecting the indent level.
M-m -> Consumes one letter to the left, and sets the current cursor position
  (after deleting the character) to a mark corresponding to the consumed
  character. Overwrites previous data in that mark.
M-g -> Consumes one letter to the left, and tries to go to the cursor position
  stored in the marklist. If the mark has not been set previously, it does
  nothing.
  Note that this sets the mark '_' to the cursor position so you can jump back.
M-x -> Enters switch mode, where typing writes to the temp_str buffer.
  Upon pressing enter, the temp_str buffer's content is interpreted as a mode,
  and you will be switched to that mode. If the contents are not interpreted as
  any mode, Default mode will be used.
M-r -> Quick switch to ReplaceStr mode, where typed characters are typed into
  the replace_str buffer, which is used in replacing.
M-h -> Where n is the length of the contents of find_str, this command replaces
  the n preceding character to the cursor with the contents of replace_str.
  Newlines halt this backwards seeking.
M-j -> Joins the current line with the line below, separated by a space.
M-k -> Kill until indent level.
M-K -> Kill until end of line (forwards).

C-r -> Reloads the file.
C-Backspace -> Deletes word backwards (this effectively deletes whitespace
  until finding a non-whitespace charatcer, then deletes until finding a
  non-alphanumeric character).
```

## Modes

### Default mode

Aliases in `M-x`: any, as long as they aren't aliases of other modes.

Shown in bottom bar as "default".

This is the default editing mode.

### Paste mode

Aliases in `M-x`: "paste", "p"

Shown in bottom bar as "paste".

This is used in pasting. It is toggled by doing `M-x` p enter.
In this mode, autopairs are disabled.

### Replace mode

Aliases in `M-x`: "replace", "r"

Shown in bottom bar as "replace".

Essentially like R in vim. Overwrites instead of inserting.
Exit with escape.

### Find mode

Aliases in `M-x`: "find", "f"

Shown in bottom bar as "find".

Same as `M-/`. Using `M-x` to enter Find mode is discouraged.
All typed characters are appended to find_str.
Backspace pops a character from find_str.

### ReplaceStr mode

Aliases in `M-x`: "replacestr", "rs"

Shown in bottom bar as "replace str".

Same as `M-r`. Using `M-x` to enter ReplaceStr mode is discouraged.
All typed characters are appended to replace_str.
Backspace pops a character from replace_str.

### Goto mode

Aliases in `M-x`: "goto", "g"

Shown in bottom bar as "goto: ".

The contents of the buffer will be interpreted as a line number and the cursor
will be moved to that line number.
If the line number provided is unparseable, nothing will happen.
If the line number provided is greater than the length of the file, the cursor
will be put on the bottom line.

### OpenFile mode

Aliasses in `M-x`: "open", "o", "openfile"

Shown in bottom bar as "open file: ".

The contents of the buffer will be interpreted as a filepath for the program to
open.
If the filepath is illegal, the program will crash.

### Switch mode

Aliases in `M-x`: "switch", "s"

Shown in bottom bar as "switch to mode: ".

Exit by pressing enter or escape.
Mode for finding modes.
Writes directly to temp_str.

## Syntax highlighting

Currently syntax highlighting is implemented as a trait on structs.

A sample implementation would be:
```rs
// ./src/languages/<langname>.rs
use crate::languages::{
    StyledChar,
    Language,
}
pub struct <langname> {};
pub const <langname but all caps>: <langname> = <langname> {};
impl Language for <langname> {
    ...
}
```

```rs
// ./src/languages.mod.rs
+ pub mod <langname>;
+ use <langname>::*;

...

    Box::new(MARKDOWN)
+ } else if <langname but all caps>.is_kind(path) {
+     Box::new(<langname but all caps>)
  } else {

...

```
