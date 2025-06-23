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

## Snippet support

Snippets are implemented in a similar way to language syntax highlighting.
