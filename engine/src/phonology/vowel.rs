/// Base ASCII vowel letter independent of shape, tone, and case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
pub enum RootVowel {
    A = 0,
    E = 1,
    I = 2,
    O = 3,
    U = 4,
    Y = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
pub enum Shape {
    None = 0,
    Circumflex = 1,
    Breve = 2,
    Horn = 3,
    Stroke = 4,
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
pub enum Tone {
    #[default]
    Flat = 0, // Ngang
    Acute = 1, // Sắc
    Grave = 2, // Huyền
    Hook = 3,  // Hỏi
    Tilde = 4, // Ngã
    Dot = 5,   // Nặng
}

impl Tone {
    /// Returns the tone at index `id` (`0..=5`).
    ///
    /// Out-of-range indices map to [`Tone::Flat`] instead of panicking.
    #[inline(always)]
    pub const fn from_id(id: usize) -> Self {
        match id {
            0 => Self::Flat,
            1 => Self::Acute,
            2 => Self::Grave,
            3 => Self::Hook,
            4 => Self::Tilde,
            5 => Self::Dot,
            _ => Self::Flat,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
pub enum Case {
    Lower = 0,
    Upper = 1,
}

/// The 12 base vowels of Vietnamese, declared in tone-placement priority order.
///
/// # Bit Layout (`u16`)
/// `[7 bits Reserved | 4 bits Priority ID | 2 bits Shape | 3 bits Root]`
///
/// - **Bits 5..=8**: Tone priority ID (`0..=11`); smaller means higher priority.
/// - **Bits 3..=4**: [`Shape`] (`0..=3`).
/// - **Bits 0..=2**: [`RootVowel`] (`0..=5`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u16)]
pub enum BaseVowel {
    // Priority 0..=1: highest placement priority (ơ, ê)
    OHorn = (Shape::Horn as u16) << 3 | RootVowel::O as u16,
    ECircumflex = (1 << 5) | (Shape::Circumflex as u16) << 3 | RootVowel::E as u16,

    // Priority 2..=5: vowels with diacritics (ă, ô, â, ư)
    ABreve = (2 << 5) | (Shape::Breve as u16) << 3 | RootVowel::A as u16,
    OCircumflex = (3 << 5) | (Shape::Circumflex as u16) << 3 | RootVowel::O as u16,
    ACircumflex = (4 << 5) | (Shape::Circumflex as u16) << 3 | RootVowel::A as u16,
    UHorn = (5 << 5) | (Shape::Horn as u16) << 3 | RootVowel::U as u16,

    // Priority 6..=8: open single vowels (a, o, e)
    A = (6 << 5) | (Shape::None as u16) << 3 | RootVowel::A as u16,
    O = (7 << 5) | (Shape::None as u16) << 3 | RootVowel::O as u16,
    E = (8 << 5) | (Shape::None as u16) << 3 | RootVowel::E as u16,

    // Priority 9..=11: close vowels / semivowels (i, u, y)
    I = (9 << 5) | (Shape::None as u16) << 3 | RootVowel::I as u16,
    U = (10 << 5) | (Shape::None as u16) << 3 | RootVowel::U as u16,
    Y = (11 << 5) | (Shape::None as u16) << 3 | RootVowel::Y as u16,
}

impl BaseVowel {
    pub const LEN: u8 = 12;
    pub const MAX_ID: u8 = Self::LEN - 1;

    /// All base vowels in priority order; the index equals the priority ID.
    const ALL: [BaseVowel; Self::LEN as usize] = [
        BaseVowel::OHorn,
        BaseVowel::ECircumflex,
        BaseVowel::ABreve,
        BaseVowel::OCircumflex,
        BaseVowel::ACircumflex,
        BaseVowel::UHorn,
        BaseVowel::A,
        BaseVowel::O,
        BaseVowel::E,
        BaseVowel::I,
        BaseVowel::U,
        BaseVowel::Y,
    ];

    /// Returns the tone-placement priority ID (`0..=11`); smaller = higher priority.
    #[inline(always)]
    pub const fn id(self) -> usize {
        (self as u16 >> 5) as usize
    }

    #[inline(always)]
    pub const fn from_id(id: usize) -> Result<Self, ()> {
        if id < Self::LEN as usize {
            Ok(Self::ALL[id])
        } else {
            Err(())
        }
    }

    /// Returns the [`BaseVowel`] with the given priority `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not in `0..=11`.
    #[inline(always)]
    pub unsafe fn from_id_unchecked(id: usize) -> Self {
        *Self::ALL.get_unchecked(id)
    }

    /// The [`BaseVowel`] for a root letter and shape, if that combination
    /// exists. Returns `None` for [`Shape::Stroke`] (a consonant stroke, never
    /// a vowel) and for shapes Vietnamese does not attach to that root.
    #[inline(always)]
    pub const fn from_parts(root: RootVowel, shape: Shape) -> Result<Self, ()> {
        match (root, shape) {
            (RootVowel::O, Shape::Horn) => Ok(BaseVowel::OHorn),
            (RootVowel::E, Shape::Circumflex) => Ok(BaseVowel::ECircumflex),
            (RootVowel::A, Shape::Breve) => Ok(BaseVowel::ABreve),
            (RootVowel::O, Shape::Circumflex) => Ok(BaseVowel::OCircumflex),
            (RootVowel::A, Shape::Circumflex) => Ok(BaseVowel::ACircumflex),
            (RootVowel::U, Shape::Horn) => Ok(BaseVowel::UHorn),
            (RootVowel::A, Shape::None) => Ok(BaseVowel::A),
            (RootVowel::O, Shape::None) => Ok(BaseVowel::O),
            (RootVowel::E, Shape::None) => Ok(BaseVowel::E),
            (RootVowel::I, Shape::None) => Ok(BaseVowel::I),
            (RootVowel::U, Shape::None) => Ok(BaseVowel::U),
            (RootVowel::Y, Shape::None) => Ok(BaseVowel::Y),
            _ => Err(()),
        }
    }

    /// Returns the [`Shape`] component of this vowel.
    #[inline(always)]
    pub const fn shape(self) -> Shape {
        let raw = (self as u16 >> 3) & 0x03;
        unsafe { std::mem::transmute(raw) }
    }

    /// Returns the [`RootVowel`] component of this vowel.
    #[inline(always)]
    pub const fn root(self) -> RootVowel {
        let raw = self as u16 & 0x07;
        unsafe { std::mem::transmute(raw) }
    }
}

impl Ord for BaseVowel {
    /// Higher priority (smaller `u16`) sorts as `Greater`.
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse raw comparison: smaller u16 = higher priority.
        (*other as u16).cmp(&(*self as u16))
    }
}

impl PartialOrd for BaseVowel {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// All 144 precomposed Vietnamese vowel characters.
//
// Layout: one block of 12 entries per base vowel, in priority-ID order.
// Within each block, the 6 tones come in Lower/Upper order (Lower, Upper,
// Lower, Upper, ...), so each block is 12 entries long. See `encode`.
const ENCODED_VOWELS: [char; 144] = [
    'ơ', 'Ơ', 'ớ', 'Ớ', 'ờ', 'Ờ', 'ở', 'Ở', 'ỡ', 'Ỡ', 'ợ', 'Ợ', // ID 0: OHorn (ơ)
    'ê', 'Ê', 'ế', 'Ế', 'ề', 'Ề', 'ể', 'Ể', 'ễ', 'Ễ', 'ệ', 'Ệ', // ID 1: ECircumflex (ê)
    'ă', 'Ă', 'ắ', 'Ắ', 'ằ', 'Ằ', 'ẳ', 'Ẳ', 'ẵ', 'Ẵ', 'ặ', 'Ặ', // ID 2: ABreve (ă)
    'ô', 'Ô', 'ố', 'Ố', 'ồ', 'Ồ', 'ổ', 'Ổ', 'ỗ', 'Ỗ', 'ộ', 'Ộ', // ID 3: OCircumflex (ô)
    'â', 'Â', 'ấ', 'Ấ', 'ầ', 'Ầ', 'ẩ', 'Ẩ', 'ẫ', 'Ẫ', 'ậ', 'Ậ', // ID 4: ACircumflex (â)
    'ư', 'Ư', 'ứ', 'Ứ', 'ừ', 'Ừ', 'ử', 'Ử', 'ữ', 'Ữ', 'ự', 'Ự', // ID 5: UHorn (ư)
    'a', 'A', 'á', 'Á', 'à', 'À', 'ả', 'Ả', 'ã', 'Ã', 'ạ', 'Ạ', // ID 6: A (a)
    'o', 'O', 'ó', 'Ó', 'ò', 'Ò', 'ỏ', 'Ỏ', 'õ', 'Õ', 'ọ', 'Ọ', // ID 7: O (o)
    'e', 'E', 'é', 'É', 'è', 'È', 'ẻ', 'Ẻ', 'ẽ', 'Ẽ', 'ẹ', 'Ẹ', // ID 8: E (e)
    'i', 'I', 'í', 'Í', 'ì', 'Ì', 'ỉ', 'Ỉ', 'ĩ', 'Ĩ', 'ị', 'Ị', // ID 9: I (i)
    'u', 'U', 'ú', 'Ú', 'ù', 'Ù', 'ủ', 'Ủ', 'ũ', 'Ũ', 'ụ', 'Ụ', // ID 10: U (u)
    'y', 'Y', 'ý', 'Ý', 'ỳ', 'Ỳ', 'ỷ', 'Ỷ', 'ỹ', 'Ỹ', 'ỵ', 'Ỵ', // ID 11: Y (y)
];

/// Encodes a `(base, tone, case)` triple as the single precomposed character.
///
/// The index is: `(block id * 6 tones + tone) * 2 cases + case`. Because
/// `base.id() <= 11`, `tone <= 5` and `case <= 1`, the index is always in
/// `0..=143`, so the lookup below can never go out of bounds.
#[inline(always)]
pub const fn encode_vowel(base: BaseVowel, tone: Tone, case: Case) -> char {
    let idx = ((base.id() * 6 + tone as usize) << 1) | (case as usize);
    ENCODED_VOWELS[idx]
}

/// Decodes a precomposed Vietnamese vowel character into `(base, tone, case)`.
///
/// Returns `None` if the character is not a Vietnamese vowel.
///
/// # How it works
///
/// This is the fastest decoder variant measured: instead of one giant
/// 144-arm jump table or the previous arithmetic codec, the characters
/// are split into four compact, non-overlapping `match`es keyed by
/// code-point region, so each inner table stays small and predictable
/// (better branch-prediction and cache behavior):
///
/// 1. **ASCII** — `a e i o u y` in both cases (12 arms).
/// 2. **Latin-1 Supplement** — `à á ả ...` scattered over U+00C0..U+00FF (32 arms).
/// 3. **Latin Extended** — `ă ĩ ũ ơ ư` and capitals in U+0100..U+01B0 (10 arms).
/// 4. **Vietnamese block** U+1EA0..U+1EF9 — every toned form (90 arms).
#[inline(always)]
pub const fn decode_vowel(character: char) -> Result<(BaseVowel, Tone, Case), ()> {
    match character as u32 {
        // ─────────────── 1. ASCII (raw keystrokes hit here most) ───────────────
        0x00..=0x7F => match character {
            'a' => Ok((BaseVowel::A, Tone::Flat, Case::Lower)),
            'A' => Ok((BaseVowel::A, Tone::Flat, Case::Upper)),
            'o' => Ok((BaseVowel::O, Tone::Flat, Case::Lower)),
            'O' => Ok((BaseVowel::O, Tone::Flat, Case::Upper)),
            'e' => Ok((BaseVowel::E, Tone::Flat, Case::Lower)),
            'E' => Ok((BaseVowel::E, Tone::Flat, Case::Upper)),
            'i' => Ok((BaseVowel::I, Tone::Flat, Case::Lower)),
            'I' => Ok((BaseVowel::I, Tone::Flat, Case::Upper)),
            'u' => Ok((BaseVowel::U, Tone::Flat, Case::Lower)),
            'U' => Ok((BaseVowel::U, Tone::Flat, Case::Upper)),
            'y' => Ok((BaseVowel::Y, Tone::Flat, Case::Lower)),
            'Y' => Ok((BaseVowel::Y, Tone::Flat, Case::Upper)),
            _ => Err(()),
        },

        // ─────────────── 2. Latin-1 Supplement (U+00C0..U+00FF) ───────────────
        0x80..=0xFF => match character {
            'ê' => Ok((BaseVowel::ECircumflex, Tone::Flat, Case::Lower)),
            'Ê' => Ok((BaseVowel::ECircumflex, Tone::Flat, Case::Upper)),
            'ô' => Ok((BaseVowel::OCircumflex, Tone::Flat, Case::Lower)),
            'Ô' => Ok((BaseVowel::OCircumflex, Tone::Flat, Case::Upper)),
            'â' => Ok((BaseVowel::ACircumflex, Tone::Flat, Case::Lower)),
            'Â' => Ok((BaseVowel::ACircumflex, Tone::Flat, Case::Upper)),
            'á' => Ok((BaseVowel::A, Tone::Acute, Case::Lower)),
            'Á' => Ok((BaseVowel::A, Tone::Acute, Case::Upper)),
            'à' => Ok((BaseVowel::A, Tone::Grave, Case::Lower)),
            'À' => Ok((BaseVowel::A, Tone::Grave, Case::Upper)),
            'ã' => Ok((BaseVowel::A, Tone::Tilde, Case::Lower)),
            'Ã' => Ok((BaseVowel::A, Tone::Tilde, Case::Upper)),
            'ó' => Ok((BaseVowel::O, Tone::Acute, Case::Lower)),
            'Ó' => Ok((BaseVowel::O, Tone::Acute, Case::Upper)),
            'ò' => Ok((BaseVowel::O, Tone::Grave, Case::Lower)),
            'Ò' => Ok((BaseVowel::O, Tone::Grave, Case::Upper)),
            'õ' => Ok((BaseVowel::O, Tone::Tilde, Case::Lower)),
            'Õ' => Ok((BaseVowel::O, Tone::Tilde, Case::Upper)),
            'é' => Ok((BaseVowel::E, Tone::Acute, Case::Lower)),
            'É' => Ok((BaseVowel::E, Tone::Acute, Case::Upper)),
            'è' => Ok((BaseVowel::E, Tone::Grave, Case::Lower)),
            'È' => Ok((BaseVowel::E, Tone::Grave, Case::Upper)),
            'í' => Ok((BaseVowel::I, Tone::Acute, Case::Lower)),
            'Í' => Ok((BaseVowel::I, Tone::Acute, Case::Upper)),
            'ì' => Ok((BaseVowel::I, Tone::Grave, Case::Lower)),
            'Ì' => Ok((BaseVowel::I, Tone::Grave, Case::Upper)),
            'ú' => Ok((BaseVowel::U, Tone::Acute, Case::Lower)),
            'Ú' => Ok((BaseVowel::U, Tone::Acute, Case::Upper)),
            'ù' => Ok((BaseVowel::U, Tone::Grave, Case::Lower)),
            'Ù' => Ok((BaseVowel::U, Tone::Grave, Case::Upper)),
            'ý' => Ok((BaseVowel::Y, Tone::Acute, Case::Lower)),
            'Ý' => Ok((BaseVowel::Y, Tone::Acute, Case::Upper)),
            _ => Err(()),
        },

        // ─────────────── 3. Latin Extended (U+0100..U+01B0) ───────────────
        0x100..=0x1DFF => match character {
            'ơ' => Ok((BaseVowel::OHorn, Tone::Flat, Case::Lower)),
            'Ơ' => Ok((BaseVowel::OHorn, Tone::Flat, Case::Upper)),
            'ă' => Ok((BaseVowel::ABreve, Tone::Flat, Case::Lower)),
            'Ă' => Ok((BaseVowel::ABreve, Tone::Flat, Case::Upper)),
            'ư' => Ok((BaseVowel::UHorn, Tone::Flat, Case::Lower)),
            'Ư' => Ok((BaseVowel::UHorn, Tone::Flat, Case::Upper)),
            'ĩ' => Ok((BaseVowel::I, Tone::Tilde, Case::Lower)),
            'Ĩ' => Ok((BaseVowel::I, Tone::Tilde, Case::Upper)),
            'ũ' => Ok((BaseVowel::U, Tone::Tilde, Case::Lower)),
            'Ũ' => Ok((BaseVowel::U, Tone::Tilde, Case::Upper)),
            _ => Err(()),
        },

        // ─────────────── 4. Vietnamese block (U+1EA0..U+1EF9) ───────────────
        0x1E00..=0x1EFF => match character {
            'ớ' => Ok((BaseVowel::OHorn, Tone::Acute, Case::Lower)),
            'Ớ' => Ok((BaseVowel::OHorn, Tone::Acute, Case::Upper)),
            'ờ' => Ok((BaseVowel::OHorn, Tone::Grave, Case::Lower)),
            'Ờ' => Ok((BaseVowel::OHorn, Tone::Grave, Case::Upper)),
            'ở' => Ok((BaseVowel::OHorn, Tone::Hook, Case::Lower)),
            'Ở' => Ok((BaseVowel::OHorn, Tone::Hook, Case::Upper)),
            'ỡ' => Ok((BaseVowel::OHorn, Tone::Tilde, Case::Lower)),
            'Ỡ' => Ok((BaseVowel::OHorn, Tone::Tilde, Case::Upper)),
            'ợ' => Ok((BaseVowel::OHorn, Tone::Dot, Case::Lower)),
            'Ợ' => Ok((BaseVowel::OHorn, Tone::Dot, Case::Upper)),
            'ế' => Ok((BaseVowel::ECircumflex, Tone::Acute, Case::Lower)),
            'Ế' => Ok((BaseVowel::ECircumflex, Tone::Acute, Case::Upper)),
            'ề' => Ok((BaseVowel::ECircumflex, Tone::Grave, Case::Lower)),
            'Ề' => Ok((BaseVowel::ECircumflex, Tone::Grave, Case::Upper)),
            'ể' => Ok((BaseVowel::ECircumflex, Tone::Hook, Case::Lower)),
            'Ể' => Ok((BaseVowel::ECircumflex, Tone::Hook, Case::Upper)),
            'ễ' => Ok((BaseVowel::ECircumflex, Tone::Tilde, Case::Lower)),
            'Ễ' => Ok((BaseVowel::ECircumflex, Tone::Tilde, Case::Upper)),
            'ệ' => Ok((BaseVowel::ECircumflex, Tone::Dot, Case::Lower)),
            'Ệ' => Ok((BaseVowel::ECircumflex, Tone::Dot, Case::Upper)),
            'ắ' => Ok((BaseVowel::ABreve, Tone::Acute, Case::Lower)),
            'Ắ' => Ok((BaseVowel::ABreve, Tone::Acute, Case::Upper)),
            'ằ' => Ok((BaseVowel::ABreve, Tone::Grave, Case::Lower)),
            'Ằ' => Ok((BaseVowel::ABreve, Tone::Grave, Case::Upper)),
            'ẳ' => Ok((BaseVowel::ABreve, Tone::Hook, Case::Lower)),
            'Ẳ' => Ok((BaseVowel::ABreve, Tone::Hook, Case::Upper)),
            'ẵ' => Ok((BaseVowel::ABreve, Tone::Tilde, Case::Lower)),
            'Ẵ' => Ok((BaseVowel::ABreve, Tone::Tilde, Case::Upper)),
            'ặ' => Ok((BaseVowel::ABreve, Tone::Dot, Case::Lower)),
            'Ặ' => Ok((BaseVowel::ABreve, Tone::Dot, Case::Upper)),
            'ố' => Ok((BaseVowel::OCircumflex, Tone::Acute, Case::Lower)),
            'Ố' => Ok((BaseVowel::OCircumflex, Tone::Acute, Case::Upper)),
            'ồ' => Ok((BaseVowel::OCircumflex, Tone::Grave, Case::Lower)),
            'Ồ' => Ok((BaseVowel::OCircumflex, Tone::Grave, Case::Upper)),
            'ổ' => Ok((BaseVowel::OCircumflex, Tone::Hook, Case::Lower)),
            'Ổ' => Ok((BaseVowel::OCircumflex, Tone::Hook, Case::Upper)),
            'ỗ' => Ok((BaseVowel::OCircumflex, Tone::Tilde, Case::Lower)),
            'Ỗ' => Ok((BaseVowel::OCircumflex, Tone::Tilde, Case::Upper)),
            'ộ' => Ok((BaseVowel::OCircumflex, Tone::Dot, Case::Lower)),
            'Ộ' => Ok((BaseVowel::OCircumflex, Tone::Dot, Case::Upper)),
            'ấ' => Ok((BaseVowel::ACircumflex, Tone::Acute, Case::Lower)),
            'Ấ' => Ok((BaseVowel::ACircumflex, Tone::Acute, Case::Upper)),
            'ầ' => Ok((BaseVowel::ACircumflex, Tone::Grave, Case::Lower)),
            'Ầ' => Ok((BaseVowel::ACircumflex, Tone::Grave, Case::Upper)),
            'ẩ' => Ok((BaseVowel::ACircumflex, Tone::Hook, Case::Lower)),
            'Ẩ' => Ok((BaseVowel::ACircumflex, Tone::Hook, Case::Upper)),
            'ẫ' => Ok((BaseVowel::ACircumflex, Tone::Tilde, Case::Lower)),
            'Ẫ' => Ok((BaseVowel::ACircumflex, Tone::Tilde, Case::Upper)),
            'ậ' => Ok((BaseVowel::ACircumflex, Tone::Dot, Case::Lower)),
            'Ậ' => Ok((BaseVowel::ACircumflex, Tone::Dot, Case::Upper)),
            'ứ' => Ok((BaseVowel::UHorn, Tone::Acute, Case::Lower)),
            'Ứ' => Ok((BaseVowel::UHorn, Tone::Acute, Case::Upper)),
            'ừ' => Ok((BaseVowel::UHorn, Tone::Grave, Case::Lower)),
            'Ừ' => Ok((BaseVowel::UHorn, Tone::Grave, Case::Upper)),
            'ử' => Ok((BaseVowel::UHorn, Tone::Hook, Case::Lower)),
            'Ử' => Ok((BaseVowel::UHorn, Tone::Hook, Case::Upper)),
            'ữ' => Ok((BaseVowel::UHorn, Tone::Tilde, Case::Lower)),
            'Ữ' => Ok((BaseVowel::UHorn, Tone::Tilde, Case::Upper)),
            'ự' => Ok((BaseVowel::UHorn, Tone::Dot, Case::Lower)),
            'Ự' => Ok((BaseVowel::UHorn, Tone::Dot, Case::Upper)),
            'ả' => Ok((BaseVowel::A, Tone::Hook, Case::Lower)),
            'Ả' => Ok((BaseVowel::A, Tone::Hook, Case::Upper)),
            'ạ' => Ok((BaseVowel::A, Tone::Dot, Case::Lower)),
            'Ạ' => Ok((BaseVowel::A, Tone::Dot, Case::Upper)),
            'ỏ' => Ok((BaseVowel::O, Tone::Hook, Case::Lower)),
            'Ỏ' => Ok((BaseVowel::O, Tone::Hook, Case::Upper)),
            'ọ' => Ok((BaseVowel::O, Tone::Dot, Case::Lower)),
            'Ọ' => Ok((BaseVowel::O, Tone::Dot, Case::Upper)),
            'ẻ' => Ok((BaseVowel::E, Tone::Hook, Case::Lower)),
            'Ẻ' => Ok((BaseVowel::E, Tone::Hook, Case::Upper)),
            'ẽ' => Ok((BaseVowel::E, Tone::Tilde, Case::Lower)),
            'Ẽ' => Ok((BaseVowel::E, Tone::Tilde, Case::Upper)),
            'ẹ' => Ok((BaseVowel::E, Tone::Dot, Case::Lower)),
            'Ẹ' => Ok((BaseVowel::E, Tone::Dot, Case::Upper)),
            'ỉ' => Ok((BaseVowel::I, Tone::Hook, Case::Lower)),
            'Ỉ' => Ok((BaseVowel::I, Tone::Hook, Case::Upper)),
            'ị' => Ok((BaseVowel::I, Tone::Dot, Case::Lower)),
            'Ị' => Ok((BaseVowel::I, Tone::Dot, Case::Upper)),
            'ủ' => Ok((BaseVowel::U, Tone::Hook, Case::Lower)),
            'Ủ' => Ok((BaseVowel::U, Tone::Hook, Case::Upper)),
            'ụ' => Ok((BaseVowel::U, Tone::Dot, Case::Lower)),
            'Ụ' => Ok((BaseVowel::U, Tone::Dot, Case::Upper)),
            'ỳ' => Ok((BaseVowel::Y, Tone::Grave, Case::Lower)),
            'Ỳ' => Ok((BaseVowel::Y, Tone::Grave, Case::Upper)),
            'ỷ' => Ok((BaseVowel::Y, Tone::Hook, Case::Lower)),
            'Ỷ' => Ok((BaseVowel::Y, Tone::Hook, Case::Upper)),
            'ỹ' => Ok((BaseVowel::Y, Tone::Tilde, Case::Lower)),
            'Ỹ' => Ok((BaseVowel::Y, Tone::Tilde, Case::Upper)),
            'ỵ' => Ok((BaseVowel::Y, Tone::Dot, Case::Lower)),
            'Ỵ' => Ok((BaseVowel::Y, Tone::Dot, Case::Upper)),
            _ => Err(()),
        },

        _ => Err(()),
    }
}

/// Decodes and keeps only the [`BaseVowel`] part.
#[inline(always)]
pub const fn decode_vowel_base(character: char) -> Result<BaseVowel, ()> {
    match decode_vowel(character) {
        Ok((base, _, _)) => Ok(base),
        Err(()) => Err(()),
    }
}

/// Decodes and keeps only the [`Tone`] part.
#[inline(always)]
pub const fn decode_vowel_tone(character: char) -> Result<Tone, ()> {
    match decode_vowel(character) {
        Ok((_, tone, _)) => Ok(tone),
        Err(()) => Err(()),
    }
}

/// Decodes and keeps only the [`Case`] part.
#[inline(always)]
pub const fn decode_vowel_case(character: char) -> Result<Case, ()> {
    match decode_vowel(character) {
        Ok((_, _, case)) => Ok(case),
        Err(()) => Err(()),
    }
}

/// Whether `ch` is a Vietnamese vowel, without decoding its parts.
///
/// Optimized with compact range checks and lookup masks.
#[inline(always)]
pub const fn is_vowel(ch: char) -> bool {
    let code = ch as u32;

    // 1. ASCII: A..Z and a..z.
    let ascii_shift = code.wrapping_sub(65);
    if ascii_shift <= 57 {
        const ASCII_MASK: u64 = 0x0110411101104111;
        return ((ASCII_MASK >> ascii_shift) & 1) != 0;
    }

    // 2. Vietnamese Extended: every code point in this range is a vowel.
    if code >= 7840 && code <= 7929 {
        return true;
    }

    // 3. Latin-1 Supplement.
    if code >= 192 && code <= 253 {
        let shift = code - 192;
        const LATIN1_MASK_64: u64 = 0x263C370F_263C370F;
        return ((LATIN1_MASK_64 >> shift) & 1) != 0;
    }

    // 4. Latin Extended.
    if code >= 258 && code <= 432 {
        return matches!(
            code - 258,
            0 | 1 | 38 | 39 | 102 | 103 | 158 | 159 | 173 | 174
        );
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected lowercase char for every base (rows = priority ID) × tone (cols = `Tone`).
    const ALL_LOWER: [[char; 6]; 12] = [
        ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'], // 0: OHorn
        ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'], // 1: ECircumflex
        ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'], // 2: ABreve
        ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'], // 3: OCircumflex
        ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'], // 4: ACircumflex
        ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'], // 5: UHorn
        ['a', 'á', 'à', 'ả', 'ã', 'ạ'], // 6: A
        ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'], // 7: O
        ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'], // 8: E
        ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'], // 9: I
        ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'], // 10: U
        ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'], // 11: Y
    ];

    /// Priority IDs 0..=11 must match the discriminants exactly.
    #[test]
    fn base_vowels_are_priority_ordered() {
        for id in 0..12 {
            let base = BaseVowel::from_id(id).unwrap();
            assert_eq!(
                base.id(),
                id,
                "{:?} must have Priority ID {id}",
                base
            );
        }
        // Highest (0) outranks everything; lowest (11) outranks nothing but Y.
        assert!(BaseVowel::OHorn <= BaseVowel::OHorn);
        assert!(BaseVowel::OHorn > BaseVowel::Y);
        assert!(BaseVowel::Y <= BaseVowel::OHorn);
        assert!(BaseVowel::A <= BaseVowel::OHorn);
        assert!(BaseVowel::A > BaseVowel::Y);
    }

    /// Every `(base, tone)` lower-case char both encodes to the expected char and decodes back.
    #[test]
    fn encodes_and_decodes_every_lowercase_vowel() {
        for (id, row) in ALL_LOWER.iter().enumerate() {
            let base = BaseVowel::from_id(id).unwrap();
            for (tone_idx, &expected) in row.iter().enumerate() {
                let tone = Tone::from_id(tone_idx);
                assert_eq!(
                    encode_vowel(base, tone, Case::Lower),
                    expected,
                    "encode({base:?}, {tone:?}, Lower)"
                );
                assert_eq!(
                    decode_vowel(expected),
                    Ok((base, tone, Case::Lower)),
                    "decode('{expected}')"
                );
            }
        }
    }

    /// Every `(base, tone)` upper-case char both encodes to the expected char and decodes back.
    #[test]
    fn encodes_and_decodes_every_uppercase_vowel() {
        for (id, row) in ALL_LOWER.iter().enumerate() {
            let base = BaseVowel::from_id(id).unwrap();
            for (tone_idx, &lower) in row.iter().enumerate() {
                let tone = Tone::from_id(tone_idx);
                let expected = lower.to_uppercase().next().unwrap();
                assert_eq!(
                    encode_vowel(base, tone, Case::Upper),
                    expected,
                    "encode({base:?}, {tone:?}, Upper)"
                );
                assert_eq!(
                    decode_vowel(expected),
                    Ok((base, tone, Case::Upper)),
                    "decode('{expected}')"
                );
            }
        }
    }

    /// Round-trip: encode→decode identity for all 12 × 6 × 2 = 144 combinations.
    #[test]
    fn round_trips_all_vowels() {
        for id in 0..12 {
            let base = BaseVowel::from_id(id).unwrap();
            for tone_idx in 0..6 {
                for &case in &[Case::Lower, Case::Upper] {
                    let tone = Tone::from_id(tone_idx);
                    let ch = encode_vowel(base, tone, case);
                    assert_eq!(
                        decode_vowel(ch),
                        Ok((base, tone, case)),
                        "round trip {base:?} {tone:?} {case:?} -> '{ch}'"
                    );
                }
            }
        }
    }

    /// Every singleton char in the Vietnamese block + Latin Extended-A area must decode.
    #[test]
    fn decodes_every_precomposed_char() {
        let mut seen = 0;
        for c in ALL_LOWER.into_iter().flatten() {
            let decoded = decode_vowel(c).unwrap_or_else(|()| panic!("'{c}' should decode"));
            seen += 1;
            let (base, tone, case) = decoded;
            assert_eq!(
                encode_vowel(base, tone, case),
                c,
                "encode of decode('{c}') must reconstruct it"
            );
        }
        assert_eq!(seen, 72); // 12 × 6: flat rows included
    }

    /// Non-vowel characters must be rejected.
    #[test]
    fn rejects_non_vowels() {
        for ch in ['q', 'w', 'x', 'z', 'đ', '1', '!', ' '] {
            assert_eq!(decode_vowel(ch), Err(()), "decode('{ch}') should fail");
        }
    }

    /// Decoding keeps the signed character's own punctuation-vertical case.
    #[test]
    fn round_trips_ascii_tone_hosts() {
        for ch in ['a', 'e', 'i', 'o', 'u', 'y'] {
            let (base, tone, case) = decode_vowel(ch).unwrap();
            assert_eq!((tone, case), (Tone::Flat, Case::Lower));
            assert_eq!(encode_vowel(base, tone, case), ch);
        }
    }

    /// `is_vowel` must agree with `decode_vowel` on every code point in the
    /// whole Unicode range.
    #[test]
    fn is_vowel_agrees_with_decoder() {
        for cp in 0..=0x10FFFF {
            let Some(ch) = char::from_u32(cp) else {
                continue;
            };
            assert_eq!(
                is_vowel(ch),
                decode_vowel(ch).is_ok(),
                "mismatch at U+{cp:04X}"
            );
        }
    }

    #[test]
    fn test_ascii_vowels_and_consonants() {
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y', 'A', 'E', 'I', 'O', 'U', 'Y'];
        for v in vowels {
            assert!(is_vowel(v), "ASCII Vowel '{}' should return true", v);
        }

        let consonants = [
            'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v',
            'w', 'x', 'z',
        ];
        for c in consonants {
            assert!(!is_vowel(c), "ASCII Consonant '{}' should return false", c);
        }

        // Test ký tự đặc biệt ASCII / số
        for c in ['0', '9', '!', '@', '#', ' ', '\n', '\t'] {
            assert!(
                !is_vowel(c),
                "ASCII Special Symbol '{}' should return false",
                c
            );
        }
    }

    #[test]
    fn test_vietnamese_d_exclusion() {
        // Đ (7872 / U+1EA0) và đ (7873 / U+1EA1) BẮT BUỘC phải là false
        assert!(!is_vowel('Đ'), "Capital Đ (7872) MUST NOT be a vowel");
        assert!(!is_vowel('đ'), "Lowercase đ (7873) MUST NOT be a vowel");
    }

    #[test]
    fn test_vietnamese_diacritics_extended() {
        // Dải Vietnamese Extended (U+1EA0..=U+1EF9)
        let vn_vowels = [
            'Ạ', 'ạ', 'Ả', 'ả', 'Ấ', 'ấ', 'Ầ', 'ầ', 'Ẩ', 'ẩ', 'Ẫ', 'ẫ', 'Ậ', 'ậ', 'Ắ', 'ắ', 'Ằ',
            'ằ', 'Ẳ', 'ẳ', 'Ẵ', 'ẵ', 'Ặ', 'ặ', 'Ẹ', 'ẹ', 'Ẻ', 'ẻ', 'Ẽ', 'ẽ', 'Ế', 'ế', 'Ề', 'ề',
            'Ể', 'ể', 'Ễ', 'ễ', 'Ệ', 'ệ', 'Ỉ', 'ỉ', 'Ị', 'ị', 'Ọ', 'ọ', 'Ỏ', 'ỏ', 'Ố', 'ố', 'Ồ',
            'ồ', 'Ổ', 'ổ', 'Ỗ', 'ỗ', 'Ộ', 'ộ', 'Ớ', 'ớ', 'Ờ', 'ờ', 'Ở', 'ở', 'Ỡ', 'ỡ', 'Ợ', 'ợ',
            'Ụ', 'ụ', 'Ủ', 'ủ', 'Ứ', 'ứ', 'Ừ', 'ừ', 'Ử', 'ử', 'Ữ', 'ữ', 'Ự', 'ự', 'Ỳ', 'ỳ', 'Ỵ',
            'ỵ', 'Ỷ', 'ỷ', 'Ỹ', 'ỹ',
        ];
        for v in vn_vowels {
            assert!(
                is_vowel(v),
                "Vietnamese Extended Vowel '{}' should return true",
                v
            );
        }
    }

    #[test]
    fn test_latin1_and_latin_extended() {
        // Latin-1 Supplement (À..Ý, à..ý)
        let latin1 = [
            'À', 'Á', 'Â', 'Ã', 'È', 'É', 'Ê', 'Ì', 'Í', 'Ò', 'Ó', 'Ô', 'Õ', 'Ù', 'Ú', 'Ý', 'à',
            'á', 'â', 'ã', 'è', 'é', 'ê', 'ì', 'í', 'ò', 'ó', 'ô', 'õ', 'ù', 'ú', 'ý',
        ];
        for v in latin1 {
            assert!(is_vowel(v), "Latin-1 Vowel '{}' should return true", v);
        }

        // Latin Extended (Ă, ă, Ĩ, ĩ, Ũ, ũ, Ơ, ơ, Ư, ư)
        let latin_ext = ['Ă', 'ă', 'Ĩ', 'ĩ', 'Ũ', 'ũ', 'Ơ', 'ơ', 'Ư', 'ư'];
        for v in latin_ext {
            assert!(
                is_vowel(v),
                "Latin Extended Vowel '{}' should return true",
                v
            );
        }

        // Ký tự trong dải Latin-1 nhưng KHÔNG Phải nguyên âm tiếng Việt
        let non_vowels_latin1 = ['Ç', 'ç', 'Ñ', 'ñ', '×', '÷', 'ß', 'ÿ'];
        for nv in non_vowels_latin1 {
            assert!(
                !is_vowel(nv),
                "Latin-1 Non-Vowel '{}' should return false",
                nv
            );
        }
    }

    #[test]
    fn test_full_unicode_range_fuzz() {
        // Quét cạn toàn bộ ký tự Unicode để đảm bảo không bị crash / panic
        for code in 0..=0x10FFFF {
            if let Some(ch) = char::from_u32(code) {
                let _ = is_vowel(ch);
            }
        }
    }
}
