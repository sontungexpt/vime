//! B. Telex tone keys.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── all five tones on single vowels ──
    // a
    case!(['a', 's'], "á"),
    case!(['a', 'f'], "à"),
    case!(['a', 'r'], "ả"),
    case!(['a', 'x'], "ã"),
    case!(['a', 'j'], "ạ"),
    // e
    case!(['e', 's'], "é"),
    case!(['e', 'f'], "è"),
    case!(['e', 'r'], "ẻ"),
    case!(['e', 'x'], "ẽ"),
    case!(['e', 'j'], "ẹ"),
    // i
    case!(['i', 's'], "í"),
    case!(['i', 'f'], "ì"),
    case!(['i', 'r'], "ỉ"),
    case!(['i', 'x'], "ĩ"),
    case!(['i', 'j'], "ị"),
    // o
    case!(['o', 's'], "ó"),
    case!(['o', 'f'], "ò"),
    case!(['o', 'r'], "ỏ"),
    case!(['o', 'x'], "õ"),
    case!(['o', 'j'], "ọ"),
    // u
    case!(['u', 's'], "ú"),
    case!(['u', 'f'], "ù"),
    case!(['u', 'r'], "ủ"),
    case!(['u', 'x'], "ũ"),
    case!(['u', 'j'], "ụ"),
    // y is a vowel; `yếu`-style spellings work (see KNOWN_DEVIATIONS for `nhà`)
    case!(['y', 's'], "ý"),
    case!(['y', 'f'], "ỳ"),
    case!(['y', 'r'], "ỷ"),
    // ── `z` resets an existing tone to flat ──
    case!(['á', 'z'], "a"),
    case!(['à', 'z'], "a"),
    // ── tone after an onset ──
    case!(['b', 'a', 's'], "bá"),
    case!(['c', 'h', 'a', 'f'], "chà"),
    case!(['g', 'i', 'a', 's'], "giá"),
    // ── tone after a coda ──
    case!(['a', 'n', 's'], "án"),
    case!(['b', 'a', 'c', 'j'], "bạc"),
    case!(['t', 'o', 'a', 'n', 'j'], "toạn"),
    // ── tone on two-vowel nuclei (flat placement on the first) ──
    case!(['a', 'i', 's'], "ái"),
    case!(['o', 'i', 's'], "ói"),
    case!(['u', 'i', 's'], "úi"),
    case!(['a', 'u', 's'], "áu"),
    case!(['a', 'y', 's'], "áy"),
    case!(['u', 'a', 's'], "úa"),
    case!(['i', 'a', 's'], "ía"),
    // ── tone replacement ──
    case!(['a', 's', 'f'], "à"),
    case!(['a', 'f', 'r'], "ả"),
    case!(['e', 's', 'r'], "ẻ"),
    case!(['o', 'f', 's'], "ó"),
    case!(['u', 's', 'j'], "ụ"),
];