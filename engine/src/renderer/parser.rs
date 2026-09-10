use crate::{
    interpreter::{KeyMapping, KeyTarget},
    phonology::{
        decode_vowel,
        rule::{self, match_vowel_sequence, SequenceStatus},
        BaseVowel, Case, Coda, Onset, Shape, Tone,
    },
    BufferChar, RootVowel,
};
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
const fn has_d(ch: char) -> bool {
    matches!(ch, 'd' | 'Đ' | 'D' | 'đ')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformEffect {
    Applied,
    Undone,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cased<T> {
    pub value: T,
    pub case: Case,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Syllable {
    pub onset: Option<Onset>,
    pub onset_chars: Vec<char>,

    pub vowels: Vec<Cased<BaseVowel>>,

    pub coda: Option<Coda>,
    pub coda_chars: Vec<char>,

    pub tone: Tone,
}

impl Syllable {
    fn vowel_bases(&self) -> (usize, [BaseVowel; 3]) {
        debug_assert!(self.vowels.len() <= 3);
        let mut buf = [BaseVowel::A; 3];
        for (i, v) in self.vowels.iter().enumerate() {
            buf[i] = v.value;
        }
        (self.vowels.len(), buf)
    }
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
    fn new(mapping: KM) -> Self {
        Self {
            syllable: Syllable::default(),
            phase: ParsePhase::Onset,
            status: ParseStatus::Incomplete,
            mapping,
        }
    }

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
            ParsePhase::Coda => self.push_coda(input),
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

            self.change_phase(ParsePhase::Vowel);
            return self.status;
        }

        // Không phải phụ âm cũng không phải nguyên âm túc là các phím rác khác như ?, /
        self.kill(DeadReason::SpecialBurden)
    }

    #[inline(always)]
    fn push_onset_transform(&mut self, key: char) -> ParseStatus {
        if self.try_d_stroke(key) != TransformEffect::Applied {
            // Không phải stroke transform.
            // Tức là bị undo về key thường hoặc là ko apply được
            return self.push_onset_literal(key);
        }

        // Đã xử lý stroke transform, trả về trạng thái hiện tại.
        self.status
    }

    #[inline(always)]
    fn try_d_stroke(&mut self, key: char) -> TransformEffect {
        if !self.mapping.stroke(key) {
            return TransformEffect::NotApplicable;
        }

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
                    return TransformEffect::Undone;
                }
                'Đ' => {
                    *ch = 'D';
                    return TransformEffect::Undone;
                }
                _ => {}
            }
        }

        TransformEffect::NotApplicable
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
    #[inline]
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
            if self.syllable.onset == Some(Onset::G)
                // Mới chỉ có i là nguyên âm
                && self.syllable.vowels[0].value == BaseVowel::I
                && self.syllable.vowels.len() == 1
            {
                // Phía trước là gi rồi và theo sau là một nguyên âm nữa thì lúc này gi sẽ thành phụ âm
                if !self.promote_gi_onset() {
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

            // Chỉ cần sau uo có thêm một kí tự thì sẽ tự động thành ươ
            self.reformat_uo();

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
        {
            let (len, bases) = self.syllable.vowel_bases();
            if SequenceStatus::Dead == match_vowel_sequence(&bases[..len]) {
                return self.kill(DeadReason::InvalidVowelSequence);
            }
        }

        self.reformat_uo();

        self.syllable.coda_chars.push(ch);
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
            match self.try_tone(tone) {
                TransformEffect::Applied => {}
                TransformEffect::Undone => return self.push_vowel_literal(key),
                TransformEffect::NotApplicable => {
                    return self.push_vowel_literal(key);
                }
            }
        }

        match self.try_d_stroke(key) {
            TransformEffect::Applied => {
                // Đã Chuyển đổi thành d-stroke
                return self.status;
            }
            TransformEffect::Undone => {
                // Bị undo lại thành kí tự thì push vô như bình thường
                return self.push_vowel_literal(key);
            }
            TransformEffect::NotApplicable => {}
        }

        // ---------------------------------------------------------
        // 2. Shape
        //
        // Tìm vowel gần nhất có thể nhận shape.
        // ---------------------------------------------------------
        let vseq_len = self.syllable.vowels.len();
        for index in (0..vseq_len).rev() {
            let base = self.syllable.vowels[index].value;

            let Some(shape) = self.mapping.shape(key, KeyTarget::BaseVowel(base)) else {
                continue;
            };

            // Trường hợp đặc biệt có thể xảy ra với các âm uo hoặc uou
            //
            // Phát hiện có hiệu ứng Horn tức là chỉ có thể có trên o hoặc u
            if shape == Shape::Horn
                // Phải có ít nhất hai nguyên âm thì mới có hiệu ứng đặc biệt
                && vseq_len > 1
                // Nguyên âm đầu tiên phải là u mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[0].value.root() == RootVowel::U
                // Nguyên âm thứ hai phải là o mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[1].value.root() == RootVowel::O
            {
                match self.try_uo_horn(index) {
                    TransformEffect::Applied => {
                        return self.status;
                    }
                    TransformEffect::Undone => {
                        return self.push_vowel_literal(key);
                    }
                    TransformEffect::NotApplicable => {}
                }
            }
            // Hiệu ứng uô hoặc uơ -> uô hoặc ưo thành uô
            else if shape == Shape::Circumflex
                && vseq_len > 1
                // Nguyên âm đầu tiên phải là u mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[0].value.root() == RootVowel::U
                // Nguyên âm thứ hai phải là o mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[1].value.root() == RootVowel::O
            {
                // uô
                match self.try_uo_circumflex(index) {
                    TransformEffect::Applied => {
                        return self.status;
                    }
                    TransformEffect::Undone => {
                        return self.push_vowel_literal(key);
                    }
                    TransformEffect::NotApplicable => {}
                }
            }

            match self.try_vowel_shape(index, shape) {
                TransformEffect::Applied => {
                    return self.status;
                }
                TransformEffect::Undone => {
                    return self.push_vowel_literal(key);
                }
                TransformEffect::NotApplicable => {
                    // Shape này không tạo được sequence hợp lệ.
                    // Thử vowel đứng trước.
                }
            }
        }

        // Không phải transform hợp lệ thì coi như key thường
        return self.push_vowel_literal(key);
    }
    #[inline(always)]
    fn try_uo_horn(&mut self, paused_index: usize) -> TransformEffect {
        let vowels = &self.syllable.vowels;

        return match (vowels[0].value, vowels[1].value) {
            // ươ -> uow
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::U;
                self.syllable.vowels[1].value = BaseVowel::O;
                return TransformEffect::Undone;
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

    #[inline(always)]
    fn try_uo_circumflex(&mut self, paused_index: usize) -> TransformEffect {
        let vowels = &self.syllable.vowels;
        let first = vowels[0].value;
        let second = vowels[1].value;

        return match (first, second) {
            // ươ -> uô
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::U;
                match self.try_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Applied => TransformEffect::Applied,
                    TransformEffect::Undone => TransformEffect::Undone,
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
                match self.try_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Applied => TransformEffect::Applied,
                    TransformEffect::Undone => TransformEffect::Undone,
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
                self.syllable.vowels[1].value = BaseVowel::O;
                return TransformEffect::Undone;
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
            return TransformEffect::Undone;
        }

        // Khác tone → thay tone hiện tại.
        syllable.tone = tone;
        TransformEffect::Applied
    }

    #[inline(always)]
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
            return TransformEffect::Undone;
        }

        let Ok(new) = old.replace_shape(shape) else {
            // Shape này không tạo được sequence hợp lệ.
            // Tức là người dùng cố ý tạo shape sai trên mapping ví dụ a + horn
            return TransformEffect::NotApplicable;
        };

        // Apply thử.
        self.syllable.vowels[index].value = new;

        let status = {
            let (len, bases) = self.syllable.vowel_bases();
            match_vowel_sequence(&bases[..len])
        };

        // Kiểm tra toàn bộ chuỗi nguyên âm sau khi apply.
        match status {
            rule::SequenceStatus::Valid | rule::SequenceStatus::InComplete => {
                TransformEffect::Applied
            }

            rule::SequenceStatus::Dead => {
                // Sequence không hợp lệ -> rollback.
                self.syllable.vowels[index].value = old;
                TransformEffect::NotApplicable
            }
        }
    }

    fn reformat_uo(&mut self) {
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
            _ => {
                // Không có trường hợp cần reformat
                return;
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

    #[inline]
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

            Err(_) => self.kill(DeadReason::InvalidVowelSequence),
        }
    }

    #[inline]
    fn push_coda_transform(&mut self, key: char) -> ParseStatus {
        if let Some(tone) = self.mapping.tone(key) {
            // Applied ko được hoặc undo thì coi key như một coda literal mới
            match self.try_tone(tone) {
                TransformEffect::Applied => return self.status,
                TransformEffect::Undone => return self.push_coda_literal(key),
                TransformEffect::NotApplicable => {
                    return self.push_coda_literal(key);
                }
            }
        }

        match self.try_d_stroke(key) {
            TransformEffect::Applied => {
                // Đã Chuyển đổi thành d-stroke
                return self.status;
            }
            TransformEffect::Undone => {
                // Bị undo lại thành kí tự thì push vô như bình thường
                return self.push_coda_literal(key);
            }
            TransformEffect::NotApplicable => {}
        }

        // ---------------------------------------------------------
        // 2. Shape
        //
        // Tìm vowel gần nhất có thể nhận shape.
        // ---------------------------------------------------------
        let vseq_len = self.syllable.vowels.len();
        for index in (0..vseq_len).rev() {
            let base = self.syllable.vowels[index].value;

            let Some(shape) = self.mapping.shape(key, KeyTarget::BaseVowel(base)) else {
                continue;
            };

            // Trường hợp đặc biệt có thể xảy ra với các âm uo hoặc uou
            //
            // Phát hiện có hiệu ứng Horn tức là chỉ có thể có trên o hoặc u
            if shape == Shape::Horn
                // Phải có ít nhất hai nguyên âm thì mới có hiệu ứng đặc biệt
                && vseq_len > 1
                // Nguyên âm đầu tiên phải là u mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[0].value.root() == RootVowel::U
                // Nguyên âm thứ hai phải là o mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[1].value.root() == RootVowel::O
            {
                match self.try_uo_horn(index) {
                    TransformEffect::Applied => {
                        return self.status;
                    }
                    TransformEffect::Undone => {
                        return self.push_coda_literal(key);
                    }
                    TransformEffect::NotApplicable => {}
                }
            }
            // Hiệu ứng uô hoặc uơ -> uô hoặc ưo thành uô
            else if shape == Shape::Circumflex
                && vseq_len > 1
                // Nguyên âm đầu tiên phải là u mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[0].value.root() == RootVowel::U
                // Nguyên âm thứ hai phải là o mới ra hiệu ứng đặc biệt
                && self.syllable.vowels[1].value.root() == RootVowel::O
            {
                // uô
                match self.try_uo_circumflex(index) {
                    TransformEffect::Applied => {
                        return self.status;
                    }
                    TransformEffect::Undone => {
                        return self.push_coda_literal(key);
                    }
                    TransformEffect::NotApplicable => {}
                }
            }

            match self.try_vowel_shape(index, shape) {
                TransformEffect::Applied => {
                    return self.status;
                }
                TransformEffect::Undone => {
                    return self.push_coda_literal(key);
                }
                TransformEffect::NotApplicable => {
                    // Shape này không tạo được sequence hợp lệ.
                    // Thử vowel đứng trước.
                }
            }
        }

        // Không phải transform hợp lệ thì coi như key thường
        return self.push_coda_literal(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::DefaultKeyMapping;

    fn parser() -> Parser<DefaultKeyMapping<'static>> {
        Parser::new(DefaultKeyMapping::telex())
    }

    #[test]
    fn onset_b() {
        let mut p = parser();

        p.push(BufferChar::Literal('b'));

        assert_eq!(p.syllable.onset, None);
        assert_eq!(p.syllable.onset_chars, vec!['b']);
        assert_eq!(p.phase, ParsePhase::Onset);
    }

    #[test]
    fn onset_ch() {
        let mut p = parser();

        p.push(BufferChar::Literal('c'));
        p.push(BufferChar::Literal('h'));

        assert_eq!(p.syllable.onset_chars, vec!['c', 'h']);
        assert_eq!(p.phase, ParsePhase::Onset);

        // Khi gặp vowel mới xác nhận onset.
        p.push(BufferChar::Literal('a'));

        assert_eq!(p.syllable.onset, Some(Onset::Ch));
        assert_eq!(p.syllable.vowels.len(), 1);
        assert_eq!(p.phase, ParsePhase::Vowel);
    }

    #[test]
    fn onset_invalid() {
        let mut p = parser();

        p.push(BufferChar::Literal('b'));
        p.push(BufferChar::Literal('c'));
        let status = p.push(BufferChar::Literal('a'));

        assert_eq!(status, ParseStatus::Dead(DeadReason::InvalidOnset));
    }

    #[test]
    fn onset_qu() {
        let mut p = parser();

        p.push(BufferChar::Literal('q'));
        p.push(BufferChar::Literal('u'));

        // "qu" vẫn đang được giữ trong onset.
        assert_eq!(p.syllable.onset_chars, vec!['q', 'u']);
        assert_eq!(p.phase, ParsePhase::Onset);

        p.push(BufferChar::Literal('a'));

        assert_eq!(p.syllable.onset, Some(Onset::QU));
        assert_eq!(p.syllable.vowels.len(), 1);
        assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
        assert_eq!(p.phase, ParsePhase::Vowel);
    }

    #[test]
    fn onset_gi() {
        let mut p = parser();

        p.push(BufferChar::Literal('g'));
        p.push(BufferChar::Literal('i'));

        // Chưa gặp vowel thứ hai nên vẫn chưa promote "gi".
        assert_eq!(p.syllable.onset, Some(Onset::G));
        assert_eq!(p.syllable.onset_chars, vec!['g']);
        assert_eq!(p.syllable.vowels.len(), 1);
        assert_eq!(p.syllable.vowels[0].value, BaseVowel::I);

        // i + a => promote i thành onset.
        p.push(BufferChar::Literal('a'));

        assert_eq!(p.syllable.onset, Some(Onset::Gi));
        assert_eq!(p.syllable.onset_chars, vec!['g', 'i']);
        assert_eq!(p.syllable.vowels.len(), 1);
        assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
    }
}
