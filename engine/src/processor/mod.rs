use crate::phonology::rule::match_vowel_sequence;
use crate::phonology::{encode_vowel, BaseVowel, Case, Coda, Onset, RootVowel, Shape, Tone};
use crate::Renderer;

mod api;

pub use api::Processor;

impl<T: crate::Renderer> Processor for T {
    #[inline]
    fn render(&self, raw: &[char], cursor: usize) -> String {
        Renderer::render(self, raw, cursor)
    }
}

#[inline]
fn root(ch: char) -> Option<(RootVowel, Case)> {
    Some((
        match ch.to_ascii_lowercase() {
            'a' => RootVowel::A,
            'e' => RootVowel::E,
            'i' => RootVowel::I,
            'o' => RootVowel::O,
            'u' => RootVowel::U,
            'y' => RootVowel::Y,
            _ => return None,
        },
        if ch.is_ascii_uppercase() {
            Case::Upper
        } else {
            Case::Lower
        },
    ))
}

#[inline]
fn shaped(root: RootVowel, shape: Shape) -> BaseVowel {
    match (root, shape) {
        (RootVowel::A, Shape::Breve) => BaseVowel::ABreve,
        (RootVowel::A, Shape::Circumflex) => BaseVowel::ACircumflex,
        (RootVowel::E, Shape::Circumflex) => BaseVowel::ECircumflex,
        (RootVowel::O, Shape::Circumflex) => BaseVowel::OCircumflex,
        (RootVowel::O, Shape::Horn) => BaseVowel::OHorn,
        (RootVowel::U, Shape::Horn) => BaseVowel::UHorn,
        (RootVowel::A, Shape::None) => BaseVowel::A,
        (RootVowel::E, Shape::None) => BaseVowel::E,
        (RootVowel::I, Shape::None) => BaseVowel::I,
        (RootVowel::O, Shape::None) => BaseVowel::O,
        (RootVowel::U, Shape::None) => BaseVowel::U,
        (RootVowel::Y, Shape::None) => BaseVowel::Y,
        _ => shaped(root, Shape::None),
    }
}

/// Converts one canonical input buffer into rendered Vietnamese text.
pub fn render_raw(raw: &[char]) -> String {
    if raw.is_empty() {
        return String::new();
    }
    if raw.len() == 1 && raw[0].eq_ignore_ascii_case(&'w') {
        return "ư".to_string();
    }
    if raw.iter().any(|ch| !ch.is_ascii()) {
        return raw.iter().collect();
    }
    let vni = raw.iter().any(|ch| matches!(ch, '0'..='9'));

    let mut onset_end = 0;
    while onset_end < raw.len() && root(raw[onset_end]).is_none() {
        onset_end += 1;
    }
    if onset_end == raw.len() || onset_end > Onset::MAX_ONSET_BYTES {
        if raw.last().is_some_and(|ch| ch.eq_ignore_ascii_case(&'w')) {
            let prefix: String = raw[..raw.len() - 1].iter().collect();
            if let Some(onset) = Onset::from_str(&prefix) {
                return format!("{}ư", onset.to_string());
            }
        }
        return raw.iter().collect();
    }

    if onset_end + 1 < raw.len()
        && raw[onset_end].eq_ignore_ascii_case(&'q')
        && raw[onset_end + 1].eq_ignore_ascii_case(&'u')
    {
        onset_end += 1;
    } else if onset_end + 1 < raw.len()
        && raw[onset_end].eq_ignore_ascii_case(&'g')
        && raw[onset_end + 1].eq_ignore_ascii_case(&'i')
        && onset_end + 2 < raw.len()
        && root(raw[onset_end + 2]).is_some()
    {
        onset_end += 1;
    }

    let onset = Onset::from_str(&raw[..onset_end].iter().collect::<String>());
    let Some(onset) = onset else {
        return raw.iter().collect();
    };

    let mut vowels = Vec::new();
    let mut cases = Vec::new();
    let mut tone = Tone::Flat;
    let mut idx = onset_end;
    while idx < raw.len() {
        if let Some((r, case)) = root(raw[idx]) {
            let mut shape = Shape::None;
            if idx + 1 < raw.len() && raw[idx + 1].eq_ignore_ascii_case(&raw[idx]) {
                shape = Shape::Circumflex;
                idx += 1;
            }
            vowels.push(shaped(r, shape));
            cases.push(case);
            idx += 1;
            continue;
        }
        if (!vni && raw[idx].eq_ignore_ascii_case(&'w'))
            || (vni && raw[idx] == '6')
            || (vni && raw[idx] == '7' && vowels.last() == Some(&BaseVowel::A))
            || (vni && raw[idx] == '7' && vowels.last() == Some(&BaseVowel::O))
            || (vni && raw[idx] == '8' && vowels.last() == Some(&BaseVowel::U))
        {
            if !vowels.is_empty() {
                let last_index = vowels.len() - 1;
                if vowels[last_index] == BaseVowel::ACircumflex
                    || vowels[last_index] == BaseVowel::ECircumflex
                    || vowels[last_index] == BaseVowel::OCircumflex
                {
                    return raw.iter().collect();
                }
                if vni && raw[idx] == '6' {
                    vowels[last_index] = match vowels[last_index] {
                        BaseVowel::A => BaseVowel::ACircumflex,
                        BaseVowel::E => BaseVowel::ECircumflex,
                        BaseVowel::O => BaseVowel::OCircumflex,
                        _ => return raw.iter().collect(),
                    };
                    idx += 1;
                    continue;
                }
                vowels[last_index] = match vowels[last_index] {
                    BaseVowel::A => BaseVowel::ABreve,
                    BaseVowel::O => BaseVowel::OHorn,
                    BaseVowel::U => BaseVowel::UHorn,
                    value => value,
                };
                if last_index > 0
                    && vowels[last_index - 1] == BaseVowel::U
                    && vowels[last_index] == BaseVowel::OHorn
                {
                    vowels[last_index - 1] = BaseVowel::UHorn;
                }
                idx += 1;
                continue;
            } else if !vni {
                vowels.push(BaseVowel::UHorn);
                cases.push(Case::Lower);
                idx += 1;
                continue;
            }
        }
        tone = match if vni {
            match raw[idx] {
                '1' => 's',
                '2' => 'f',
                '3' => 'r',
                '4' => 'x',
                '5' => 'j',
                '0' => 'z',
                _ => raw[idx],
            }
        } else {
            raw[idx].to_ascii_lowercase()
        } {
            's' => Tone::Acute,
            'f' => Tone::Grave,
            'r' => Tone::Hook,
            'x' => Tone::Tilde,
            'j' => Tone::Dot,
            'z' => Tone::Flat,
            _ => break,
        };
        idx += 1;
    }

    if idx < raw.len() && !vni && raw[idx].eq_ignore_ascii_case(&'w') {
        return raw.iter().collect();
    }

    let mut coda_end = raw.len();
    if !vni
        && coda_end > idx
        && matches!(
            raw[coda_end - 1].to_ascii_lowercase(),
            's' | 'f' | 'r' | 'x' | 'j' | 'z'
        )
    {
        tone = match raw[coda_end - 1].to_ascii_lowercase() {
            's' => Tone::Acute,
            'f' => Tone::Grave,
            'r' => Tone::Hook,
            'x' => Tone::Tilde,
            'j' => Tone::Dot,
            _ => Tone::Flat,
        };
        coda_end -= 1;
    }
    let coda_text: String = raw[idx..coda_end].iter().collect();
    let Some(coda) = Coda::from_str(&coda_text) else {
        return raw.iter().collect();
    };
    let Some(rule) = match_vowel_sequence(&vowels) else {
        return raw.iter().collect();
    };
    if !rule.valid_codas.contains(coda) {
        return raw.iter().collect();
    }

    let mut output = String::new();
    output.push_str(onset.to_string());
    for (index, &vowel) in vowels.iter().enumerate() {
        let applied_tone = if index == rule.tone_target_idx {
            tone
        } else {
            Tone::Flat
        };
        output.push(encode_vowel(vowel, applied_tone, cases[index]));
    }
    output.push_str(coda.as_str());
    output
}
