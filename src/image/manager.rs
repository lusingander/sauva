use std::{
    io::{self, Write},
    process,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::image::{
    ImageKey, RgbaImage,
    kitty::{self, ImageId, VirtualPlacement},
};

static IMAGE_ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static IMAGE_ID_SEED: OnceLock<u32> = OnceLock::new();
const IMAGE_ID_SPACE: u64 = 0x00ff_ffff;

pub struct KittyImageManager<Content> {
    image_id: ImageId,
    active: Option<ImageKey<Content, VirtualPlacement>>,
}

impl<Content> KittyImageManager<Content> {
    pub fn new() -> Self {
        Self::with_id(process_image_id())
    }

    pub const fn with_id(image_id: ImageId) -> Self {
        Self {
            image_id,
            active: None,
        }
    }

    pub const fn image_id(&self) -> ImageId {
        self.image_id
    }

    /// Transmits a changed image and replaces the manager's existing Kitty image.
    ///
    /// `content` must identify the image pixels and dimensions.
    ///
    /// Returns `true` when a transmission was written and `false` when the
    /// requested image and placement were already active.
    pub fn show(
        &mut self,
        writer: &mut impl Write,
        content: Content,
        image: RgbaImage<'_>,
        placement: VirtualPlacement,
    ) -> io::Result<bool>
    where
        Content: Copy + PartialEq,
    {
        let key = ImageKey::new(content, placement);
        if self.active == Some(key) {
            return Ok(false);
        }

        kitty::transmit_rgba(writer, self.image_id, image, placement)?;
        self.active = Some(key);
        Ok(true)
    }

    /// Deletes the active image and all of its placements.
    ///
    /// Returns `true` when deletion was written and `false` when there was no
    /// active image to clean up.
    pub fn cleanup(&mut self, writer: &mut impl Write) -> io::Result<bool> {
        if self.active.is_none() {
            return Ok(false);
        }

        kitty::delete(writer, self.image_id)?;
        self.active = None;
        Ok(true)
    }
}

fn process_image_id() -> ImageId {
    let sequence = IMAGE_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let seed = *IMAGE_ID_SEED.get_or_init(|| {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos() as u64);
        ((splitmix64(timestamp ^ u64::from(process::id())) % IMAGE_ID_SPACE) + 1) as u32
    });
    let value = ((u64::from(seed) - 1 + sequence) % IMAGE_ID_SPACE + 1) as u32;
    ImageId::new(value).expect("the generated Kitty image ID must fit in 24 bits")
}

const fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use crate::image::{
        RgbaImage,
        kitty::{ImageId, VirtualPlacement},
        manager::KittyImageManager,
    };

    const IMAGE_ID: ImageId = match ImageId::new(7) {
        Some(image_id) => image_id,
        None => panic!("test image ID must be valid"),
    };
    const PLACEMENT: VirtualPlacement = match VirtualPlacement::new(2, 2) {
        Ok(placement) => placement,
        Err(_) => panic!("test placement must be valid"),
    };
    const SMALLER_PLACEMENT: VirtualPlacement = match VirtualPlacement::new(1, 2) {
        Ok(placement) => placement,
        Err(_) => panic!("test placement must be valid"),
    };
    const PIXELS: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    const CONTENT: u8 = 1;
    const CHANGED_CONTENT: u8 = 2;

    #[test]
    fn creates_valid_process_scoped_ids() {
        let first = KittyImageManager::<u8>::new().image_id().value();
        let second = KittyImageManager::<u8>::new().image_id().value();

        assert!((1..=0x00ff_ffff).contains(&first));
        assert!((1..=0x00ff_ffff).contains(&second));
        assert_ne!(first, second);
    }

    #[test]
    fn suppresses_an_unchanged_retransmission() {
        let mut manager = KittyImageManager::with_id(IMAGE_ID);
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        let mut first_output = Vec::new();
        let mut second_output = Vec::new();

        assert!(
            manager
                .show(&mut first_output, CONTENT, image, PLACEMENT)
                .unwrap()
        );
        assert!(
            !manager
                .show(&mut second_output, CONTENT, image, PLACEMENT)
                .unwrap()
        );

        assert!(!first_output.is_empty());
        assert!(second_output.is_empty());
    }

    #[test]
    fn retransmits_changed_content_with_the_same_image_id() {
        let mut manager = KittyImageManager::with_id(IMAGE_ID);
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        let mut output = Vec::new();

        assert!(
            manager
                .show(&mut output, CONTENT, image, PLACEMENT)
                .unwrap()
        );
        assert!(
            manager
                .show(&mut output, CHANGED_CONTENT, image, PLACEMENT)
                .unwrap()
        );

        assert_eq!(count_occurrences(&output, b"i=7"), 2);
    }

    #[test]
    fn retransmits_a_changed_placement_with_the_same_image_id() {
        let mut manager = KittyImageManager::with_id(IMAGE_ID);
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        let mut output = Vec::new();

        assert!(
            manager
                .show(&mut output, CONTENT, image, PLACEMENT)
                .unwrap()
        );
        assert!(
            manager
                .show(&mut output, CONTENT, image, SMALLER_PLACEMENT)
                .unwrap()
        );

        assert_eq!(count_occurrences(&output, b"i=7"), 2);
        assert_eq!(count_occurrences(&output, b"p=1"), 2);
    }

    #[test]
    fn cleans_up_an_active_image_only_once() {
        let mut manager = KittyImageManager::with_id(IMAGE_ID);
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        let mut transfer = Vec::new();
        let mut cleanup = Vec::new();
        let mut repeated_cleanup = Vec::new();
        manager
            .show(&mut transfer, CONTENT, image, PLACEMENT)
            .unwrap();

        assert!(manager.cleanup(&mut cleanup).unwrap());
        assert!(!manager.cleanup(&mut repeated_cleanup).unwrap());

        assert_eq!(cleanup, b"\x1b_Ga=d,d=I,i=7,q=2\x1b\\");
        assert!(repeated_cleanup.is_empty());
    }

    #[test]
    fn retries_a_transfer_after_a_write_failure() {
        let mut manager = KittyImageManager::with_id(IMAGE_ID);
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        let mut failure = AlwaysFails;
        let mut retry = Vec::new();

        assert!(
            manager
                .show(&mut failure, CONTENT, image, PLACEMENT)
                .is_err()
        );
        assert!(manager.show(&mut retry, CONTENT, image, PLACEMENT).unwrap());
        assert!(!retry.is_empty());
    }

    #[test]
    fn retries_cleanup_after_a_write_failure() {
        let mut manager = KittyImageManager::with_id(IMAGE_ID);
        let image = RgbaImage::new(2, 2, &PIXELS).unwrap();
        manager
            .show(&mut Vec::new(), CONTENT, image, PLACEMENT)
            .unwrap();
        let mut failure = AlwaysFails;
        let mut retry = Vec::new();

        assert!(manager.cleanup(&mut failure).is_err());
        assert!(manager.cleanup(&mut retry).unwrap());
        assert_eq!(retry, b"\x1b_Ga=d,d=I,i=7,q=2\x1b\\");
    }

    struct AlwaysFails;

    impl Write for AlwaysFails {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("injected write failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("injected flush failure"))
        }
    }

    fn count_occurrences(haystack: &[u8], needle: &[u8]) -> usize {
        haystack
            .windows(needle.len())
            .filter(|window| *window == needle)
            .count()
    }
}
