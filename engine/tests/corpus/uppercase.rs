//! H. Uppercase input.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── uppercase letters ──
    case!(['A'], "A"),
    case!(['E'], "E"),
    case!(['I'], "I"),
    case!(['O'], "O"),
    case!(['U'], "U"),
    case!(['Y'], "Y"),
    case!(['B', 'A'], "BA"),
    case!(['C', 'H', 'A'], "CHA"),
    case!(['D', 'D', 'A'], "ĐA"),
    case!(['Q', 'U', 'A'], "QUA"),
    // ── shapes ──
    case!(['A', 'W'], "Ă"),
    case!(['A', 'A'], "Â"),
    case!(['E', 'E'], "Ê"),
    case!(['O', 'O'], "Ô"),
    case!(['O', 'W'], "Ơ"),
    case!(['U', 'W'], "Ư"),
    // ── shapes after a lowercase vowel keep lowercase; here uppercase → uppercase ──
    case!(['A', 'W', 'S'], "Ắ"),
    case!(['A', 'A', 'S'], "Ấ"),
    case!(['U', 'W', 'S'], "Ứ"),
    case!(['O', 'W', 'S'], "Ớ"),
    // ── tones on uppercase vowels ──
    case!(['A', 'S'], "Á"),
    case!(['A', 'F'], "À"),
    case!(['A', 'R'], "Ả"),
    case!(['A', 'X'], "Ã"),
    case!(['A', 'J'], "Ạ"),
    case!(['E', 'F'], "È"),
    case!(['U', 'S'], "Ú"),
    // uppercase `Y` is a vowel, tone applies to it
    case!(['Y', 'S'], "Ý"),
    // ── uppercase precomposed stays uppercase ──
    case!(['Ắ'], "Ắ"),
    case!(['Ằ'], "Ằ"),
    case!(['Ấ'], "Ấ"),
    case!(['Ệ'], "Ệ"),
    // ── uppercase precomposed transforms ──
    case!(['Ắ', 'f'], "Ằ"),
    case!(['Ắ', 's'], "Ăs"),
    case!(['Ắ', 'w'], "Áw"),
    case!(['Ấ', 'f'], "Ầ"),
    case!(['Ạ', 'w'], "Ặ"),
];