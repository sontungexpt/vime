//! Compact IDs, static vowel grammar, and character spelling tables.
//!
//! Everything here is `const`/`static` data plus small total functions.
//! After parsing, the hot path works on these IDs and masks only.

use crate::character::codec::encode_vowel;
use crate::character::{BaseVowel, Case, Tone};
use crate::processor::syllable::SequenceState;

/// Maximum number of vowel qualities in one vowel sequence.
pub(crate) const MAX_ATOMS: usize = 3;

/// Sentinel quality for unused atom slots.
pub(crate) const Q_NONE: u8 = 15;

pub(crate) const Q_OHORN: u8 = 0;
pub(crate) const Q_ECIRC: u8 = 1;
pub(crate) const Q_ABREVE: u8 = 2;
pub(crate) const Q_OCIRC: u8 = 3;
pub(crate) const Q_ACIRC: u8 = 4;
pub(crate) const Q_UHORN: u8 = 5;
pub(crate) const Q_A: u8 = 6;
pub(crate) const Q_O: u8 = 7;
pub(crate) const Q_E: u8 = 8;
pub(crate) const Q_I: u8 = 9;
pub(crate) const Q_U: u8 = 10;
pub(crate) const Q_Y: u8 = 11;

/// Horned counterpart of a plain quality, if any.
pub(crate) const fn horn_of(quality: u8) -> Option<u8> {
    match quality {
        Q_O => Some(Q_OHORN),
        Q_U => Some(Q_UHORN),
        _ => None,
    }
}

macro_rules! quality {
    (A) => {
        Q_A
    };
    (ABreve) => {
        Q_ABREVE
    };
    (ACirc) => {
        Q_ACIRC
    };
    (E) => {
        Q_E
    };
    (ECirc) => {
        Q_ECIRC
    };
    (I) => {
        Q_I
    };
    (O) => {
        Q_O
    };
    (OCirc) => {
        Q_OCIRC
    };
    (OHorn) => {
        Q_OHORN
    };
    (U) => {
        Q_U
    };
    (UHorn) => {
        Q_UHORN
    };
    (Y) => {
        Q_Y
    };
    (_) => {
        Q_NONE
    };
}

/// Coda acceptance classes for vowel sequences.
///
/// Bit 0 of a coda mask means "empty coda". Class codes select precomputed
/// masks: `F` final-only, `C` centering rhymes (take most codas), `S` simple
/// vowels (take any coda).
macro_rules! coda_class {
    (S) => {
        (1 << Coda::COUNT as u64) - 1
    };
    (F) => {
        1
    };
    (C) => {
        (1 << Coda::None as u64)
            | (1 << Coda::C as u64)
            | (1 << Coda::M as u64)
            | (1 << Coda::N as u64)
            | (1 << Coda::P as u64)
            | (1 << Coda::T as u64)
            | (1 << Coda::Ng as u64)
    };
}

/// One definition list drives the `Vowel` enum, its spellings, and the
/// per-sequence metadata, so they can never drift apart.
macro_rules! define_vowels {
    ($( $variant:ident = [ $a:tt $b:tt $c:tt ] , $state:ident , $main:expr , $class:tt ; )*) => {
        /// A Vietnamese vowel sequence as a compact ID.
        ///
        /// Variant names concatenate one token per vowel quality:
        /// `A ABreve ACirc E ECirc I O OCirc OHorn U UHorn Y`.
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        #[repr(u8)]
        pub enum Vowel {
            $( $variant, )*
        }

        impl Vowel {
            pub const COUNT: usize = [ $( stringify!($variant), )* ].len();

            pub(crate) const fn from_u8(value: u8) -> Self {
                match value {
                    $( value if value == Self::$variant as u8 => Self::$variant, )*
                    _ => Self::A,
                }
            }

        }

        pub(super) const ATOM_ROWS: [[u8; MAX_ATOMS]; Vowel::COUNT] =
            [ $( [ quality!($a), quality!($b), quality!($c) ], )* ];

        pub(super) const STATE_ROWS: [SequenceState; Vowel::COUNT] =
            [ $( SequenceState::$state, )* ];

        pub(super) const MAIN_ROWS: [u8; Vowel::COUNT] = [ $( $main, )* ];

        pub(super) const CODA_MASK_ROWS: [u64; Vowel::COUNT] = [ $( coda_class!($class), )* ];
    };
}

define_vowels! {
    A          = [A _ _],         Valid,        0,         S;
    ABreve     = [ABreve _ _],    Valid,        0,         S;
    ACirc      = [ACirc _ _],     Valid,        0,         S;
    E          = [E _ _],         Valid,        0,         S;
    ECirc      = [ECirc _ _],     Valid,        0,         S;
    I          = [I _ _],         Valid,        0,         S;
    O          = [O _ _],         Valid,        0,         S;
    OCirc      = [OCirc _ _],     Valid,        0,         S;
    OHorn      = [OHorn _ _],     Valid,        0,         S;
    U          = [U _ _],         Valid,        0,         S;
    UHorn      = [UHorn _ _],     Valid,        0,         S;
    Y          = [Y _ _],         Valid,        0,         S;
    Ai         = [A I _],         Valid,        0,         F;
    Ao         = [A O _],         Valid,        0,         F;
    Au         = [A U _],         Valid,        0,         F;
    AuCirc     = [ACirc U _],     Valid,        0,         F;
    Ay         = [A Y _],         Valid,        0,         F;
    AyCirc     = [ACirc Y _],     Valid,        0,         F;
    Ia         = [I A _],         Valid,        0,         F;
    Ie         = [I E _],         Transitional, MAIN_NONE, C;
    IeCirc     = [I ECirc _],     Valid,        1,         C;
    Iu         = [I U _],         Valid,        0,         F;
    Ieu        = [I E U],         Transitional, MAIN_NONE, F;
    IeuCirc    = [I ECirc U],     Valid,        1,         F;
    Ye         = [Y E _],         Transitional, MAIN_NONE, C;
    YeCirc     = [Y ECirc _],     Valid,        1,         C;
    Yeu        = [Y E U],         Transitional, MAIN_NONE, F;
    YeuCirc    = [Y ECirc U],     Valid,        1,         F;
    Eo         = [E O _],         Valid,        0,         F;
    Eu         = [E U _],         Transitional, MAIN_NONE, F;
    EuCirc     = [ECirc U _],     Valid,        0,         F;
    Oa         = [O A _],         Valid,        1,         F;
    OaBreve    = [O ABreve _],    Valid,        1,         F;
    Oay        = [O A Y],         Valid,        1,         F;
    Oai        = [O A I],         Valid,        1,         F;
    Oau        = [O A U],         Valid,        1,         F;
    Oao        = [O A O],         Valid,        1,         F;
    Oo         = [O O _],         Transitional, MAIN_NONE, F;
    Oi         = [O I _],         Valid,        0,         F;
    OiCirc     = [O OCirc _],     Valid,        0,         F;
    OiHorn     = [O OHorn _],     Valid,        0,         F;
    Oe         = [O E _],         Valid,        1,         F;
    Oeo        = [O E O],         Valid,        1,         F;
    Ua         = [U A _],         Valid,        0,         F;
    Uao        = [U A O],         Valid,        1,         F;
    UaHorn     = [UHorn A _],     Valid,        0,         C;
    UaCirc     = [U ACirc _],     Valid,        1,         C;
    UayCirc    = [U ACirc Y],     Valid,        1,         F;
    Ue         = [U E _],         Transitional, MAIN_NONE, C;
    UeCirc     = [U ECirc _],     Valid,        1,         C;
    Ui         = [U I _],         Valid,        0,         F;
    UiHorn     = [UHorn I _],     Valid,        0,         F;
    Uy         = [U Y _],         Valid,        1,         F;
    Uyu        = [U Y U],         Valid,        1,         F;
    Uya        = [U Y A],         Valid,        1,         F;
    Uye        = [U Y E],         Transitional, MAIN_NONE, C;
    UyeCirc    = [U Y ECirc],     Valid,        2,         C;
    Uu         = [U U _],         Transitional, MAIN_NONE, F;
    UuHorn     = [UHorn U _],     Valid,        0,         F;
    Uo         = [U O _],         Transitional, MAIN_NONE, C;
    UoCirc     = [U OCirc _],     Valid,        1,         C;
    UoiCirc    = [U OCirc I],     Valid,        1,         F;
    UhornO     = [UHorn O _],     Transitional, MAIN_NONE, C;
    UOhorn     = [U OHorn _],     Valid,        1,         F;
    UoHorn     = [UHorn OHorn _], Valid,        1,         C;
    Uoi        = [U O I],         Transitional, MAIN_NONE, F;
    UOiHorn    = [U OHorn I],     Transitional, MAIN_NONE, F;
    UhornOi    = [UHorn O I],     Transitional, MAIN_NONE, F;
    UoiHorn    = [UHorn OHorn I], Valid,        1,         F;
    Uou        = [U O U],         Transitional, MAIN_NONE, F;
    UOuHorn    = [U OHorn U],     Transitional, MAIN_NONE, F;
    UhornOu    = [UHorn O U],     Transitional, MAIN_NONE, F;
    UouHorn    = [UHorn OHorn U], Valid,        1,         F;
}

pub(crate) const MAIN_NONE: u8 = 0xff;

/// Static grammar entry for one vowel sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VowelRule {
    /// Vowel qualities of the sequence; trailing slots are [`Q_NONE`].
    pub atoms: [u8; MAX_ATOMS],
    /// Number of meaningful entries in `atoms`.
    pub count: u8,
    /// Status when the sequence stands alone (empty coda).
    pub state: SequenceState,
    /// Main-vowel atom index for modern orthography, or [`MAIN_NONE`].
    pub main: u8,
    /// Coda IDs this sequence may precede; bit 0 covers the empty coda.
    pub coda_mask: u64,
}

impl VowelRule {
    /// Can this sequence precede the given coda?
    #[inline]
    pub fn allows_coda(&self, coda: Coda) -> bool {
        self.coda_mask & (1 << coda as u64) != 0
    }
}

const fn atom_count(atoms: [u8; MAX_ATOMS]) -> u8 {
    if atoms[1] == Q_NONE {
        1
    } else if atoms[2] == Q_NONE {
        2
    } else {
        MAX_ATOMS as u8
    }
}

pub(crate) static VOWEL_RULES: [VowelRule; Vowel::COUNT] = build_rules();

const fn build_rules() -> [VowelRule; Vowel::COUNT] {
    let mut rules = [VowelRule {
        atoms: [Q_NONE; MAX_ATOMS],
        count: 0,
        state: SequenceState::Invalid,
        main: MAIN_NONE,
        coda_mask: 0,
    }; Vowel::COUNT];
    let mut i = 0;
    while i < Vowel::COUNT {
        rules[i] = VowelRule {
            count: atom_count(ATOM_ROWS[i]),
            atoms: ATOM_ROWS[i],
            state: STATE_ROWS[i],
            main: MAIN_ROWS[i],
            coda_mask: CODA_MASK_ROWS[i],
        };
        i += 1;
    }
    rules
}

/// Direct-index lookup from up to three qualities to a `Vowel`.
///
/// Key layout: `(q0 * 13 + q1) * 13 + q2` where absent slots use
/// [`LOOKUP_PAD`] (=12). Quality 0 (`a`) never pads a stored key because
/// sequences live at their exact length, so `[u, a]` and `[u, a, a]` land on
/// distinct keys. Value 0 means "no sequence".
const LOOKUP_PAD: usize = 12;
const LOOKUP_STRIDE: usize = 13;
static VOWEL_LOOKUP: [u16; LOOKUP_STRIDE * LOOKUP_STRIDE * LOOKUP_STRIDE] = build_lookup();

pub(crate) const ATOM_COUNTS: [u8; Vowel::COUNT] = build_atom_counts();

const fn build_atom_counts() -> [u8; Vowel::COUNT] {
    let mut counts = [0u8; Vowel::COUNT];
    let mut i = 0;
    while i < Vowel::COUNT {
        counts[i] = atom_count(ATOM_ROWS[i]);
        i += 1;
    }
    counts
}

const fn lookup_key(atoms: [u8; MAX_ATOMS], count: usize) -> usize {
    let q0 = if count > 0 {
        atoms[0] as usize
    } else {
        LOOKUP_PAD
    };
    let q1 = if count > 1 {
        atoms[1] as usize
    } else {
        LOOKUP_PAD
    };
    let q2 = if count > 2 {
        atoms[2] as usize
    } else {
        LOOKUP_PAD
    };
    (q0 * LOOKUP_STRIDE + q1) * LOOKUP_STRIDE + q2
}

const fn build_lookup() -> [u16; LOOKUP_STRIDE * LOOKUP_STRIDE * LOOKUP_STRIDE] {
    let mut table = [0u16; LOOKUP_STRIDE * LOOKUP_STRIDE * LOOKUP_STRIDE];
    let mut i = 0;
    while i < Vowel::COUNT {
        let key = lookup_key(ATOM_ROWS[i], ATOM_COUNTS[i] as usize);
        table[key] = (i + 1) as u16;
        i += 1;
    }
    table
}

/// O(1) vowel identification from parsed qualities.
#[inline]
pub(crate) fn vowel_for(atoms: [u8; MAX_ATOMS], count: usize) -> Option<Vowel> {
    if count == 0 || count > MAX_ATOMS {
        return None;
    }
    let hit = VOWEL_LOOKUP[lookup_key(atoms, count)];
    (hit != 0).then(|| Vowel::from_u8((hit - 1) as u8))
}

#[inline]
pub(crate) fn rule(vowel: Vowel) -> &'static VowelRule {
    &VOWEL_RULES[vowel as usize]
}

/// An initial consonant or consonant cluster as a compact ID.
///
/// Canonical raw spellings use ASCII letters; `đ` is spelled `dd`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Onset {
    None = 0,
    B,
    C,
    Ch,
    D,
    DStroke,
    G,
    Gh,
    Gi,
    H,
    K,
    Kh,
    L,
    M,
    N,
    Ng,
    Ngh,
    Nh,
    P,
    Ph,
    Qu,
    R,
    S,
    T,
    Th,
    Tr,
    V,
    X,
}

impl Onset {
    pub const COUNT: usize = 28;

    /// Identifies an onset from canonical ASCII letters.
    pub(crate) fn identify(bytes: &[u8]) -> Option<Self> {
        Some(match bytes {
            b"" => Self::None,
            b"b" => Self::B,
            b"c" => Self::C,
            b"ch" => Self::Ch,
            b"d" => Self::D,
            b"dd" => Self::DStroke,
            b"g" => Self::G,
            b"gh" => Self::Gh,
            b"gi" => Self::Gi,
            b"h" => Self::H,
            b"k" => Self::K,
            b"kh" => Self::Kh,
            b"l" => Self::L,
            b"m" => Self::M,
            b"n" => Self::N,
            b"ng" => Self::Ng,
            b"ngh" => Self::Ngh,
            b"nh" => Self::Nh,
            b"p" => Self::P,
            b"ph" => Self::Ph,
            b"qu" => Self::Qu,
            b"r" => Self::R,
            b"s" => Self::S,
            b"t" => Self::T,
            b"th" => Self::Th,
            b"tr" => Self::Tr,
            b"v" => Self::V,
            b"x" => Self::X,
            _ => return None,
        })
    }

    /// Vietnamese spelling used when composing output.
    pub fn spelling(self) -> &'static str {
        match self {
            Self::None => "",
            Self::B => "b",
            Self::C => "c",
            Self::Ch => "ch",
            Self::D => "d",
            Self::DStroke => "đ",
            Self::G => "g",
            Self::Gh => "gh",
            Self::Gi => "gi",
            Self::H => "h",
            Self::K => "k",
            Self::Kh => "kh",
            Self::L => "l",
            Self::M => "m",
            Self::N => "n",
            Self::Ng => "ng",
            Self::Ngh => "ngh",
            Self::Nh => "nh",
            Self::P => "p",
            Self::Ph => "ph",
            Self::Qu => "qu",
            Self::R => "r",
            Self::S => "s",
            Self::T => "t",
            Self::Th => "th",
            Self::Tr => "tr",
            Self::V => "v",
            Self::X => "x",
        }
    }
}

/// A final consonant as a compact ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Coda {
    None = 0,
    C,
    Ch,
    M,
    N,
    Ng,
    Nh,
    P,
    T,
}

impl Coda {
    pub const COUNT: usize = 9;

    /// Identifies a coda from canonical ASCII letters.
    pub(crate) fn identify(bytes: &[u8]) -> Option<Self> {
        Some(match bytes {
            b"" => Self::None,
            b"c" => Self::C,
            b"ch" => Self::Ch,
            b"m" => Self::M,
            b"n" => Self::N,
            b"ng" => Self::Ng,
            b"nh" => Self::Nh,
            b"p" => Self::P,
            b"t" => Self::T,
            _ => return None,
        })
    }

    /// Vietnamese spelling used when composing output.
    pub fn spelling(self) -> &'static str {
        match self {
            Self::None => "",
            Self::C => "c",
            Self::Ch => "ch",
            Self::M => "m",
            Self::N => "n",
            Self::Ng => "ng",
            Self::Nh => "nh",
            Self::P => "p",
            Self::T => "t",
        }
    }
}

/// Base ASCII letter of each vowel quality.
pub(crate) const BASE_LETTERS: [char; 12] =
    ['o', 'e', 'a', 'o', 'a', 'u', 'a', 'o', 'e', 'i', 'u', 'y'];

/// Canonical ASCII body spelling of each vowel quality, matching how
/// `serialize` re-encodes parsed words.
const QUALITY_BODIES: [&str; 12] = [
    "ow", "ee", "aw", "oo", "aa", "uw", "a", "o", "e", "i", "u", "y",
];

const fn tone_marker_byte(tone: usize) -> Option<u8> {
    match tone {
        1 => Some(b's'),
        2 => Some(b'f'),
        3 => Some(b'r'),
        4 => Some(b'x'),
        5 => Some(b'j'),
        _ => None,
    }
}

static SPELLINGS: [(char, &str, Option<u8>); 12 * 6] = build_spellings();

/// One row per precomposed Vietnamese character, derived from the codec's
/// closed-form codepoint formula so decoding and rendering never drift apart.
const fn build_spellings() -> [(char, &'static str, Option<u8>); 12 * 6] {
    let mut rows = [('\0', "", None); 12 * 6];
    let mut quality = 0;
    while quality < 12 {
        let mut tone = 0;
        while tone < 6 {
            rows[quality * 6 + tone] = (
                encode_vowel(BaseVowel::from_id(quality), Tone::from_id(tone), Case::Lower),
                QUALITY_BODIES[quality],
                tone_marker_byte(tone),
            );
            tone += 1;
        }
        quality += 1;
    }
    rows
}

/// Maps one precomposed Vietnamese character to its canonical ASCII body and
/// optional tone marker. This is the single expensive-Unicode surface of the
/// engine; combining marks are handled next to it in `normalize`.
pub(crate) fn vietnamese_spelling(character: char) -> Option<(&'static str, Option<u8>)> {
    if character == 'đ' {
        return Some(("dd", None));
    }
    SPELLINGS
        .iter()
        .find(|row| row.0 == character)
        .map(|row| (row.1, row.2))
}
