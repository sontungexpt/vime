//! C. Telex shape keys.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── core shapes ──
    case!(['a', 'w'], "ă"),
    case!(['a', 'a'], "â"),
    case!(['e', 'e'], "ê"),
    case!(['o', 'o'], "ô"),
    case!(['o', 'w'], "ơ"),
    case!(['u', 'w'], "ư"),
    // ── shapes on shaped vowels: replace or revert the shape ──
    case!(['â', 'w'], "ă"),
    case!(['ă', 'a'], "â"),
    case!(['â', 'a'], "aa"), // same shape reverts â → a, key leaks as a vowel
    // ── shapes after an onset ──
    case!(['c', 'h', 'a', 'w'], "chă"),
    case!(['c', 'h', 'a', 'a'], "châ"),
    case!(['c', 'o', 'o'], "cô"),
    case!(['b', 'u', 'w'], "bư"),
    case!(['t', 'h', 'o', 'w'], "thơ"),
    // ── shape on plain a after a coda already started ──
    case!(['a', 'n', 'w'], "ăn"),
    case!(['a', 'n', 'a'], "ân"),
    // ── o/w interplay outside uo context (single vowel) ──
    case!(['o', 'w'], "ơ"),
    case!(['o', 'w', 'o'], "ô"),
    // ── shape toggles ──
    case!(['a', 'w', 'w'], "aw"),
    case!(['e', 'e', 'e'], "ee"),
    case!(['o', 'o', 'o'], "oo"),
];