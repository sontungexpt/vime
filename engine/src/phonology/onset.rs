/// Vietnamese syllable onset.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Onset {
    #[default]
    None = 0,
    B,
    C,
    Ch,
    D,
    Đ,
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
    P,
    Ph,
    QU,
    R,
    S,
    T,
    Th,
    Tr,
    V,
    X,

    /// Sentinel marking the number of real onset variants.
    _Count,
}

impl Onset {
    pub const MAX_ONSET_BYTES: usize = 3;
    pub const LEN: u8 = Self::_Count as u8;
    pub const MAX_ID: u8 = Self::LEN - 1;

    /// O(1) lookup from a numeric ID. Returns `None` for out-of-bounds IDs.
    #[inline(always)]
    pub const fn from_id(id: u8) -> Option<Self> {
        // Safety: real discriminants are contiguous from 0 through MAX_ID.
        if id < Self::LEN {
            Some(unsafe { std::mem::transmute::<u8, Self>(id) })
        } else {
            None
        }
    }

    /// O(1) case-insensitive lookup, compatible with const evaluation.
    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            0 => Some(Self::None),

            // -------------------------------------------------------------
            // 1. One-byte onset (A-Z, a-z).
            // -------------------------------------------------------------
            1 => {
                // `b'a' | 0x20` converts ASCII uppercase to lowercase.
                match bytes[0] | 0x20 {
                    b'b' => Some(Self::B),
                    b'c' => Some(Self::C),
                    b'd' => Some(Self::D),
                    b'g' => Some(Self::G),
                    b'h' => Some(Self::H),
                    b'k' => Some(Self::K),
                    b'l' => Some(Self::L),
                    b'm' => Some(Self::M),
                    b'n' => Some(Self::N),
                    b'p' => Some(Self::P),
                    b'r' => Some(Self::R),
                    b's' => Some(Self::S),
                    b't' => Some(Self::T),
                    b'v' => Some(Self::V),
                    b'x' => Some(Self::X),
                    _ => None,
                }
            }

            // -------------------------------------------------------------
            // 2. Two-byte onset ("ch", "gi", "đ", "gh", "kh", "ng", "ph", "qu", "th", "tr").
            // -------------------------------------------------------------
            2 => {
                // Handle the UTF-8 bytes for 'đ' / 'Đ' separately.
                if bytes[0] == 0xC4 && (bytes[1] == 0x91 || bytes[1] == 0x90) {
                    return Some(Self::Đ);
                }

                // Pack the two ASCII bytes into a lowercase `u16` value.
                let b0 = bytes[0] | 0x20;
                let b1 = bytes[1] | 0x20;
                let pair = ((b0 as u16) << 8) | (b1 as u16);

                match pair {
                    0x6368 => Some(Self::Ch), // b"ch"
                    0x6768 => Some(Self::Gh), // b"gh"
                    0x6769 => Some(Self::Gi), // b"gi"
                    0x6B68 => Some(Self::Kh), // b"kh"
                    0x6E67 => Some(Self::Ng), // b"ng"
                    0x7068 => Some(Self::Ph), // b"ph"
                    0x7175 => Some(Self::QU), // b"qu"
                    0x7468 => Some(Self::Th), // b"th"
                    0x7472 => Some(Self::Tr), // b"tr"
                    _ => None,
                }
            }

            // -------------------------------------------------------------
            // 3. Three-byte onset ("ngh").
            // -------------------------------------------------------------
            3 => {
                let b0 = bytes[0] | 0x20;
                let b1 = bytes[1] | 0x20;
                let b2 = bytes[2] | 0x20;

                if b0 == b'n' && b1 == b'g' && b2 == b'h' {
                    Some(Self::Ngh)
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    /// O(1) lookup from a `&str`, handled case-insensitively.
    #[inline(always)]
    pub const fn from_str(s: &str) -> Option<Self> {
        Self::from_bytes(s.as_bytes())
    }

    const ENCODED_CHARS: &[&str] = &[
        "NONE", "b", "c", "ch", "d", "gi", "đ", "g", "gh", "h", "k", "kh", "l", "m", "n", "ng",
        "ngh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x",
    ];

    /// Converts the onset to a human-readable string.
    #[inline(always)]
    pub const fn to_string(self) -> &'static str {
        Self::ENCODED_CHARS[self as usize]
    }
}
