use crate::{
    keymapping::{KeyMapping, KeyTarget},
    phonology::{
        decode_vowel,
        rule::{self, check_nucleus_validity, NucleusStatus},
        BaseVowel, Case, Coda, Onset, Shape, Tone,
    },
    BufferChar, RootVowel,
};

use super::syllable::{Cased, Syllable};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformEffect {
    Applied,
    Reverted,
    NotApplicable,
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
    InvalidCoda,
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
pub struct ParseSnapshot {
    phase: ParsePhase,
    syllable: Syllable,
    status: ParseStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser<'a, KM: KeyMapping> {
    syllable: Syllable,
    phase: ParsePhase,
    status: ParseStatus,
    mapping: &'a KM,
}

impl<'a, KM: KeyMapping> Parser<'a, KM> {
    #[inline(always)]
    pub fn new(mapping: &'a KM) -> Self {
        Self {
            syllable: Syllable::default(),
            phase: ParsePhase::Onset,
            status: ParseStatus::Incomplete,
            mapping,
        }
    }

    #[inline(always)]
    pub fn snapshot(&self) -> ParseSnapshot {
        ParseSnapshot {
            phase: self.phase,
            syllable: self.syllable.clone(),
            status: self.status,
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.syllable = Syllable::default();
        self.phase = ParsePhase::Onset;
        self.status = ParseStatus::Incomplete;
    }

    #[inline(always)]
    pub const fn syllable(&self) -> &Syllable {
        &self.syllable
    }

    #[inline(always)]
    pub const fn status(&self) -> ParseStatus {
        self.status
    }

    #[inline(always)]
    pub const fn phase(&self) -> ParsePhase {
        self.phase
    }

    #[inline(always)]
    const fn kill(&mut self, reason: DeadReason) -> ParseStatus {
        self.status = ParseStatus::Dead(reason);
        self.status
    }

    #[inline(always)]
    const fn set_phase(&mut self, phase: ParsePhase) {
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
            ParsePhase::Coda => self.push_coda(input),
        }
    }

    #[inline]
    pub fn parse(&mut self, chars: &[BufferChar]) -> ParseStatus {
        for ch in chars {
            self.push(*ch);
        }
        self.status
    }

    #[inline]
    fn push_onset(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_onset_literal(ch),
            BufferChar::Transform(key) => self.push_onset_transform(key),
        }
    }

    fn push_onset_literal(&mut self, ch: char) -> ParseStatus {
        let syllable = &mut self.syllable;

        // Là phụ âm thì cứ thêm vào
        if is_ascii_consonant(ch) {
            syllable.onset_chars.push(ch);
            return self.status;
        }
        // Nếu là nguyên âm
        else if let Some((base, tone, case)) = decode_vowel(ch) {
            // qu
            // Nếu gặp nguyên âm u
            // Chỉ lấy u nếu là ư thì coi như loại
            if base == BaseVowel::U
                && syllable.onset_chars.len() == 1
                && syllable.onset_chars[0].to_ascii_lowercase() == 'q'
            {
                // Coi u là một consonant trong trường hợp này
                syllable.onset_chars.push(ch);
                return self.status;
            }

            // Xử lí nguyên âm đầu tiên gặp
            //
            // Không có onset cũng hợp lệ:
            // "a", "ă", "â", ...
            if syllable.onset_chars.len() > 0 {
                match Onset::from_chars(&syllable.onset_chars) {
                    Ok(kind) => {
                        syllable.onset = Some(kind);
                    }
                    Err(_) => return self.kill(DeadReason::InvalidOnset),
                }
            }

            syllable.vowels.push(Cased { value: base, case });
            syllable.tone = tone;

            self.set_phase(ParsePhase::Vowel);
            return self.status;
        }

        // Không phải phụ âm cũng không phải nguyên âm túc là các phím rác khác như ?, /
        self.kill(DeadReason::SpecialBurden)
    }

    fn push_onset_transform(&mut self, key: char) -> ParseStatus {
        // Không phải stroke thì coi như key bình thường
        if !self.mapping.stroke(key) {
            return self.push_onset_literal(key);
        }

        match self.try_d_stroke() {
            TransformEffect::Applied => {
                return self.status;
            }
            TransformEffect::Reverted => {
                // Không phải stroke transform.
                // Tức là bị undo về và xem như key thường
                // Ví dụ 'đ' thành 'd' và thêm key vào (trong telex là thêm 'd') thành dd
                return self.push_onset_literal(key);
            }
            TransformEffect::NotApplicable => {
                return self.push_onset_literal(key);
            }
        }
    }

    /// "gi" + nguyên âm nữa -> chuyển 'i' từ chuỗi nguyên âm về onset,
    /// lúc này "gi" trở thành phụ âm (Onset::Gi).
    #[inline(always)]
    fn resolve_gi_onset(&mut self) -> bool {
        let syllable = &mut self.syllable;

        let Some(i) = syllable.vowels.pop() else {
            return false;
        };

        syllable
            .onset_chars
            .push(if i.case == Case::Lower { 'i' } else { 'I' });
        syllable.onset = Some(Onset::Gi);

        true
    }

    #[inline]
    fn push_vowel(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_vowel_literal(ch),
            BufferChar::Transform(key) => self.push_vowel_transform(key),
        }
    }

    #[inline(always)]
    fn push_vowel_precomposed(&mut self, base: BaseVowel, tone: Tone, case: Case) -> bool {
        let syllable = &mut self.syllable;

        // Nếu input là precomposed vowel có tone,
        // không được tạo conflict với tone hiện tại.
        // Ví dụ đang có á rồi mà dùng kĩ thuật uinput truyền thẳng một kí tự ắ vào buffer
        if tone != Tone::Flat && syllable.tone != Tone::Flat {
            return false; // push failed
        }

        syllable.vowels.push(Cased { value: base, case });

        if tone != Tone::Flat {
            syllable.tone = tone;
        }

        true // push succeeded
    }

    fn push_vowel_literal(&mut self, ch: char) -> ParseStatus {
        if let Some((base, tone, case)) = decode_vowel(ch) {
            // ---------------------------------------------------------
            // Special case: gi
            // "gi" chưa quyết định được là gi là onset hay g là onset vì nếu sau i là nguyên âm thì gi là onset
            // còn không thì g là onset
            // ---------------------------------------------------------
            if self.syllable.onset == Some(Onset::G)
                // Mới chỉ có i là nguyên âm
                && self.syllable.vowels[0].value == BaseVowel::I
                && self.syllable.vowels.len() == 1
            {
                // Phía trước là gi rồi
                // và theo sau là một nguyên âm nữa thì
                // lúc này gi sẽ thành phụ âm
                if !self.resolve_gi_onset() {
                    // Không bao giờ failed
                    return self.kill(DeadReason::Unknown);
                }
            }

            // Vietnamese vowel sequence supports at most 3 vowels.
            if self.syllable.vowels.len() >= 3 {
                return self.kill(DeadReason::InvalidVowelSequence);
            }
            // Thêm nguyên âm mới vào chuỗi nguyên âm
            else if !self.push_vowel_precomposed(base, tone, case) {
                return self.kill(DeadReason::InvalidVowelSequence);
            }

            // Hàm này tự xử lí check len rồi
            // chỉ khi có từ hai nguyên âm đổ lên thì mới thực hiện
            // Tức là trong lượt mà nó normalize thì chúng ta đang push
            // nguyên âm thứ 3 vào
            self.normalize_uo();

            return self.status;
        }

        // ---------------------------------------------------------
        // Không phải vowel.
        //
        // Đây là coda → chuyển phase.
        // ---------------------------------------------------------
        if !is_ascii_consonant(ch) {
            // Không phải consonant ASCII, coi như là kí tự đặc biệt
            return self.kill(DeadReason::SpecialBurden);
        }

        // Kiểm tra chuỗi nguyên âm hiện tại có chết chưa có khả thi để pass tiếp ko.
        // Nếu ko valid, coi như dead vì ko thể tạo được nữa do đã kết hợp với coda rồi
        // Ví dụ 'chơa' + 'c' -> 'chơac' (coda invalid)
        if NucleusStatus::Dead == self.validate_nucleus::<3>() {
            return self.kill(DeadReason::InvalidVowelSequence);
        }

        // Hàm này tự xử lí check len rồi
        // chỉ khi có từ hai nguyên âm đổ lên thì mới thực hiện
        // Tức là trong lượt mà nó normalize thì chúng ta đang push
        // một thằng kí tự coda vào
        self.normalize_uo();

        self.syllable.coda_chars.push(ch);
        self.set_phase(ParsePhase::Coda);
        self.status
    }

    #[inline(always)]
    fn try_common_transform(&mut self, key: char) -> Option<TransformEffect> {
        // ---------------------------------------------------------
        // 1. Tone
        // ---------------------------------------------------------
        if let Some(tone) = self.mapping.tone(key) {
            return Some(self.try_tone(tone));
        }

        // ---------------------------------------------------------
        // 2. D-stroke
        // ---------------------------------------------------------
        if self.mapping.stroke(key) {
            return Some(self.try_d_stroke());
        }

        None
    }

    fn push_vowel_transform(&mut self, key: char) -> ParseStatus {
        // Some tức là đã có effect hợp lệ tức là key đã chuyển thành tone hoặc stroke
        if let Some(effect) = self.try_common_transform(key) {
            match effect {
                TransformEffect::Applied => {
                    return self.status;
                }
                TransformEffect::Reverted => {
                    return self.push_vowel_literal(key);
                }
                TransformEffect::NotApplicable => {
                    return self.push_vowel_literal(key);
                }
            }
        }

        // Chưa có effect hợp lệ, check xem có phải shape ko

        // ---------------------------------------------------------
        // 2. Shape
        //
        // Tìm vowel gần nhất có thể nhận shape.
        // ---------------------------------------------------------

        match self.try_shape_transform(key) {
            TransformEffect::Applied => self.status,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.push_vowel_literal(key)
            }
        }
    }

    #[inline]
    fn push_coda(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_coda_literal(ch),
            BufferChar::Transform(key) => self.push_coda_transform(key),
        }
    }

    fn push_coda_literal(&mut self, ch: char) -> ParseStatus {
        if !is_ascii_consonant(ch) {
            // Lúc này nếu có nguyên âm xen giữa coda cũng coi như là key rác
            // Ví dụ 'tiếnog' thì 'o' xen giữa 'ng' xem là rác
            return self.kill(DeadReason::SpecialBurden);
        }

        self.syllable.coda_chars.push(ch);

        // Check ngay sau khi xem để phát hiện coda chết
        match Coda::from_chars(&self.syllable.coda_chars) {
            Ok(kind) => {
                self.syllable.coda = Some(kind);
                self.status
            }

            Err(_) => self.kill(DeadReason::InvalidCoda),
        }
    }

    fn push_coda_transform(&mut self, key: char) -> ParseStatus {
        if let Some(effect) = self.try_common_transform(key) {
            return match effect {
                TransformEffect::Applied => self.status,
                TransformEffect::Reverted | TransformEffect::NotApplicable => {
                    self.push_coda_literal(key)
                }
            };
        }

        match self.try_shape_transform(key) {
            TransformEffect::Applied => self.status,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.push_coda_literal(key)
            }
        }
    }

    #[inline(always)]
    fn is_uo_first(&self) -> bool {
        return self.syllable.vowels.len() > 1
            && self.syllable.vowels[0].value.root() == RootVowel::U
            && self.syllable.vowels[1].value.root() == RootVowel::O;
    }

    #[inline]
    fn try_uo_horn(&mut self) -> TransformEffect {
        let vowels = &self.syllable.vowels;

        return match (vowels[0].value, vowels[1].value) {
            // ươ -> uow
            // Bất cứ trường hợp nào cũng revert kể cả có 3 nguyên âm
            // Ví dụ:
            //
            //  'ươ' -> 'uow'
            //  'ươu' -> 'uouw'
            //  'ươơ' -> 'uoơw'
            //  'ươư' -> 'uoưw'
            //  'ươi' -> 'uoiw'
            //  'ươa' -> 'uoaw'
            //  'ươă' -> 'uoăw'
            //  'ươy' -> 'uoyw'
            //  'ươe' -> 'uoew'
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                // Considering if we should do this or not because this case will be revert to ascii by renderer
                // if vowels.len() == 3
                //     && (vowels[2].value != BaseVowel::U && vowels[2].value != BaseVowel::I)
                // {
                //     return TransformEffect::NotApplicable;
                // }

                self.syllable.vowels[0].value = BaseVowel::U;
                self.syllable.vowels[1].value = BaseVowel::O;
                return TransformEffect::Reverted;
            }

            // ưô -> ươ
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => {
                return TransformEffect::NotApplicable;
            }

            // ưo -> ươ
            (BaseVowel::UHorn, BaseVowel::O) => {
                return self.try_vowel_shape(1, Shape::Horn);
            }

            // uo -> uơ
            (BaseVowel::U, BaseVowel::O) => {
                return self.try_vowel_shape(1, Shape::Horn);
            }
            // uơ -> ươ
            (BaseVowel::U, BaseVowel::OHorn) => {
                return self.try_vowel_shape(0, Shape::Horn);
            }
            // uô -> uơ
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                return self.try_vowel_shape(1, Shape::Horn);
            }
            _ => TransformEffect::NotApplicable,
        };
    }

    #[inline]
    fn try_uo_circumflex(&mut self) -> TransformEffect {
        let vowels = &self.syllable.vowels;
        let first = vowels[0].value;
        let second = vowels[1].value;

        return match (first, second) {
            // ươ -> uô
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::U;

                // Thử thay circumflex vào ơ
                match self.try_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Applied => TransformEffect::Applied,
                    TransformEffect::Reverted => TransformEffect::Reverted, // Never happen vì ư có shape là horn nên ko thể reverted khi nhấn circumflex
                    TransformEffect::NotApplicable => {
                        // rollback
                        self.syllable.vowels[0].value = first;
                        TransformEffect::NotApplicable
                    }
                }
            }

            // ưô ->
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => {
                return TransformEffect::NotApplicable;
            }

            // ưo -> uô
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.syllable.vowels[0].value = BaseVowel::U;
                // Thử thêm circumflex vào o
                match self.try_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Applied => TransformEffect::Applied,
                    TransformEffect::Reverted => TransformEffect::Reverted, // Never happen vì ư có shape là không dấu nên ko thể reverted khi nhấn circumflex
                    TransformEffect::NotApplicable => {
                        // rollback
                        self.syllable.vowels[0].value = first;
                        TransformEffect::NotApplicable
                    }
                }
            }

            // uo -> uô
            (BaseVowel::U, BaseVowel::O) => {
                return self.try_vowel_shape(1, Shape::Circumflex);
            }

            // uơ -> uô
            (BaseVowel::U, BaseVowel::OHorn) => {
                return self.try_vowel_shape(1, Shape::Circumflex);
            }

            // uô -> undo
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                // Considering if we should do this or not because this case will be revert to ascii by renderer
                // if vowels.len() == 3 && vowels[2].value != BaseVowel::I
                // {
                //     return TransformEffect::NotApplicable;
                // }
                self.syllable.vowels[1].value = BaseVowel::O;
                return TransformEffect::Reverted;
            }
            _ => TransformEffect::NotApplicable,
        };
    }

    #[inline(always)]
    fn try_tone(&mut self, tone: Tone) -> TransformEffect {
        let syllable = &mut self.syllable;

        if syllable.vowels.is_empty() {
            return TransformEffect::NotApplicable;
        }

        // Cùng tone → toggle về Flat.
        if syllable.tone == tone {
            syllable.tone = Tone::Flat;
            return TransformEffect::Reverted;
        }

        // Khác tone → thay tone hiện tại.
        syllable.tone = tone;
        TransformEffect::Applied
    }

    #[inline]
    fn try_d_stroke(&mut self) -> TransformEffect {
        for ch in self.syllable.onset_chars.iter_mut().rev() {
            match *ch {
                'd' => {
                    *ch = 'đ';
                    return TransformEffect::Applied;
                }
                'D' => {
                    *ch = 'Đ';
                    return TransformEffect::Applied;
                }
                'đ' => {
                    *ch = 'd';
                    return TransformEffect::Reverted;
                }
                'Đ' => {
                    *ch = 'D';
                    return TransformEffect::Reverted;
                }
                _ => {}
            }
        }

        TransformEffect::NotApplicable
    }

    fn try_shape_transform(&mut self, key: char) -> TransformEffect {
        let vseq_len = self.syllable.vowels.len();

        for index in (0..vseq_len).rev() {
            let base = self.syllable.vowels[index].value;

            let Some(shape) = self.mapping.shape(key, KeyTarget::BaseVowel(base)) else {
                continue;
            };

            // Xử lí special case liên quan tới u và o

            if shape == Shape::Horn && self.is_uo_first() {
                return self.try_uo_horn();
            } else if shape == Shape::Circumflex && self.is_uo_first() {
                return self.try_uo_circumflex();
            }

            let effect = self.try_vowel_shape(index, shape);
            // Chưa thành công thì thử tiếp với các vowel phía trước
            if effect != TransformEffect::NotApplicable {
                return effect;
            }
        }

        TransformEffect::NotApplicable
    }

    #[inline(always)]
    fn validate_nucleus<const N: usize>(&self) -> NucleusStatus {
        let mut buf = [BaseVowel::A; N]; // Mảng cố định N trên Stack (sửa lỗi mảng động)
        let len = self.syllable.vowels.len().min(N);

        for (i, v) in self.syllable.vowels.iter().take(len).enumerate() {
            buf[i] = v.value;
        }

        // Cắt slice chính xác theo độ dài thực tế và kiểm tra
        check_nucleus_validity(&buf[..len])
    }

    #[inline]
    fn try_vowel_shape(&mut self, index: usize, shape: Shape) -> TransformEffect {
        let old = self.syllable.vowels[index].value;

        // ---------------------------------------------------------
        // 1. Undo trước.
        //
        // Nếu vowel hiện tại đã có đúng shape này thì chắc chắn
        // state hiện tại đã hợp lệ. Chỉ cần remove shape.
        // Không cần validate candidate.
        // ---------------------------------------------------------
        if old.shape() == shape && shape != Shape::None {
            self.syllable.vowels[index].value = old.without_shape();
            return TransformEffect::Reverted;
        }

        let Ok(new) = old.replace_shape(shape) else {
            // Shape này không tạo được sequence hợp lệ.
            // Tức là người dùng cố ý tạo shape sai trên mapping ví dụ a + horn
            return TransformEffect::NotApplicable;
        };

        // Apply thử.
        self.syllable.vowels[index].value = new;

        // Nếu chỉ có 1 vowel thì không cần validate sequence.
        if self.syllable.vowels.len() < 2 {
            return TransformEffect::Applied;
        }

        match self.validate_nucleus::<3>() {
            rule::NucleusStatus::Valid => TransformEffect::Applied,
            rule::NucleusStatus::InComplete => TransformEffect::Applied,
            rule::NucleusStatus::Dead => {
                self.syllable.vowels[index].value = old;
                TransformEffect::NotApplicable
            }
        }
    }

    #[inline]
    fn normalize_uo(&mut self) {
        let vowels = &self.syllable.vowels;
        if vowels.len() < 2 {
            return;
        }

        match (vowels[0].value, vowels[1].value) {
            // uơ → ươ
            (BaseVowel::U, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::UHorn;
            }
            // ưo → ươ
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.syllable.vowels[1].value = BaseVowel::OHorn;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
