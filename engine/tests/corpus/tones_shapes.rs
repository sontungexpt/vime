//! D. Tone + shape combinations.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── shape then tone ──
    case!(['a', 'w', 's'], "ắ"),
    case!(['a', 'w', 'f'], "ằ"),
    case!(['a', 'w', 'r'], "ẳ"),
    case!(['a', 'w', 'x'], "ẵ"),
    case!(['a', 'w', 'j'], "ặ"),
    case!(['a', 'a', 's'], "ấ"),
    case!(['a', 'a', 'f'], "ầ"),
    case!(['a', 'a', 'j'], "ậ"),
    case!(['e', 'e', 's'], "ế"),
    case!(['o', 'o', 's'], "ố"),
    case!(['o', 'w', 's'], "ớ"),
    case!(['u', 'w', 's'], "ứ"),
    // ── tone then shape: the shape applies, the tone is preserved ──
    case!(['a', 's', 'w'], "ắ"),
    case!(['a', 'f', 'w'], "ằ"),
    case!(['a', 's', 'a'], "ấ"),
    case!(['e', 's', 'e'], "ế"),
    case!(['o', 's', 'o'], "ố"),
    case!(['o', 's', 'w'], "ớ"),
    case!(['u', 's', 'w'], "ứ"),
    // ── shape then second tone replaces ──
    case!(['a', 'w', 's', 'f'], "ằ"),
    case!(['a', 'a', 's', 'r'], "ẩ"),
    // ── shapes + tones after an onset / with coda ──
    case!(['c', 'h', 'a', 'w', 'n', 's'], "chắn"),
    case!(['m', 'a', 'a', 'n', 's'], "mấn"),
    case!(['k', 'h', 'a', 'a', 's'], "khấ"),
    // ── shape swap keeps tone ──
    case!(['ấ', 'w'], "ắ"),
];