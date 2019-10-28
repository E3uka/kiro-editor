use crate::error::{Error, Result};
use std::ops;
use unicode_width::UnicodeWidthChar;

const TAB_STOP: usize = 8;

#[derive(Default)]
pub struct Row {
    buf: String,
    render: String,
    // Cache of byte indices of characters in `buf`. This will be empty when `buf` only contains
    // single byte characters not to allocate memory.
    indices: Vec<usize>,
}

impl Row {
    pub fn empty() -> Row {
        Row {
            buf: "".to_string(),
            render: "".to_string(),
            indices: Vec::with_capacity(0),
        }
    }

    pub fn new<S: Into<String>>(line: S) -> Result<Row> {
        let mut row = Row {
            buf: line.into(),
            render: "".to_string(),
            indices: Vec::with_capacity(0),
        };
        row.update_render()?;
        Ok(row)
    }

    // Returns number of characters
    pub fn len(&self) -> usize {
        if self.indices.is_empty() {
            self.buf.len()
        } else {
            self.indices.len()
        }
    }

    pub fn byte_idx_of(&self, char_idx: usize) -> usize {
        let len = self.indices.len();
        if len == 0 {
            char_idx
        } else if len == char_idx {
            self.buf.len()
        } else {
            self.indices[char_idx]
        }
    }

    pub fn char_idx_of(&self, byte_idx: usize) -> usize {
        if self.indices.is_empty() {
            return byte_idx;
        }
        if self.buf.len() == byte_idx {
            return self.indices.len(); // When the byte index points after the last character
        }
        self.indices
            .iter()
            .position(|bi| *bi == byte_idx)
            .expect("byte index is not correct boundary of UTF-8")
    }

    pub fn buffer(&self) -> &str {
        self.buf.as_str()
    }

    pub fn render_text(&self) -> &str {
        self.render.as_str()
    }

    pub fn char_at(&self, at: usize) -> char {
        self.char_at_checked(at).unwrap()
    }

    pub fn char_at_checked(&self, at: usize) -> Option<char> {
        self[at..].chars().next()
    }

    fn update_render(&mut self) -> Result<()> {
        self.render.clear();
        self.render.reserve(self.buf.len());
        let mut index = 0;
        let mut num_chars = 0;

        for c in self.buf.chars() {
            if c == '\t' {
                loop {
                    self.render.push(' ');
                    index += 1;
                    if index % TAB_STOP == 0 {
                        break;
                    }
                }
            } else if let Some(width) = c.width_cjk() {
                index += width;
                self.render.push(c);
            } else {
                // Control sequences are valid for UTF-8 but they should not appear in text.
                return Err(Error::ControlCharInText(c));
            }
            num_chars += 1;
        }

        if num_chars == self.buf.len() {
            // If number of chars is the same as byte length, this line includes no multi-byte char.
            // Vector with zero capacity is guaranteed not to allocate heap memory.
            self.indices = Vec::with_capacity(0)
        } else {
            self.indices.clear();
            self.indices.reserve(num_chars);
            for (idx, _) in self.buf.char_indices() {
                self.indices.push(idx);
            }
        }

        Ok(())
    }

    pub fn rx_from_cx(&self, cx: usize) -> usize {
        self[..cx].chars().fold(0, |rx, ch| {
            if ch == '\t' {
                // Proceed TAB_STOP spaces then subtract spaces by mod TAB_STOP
                rx + TAB_STOP - (rx % TAB_STOP)
            } else {
                rx + ch.width_cjk().unwrap()
            }
        })
    }

    pub fn insert_char(&mut self, at: usize, c: char) {
        if self.len() <= at {
            self.buf.push(c);
        } else {
            self.buf.insert(self.byte_idx_of(at), c);
        }
        // TODO: More efficient update for self.render
        self.update_render().unwrap();
    }

    pub fn insert_str<S: AsRef<str>>(&mut self, at: usize, s: S) {
        if self.len() <= at {
            self.buf.push_str(s.as_ref());
        } else {
            self.buf.insert_str(self.byte_idx_of(at), s.as_ref());
        }
        self.update_render().unwrap();
    }

    pub fn delete_char(&mut self, at: usize) {
        if at < self.len() {
            self.buf.remove(self.byte_idx_of(at));
            self.update_render().unwrap();
        }
    }

    pub fn append<S: AsRef<str>>(&mut self, s: S) {
        let s = s.as_ref();
        if s.is_empty() {
            return;
        }
        self.buf.push_str(s);
        self.update_render().unwrap();
    }

    pub fn truncate(&mut self, at: usize) {
        if at < self.len() {
            self.buf.truncate(self.byte_idx_of(at));
            self.update_render().unwrap();
        }
    }

    pub fn remove_char(&mut self, at: usize) {
        self.buf.remove(self.byte_idx_of(at));
        self.update_render().unwrap();
    }

    pub fn remove(&mut self, start: usize, end: usize) {
        if start < end {
            let start_idx = self.byte_idx_of(start);
            let end_idx = self.byte_idx_of(end);
            self.buf.drain(start_idx..end_idx);
            self.update_render().unwrap();
        }
    }
}

impl ops::Index<ops::Range<usize>> for Row {
    type Output = str;

    fn index(&self, r: ops::Range<usize>) -> &Self::Output {
        let start = self.byte_idx_of(r.start);
        let end = self.byte_idx_of(r.end);
        &self.buf[start..end]
    }
}

impl ops::Index<ops::RangeFrom<usize>> for Row {
    type Output = str;

    fn index(&self, r: ops::RangeFrom<usize>) -> &Self::Output {
        let start = self.byte_idx_of(r.start);
        &self.buf[start..]
    }
}

impl ops::Index<ops::RangeTo<usize>> for Row {
    type Output = str;

    fn index(&self, r: ops::RangeTo<usize>) -> &Self::Output {
        let end = self.byte_idx_of(r.end);
        &self.buf[..end]
    }
}

impl ops::Index<ops::RangeInclusive<usize>> for Row {
    type Output = str;

    fn index(&self, r: ops::RangeInclusive<usize>) -> &Self::Output {
        let start = self.byte_idx_of(*r.start());
        let end = self.byte_idx_of(*r.end());
        &self.buf[start..=end]
    }
}

impl ops::Index<ops::RangeToInclusive<usize>> for Row {
    type Output = str;

    fn index(&self, r: ops::RangeToInclusive<usize>) -> &Self::Output {
        let end = self.byte_idx_of(r.end);
        &self.buf[..=end]
    }
}
