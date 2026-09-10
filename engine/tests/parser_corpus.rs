//! Data-driven behavior corpus for the real `Parser` pipeline:
//!
//! ```text
//! KeyMapping → BufferChar → Parser::push() → Syllable → Renderer
//! ```
//!
//! Every case lists the *exact* order in which characters are pushed. A bare
//! ASCII letter like `'a'` arrives as `BufferChar::Literal` unless the mapping
//! declares it a transform key (tone / shape / stroke). Precomposed
//! Vietnamese vowels (``ạ``, `ắ`, `Ắ`, …) are kept as-is; the corpus never
//! decomposes them.
//!
//! # Reading a case
//!
//! ```text
//! case!(['a', 's'], "á"),
//! ```
//!
//! means:
//!
//! ```text
//! push('a')   // Literal: plain vowel
//! push('s')   // Transform: acute tone
//! → "á"
//! ```
//!
//! Anything the mapping does *not* treat as a transform key is pushed as a
//! literal with `BufferChar::Literal`, exactly like `Engine` does.

use vietnamese_engine::{
    renderer::parser::{DeadReason, ParseStatus},
    BufferChar, DefaultKeyMapping, DefaultRenderer, KeyMapping, Parser, Renderer,
};

// ─────────────────────────────────────────────────────────────────────────────
// Test data types
// ─────────────────────────────────────────────────────────────────────────────

struct TestCase {
    input: &'static [char],
    expected: &'static str,
}

struct DeadCase {
    input: &'static [char],
    expected: ParseStatus,
}

macro_rules! case {
    ([$($ch:expr),* $(,)?], $expected:expr) => {
        TestCase {
            input: &[$($ch),*],
            expected: $expected,
        }
    };
}

macro_rules! dead_case {
    ([$($ch:expr),* $(,)?], $expected:expr) => {
        DeadCase {
            input: &[$($ch),*],
            expected: $expected,
        }
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Runner helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Classifies a character exactly like the engine: a key is `Transform` when
/// the mapping declares it a tone, shape, or stroke key; everything else is a
/// `Literal`. Precomposed vowels never classify as transforms.
fn classify(mapping: &DefaultKeyMapping<'_>, ch: char) -> BufferChar {
    if mapping.is_transform(ch) {
        BufferChar::Transform(ch)
    } else {
        BufferChar::Literal(ch)
    }
}

/// Pushes every character in order, then renders the resulting syllable.
fn run_case(case: &TestCase, mapping: &DefaultKeyMapping<'_>) {
    let mut parser = Parser::new(mapping);

    for &ch in case.input {
        parser.push(classify(mapping, ch));
    }

    let rendered = DefaultRenderer::default().render(parser.syllable());

    assert_eq!(
        rendered,
        case.expected,
        "\ninput={:?}\n  expected = {:?}\n  actual   = {:?}\n  syllable = {:?}\n  status   = {:?}\n  phase    = {:?}",
        case.input,
        case.expected,
        rendered,
        parser.syllable(),
        parser.status(),
        parser.phase(),
    );
}

/// Pushes every character in order and asserts the parser ends up in the
/// expected (dead) status. The syllable that a dead parse renders is not
/// compared, only the `ParseStatus`.
fn run_dead(case: &DeadCase, mapping: &DefaultKeyMapping<'_>) {
    let mut parser = Parser::new(mapping);

    for &ch in case.input {
        parser.push(classify(mapping, ch));
    }

    assert_eq!(
        parser.status(),
        case.expected,
        "\ninput={:?}\n  expected status = {:?}\n  actual status   = {:?}\n  phase = {:?}\n  syllable = {:?}",
        case.input,
        case.expected,
        parser.status(),
        parser.phase(),
        parser.syllable(),
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// A. Basic onset + vowel
// ─────────────────────────────────────────────────────────────────────────────

const A_BASIC: &[TestCase] = &[
    // ── single vowels ──
    case!(['a'], "a"),
    case!(['e'], "e"),
    case!(['i'], "i"),
    case!(['o'], "o"),
    case!(['u'], "u"),
    case!(['y'], "y"),
    // zero onset requires a vowel; plain onset chars render as themselves
    case!(['b'], "b"),
    case!(['c'], "c"),
    case!(['d'], "d"),
    case!(['g'], "g"),
    case!(['h'], "h"),
    case!(['k'], "k"),
    case!(['l'], "l"),
    case!(['m'], "m"),
    case!(['n'], "n"),
    case!(['q'], "q"),
    case!(['r'], "r"),
    case!(['s'], "s"),
    case!(['t'], "t"),
    case!(['v'], "v"),
    case!(['x'], "x"),
    // ── single consonant + vowel ──
    case!(['b', 'a'], "ba"),
    case!(['c', 'a'], "ca"),
    case!(['g', 'a'], "ga"),
    case!(['h', 'a'], "ha"),
    case!(['k', 'a'], "ka"),
    case!(['l', 'a'], "la"),
    case!(['m', 'a'], "ma"),
    case!(['n', 'a'], "na"),
    case!(['p', 'a'], "pa"),
    case!(['r', 'a'], "ra"),
    case!(['t', 'a'], "ta"),
    case!(['v', 'a'], "va"),
    case!(['x', 'a'], "xa"),
    // tone keys double as consonants while no vowel exists yet
    case!(['s', 'a'], "sa"),
    case!(['x', 'a'], "xa"),
    // ── digraph onsets ──
    case!(['c', 'h', 'a'], "cha"),
    case!(['g', 'h', 'a'], "gha"),
    case!(['g', 'i', 'a'], "gia"),
    case!(['k', 'h', 'a'], "kha"),
    case!(['n', 'g', 'a'], "nga"),
    case!(['n', 'g', 'h', 'a'], "ngha"),
    case!(['p', 'h', 'a'], "pha"),
    case!(['t', 'h', 'a'], "tha"),
    case!(['t', 'r', 'a'], "tra"),
    // ── d / đ stroke ──
    case!(['d', 'a'], "da"),
    case!(['d', 'd', 'a'], "đa"),
    // qu keeps `u` as a consonant inside the onset
    case!(['q', 'u', 'a'], "qua"),
    // ── vowel sequences without onset ──
    case!(['a', 'i'], "ai"),
    case!(['a', 'o'], "ao"),
    case!(['a', 'u'], "au"),
    case!(['a', 'y'], "ay"),
    case!(['i', 'a'], "ia"),
    case!(['u', 'a'], "ua"),
    case!(['ư', 'a'], "ưa"),
    case!(['u', 'y'], "uy"),
    case!(['o', 'a'], "oa"),
    case!(['o', 'e'], "oe"),
];

// ─────────────────────────────────────────────────────────────────────────────
// B. Telex tones
// ─────────────────────────────────────────────────────────────────────────────

const B_TONES: &[TestCase] = &[
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
    // y as first letter is an onset consonant (see KNOWN_DEVIATIONS for the
    // intended vowel behavior), so `ý` sentence-final words are unreachable.
    case!(['y', 's'], "ys"),
    case!(['y', 'f'], "yf"),
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
    case!(['t', 'o', 'a', 'n', 'j'], "tọan"),
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

// ─────────────────────────────────────────────────────────────────────────────
// C. Telex shapes
// ─────────────────────────────────────────────────────────────────────────────

const C_SHAPES: &[TestCase] = &[
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

// ─────────────────────────────────────────────────────────────────────────────
// D. Tone + shape combinations
// ─────────────────────────────────────────────────────────────────────────────

const D_SHAPES_TONES: &[TestCase] = &[
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

// ─────────────────────────────────────────────────────────────────────────────
// E. uo / ươ special cases
// ─────────────────────────────────────────────────────────────────────────────

const E_UO: &[TestCase] = &[
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

// ─────────────────────────────────────────────────────────────────────────────
// F. VNI
// ─────────────────────────────────────────────────────────────────────────────

// VNI layout (config/keymapping/default/vni.rs):
//   tones 1..=5 = acute/grave/hook/tilde/dot, 0 = flat reset
//   shapes 6 = circumflex (a e o), 7 = breve(a) / horn(o), 8 = horn(u)
//   stroke 9 = d → đ

const F_VNI: &[TestCase] = &[
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
    case!(['c', 'h', 'o', 'a', '1'], "chóa"),
    case!(['d', '9', 'e', 'p', '5'], "đẹp"),
    case!(['v', 'i', 'e', '6', 't', '5'], "việt"),
    case!(['n', 'g', 'ư', 'ơ', 'i', '2'], "người"),
    case!(['t', 'h', 'ư', 'ơ', 'n', 'g'], "thương"),
    case!(['x', 'o', 'n', 'g'], "xong"),
];

// ─────────────────────────────────────────────────────────────────────────────
// G. Precomposed Vietnamese input
// ─────────────────────────────────────────────────────────────────────────────

const G_PRECOMPOSED: &[TestCase] = &[
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

// ─────────────────────────────────────────────────────────────────────────────
// H. Uppercase
// ─────────────────────────────────────────────────────────────────────────────

const H_UPPERCASE: &[TestCase] = &[
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
    // uppercase `Y` is an onset consonant, tone key falls through as a letter
    case!(['Y', 'S'], "YS"),
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

// ─────────────────────────────────────────────────────────────────────────────
// I. Toggle / revert behavior
// ─────────────────────────────────────────────────────────────────────────────

// Transform keys produce one of three effects:
//   Applied      → the transform took effect (tone/shape set)
//   Reverted     → an equal tone/shape was toggled off; the key then falls
//                  through as a literal and lands in the coda
//   NotApplicable→ nothing to transform; the key falls through as literal
// The expected strings below reflect that *exact* semantics.

const I_TOGGLE: &[TestCase] = &[
    // ── tone → different tone (Applied) ──
    case!(['a', 's'], "á"),
    case!(['á', 'f'], "à"),
    case!(['à', 'r'], "ả"),
    case!(['ả', 'x'], "ã"),
    case!(['ã', 'j'], "ạ"),
    // ── tone → same tone (Reverted + literal spill) ──
    case!(['a', 's', 's'], "as"),
    case!(['á', 's'], "as"),
    case!(['à', 'f'], "af"),
    // ── `z` resets a tone (Applied when the vowel is toned) ──
    case!(['á', 'z'], "a"),
    case!(['ạ', 'z'], "a"),
    // ── shape → same shape (Reverted + literal spill) ──
    case!(['a', 'w', 'w'], "aw"),
    case!(['ă', 'w'], "aw"),
    case!(['â', 'a'], "aa"),
    // ── shape → different shape (Applied) ──
    case!(['a', 'a'], "â"),
    case!(['â', 'w'], "ă"),
    case!(['ă', 'a'], "â"),
    case!(['a', 'w'], "ă"),
    case!(['o', 'o'], "ô"),
    // ── tone + shape toggles keep the other attribute ──
    case!(['a', 's', 'w'], "ắ"),
    case!(['ă', 's'], "ắ"),
    case!(['ắ', 'w'], "áw"),
    case!(['a', 'w', 's'], "ắ"),
    // ── d-stroke toggle ──
    case!(['d', 'd'], "đ"),
    case!(['D', 'D'], "Đ"),
    case!(['d', 'd', 'a'], "đa"),
    // ── uo/ươ revert cycles (see section E) ──
    case!(['u', 'o', 'w'], "uơ"),
    case!(['u', 'o', 'w', 'w'], "ươ"),
    case!(['u', 'o', 'w', 'w', 'w'], "uow"),
    // ── tone on a literal that is a tone key in onset: `s` becomes onset ──
    case!(['s'], "s"),
];

// ─────────────────────────────────────────────────────────────────────────────
// J. Invalid / dead cases
// ─────────────────────────────────────────────────────────────────────────────

const J_DEAD_TELEX: &[DeadCase] = &[
    // ── InvalidOnset ──
    dead_case!(['b', 'c', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['w', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['q', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)), // q needs u
    dead_case!(['z', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['j', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['f', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['g', 'r', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['y', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    // TODO(known gap): Onset has no `Nh` variant, so `nh…` cannot be formed.
    // `nha` should reach the vowel phase; today it dies.
    dead_case!(
        ['n', 'h', 'a'],
        ParseStatus::Dead(DeadReason::InvalidOnset)
    ),
    // ── InvalidVowelSequence ──
    dead_case!(
        ['i', 'e', 'u', 'n'],
        ParseStatus::Dead(DeadReason::InvalidVowelSequence)
    ),
    dead_case!(
        ['a', 'i', 'u', 'n'],
        ParseStatus::Dead(DeadReason::InvalidVowelSequence)
    ),
    dead_case!(
        ['a', 'o', 'i', 'u'],
        ParseStatus::Dead(DeadReason::InvalidVowelSequence)
    ), // fourth vowel
    dead_case!(
        ['b', 'a', 'o', 'i', 'u', 'n'],
        ParseStatus::Dead(DeadReason::InvalidVowelSequence)
    ), // fourth vowel after onset
    // ── InvalidCoda ──
    dead_case!(
        ['a', 'k', 't'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    dead_case!(
        ['a', 'b', 'd'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    dead_case!(
        ['a', 'n', 'n'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    dead_case!(
        ['c', 'h', 'a', 't', 'c'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    dead_case!(
        ['a', 'm', 'h'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    // ── SpecialBurden ──
    dead_case!(['?'], ParseStatus::Dead(DeadReason::SpecialBurden)),
    dead_case!(['!'], ParseStatus::Dead(DeadReason::SpecialBurden)),
    dead_case!(['#'], ParseStatus::Dead(DeadReason::SpecialBurden)),
    dead_case!(['a', '?'], ParseStatus::Dead(DeadReason::SpecialBurden)),
    dead_case!(
        ['a', 'n', 'o'],
        ParseStatus::Dead(DeadReason::SpecialBurden)
    ), // vowel inside a coda
];

const J_DEAD_VNI: &[DeadCase] = &[
    // tone/shape/stroke keys before any vowel are treated as literals in the
    // onset, which are neither consonants nor vowels → SpecialBurden
    dead_case!(['1', 'a'], ParseStatus::Dead(DeadReason::SpecialBurden)),
    dead_case!(['9', 'a'], ParseStatus::Dead(DeadReason::SpecialBurden)),
    dead_case!(['6', 'a'], ParseStatus::Dead(DeadReason::SpecialBurden)),
    // stroke key with no `d` to revert → literal in the vowel phase kills
    dead_case!(
        ['a', '9'],
        ParseStatus::Dead(DeadReason::SpecialBurden)
    ),
    // coda-invalid via VNI layout
    dead_case!(
        ['a', 'b', 'd'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
];

// ─────────────────────────────────────────────────────────────────────────────
// K. Real Vietnamese syllables (telex)
// ─────────────────────────────────────────────────────────────────────────────

const K_SYLLABLES: &[TestCase] = &[
    case!(['t', 'h', 'ủ', 'y'], "thủy"),
    case!(['h', 'ủ', 'y'], "hủy"),
    case!(['k', 'h', 'u', 'y', 'u', 'r'], "khuỷu"), // kh + u + y + u + hook → tone on middle y
    case!(['t', 'h', 'u', 'y', 'r'], "thủy"),
    case!(['t', 'h', 'u', 'o', 'w', 'r'], "thuở"), // u,o,w then tone: keeps [U,OHorn]; tone on ơ
    case!(['c', 'h', 'u', 'y', 'ệ', 'n'], "chuyện"),
    case!(['t', 'h', 'u', 'y', 'ê', 'n'], "thuyên"),
    case!(['n', 'g', 'ư', 'ơ', 'i', 'f'], "người"),
    case!(['n', 'g', 'u', 'o', 'w', 'i', 'f'], "người"),
    case!(['m', 'u', 'ố', 'n'], "muốn"),
    case!(['q', 'u', 'á'], "quá"),
    case!(['q', 'u', 'o', 'o', 'c', 's'], "quốc"),
    case!(['n', 'u', 'o', 'w', 'c', 's'], "nước"),
    case!(['t', 'i', 'e', 'e', 'n', 'f'], "tiền"),
    case!(['c', 'ư', 'ờ', 'i'], "cười"),
    case!(['n', 'g', 'h', 'i', 'a', 'f'], "nghìa"),
    case!(['g', 'i', 'ờ'], "giờ"),
    case!(['c', 'h', 'à', 'n'], "chàn"),
    case!(['a', 'n', 'h'], "anh"),
    case!(['e', 'm'], "em"),
    case!(['ơ', 'n'], "ơn"),
    case!(['ô', 'n', 'g'], "ông"),
    case!(['s', 'ư', 'a', 'r'], "sửa"),
    case!(['m', 'ư', 'a', 's'], "mứa"),
    case!(['q', 'u', 'y'], "quy"),
];

// ─────────────────────────────────────────────────────────────────────────────
// Documented deviations from the standard orthography (the parser today
// produces these; the correct Vietnamese spelling is asserted instead so the
// failure is visible in the test report).
//
// See: renderer/default.rs tone placement (oa/oe + ao), and the missing
// y-after-qu and Nh-onset handling in the parser.
// ─────────────────────────────────────────────────────────────────────────────

const DEVIATIONS_TELEX: &[TestCase] = &[
    // modern orthography: tone on `a` of `oa`
    case!(['t', 'o', 'a', 'n', 's'], "toán"), // today → "tóan"
    case!(['t', 'o', 'a', 'f'], "toà"),       // today → "tòa"
    case!(['h', 'o', 'a', 'j'], "hoạ"),       // today → "họa"
    // tone on `o` of `ao`
    case!(['a', 'o', 's'], "áo"), // today → "aó"
    // `quý`: the `y` must be a vowel after `qu`
    case!(['q', 'u', 'y', 's'], "quý"), // today → "quys"
    // `nh` onset is missing entirely
    case!(['n', 'h', 'a', 'f'], "nhà"), // today → Dead(InvalidOnset)
    // vowel-initial `y` is treated as a consonant onset
    case!(['y', 'e', 'e', 'u', 's'], "yếu"), // today → Dead(InvalidOnset)
];

// ─────────────────────────────────────────────────────────────────────────────
// Test entry points
// ─────────────────────────────────────────────────────────────────────────────

fn run_all<'a>(cases: &'a [TestCase], mapping: &DefaultKeyMapping<'a>) -> usize {
    for case in cases {
        run_case(case, mapping);
    }
    cases.len()
}

fn run_all_dead<'a>(cases: &'a [DeadCase], mapping: &DefaultKeyMapping<'a>) -> usize {
    for case in cases {
        run_dead(case, mapping);
    }
    cases.len()
}

#[test]
fn telex_corpus() {
    let telex = DefaultKeyMapping::telex();
    let n = run_all(A_BASIC, &telex)
        + run_all(B_TONES, &telex)
        + run_all(C_SHAPES, &telex)
        + run_all(D_SHAPES_TONES, &telex)
        + run_all(E_UO, &telex)
        + run_all(G_PRECOMPOSED, &telex)
        + run_all(H_UPPERCASE, &telex)
        + run_all(I_TOGGLE, &telex)
        + run_all(K_SYLLABLES, &telex);
    assert!(
        n >= 300,
        "expected the telex corpus to stay large; got {n} cases"
    );
}

#[test]
fn vni_corpus() {
    let vni = DefaultKeyMapping::vni();
    let n = run_all(F_VNI, &vni);
    assert!(n >= 40, "expected at least 40 VNI cases; got {n}");
}

#[test]
fn dead_corpus() {
    let telex = DefaultKeyMapping::telex();
    let vni = DefaultKeyMapping::vni();
    let n = run_all_dead(J_DEAD_TELEX, &telex) + run_all_dead(J_DEAD_VNI, &vni);
    assert!(n >= 25, "expected at least 25 dead cases; got {n}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Known-deviation test: intentionally FAILS while the underlying bugs exist so
// the report surfaces them. Rename/remove the expected values once fixed.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn known_deviations_from_standard_orthography() {
    let telex = DefaultKeyMapping::telex();
    run_all(DEVIATIONS_TELEX, &telex);
}