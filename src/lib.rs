// Refs:
//   Build Your Own Text Editor: https://viewsourcecode.org/snaptoken/kilo/index.html
//   VT100 User Guide:           https://vt100.net/docs/vt100-ug/chapter3.html
//   Xterm Control Sequences:    https://www.xfree86.org/current/ctlseqs.html

#![allow(clippy::unused_io_amount)]
#![allow(clippy::match_overlapping_arm)]
#![allow(clippy::useless_let_if_seq)]
#![allow(clippy::cognitive_complexity)]

mod edit_diff;
mod editor;
mod error;
mod highlight;
mod history;
mod input;
mod language;
mod prompt;
mod row;
mod screen;
mod signal;
mod status_bar;
mod term_color;
mod text_buffer;

pub use editor::Editor;
pub use error::{Error, Result};
pub use input::{InputSeq, KeySeq, StdinRawMode};
pub use language::Language;
pub use screen::{Screen, HELP, VERSION};
pub use text_buffer::{Lines, TextBuffer};
