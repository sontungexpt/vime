use crate::phonology::{BaseVowel, Case, Coda, Onset, Tone};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cased<T> {
    pub value: T,
    pub case: Case,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllable {
    pub onset: Option<Onset>,
    pub onset_chars: Vec<char>,

    pub vowels: Vec<Cased<BaseVowel>>,

    pub coda: Option<Coda>,
    pub coda_chars: Vec<char>,

    pub tone: Tone,
}

impl Default for Syllable {
    #[inline(always)]
    fn default() -> Self {
        Self {
            onset: None,
            onset_chars: Vec::with_capacity(3),

            vowels: Vec::with_capacity(3),

            coda: None,
            coda_chars: Vec::with_capacity(2),

            tone: Tone::Flat,
        }
    }
}

impl Syllable {
    #[inline(always)]
    pub(super) fn vowel_bases(&self) -> (usize, [BaseVowel; 3]) {
        debug_assert!(self.vowels.len() <= 3);
        let mut buf = [BaseVowel::A; 3];
        for (i, v) in self.vowels.iter().enumerate() {
            buf[i] = v.value;
        }
        (self.vowels.len(), buf)
    }
}
