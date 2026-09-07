use std::fmt;
use std::str::FromStr;

/// Syllable Coda (Phụ âm cuối)
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Coda {
    #[default]
    None = 0,
    C = 1,
    Ch = 2,
    M = 3,
    N = 4,
    Ng = 5,
    Nh = 6,
    P = 7,
    T = 8,

    /// Sentinel marking the number of real coda variants.
    _Count,
}

impl Coda {
    pub const MAX_CODA_BYTES: usize = 2;
    pub const LEN: u8 = Self::_Count as u8;
    pub const MAX_ID: u8 = Self::LEN - 1;

    /// Normalized string representations corresponding 1-to-1 with enum discriminants.
    const ENCODED_STRS: [&'static str; Self::LEN as usize] =
        ["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

    /// O(1) lookup from a numeric ID. Returns `None` for out-of-bounds IDs.
    #[inline(always)]
    pub const fn from_id(id: u8) -> Option<Self> {
        if id < Self::LEN {
            // Safety: Real discriminants are contiguous from 0 through MAX_ID.
            Some(unsafe { std::mem::transmute::<u8, Self>(id) })
        } else {
            None
        }
    }

    /// O(1) case-insensitive lookup from an ASCII byte slice (fully optimized for `const fn`).
    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            0 => Some(Self::None),

            // Single-byte codas ("c", "m", "n", "p", "t")
            1 => match bytes[0] | 0x20 {
                b'c' => Some(Self::C),
                b'm' => Some(Self::M),
                b'n' => Some(Self::N),
                b'p' => Some(Self::P),
                b't' => Some(Self::T),
                _ => None,
            },

            // Two-byte codas ("ch", "ng", "nh")
            2 => {
                let b0 = bytes[0] | 0x20;
                let b1 = bytes[1] | 0x20;
                let pair = ((b0 as u16) << 8) | (b1 as u16);

                match pair {
                    0x6368 => Some(Self::Ch), // b"ch"
                    0x6E67 => Some(Self::Ng), // b"ng"
                    0x6E68 => Some(Self::Nh), // b"nh"
                    _ => None,
                }
            }

            _ => None,
        }
    }

    /// O(1) case-insensitive lookup from a `&str`.
    #[inline(always)]
    pub const fn from_str(s: &str) -> Option<Self> {
        Self::from_bytes(s.as_bytes())
    }

    /// Returns the static string representation of the coda.
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        Self::ENCODED_STRS[self as usize]
    }
}

// Standard Display implementation (Zero-allocation string formatting)
impl fmt::Display for Coda {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Standard FromStr implementation for idiomatic parsing via .parse::<Coda>()
impl FromStr for Coda {
    type Err = ();

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes()).ok_or(())
    }
}
