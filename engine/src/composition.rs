//! Language-neutral raw composition and cursor state.
//!
//! The raw buffer holds canonical ASCII letters and is renormalized after
//! every insertion: valid syllables are re-serialized (merging duplicate horn
//! markers, relocating trailing tone markers), everything else passes through
//! verbatim so backspaces can always rebuild what was typed. Rendering the
//! buffer into Vietnamese text lives in [`crate::processor::render_raw`], not
//! here.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Composition {
    raw_chars: Vec<char>,
    cursor: usize,
}

impl Composition {
    pub(crate) const fn new() -> Self {
        Self {
            raw_chars: Vec::new(),
            cursor: 0,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw_chars.is_empty()
    }

    pub fn raw(&self) -> String {
        self.raw_chars.iter().collect()
    }

    #[inline]
    pub fn raw_chars(&self) -> &[char] {
        &self.raw_chars
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub(crate) fn insert(&mut self, ch: char) {
        self.raw_chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    /// Replaces the whole buffer, clamping the cursor to its length.
    #[inline]
    pub(crate) fn replace(&mut self, raw: String, cursor: usize) {
        self.raw_chars = raw.chars().collect();
        self.cursor = cursor.min(self.raw_chars.len());
    }

    #[inline]
    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.raw_chars.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    #[inline]
    pub(crate) fn delete(&mut self) {
        if self.cursor < self.raw_chars.len() {
            self.raw_chars.remove(self.cursor);
        }
    }

    #[inline]
    pub(crate) fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    #[inline]
    pub(crate) fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.raw_chars.len());
    }

    #[inline]
    pub(crate) fn move_home(&mut self) {
        self.cursor = 0;
    }

    #[inline]
    pub(crate) fn move_end(&mut self) {
        self.cursor = self.raw_chars.len();
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.raw_chars.clear();
        self.cursor = 0;
    }
}
