//! E. The special `uo` / `ươ` cycles.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── plain uo ──
    case!(['u', 'o'], "uo"),
    // ── the `w` cycle: uo → uơ → ươ → uow ---
    case!(['u', 'o', 'w'], "uơ"),
    case!(['u', 'o', 'w', 'w'], "ươ"),
    case!(['u', 'o', 'w', 'w', 'w'], "uow"),
    // ── the `o` cycle: uo → uô → uo(+literal o) ──
    case!(['u', 'o', 'o'], "uô"),
    // ── precomposed ư/ơ produce the canonical ươ nucleus ──
    case!(['ư', 'o'], "ươ"),
    case!(['ư', 'ơ'], "ươ"),
    case!(['u', 'ơ'], "ươ"),
    case!(['ư', 'ô'], "ưô"),
    // ── three-vowel nuclei ──
    case!(['u', 'o', 'o', 'i'], "uôi"),
    case!(['u', 'o', 'o', 'i', 's'], "uối"),
    case!(['u', 'o', 'o', 'i', 'j'], "uội"),
    case!(['u', 'o', 'w', 'i'], "ươi"),
    case!(['u', 'o', 'w', 'i', 's'], "ưới"),
    case!(['u', 'o', 'w', 'i', 'f'], "ười"),
    case!(['u', 'o', 'w', 'u'], "ươu"),
    case!(['u', 'o', 'w', 'u', 'j'], "ượu"),
    case!(['u', 'o', 'w', 'e'], "ươe"),
    case!(['u', 'o', 'w', 'a'], "ươa"),
    case!(['u', 'o', 'w', 'w', 'i'], "ươi"),
    // ── uo/ươ with coda ──
    case!(['u', 'o', 'o', 'n', 'g'], "uông"),
    case!(['u', 'o', 'o', 'n', 'g', 's'], "uống"),
    case!(['m', 'u', 'o', 'o', 'n', 's'], "muốn"),
    case!(['u', 'o', 'w', 'o', 'n', 'g'], "uông"),
    case!(['u', 'o', 'w', 's'], "uớ"),
    case!(['u', 'o', 'w', 'f'], "uờ"),
    // ── revert target after normalize: ươ + w rolls back to all-ASCII uo ──
    case!(['ư', 'ơ', 'w'], "uow"),
    case!(['u', 'o', 'w', 'o'], "uô"),
    // ── tone-then-third-vowel keeps the tone where it was placed ──
    case!(['u', 'o', 'w', 's', 'i'], "ưới"),
    case!(['u', 'o', 'o', 's', 'i'], "uối"),
];