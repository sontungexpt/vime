//! F. The VNI layout.
//!
//! VNI layout (config/keymapping/default/vni.rs):
//!   tones 1..=5 = acute/grave/hook/tilde/dot, 0 = flat reset
//!   shapes 6 = circumflex (a e o), 7 = breve(a) / horn(o), 8 = horn(u)
//!   stroke 9 = d → đ

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── tones ──
    case!(['a', '1'], "á"),
    case!(['a', '2'], "à"),
    case!(['a', '3'], "ả"),
    case!(['a', '4'], "ã"),
    case!(['a', '5'], "ạ"),
    case!(['e', '1'], "é"),
    case!(['e', '2'], "è"),
    case!(['i', '1'], "í"),
    case!(['o', '1'], "ó"),
    case!(['u', '1'], "ú"),
    case!(['u', '5'], "ụ"),
    case!(['a', '1', '2'], "à"),
    case!(['a', '1', '0'], "a"),
    // ── shapes ──
    case!(['a', '6'], "â"),
    case!(['a', '7'], "ă"),
    case!(['e', '6'], "ê"),
    case!(['o', '6'], "ô"),
    case!(['o', '7'], "ơ"),
    case!(['u', '8'], "ư"),
    // ── shape then tone ──
    case!(['a', '7', '1'], "ắ"),
    case!(['a', '7', '2'], "ằ"),
    case!(['a', '6', '1'], "ấ"),
    case!(['o', '6', '1'], "ố"),
    case!(['u', '8', '1'], "ứ"),
    // ── tone then shape keeps the tone ──
    case!(['a', '1', '7'], "ắ"),
    case!(['a', '1', '6'], "ấ"),
    // ── `d 9` stroke ──
    case!(['d', '9', 'a'], "đa"),
    case!(['d', '9', 'a', '1'], "đá"),
    // ── uo / ươ ──
    case!(['u', 'o', '7'], "uơ"),
    case!(['u', 'o', '7', 'i'], "ươi"),
    case!(['u', 'o', '7', 'i', '1'], "ưới"),
    case!(['u', 'o', '6', 'i'], "uôi"),
    case!(['u', 'o', '6'], "uô"),
    // ── qu ──
    case!(['q', 'u', 'a'], "qua"),
    case!(['q', 'u', 'a', '1'], "quá"),
    // ── real syllables ──
    case!(['m', 'u', 'o', '6', 'n', '1'], "muốn"),
    case!(['c', 'h', 'o', 'a', '1'], "choá"),
    case!(['d', '9', 'e', 'p', '5'], "đẹp"),
    case!(['v', 'i', 'e', '6', 't', '5'], "việt"),
    case!(['n', 'g', 'ư', 'ơ', 'i', '2'], "người"),
    case!(['t', 'h', 'ư', 'ơ', 'n', 'g'], "thương"),
    case!(['x', 'o', 'n', 'g'], "xong"),
];