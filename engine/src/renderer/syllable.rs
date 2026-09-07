use crate::phonology::{decode_vowel, is_vowel, BaseVowel, Case, Coda, Onset, Tone};

// =========================================================================
// 1. CẤU TRÚC SPANNED (Lưu Trữ Chữ Gốc Zero-Allocation)
// =========================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Spanned<T, const N: usize = 4> {
    pub kind: T,
    bytes: [u8; N],
    len: u8,
}

impl<T: Copy + Default, const N: usize> Default for Spanned<T, N> {
    fn default() -> Self {
        Self {
            kind: T::default(),
            bytes: [0u8; N],
            len: 0,
        }
    }
}

impl<T: Copy, const N: usize> Spanned<T, N> {
    #[inline(always)]
    pub fn new(kind: T, raw: &[u8]) -> Self {
        let mut bytes = [0u8; N];
        let copy_len = raw.len().min(N);
        bytes[..copy_len].copy_from_slice(&raw[..copy_len]);

        Self {
            kind,
            bytes,
            len: copy_len as u8,
        }
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes[..self.len as usize]) }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

pub type SpannedOnset = Spanned<Onset, 4>;
pub type SpannedCoda = Spanned<Coda, 4>;

#[inline]
fn encode_chars_to_buf<'a>(chars: &[char], buf: &'a mut [u8; 4]) -> Option<&'a [u8]> {
    let mut len = 0;
    for &ch in chars {
        let mut encoded = [0u8; 4];
        let bytes = ch.encode_utf8(&mut encoded).as_bytes();
        if len + bytes.len() > buf.len() {
            return None;
        }
        buf[len..len + bytes.len()].copy_from_slice(bytes);
        len += bytes.len();
    }
    Some(&buf[..len])
}

/// Cấu trúc dữ liệu chứa các thông tin âm tiết đã được bóc tách
#[derive(Debug, Default, Clone)]
pub struct ParsedSyllable {
    pub onset: Onset,           // Phụ âm đầu (VD: Ngh)
    pub vowels: Vec<BaseVowel>, // Chuỗi nguyên âm cốt lõi (VD: [I, ECircumflex])
    pub cases: Vec<Case>,       // Chữ hoa/thường tương ứng với từng nguyên âm
    pub tone: Tone,             // Dấu thanh gom được từ chuỗi (VD: Acute / Sắc)
    pub coda: Coda,             // Phụ âm cuối (VD: Ng)
}

impl ParsedSyllable {
    /// Hàm phân tích mảng char (đã qua xử lý Telex/VNI) thành ParsedSyllable
    pub fn parse(chars: &[char]) -> Option<Self> {
        let mut parsed = ParsedSyllable::default();
        let len = chars.len();
        if len == 0 {
            return None;
        }

        let mut idx = 0;

        // -----------------------------------------------------------------
        // 1. Quét tìm Onset (Giới hạn tối đa 3-4 ký tự, ngắt sớm ngay khi vi phạm)
        // -----------------------------------------------------------------
        let onset_start = idx;

        // Vòng lặp quét phụ âm: Chạy tối đa Onset::MAX_ONSET_BYTES lần
        while idx < len && !is_vowel(chars[idx]) {
            idx += 1;
            // Tối ưu: Dừng sớm ngay lập tức nếu phụ âm dài vượt quá quy định!
            if idx - onset_start > Onset::MAX_ONSET_BYTES as usize {
                return None;
            }
        }

        // Xử lý đặc biệt cho 'qu' (u được tính vào onset)
        // và 'gi' khi đi trước một nguyên âm khác (ví dụ: 'giá' -> onset 'gi', 'a' làm nucleus)
        if idx < len {
            let onset_len = idx - onset_start;
            if onset_len == 1 {
                let first = chars[onset_start];
                let curr = chars[idx];

                // Xử lý 'qu' (vd: "qua")
                if (first as u8 | 0x20) == b'q' && (curr as u8 | 0x20) == b'u' {
                    idx += 1;
                }
                // Xử lý 'gi' (vd: "giá", nhưng "gì" thì 'i' vẫn là nguyên âm)
                else if (first as u8 | 0x20) == b'g'
                    && (curr as u8 | 0x20) == b'i'
                    && idx + 1 < len
                    && is_vowel(chars[idx + 1])
                {
                    idx += 1;
                }
            }
        }

        // Trích xuất Onset KHÔNG tốn Allocation (Zero-Allocation Buffer)
        if idx > onset_start {
            let mut buf = [0u8; 4];
            let bytes = encode_chars_to_buf(&chars[onset_start..idx], &mut buf)?;
            parsed.onset = Onset::from_bytes(bytes).unwrap_or(Onset::None);
        }

        // -----------------------------------------------------------------
        // 2. Quét trích xuất Vowels (Chuỗi nguyên âm & Dấu thanh)
        // -----------------------------------------------------------------
        let vowel_start = idx;
        while idx < len && is_vowel(chars[idx]) {
            if let Some((base, tone, case)) = decode_vowel(chars[idx]) {
                parsed.vowels.push(base);
                parsed.cases.push(case);

                if tone != Tone::Flat {
                    parsed.tone = tone;
                }
                idx += 1;
            } else {
                break;
            }
        }

        // Một âm tiết hợp lệ bắt buộc phải có ít nhất 1 nguyên âm
        if idx == vowel_start {
            return None;
        }

        // -----------------------------------------------------------------
        // 3. Quét phần còn lại làm Coda (Phụ âm cuối - tối đa 2 bytes/chars)
        // -----------------------------------------------------------------
        if idx < len {
            let coda_slice = &chars[idx..];
            // Coda tiếng Việt tối đa 2 ký tự (ng, nh, ch, st...), nhiều hơn => Invalid
            if coda_slice.len() > 2 {
                return None;
            }

            let mut buf = [0u8; 4];
            let bytes = encode_chars_to_buf(coda_slice, &mut buf)?;
            parsed.coda = Coda::from_bytes(bytes).unwrap_or(Coda::None);
        }

        Some(parsed)
    }
}
