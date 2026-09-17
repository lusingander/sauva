use crate::unicode::CodePoint;

const HANGUL_SYLLABLE_BASE: u32 = 0xac00;
const HANGUL_LEADING_JAMO_BASE: u32 = 0x1100;
const HANGUL_VOWEL_JAMO_BASE: u32 = 0x1161;
const HANGUL_TRAILING_JAMO_BASE: u32 = 0x11a7;
const HANGUL_LEADING_COUNT: u32 = 19;
const HANGUL_VOWEL_COUNT: u32 = 21;
const HANGUL_TRAILING_COUNT: u32 = 28;
const HANGUL_SYLLABLES_PER_LEADING: u32 = HANGUL_VOWEL_COUNT * HANGUL_TRAILING_COUNT;
const HANGUL_SYLLABLE_COUNT: u32 = HANGUL_LEADING_COUNT * HANGUL_SYLLABLES_PER_LEADING;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decomposition {
    decomposition_type: DecompositionType,
    mapping: DecompositionMapping,
}

impl Decomposition {
    pub const fn canonical(mapping: &'static [u32]) -> Self {
        Self {
            decomposition_type: DecompositionType::Canonical,
            mapping: DecompositionMapping::Generated(mapping),
        }
    }

    pub const fn compatibility(tag: &'static str, mapping: &'static [u32]) -> Self {
        Self {
            decomposition_type: DecompositionType::Compatibility(tag),
            mapping: DecompositionMapping::Generated(mapping),
        }
    }

    pub fn hangul(code_point: CodePoint) -> Option<Self> {
        let syllable_index = code_point.value().checked_sub(HANGUL_SYLLABLE_BASE)?;
        if syllable_index >= HANGUL_SYLLABLE_COUNT {
            return None;
        }

        let leading = HANGUL_LEADING_JAMO_BASE + syllable_index / HANGUL_SYLLABLES_PER_LEADING;
        let vowel = HANGUL_VOWEL_JAMO_BASE
            + syllable_index % HANGUL_SYLLABLES_PER_LEADING / HANGUL_TRAILING_COUNT;
        let trailing_index = syllable_index % HANGUL_TRAILING_COUNT;
        let mut values = [leading, vowel, 0];
        let length = if trailing_index == 0 {
            2
        } else {
            values[2] = HANGUL_TRAILING_JAMO_BASE + trailing_index;
            3
        };

        Some(Self {
            decomposition_type: DecompositionType::Canonical,
            mapping: DecompositionMapping::Hangul { values, length },
        })
    }

    pub const fn decomposition_type(self) -> DecompositionType {
        self.decomposition_type
    }

    pub fn mapping(&self) -> impl ExactSizeIterator<Item = CodePoint> + '_ {
        self.mapping_values().iter().map(|&value| {
            CodePoint::new(value).expect("decomposition mappings contain valid code points")
        })
    }

    fn mapping_values(&self) -> &[u32] {
        match &self.mapping {
            DecompositionMapping::Generated(values) => values,
            DecompositionMapping::Hangul { values, length } => &values[..usize::from(*length)],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecompositionMapping {
    Generated(&'static [u32]),
    Hangul { values: [u32; 3], length: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecompositionType {
    Canonical,
    Compatibility(&'static str),
}

impl DecompositionType {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Canonical => "Canonical",
            Self::Compatibility(_) => "Compatibility",
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::unicode::{CodePoint, Decomposition, DecompositionType};

    #[rstest]
    #[case(0xac00, &[0x1100, 0x1161])]
    #[case(0xac01, &[0x1100, 0x1161, 0x11a8])]
    #[case(0xd7a3, &[0x1112, 0x1175, 0x11c2])]
    fn decomposes_hangul_syllables_directly(#[case] value: u32, #[case] expected: &[u32]) {
        let decomposition = Decomposition::hangul(CodePoint::new(value).unwrap()).unwrap();

        assert_eq!(
            decomposition.decomposition_type(),
            DecompositionType::Canonical
        );
        assert_eq!(
            decomposition
                .mapping()
                .map(CodePoint::value)
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[rstest]
    #[case(0xabff)]
    #[case(0xd7a4)]
    fn does_not_decompose_values_outside_the_hangul_syllable_range(#[case] value: u32) {
        assert!(Decomposition::hangul(CodePoint::new(value).unwrap()).is_none());
    }

    #[rustfmt::skip]
    #[rstest]
    #[case(
        DecompositionType::Canonical,
        "Canonical"
    )]
    #[case(
        DecompositionType::Compatibility("font"),
        "Compatibility"
    )]
    fn exposes_decomposition_type_labels(
        #[case] decomposition_type: DecompositionType,
        #[case] label: &str,
    ) {
        assert_eq!(decomposition_type.label(), label);
    }
}
