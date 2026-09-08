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
}

impl Coda {
    pub const MAX_CODA_BYTES: usize = 2;
    pub const LEN: usize = 9;
    pub const MAX_ID: usize = Self::LEN - 1;

    /// Normalized string representations corresponding 1-to-1 with enum discriminants.
    const ENCODED_STRS: [&'static str; Self::LEN] = ["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

    /// O(1) lookup from a numeric ID. Returns `Err(())` for out-of-bounds IDs.
    #[inline(always)]
    pub const fn from_id(id: usize) -> Result<Self, ()> {
        match id {
            0 => Ok(Self::None),
            1 => Ok(Self::C),
            2 => Ok(Self::Ch),
            3 => Ok(Self::M),
            4 => Ok(Self::N),
            5 => Ok(Self::Ng),
            6 => Ok(Self::Nh),
            7 => Ok(Self::P),
            8 => Ok(Self::T),
            _ => Err(()),
        }
    }

    /// O(1) case-insensitive lookup from an ASCII byte slice (fully optimized for `const fn`).
    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        match bytes {
            [] => Ok(Self::None),

            // Single-byte codas ("c", "m", "n", "p", "t")
            &[byte] => match byte | 0x20 {
                b'c' => Ok(Self::C),
                b'm' => Ok(Self::M),
                b'n' => Ok(Self::N),
                b'p' => Ok(Self::P),
                b't' => Ok(Self::T),
                _ => Err(()),
            },

            // Two-byte codas ("ch", "ng", "nh")
            &[first, second] => {
                let pair = ((first | 0x20) as u16) << 8 | (second | 0x20) as u16;

                match pair {
                    0x6368 => Ok(Self::Ch), // b"ch"
                    0x6E67 => Ok(Self::Ng), // b"ng"
                    0x6E68 => Ok(Self::Nh), // b"nh"
                    _ => Err(()),
                }
            }

            _ => Err(()),
        }
    }
    #[inline(always)]
    pub const fn from_chars(chars: &[char]) -> Result<Self, ()> {
        match chars {
            [] => Ok(Self::None),
            &[c] if c.is_ascii_alphabetic() => Self::from_bytes(&[c as u8]),
            &[c0, c1] if c0.is_ascii_alphabetic() && c1.is_ascii_alphabetic() => {
                Self::from_bytes(&[c0 as u8, c1 as u8])
            }
            _ => Err(()),
        }
    }

    /// Returns the static string representation of the coda.
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        Self::ENCODED_STRS[self as usize]
    }
}

impl fmt::Display for Coda {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Standard FromStr implementation for idiomatic parsing via .parse::<Coda>()
impl FromStr for Coda {
    type Err = ();

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}
