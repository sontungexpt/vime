//! Semantic character model: Vietnamese vowels as primitive `(base, tone, case)`
//! triples, plus the Unicode codec mapping them to precomposed characters.

pub mod codec;
pub mod vowel;

pub use codec::{decode_vowel, encode_vowel, BaseVowel, Case, RootVowel, Shape, Tone};
pub use vowel::Vowel;
