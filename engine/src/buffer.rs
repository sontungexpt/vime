use std::fmt::{self, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BufferChar {
    Literal(char),
    // char in transform is always a ascii char
    Transform(char),
}

impl BufferChar {
    #[inline(always)]
    pub const fn as_char(&self) -> char {
        match *self {
            BufferChar::Literal(c) | BufferChar::Transform(c) => c,
        }
    }

    #[inline(always)]
    pub const fn is_transform(&self) -> bool {
        matches!(self, BufferChar::Transform(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Buffer {
    chars: Vec<BufferChar>,
    cursor: usize,
}

impl fmt::Display for Buffer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in &self.chars {
            formatter.write_char(character.as_char())?;
        }
        Ok(())
    }
}

impl Buffer {
    pub(crate) const fn new() -> Self {
        Self {
            chars: Vec::new(),
            cursor: 0,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    #[inline]
    pub fn chars(&self) -> &[BufferChar] {
        &self.chars
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub(crate) fn insert(&mut self, ch: BufferChar) {
        self.chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    #[inline]
    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.chars.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    #[inline]
    pub(crate) fn delete(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    #[inline]
    pub(crate) fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    #[inline]
    pub(crate) fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.chars.len());
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }
}
