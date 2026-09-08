//! Language-neutral raw input buffer and cursor state.
//!
//! The raw buffer holds canonical ASCII letters and is renormalized after
//! every insertion: valid syllables are re-serialized (merging duplicate horn
//! markers, relocating trailing tone markers), everything else passes through
//! verbatim so backspaces can always rebuild what was typed. Rendering the
//! buffer into Vietnamese text lives in [`crate::processor::render_raw`], not
//! here.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Buffer {
    chars: Vec<char>,
    cursor: usize,
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

    pub fn raw(&self) -> String {
        self.chars.iter().collect()
    }

    #[inline]
    pub fn raw_chars(&self) -> &[char] {
        &self.chars
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub(crate) fn insert(&mut self, ch: char) {
        self.chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    /// Replaces the whole buffer, clamping the cursor to its length.
    #[inline]
    pub(crate) fn replace(&mut self, raw: String, cursor: usize) {
        self.chars = raw.chars().collect();
        self.cursor = cursor.min(self.chars.len());
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
