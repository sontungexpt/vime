//! G. Precomposed Vietnamese input (kept as-is, never decomposed).

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── single precomposed vowels ──
    case!(['ă'], "ă"),
    case!(['â'], "â"),
    case!(['ê'], "ê"),
    case!(['ô'], "ô"),
    case!(['ơ'], "ơ"),
    case!(['ư'], "ư"),
    case!(['ắ'], "ắ"),
    case!(['ằ'], "ằ"),
    case!(['ầ'], "ầ"),
    case!(['ấ'], "ấ"),
    case!(['ộ'], "ộ"),
    case!(['ớ'], "ớ"),
    case!(['ử'], "ử"),
    // ── precomposed + onset ──
    case!(['c', 'h', 'ả'], "chả"),
    case!(['b', 'ờ'], "bờ"),
    case!(['g', 'i', 'ờ'], "giờ"),
    case!(['n', 'ư', 'ớ', 'c'], "nước"),
    case!(['c', 'ư', 'ờ', 'i'], "cười"),
    // ── tone replacement on a precomposed vowel ──
    case!(['ắ', 'f'], "ằ"),
    case!(['ắ', 'r'], "ẳ"),
    case!(['ấ', 'f'], "ầ"),
    case!(['ấ', 'r'], "ẩ"),
    case!(['ệ', 's'], "ế"),
    case!(['ứ', 'f'], "ừ"),
    case!(['ớ', 'j'], "ợ"),
    // ── tone toggle: same tone removes it, key spills as literal ──
    case!(['ắ', 's'], "ăs"),
    case!(['ấ', 's'], "âs"),
    case!(['ộ', 'j'], "ôj"),
    // ── shape applied to a precomposed vowel (keeps tone) ──
    case!(['á', 'w'], "ắ"),
    case!(['á', 'a'], "ấ"),
    case!(['ạ', 'w'], "ặ"),
    case!(['ẹ', 'e'], "ệ"),
    case!(['o', 'w'], "ơ"),
    // ── shape removed from a precomposed vowel (keeps tone) ──
    case!(['ắ', 'w'], "áw"),
    case!(['ấ', 'a'], "áa"),
    // ── o/ư precomposed in sequences ──
    case!(['ư', 'a'], "ưa"),
    case!(['t', 'h', 'ư', 'a', 'r'], "thửa"),
    case!(['m', 'ư', 'a', 'n'], "mưan"),
    case!(['i', 'ê'], "iê"),
];