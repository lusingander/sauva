use crate::unicode::CodePoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaseMapping {
    source: u32,
    values: &'static [u32],
}

impl CaseMapping {
    pub const fn new(source: CodePoint, values: &'static [u32]) -> Self {
        Self {
            source: source.value(),
            values,
        }
    }

    pub fn is_identity(self) -> bool {
        self.values.is_empty() || self.values == [self.source]
    }

    pub fn mapping(&self) -> impl ExactSizeIterator<Item = CodePoint> + '_ {
        let values = if self.values.is_empty() {
            std::slice::from_ref(&self.source)
        } else {
            self.values
        };
        values
            .iter()
            .map(|&value| CodePoint::new(value).expect("case mappings contain valid code points"))
    }
}
