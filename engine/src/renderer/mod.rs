mod api;
pub mod parser;

pub use api::Renderer;
pub use parser::{ParsedVowel, Parser, Syllable};

use crate::{DefaultKeyMapping, KeyMapping};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Orthography {
    /// Today's standard tone placement (e.g. *hòa*, *tuyển*).
    #[default]
    Modern,
    /// Traditional (pre-reform) tone placement (e.g. *hoà*, *thuý*).
    Old,
}

/// The standard renderer, backed by an interpreter and an orthography.
pub struct SimpleRenderer<I: KeyMapping = DefaultKeyMapping<'static>> {
    interpreter: I,
    orthography: Orthography,
}

// impl ParsedSyllable {
//     /// Xác định index của nguyên âm sẽ mang Dấu Thanh chuẩn Phonology
//     pub fn get_tone_target_index(&self) -> usize {
//         let len = self.vowels.bases.len();
//         if len <= 1 {
//             return 0;
//         }

//         // 1. Ưu tiên nguyên âm có Dấu mũ / Dấu móc (ê, ơ, ô, ă, â, ư)
//         for (i, &v) in self.vowels.bases.iter().enumerate() {
//             if !matches!(v.shape(), Shape::None) {
//                 return i;
//             }
//         }

//         // 2. Quy tắc bỏ dấu Chuẩn (Bộ GD&ĐT)
//         if self.coda.kind != Coda::None {
//             1 // Có phụ âm cuối -> Dấu đặt ở nguyên âm thứ 2
//         } else {
//             0 // Không có phụ âm cuối -> Dấu đặt ở nguyên âm thứ 1
//         }
//     }
// }

impl<I: KeyMapping> SimpleRenderer<I> {
    pub const fn new(interpreter: I, orthography: Orthography) -> Self {
        Self {
            interpreter,
            orthography,
        }
    }

    pub const fn interpreter(&self) -> &I {
        &self.interpreter
    }

    pub const fn orthography(&self) -> Orthography {
        self.orthography
    }
}

impl Default for SimpleRenderer<DefaultKeyMapping<'static>> {
    fn default() -> Self {
        Self::new(DefaultKeyMapping::telex(), Orthography::default())
    }
}

impl<I: KeyMapping> Renderer for SimpleRenderer<I> {
    fn render(&self, raw: &[char], _cursor: usize) -> String {
        raw.iter().collect()
    }
}
