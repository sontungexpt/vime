//! Reading vowel qualities from canonical ASCII raw text.

use super::rules::{
    horn_of, MAX_ATOMS, Q_A, Q_ABREVE, Q_ACIRC, Q_E, Q_ECIRC, Q_I, Q_NONE, Q_O, Q_OCIRC, Q_OHORN,
    Q_U, Q_UHORN, Q_Y,
};

pub(crate) struct Run {
    pub(super) atoms: [u8; MAX_ATOMS],
    pub(super) count: usize,
    pub(super) end: usize,
}

/// Greedily reads vowel qualities starting at `start`.
///
/// Encodings accepted per letter:
///
/// - doubled letters (`aa`, `ee`, `oo`) carry circumflex;
/// - `a` followed by `w` carries breve;
/// - one `w` horns every immediately preceding run of plain `o`/`u`
///   letters (`uow` → ươ), and an unconsumed `w` is itself the quality ư.
pub(crate) fn read_run(bytes: &[u8], start: usize) -> Option<Run> {
    let mut atoms = [Q_NONE; MAX_ATOMS];
    let mut count = 0usize;
    let mut horn_from = 0usize;
    let mut i = start;
    macro_rules! push {
        ($quality:expr) => {{
            if count == MAX_ATOMS {
                return None;
            }
            atoms[count] = $quality;
            count += 1;
        }};
    }
    while i < bytes.len() {
        match bytes[i] {
            b'a' => match bytes.get(i + 1) {
                Some(b'a') => {
                    push!(Q_ACIRC);
                    horn_from = count;
                    i += 2;
                }
                Some(b'w') => {
                    push!(Q_ABREVE);
                    horn_from = count;
                    i += 2;
                }
                _ => {
                    push!(Q_A);
                    horn_from = count;
                    i += 1;
                }
            },
            b'e' => {
                if bytes.get(i + 1) == Some(&b'e') {
                    push!(Q_ECIRC);
                    i += 2;
                } else {
                    push!(Q_E);
                    i += 1;
                }
                horn_from = count;
            }
            b'i' | b'y' => {
                push!(if bytes[i] == b'i' { Q_I } else { Q_Y });
                horn_from = count;
                i += 1;
            }
            b'o' => {
                if bytes.get(i + 1) == Some(&b'o') {
                    push!(Q_OCIRC);
                    horn_from = count;
                    i += 2;
                } else {
                    push!(Q_O);
                    i += 1;
                }
            }
            b'u' => {
                push!(Q_U);
                i += 1;
            }
            b'w' => {
                if horn_from < count {
                    for quality in &mut atoms[horn_from..count] {
                        if let Some(horned) = horn_of(*quality) {
                            *quality = horned;
                        }
                    }
                } else {
                    push!(Q_UHORN);
                }
                horn_from = count;
                i += 1;
            }
            _ => break,
        }
    }
    Some(Run {
        atoms,
        count,
        end: i,
    })
}

#[inline]
pub(crate) fn is_circumflex(quality: u8) -> bool {
    matches!(quality, Q_ACIRC | Q_ECIRC | Q_OCIRC)
}

#[inline]
pub(crate) fn is_breve(quality: u8) -> bool {
    quality == Q_ABREVE
}

#[inline]
pub(crate) fn is_horn(quality: u8) -> bool {
    matches!(quality, Q_OHORN | Q_UHORN)
}

/// Byte width of one vowel atom inside its canonical spelling.
pub(crate) fn atom_width(atoms: &[u8; MAX_ATOMS], index: usize) -> usize {
    let quality = atoms[index];
    let mut width = 1;
    if is_circumflex(quality) || is_breve(quality) {
        width += 1;
    }
    if is_horn(quality) && !atoms.get(index + 1).copied().is_some_and(is_horn) {
        width += 1;
    }
    width
}
