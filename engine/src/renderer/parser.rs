use crate::{
    interpreter::{KeyMapping, KeyTarget},
    phonology::{
        decode_vowel, is_vowel,
        rule::{self, match_vowel_sequence},
        BaseVowel, Case, Coda, Onset, Shape, Tone,
    },
    BufferChar,
};
use std::str::FromStr;

// Kiểm tra xem ký tự có phải là kí tự phụ âm ascii
#[inline(always)]
const fn is_ascii_consonant(c: char) -> bool {
    matches!(
        c,
        'b'..='d' | 'f'..='h' | 'j'..='n' | 'p'..='t' | 'v'..='z' |
        'B'..='D' | 'F'..='H' | 'J'..='N' | 'P'..='T' | 'V'..='Z' |
        'đ' | 'Đ'
    )
}

#[inline(always)]
const fn hasD(ch: char) -> bool {
    matches!(ch, 'd' | 'Đ' | 'D' | 'đ')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformResult {
    Applied,
    Undone,
    NotApplicable,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OnsetSegment {
    pub kind: Onset,
    collected: Vec<char>,
}

impl OnsetSegment {
    #[inline(always)]
    pub fn push(&mut self, ch: char) {
        self.collected.push(ch);
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.collected.len()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CodaSegment {
    pub kind: Coda,
    pub collected: Vec<char>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedVowel {
    pub base: BaseVowel,
    pub case: Case,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Syllable {
    pub onset: OnsetSegment,
    pub vowels: Vec<ParsedVowel>,
    pub coda: CodaSegment,
    pub tone: Tone,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsePhase {
    #[default]
    Onset,
    Vowel,
    Coda,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadReason {
    #[default]
    Unknown,
    InvalidOnset,
    InvalidVowelSequence,
    SpecialBurden, // Các kí tự lạ ? , / , \
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    #[default]
    Incomplete,
    Valid,
    Dead(DeadReason),
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Parser<KM: KeyMapping> {
    syllable: Syllable,
    phase: ParsePhase,
    status: ParseStatus,
    mapping: KM,
}

impl<KM: KeyMapping> Parser<KM> {
    #[inline(always)]
    const fn kill(&mut self, reason: DeadReason) -> ParseStatus {
        self.status = ParseStatus::Dead(reason);
        self.status
    }

    #[inline(always)]
    const fn change_phase(&mut self, phase: ParsePhase) {
        self.phase = phase;
    }

    #[inline]
    pub fn push(&mut self, input: BufferChar) -> ParseStatus {
        if let ParseStatus::Dead(_) = self.status {
            return self.status;
        }

        match self.phase {
            ParsePhase::Onset => self.push_onset(input),
            ParsePhase::Vowel => self.push_vowel(input),
            // ParsePhase::Coda => self.push_coda(input),
            _ => ParseStatus::Incomplete,
        }
    }

    #[inline]
    fn push_onset(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_onset_literal(ch),
            BufferChar::Transform(key) => self.push_onset_transform(key),
        }
    }

    #[inline]
    fn push_onset_literal(&mut self, ch: char) -> ParseStatus {
        let syllable = &mut self.syllable;

        // Là phụ âm thì cứ thêm vào
        if is_ascii_consonant(ch) {
            syllable.onset.push(ch);
        }
        // Nếu là nguyên âm
        else if let Some((base, tone, case)) = decode_vowel(ch) {
            // qu
            // Nếu gặp nguyên âm u
            if base == BaseVowel::U
                && syllable.onset.len() == 1
                && syllable.onset.collected[0].to_ascii_lowercase() == 'q'
            // Chỉ lấy u nếu là ư thì coi như loại
            {
                // Coi u là một consonant trong trường hợp này
                syllable.onset.push(ch);
                return self.status;
            }

            // Xử lí nguyên âm đầu tiên gặp
            //
            // Không có onset cũng hợp lệ:
            // "a", "ă", "â", ...
            if syllable.onset.len() > 0 {
                match Onset::from_chars(&syllable.onset.collected) {
                    Ok(kind) => {
                        syllable.onset.kind = kind;
                    }
                    Err(_) => return self.kill(DeadReason::InvalidOnset),
                }
            }

            syllable.vowels.push(ParsedVowel { base, case });
            syllable.tone = tone;

            self.change_phase(ParsePhase::Vowel);
        }

        // Không phải phụ âm cũng không phải nguyên âm túc là các phím rác khác như ?, /
        self.kill(DeadReason::SpecialBurden)
    }

    #[inline(always)]
    fn push_onset_transform(&mut self, key: char) -> ParseStatus {
        if self.try_d_stroke_transform(key) != TransformResult::Applied {
            // Không phải stroke transform.
            return self.push_onset_literal(key);
        }

        // Đã xử lý stroke transform, trả về trạng thái hiện tại.
        self.status
    }

    // #[inline(always)]
    // fn build_onset(&mut self) -> Result<Onset, ()> {
    //     if self.syllable.onset.len() == 0 {
    //         return Ok(Onset::None);
    //     }
    //     match Onset::from_chars(&self.syllable.onset.collected) {
    //         Ok(kind) => {
    //             self.syllable.onset.kind = kind;
    //             Ok(kind)
    //         }
    //         Err(_) => Err(()),
    //     }
    // }

    #[inline(always)]
    fn try_d_stroke_transform(&mut self, key: char) -> TransformResult {
        if !self.mapping.stroke(key) {
            return TransformResult::NotApplicable;
        }

        self.update_d_stroke()
    }

    // Return true if applied, false if undone or not applicable.
    #[inline(always)]
    fn update_d_stroke(&mut self) -> TransformResult {
        for ch in self.syllable.onset.collected.iter_mut().rev() {
            match *ch {
                'd' => {
                    *ch = 'đ';
                    return TransformResult::Applied;
                }
                'D' => {
                    *ch = 'Đ';
                    return TransformResult::Applied;
                }
                'đ' => {
                    *ch = 'd';
                    return TransformResult::Undone;
                }
                'Đ' => {
                    *ch = 'D';
                    return TransformResult::Undone;
                }
                _ => {}
            }
        }

        TransformResult::NotApplicable
    }

    /// "gi" + nguyên âm nữa -> chuyển 'i' từ chuỗi nguyên âm về onset,
    /// lúc này "gi" trở thành phụ âm (Onset::Gi).
    #[inline(always)]
    fn promote_gi_onset(&mut self) -> bool {
        let syllable = &mut self.syllable;

        let Some(i) = syllable.vowels.pop() else {
            return false;
        };

        syllable
            .onset
            .push(if i.case == Case::Lower { 'i' } else { 'I' });
        syllable.onset.kind = Onset::Gi;

        true
    }

    #[inline]
    fn push_vowel(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_vowel_literal(ch),
            BufferChar::Transform(key) => self.push_vowel_transform(key),
        }
    }
    #[inline]
    fn push_vowel_precomposed(&mut self, base: BaseVowel, tone: Tone, case: Case) -> bool {
        let syllable = &mut self.syllable;

        // Nếu input là precomposed vowel có tone,
        // không được tạo conflict với tone hiện tại.
        // Ví dụ đang có á rồi mà dùng kĩ thuật uinput truyền thẳng một kí tự ắ vào buffer
        if tone != Tone::Flat && syllable.tone != Tone::Flat {
            return false; // push failed
        }

        syllable.vowels.push(ParsedVowel { base, case });

        if tone != Tone::Flat {
            syllable.tone = tone;
        }

        true // push succeeded
    }

    #[inline]
    fn push_vowel_literal(&mut self, ch: char) -> ParseStatus {
        if let Some((base, tone, case)) = decode_vowel(ch) {
            // ---------------------------------------------------------
            // Special case: g + i
            //
            // "gi" chưa thể quyết định:
            //
            //   g | i
            //   gi | ...
            //
            // ---------------------------------------------------------
            if self.syllable.onset.kind == Onset::G && self.syllable.vowels[0].base == BaseVowel::I
            {
                // Phía trước là gi rồi và theo sau là một nguyên âm nữa thì lúc này gi sẽ thành phụ âm
                if !self.promote_gi_onset() {
                    // Không bao giờ failed
                    return self.kill(DeadReason::Unknown);
                }
            }

            // Thêm nguyên âm mới vào chuỗi nguyên âm
            if !self.push_vowel_precomposed(base, tone, case) {
                return self.kill(DeadReason::InvalidVowelSequence);
            }
        }

        // ---------------------------------------------------------
        // Không phải vowel.
        //
        // Đây là coda → chuyển phase.
        // ---------------------------------------------------------
        if !is_ascii_consonant(ch) {
            return self.kill(DeadReason::SpecialBurden);
        }

        self.syllable.coda.collected.push(ch);
        self.change_phase(ParsePhase::Coda);
        self.status
    }

    #[inline]
    fn push_vowel_transform(&mut self, key: char) -> ParseStatus {
        // ---------------------------------------------------------
        // 1. Tone
        // ---------------------------------------------------------
        if let Some(tone) = self.mapping.tone(key) {
            // Applied ko được hoặc undo thì coi key như một vowel literal mới
            if self.update_tone(tone) != TransformResult::Applied {
                return self.push_vowel_literal(key);
            }
        }

        match self.try_d_stroke_transform(key) {
            TransformResult::Applied => {
                // Đã Chuyển đổi thành d-stroke
                return self.status;
            }
            TransformResult::Undone => {
                // Undo lại thành kí tự kết thúc phase sang phase coda
                self.syllable.coda.collected.push(key);
                self.change_phase(ParsePhase::Coda);
                return self.status;
            }
            TransformResult::NotApplicable => {}
        }

        // ---------------------------------------------------------
        // 2. Shape
        //
        // Tìm vowel gần nhất có thể nhận shape.
        // ---------------------------------------------------------
        for index in (0..self.syllable.vowels.len()).rev() {
            let base = self.syllable.vowels[index].base;

            let Some(shape) = self.mapping.shape(key, KeyTarget::BaseVowel(base)) else {
                continue;
            };

            if self.apply_vowel_shape(index, shape) {
                return self.update_vowel_status();
            }
        }

        // Không phải transform hợp lệ cho vowel.
        self.kill(DeadReason::Unknown)
    }

    #[inline(always)]
    fn update_tone(&mut self, tone: Tone) -> TransformResult {
        let syllable = &mut self.syllable;

        if syllable.vowels.is_empty() {
            return TransformResult::NotApplicable;
        }

        // Cùng tone → toggle về Flat.
        if syllable.tone == tone {
            syllable.tone = Tone::Flat;
            return TransformResult::Undone;
        }

        // Khác tone → thay tone hiện tại.
        syllable.tone = tone;
        TransformResult::Applied
    }
}
