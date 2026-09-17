use crate::unicode::CodePoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Plane(u8);

impl Plane {
    pub const COUNT: usize = 17;
    pub const LAST_NUMBER: u8 = 16;

    pub fn new(number: u8) -> Option<Self> {
        Self::all().nth(usize::from(number))
    }

    pub const fn for_code_point(code_point: CodePoint) -> Self {
        Self(code_point.plane())
    }

    pub fn all() -> impl ExactSizeIterator<Item = Self> {
        (0..Self::COUNT).map(|number| Self(number as u8))
    }

    pub const fn number(self) -> u8 {
        self.0
    }

    pub const fn name(self) -> Option<&'static str> {
        match self.0 {
            0 => Some("Basic Multilingual Plane"),
            1 => Some("Supplementary Multilingual Plane"),
            2 => Some("Supplementary Ideographic Plane"),
            3 => Some("Tertiary Ideographic Plane"),
            14 => Some("Supplementary Special-purpose Plane"),
            15 => Some("Supplementary Private Use Area-A"),
            16 => Some("Supplementary Private Use Area-B"),
            _ => None,
        }
    }

    pub fn start(self) -> CodePoint {
        valid_code_point(u32::from(self.0) << 16)
    }

    pub fn end(self) -> CodePoint {
        valid_code_point((u32::from(self.0) << 16) | 0xffff)
    }

    pub const fn range(self, index: u8) -> PlaneRange {
        PlaneRange::new(self, index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaneRange {
    plane: Plane,
    index: u8,
}

impl PlaneRange {
    pub const COUNT_PER_PLANE: usize = 256;
    pub const SIZE: u32 = 0x100;

    pub const fn new(plane: Plane, index: u8) -> Self {
        Self { plane, index }
    }

    pub const fn for_code_point(code_point: CodePoint) -> Self {
        Self {
            plane: Plane::for_code_point(code_point),
            index: ((code_point.value() >> 8) & (Self::COUNT_PER_PLANE as u32 - 1)) as u8,
        }
    }

    pub const fn plane(self) -> Plane {
        self.plane
    }

    pub const fn index(self) -> u8 {
        self.index
    }

    pub fn start(self) -> CodePoint {
        valid_code_point(self.start_value())
    }

    pub fn end(self) -> CodePoint {
        valid_code_point(self.start_value() + Self::SIZE - 1)
    }

    pub fn code_point(self, offset: u8) -> CodePoint {
        valid_code_point(self.start_value() + u32::from(offset))
    }

    pub fn offset_of(self, code_point: CodePoint) -> Option<u8> {
        let offset = code_point.value().checked_sub(self.start_value())?;
        (offset < Self::SIZE).then_some(offset as u8)
    }

    pub fn contains(self, code_point: CodePoint) -> bool {
        self.offset_of(code_point).is_some()
    }

    const fn start_value(self) -> u32 {
        (self.plane.number() as u32) << 16 | (self.index as u32) << 8
    }
}

fn valid_code_point(value: u32) -> CodePoint {
    CodePoint::new(value).expect("plane-derived values are valid Unicode code points")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn enumerates_exactly_the_seventeen_unicode_planes() {
        let planes = Plane::all().collect::<Vec<_>>();

        assert_eq!(planes.len(), Plane::COUNT);
        assert_eq!(planes.first().unwrap().number(), 0);
        assert_eq!(planes.last().unwrap().number(), 16);
        assert_eq!(Plane::new(16), Some(*planes.last().unwrap()));
        assert_eq!(Plane::new(17), None);
    }

    #[test]
    fn exposes_plane_names_without_naming_reserved_planes() {
        let named = [
            (0, "Basic Multilingual Plane"),
            (1, "Supplementary Multilingual Plane"),
            (2, "Supplementary Ideographic Plane"),
            (3, "Tertiary Ideographic Plane"),
            (14, "Supplementary Special-purpose Plane"),
            (15, "Supplementary Private Use Area-A"),
            (16, "Supplementary Private Use Area-B"),
        ];

        for (number, expected_name) in named {
            assert_eq!(Plane::new(number).unwrap().name(), Some(expected_name));
        }
        for number in 4..=13 {
            assert_eq!(Plane::new(number).unwrap().name(), None);
        }
    }

    #[rstest]
    #[case(0, 0x0000, 0xffff)]
    #[case(1, 0x1_0000, 0x1_ffff)]
    #[case(16, 0x10_0000, CodePoint::MAX_VALUE)]
    fn calculates_plane_boundaries(
        #[case] number: u8,
        #[case] expected_start: u32,
        #[case] expected_end: u32,
    ) {
        let plane = Plane::new(number).unwrap();

        assert_eq!(plane.start().value(), expected_start);
        assert_eq!(plane.end().value(), expected_end);
        assert_eq!(Plane::for_code_point(plane.start()), plane);
        assert_eq!(Plane::for_code_point(plane.end()), plane);
    }

    #[test]
    fn divides_every_plane_into_contiguous_ranges() {
        for plane in Plane::all() {
            let mut previous_end = None;

            for index in 0..=u8::MAX {
                let range = plane.range(index);
                assert_eq!(range.plane(), plane);
                assert_eq!(range.index(), index);
                assert_eq!(range.end().value() - range.start().value() + 1, 256);
                if let Some(end) = previous_end {
                    assert_eq!(range.start().value(), end + 1);
                }
                previous_end = Some(range.end().value());
            }

            assert_eq!(plane.range(0).start(), plane.start());
            assert_eq!(plane.range(u8::MAX).end(), plane.end());
        }
    }

    #[rstest]
    #[case(0x0000, 0, 0, 0x0000, 0x00ff)]
    #[case(0x00ff, 0, 0, 0x0000, 0x00ff)]
    #[case(0x0100, 0, 1, 0x0100, 0x01ff)]
    #[case(0xffff, 0, 255, 0xff00, 0xffff)]
    #[case(0x1_0000, 1, 0, 0x1_0000, 0x1_00ff)]
    #[case(0x10_ffff, 16, 255, 0x10_ff00, 0x10_ffff)]
    fn derives_ranges_at_boundaries(
        #[case] value: u32,
        #[case] plane: u8,
        #[case] index: u8,
        #[case] start: u32,
        #[case] end: u32,
    ) {
        let range = PlaneRange::for_code_point(CodePoint::new(value).unwrap());

        assert_eq!(range.plane().number(), plane);
        assert_eq!(range.index(), index);
        assert_eq!(range.start().value(), start);
        assert_eq!(range.end().value(), end);
    }

    #[test]
    fn converts_offsets_to_code_points_and_back() {
        let range = Plane::new(1).unwrap().range(0x23);

        assert_eq!(PlaneRange::COUNT_PER_PLANE, usize::from(u8::MAX) + 1);
        assert_eq!(range.code_point(0).value(), 0x1_2300);
        assert_eq!(range.code_point(0xab).value(), 0x1_23ab);
        assert_eq!(range.code_point(u8::MAX).value(), 0x1_23ff);
        assert_eq!(range.offset_of(range.code_point(0xab)), Some(0xab));
        assert!(range.contains(range.code_point(0xab)));
        assert_eq!(
            range.offset_of(Plane::new(1).unwrap().range(0x24).start()),
            None
        );
    }
}
