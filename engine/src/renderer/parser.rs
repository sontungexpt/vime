use crate::{
    interpreter::{KeyContext, KeyInterpreter},
    phonology::{decode_vowel, is_vowel, BaseVowel, Coda, Onset, Shape, Tone},
    BufferChar,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OnsetSegment {
    pub kind: Onset,
    pub chars: Vec<char>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VowelSegment {
    pub bases: Vec<BaseVowel>,
    pub chars: Vec<char>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CodaSegment {
    pub kind: Coda,
    pub chars: Vec<char>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParsedSyllable {
    pub onset: OnsetSegment,
    pub vowels: VowelSegment,
    pub tone: Tone,
    // usize là độ dài của chuỗi nguyên âm ngay lúc phát hiện ra tone
    pub tones: Vec<(Tone, usize)>,
    pub coda: CodaSegment,
    pub fallback_transforms: Vec<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedResult {
    /// Phân tích thành công một âm tiết Tiếng Việt chuẩn ngữ pháp
    Success(ParsedSyllable),

    /// Thành công nhưng chỉ chứa 'd'/'D' hoặc 'đ'/'Đ' + dấu stroke
    OnlyDAndStroke,

    /// Không phải từ Tiếng Việt hợp lệ, nhưng trả về kết quả parse dở dang
    /// để Engine có thể fallback/in ra chuỗi thô mà không làm mất ký tự
    Failure(ParsedSyllable),
}

// 1. Constant BitMask R2 (o, u, c, n, m, g, h, p, t)
const R2_MASK: u32 = {
    let mut mask = 0u32;
    let allowed_r2 = [b'o', b'u', b'c', b'n', b'm', b'g', b'h', b'p', b't'];
    let mut i = 0;
    while i < allowed_r2.len() {
        let bit_index = allowed_r2[i] - b'a';
        mask |= 1 << bit_index;
        i += 1;
    }
    mask
};

impl ParsedSyllable {
    /// Xác định index của nguyên âm sẽ mang Dấu Thanh chuẩn Phonology
    pub fn get_tone_target_index(&self) -> usize {
        let len = self.vowels.bases.len();
        if len <= 1 {
            return 0;
        }

        // 1. Ưu tiên nguyên âm có Dấu mũ / Dấu móc (ê, ơ, ô, ă, â, ư)
        for (i, &v) in self.vowels.bases.iter().enumerate() {
            if !matches!(v.shape(), Shape::None) {
                return i;
            }
        }

        // 2. Quy tắc bỏ dấu Chuẩn (Bộ GD&ĐT)
        if self.coda.kind != Coda::None {
            1 // Có phụ âm cuối -> Dấu đặt ở nguyên âm thứ 2
        } else {
            0 // Không có phụ âm cuối -> Dấu đặt ở nguyên âm thứ 1
        }
    }
}

// =========================================================================
// 2. PARSER ENGINE (Bộ xử lý bóc tách âm tiết)
// =========================================================================

pub struct SyllableParser;

impl SyllableParser {
    #[inline(always)]
    const fn is_ascii_consonant(c: char) -> bool {
        matches!(
            c,
            'b'..='d' | 'f'..='h' | 'j'..='n' | 'p'..='t' | 'v'..='z' |
            'B'..='D' | 'F'..='H' | 'J'..='N' | 'P'..='T' | 'V'..='Z' |
            'đ' | 'Đ'
        )
    }

    /// Parser chính nhận chuỗi BufferChar và KeyInterpreter để bóc tách âm tiết
    pub fn parse<I: KeyInterpreter>(chars: &[BufferChar], interpreter: &I) -> ParsedResult {
        let len = chars.len();

        if len == 0 {
            return ParsedResult::Failure(ParsedSyllable::default());
        } else if len == 2 {
            let first_char = chars[0].as_char();

            if matches!(first_char, 'd' | 'D' | 'đ' | 'Đ') {
                if let BufferChar::Transform(modifier_key) = chars[1] {
                    let context = KeyContext::new(Some(first_char));
                    if let Some((_, Shape::Stroke)) =
                        interpreter.interpret_shape(context, modifier_key)
                    {
                        return ParsedResult::OnlyDAndStroke;
                    }
                }
            }
        }

        let mut idx = 0;
        let mut parsed = ParsedSyllable::default();

        let mut raw_coda_chars: Vec<char> = Vec::new();

        // -----------------------------------------------------------------
        // 1. Quét tìm Onset
        // -----------------------------------------------------------------
        let onset_start = idx; // always 0

        while idx < len && Self::is_ascii_consonant(chars[idx].as_char()) {
            idx += 1;
            if idx - onset_start > Onset::MAX_ONSET_BYTES as usize {
                // Onset quá dài, không hợp lệ
                return ParsedResult::Failure(parsed);
            }
        }

        // has as least one consonant
        if idx > onset_start {
            // Xử lý đặc biệt cho 'qu' và 'gi'
            let onset_len = idx - onset_start;
            if onset_len == 1 {
                let first = chars[onset_start].as_char().to_ascii_lowercase();
                let curr = chars[idx].as_char().to_ascii_lowercase();

                // Xử lý 'qu'
                if first == 'q' && curr == 'u' {
                    idx += 1;
                }
                // Xử lý 'gi' (Ví dụ: "giá" -> onset 'gi', 'a' làm nucleus)
                else if first == 'g'
                    && curr == 'i'
                    && idx + 1 < len
                    && is_vowel(chars[idx + 1].as_char())
                {
                    idx += 1;
                }
            }

            let raw_onset_chars: Vec<char> = chars[onset_start..idx]
                .iter()
                .map(|bc| bc.as_char())
                .collect();

            parsed.onset.kind = Onset::from_chars(&raw_onset_chars).unwrap_or(Onset::None);
            parsed.onset.chars = raw_onset_chars;
        }

        // Only onset, no vowels so not a valid vietnamese word
        if idx >= len {
            return ParsedResult::Failure(parsed);
        }

        // -----------------------------------------------------------------
        // 2. TÁCH RIÊNG: Xử lý Ký tự đầu tiên sau Onset phải là nguyên âm
        // -----------------------------------------------------------------
        let first_vowel = chars[idx].as_char();
        if !is_vowel(first_vowel) {
            return ParsedResult::Failure(parsed);
        } else {
            parsed.vowels.chars.push(first_vowel);
            idx += 1;
        }

        // -----------------------------------------------------------------
        // 3. Tìm các nguyên âm
        // Tại thời điểm này, parsed.vowels BẮT BUỘC đã có ít nhất 1 phần tử
        // Đồng thời chuỗi nguyên âm phải là một chuỗi liền mạch toàn nguyên âm hoặc là transform key nếu có một kí tự nào không phải là nguyên âm và transform key xen giữa chuỗi nguyên âm thì coi như là chuỗi nguyên âm ko hợp lệ
        // Ví dụ: "aco" là chuỗi ko hợp lệ,
        // -----------------------------------------------------------------
        while idx < len {
            let b_char = chars[idx];

            match b_char {
                // A. Ký tự thường (Literal)
                BufferChar::Literal(ch) => {
                    if is_vowel(ch) {
                        parsed.vowels.chars.push(ch);
                    } else {
                        // Kết thúc chuỗi nguyên âm phần còn lại là coda hoặc các transform key xen kẻ coda
                        idx += 1;
                        break;
                    }
                }

                // B. Phím Transform biến đổi (Dấu thanh / Mũ / Móc)
                BufferChar::Transform(key) => {
                    let vowels_len = parsed.vowels.chars.len();

                    // Tone có thể apply cho tất cả nguyên âm
                    if let Some((_, tone)) = interpreter.interpret_tone(KeyContext::default(), key)
                    {
                        parsed.tones.push((tone, vowels_len));
                    } else {
                        // 2. XỬ LÝ DẤU HÌNH DẠNG (Shape - Mũ, Móc)
                        for vowel_char in parsed.vowels.chars.iter().rev() {
                            let context = KeyContext::new(Some(*vowel_char));
                            if let Some((_, shape)) = interpreter.interpret_shape(context, key) {}
                        }
                        // Không phải là transform key thì key phải là nguyên âm để tạo thành một chuỗi nguyên âm liên tiếp
                        if !is_vowel(key) {
                            idx += 1;
                            break;
                        }

                        parsed.vowels.chars.push(key);
                    }
                }
            }
            idx += 1;
        }

        while idx < len {
            if raw_coda_chars.len() > 2 {
                return ParsedResult::Failure(parsed);
            }
            raw_coda_chars.push(chars[idx].as_char());
            idx += 1;
        }

        ParsedResult::Success(parsed)
    }
}
