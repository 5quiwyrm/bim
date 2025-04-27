# Bim

Like Vim but bad.
Also Bim sounds like bin, and this reflects the shittiness of this editor.

This will be a keyboard focused editor, but it will not be much like Vim.
This will have modes that change what keybinds do.

## Usage

To run the project:

`cargo run --release` to only run the project.
`cargo build --release` to build the project as an executable to a directory.

## Keybinds

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
Tab -> Types 4 spaces, increasing indent lvl by 1

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

C-r -> Reloads the file.
C-Backspace -> Deletes word backwards (this effectively deletes whitespace
  until finding a non-whitespace charatcer, then deletes until finding a
  non-alphanumeric character).
