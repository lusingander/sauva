use std::{fmt, str::FromStr};

use garde::Validate;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use umbra::optional;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl RgbaColor {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, u8::MAX)
    }

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub const fn channels(self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

impl FromStr for RgbaColor {
    type Err = ParseRgbaColorError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hexadecimal = value
            .strip_prefix('#')
            .ok_or(ParseRgbaColorError::InvalidFormat)?;
        if !matches!(hexadecimal.len(), 6 | 8)
            || !hexadecimal.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ParseRgbaColorError::InvalidFormat);
        }

        let mut channels = [0; 4];
        channels[3] = u8::MAX;
        for (index, channel) in channels.iter_mut().enumerate().take(hexadecimal.len() / 2) {
            let offset = index * 2;
            *channel = u8::from_str_radix(&hexadecimal[offset..offset + 2], 16)
                .map_err(|_| ParseRgbaColorError::InvalidFormat)?;
        }

        Ok(Self::rgba(
            channels[0],
            channels[1],
            channels[2],
            channels[3],
        ))
    }
}

impl<'de> Deserialize<'de> for RgbaColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Serialize for RgbaColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let [red, green, blue, alpha] = self.channels();
        let value = if alpha == u8::MAX {
            format!("#{red:02x}{green:02x}{blue:02x}")
        } else {
            format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
        };
        serializer.serialize_str(&value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseRgbaColorError {
    InvalidFormat,
}

impl fmt::Display for ParseRgbaColorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected #RRGGBB or #RRGGBBAA")
    }
}

impl std::error::Error for ParseRgbaColorError {}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)],
    visibility = pub
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Validate)]
pub struct GlyphPreviewSettings {
    #[garde(inner(custom(validate_font_family)))]
    pub font_families: Vec<String>,
    #[garde(inner(custom(validate_font_family)))]
    pub emoji_font_families: Vec<String>,
    #[garde(skip)]
    pub fg: RgbaColor,
    #[garde(skip)]
    pub bg: RgbaColor,
}

impl Default for GlyphPreviewSettings {
    fn default() -> Self {
        Self {
            font_families: Vec::new(),
            emoji_font_families: Vec::new(),
            fg: RgbaColor::rgb(245, 247, 250),
            bg: RgbaColor::rgba(0, 0, 0, 0),
        }
    }
}

fn validate_font_family(value: &str, _: &()) -> garde::Result {
    if value.trim().is_empty() {
        return Err(garde::Error::new("font family must not be empty"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use garde::Validate;
    use rstest::rstest;

    use crate::glyph::settings::{GlyphPreviewSettings, OptionalGlyphPreviewSettings, RgbaColor};

    #[rstest]
    #[case("#f5f7fa", RgbaColor::rgb(245, 247, 250))]
    #[case("#10203040", RgbaColor::rgba(16, 32, 48, 64))]
    #[case("#ABCDEF", RgbaColor::rgb(171, 205, 239))]
    fn parses_rgb_and_rgba_colors(#[case] value: &str, #[case] expected: RgbaColor) {
        assert_eq!(value.parse(), Ok(expected));
    }

    #[rstest]
    #[case("f5f7fa")]
    #[case("#fff")]
    #[case("#f5f7faff00")]
    #[case("#gg0000")]
    #[case("#aéabc")]
    #[case("transparent")]
    #[case("red")]
    fn rejects_other_color_formats(#[case] value: &str) {
        assert!(value.parse::<RgbaColor>().is_err());
    }

    #[test]
    fn defaults_to_the_existing_fg_and_a_transparent_bg() {
        let settings = GlyphPreviewSettings::default();

        assert!(settings.font_families.is_empty());
        assert!(settings.emoji_font_families.is_empty());
        assert_eq!(settings.fg, RgbaColor::rgb(245, 247, 250));
        assert_eq!(settings.bg, RgbaColor::rgba(0, 0, 0, 0));
    }

    #[test]
    fn partial_config_preserves_the_unspecified_default() {
        let config: OptionalGlyphPreviewSettings = toml::from_str(
            r##"
                fg = "#10203040"
            "##,
        )
        .unwrap();

        assert_eq!(
            GlyphPreviewSettings::from(config),
            GlyphPreviewSettings {
                fg: RgbaColor::rgba(0x10, 0x20, 0x30, 0x40),
                ..GlyphPreviewSettings::default()
            }
        );
    }

    #[test]
    fn font_family_lists_replace_their_defaults() {
        let config: OptionalGlyphPreviewSettings = toml::from_str(
            r#"
                font_families = ["Iosevka", "Noto Sans"]
                emoji_font_families = []
            "#,
        )
        .unwrap();

        let settings = GlyphPreviewSettings::from(config);

        assert_eq!(settings.font_families, ["Iosevka", "Noto Sans"]);
        assert!(settings.emoji_font_families.is_empty());
    }

    #[test]
    fn rejects_empty_or_whitespace_only_font_families() {
        for family in ["", " ", "\t"] {
            let settings = GlyphPreviewSettings {
                font_families: vec![family.to_owned()],
                ..GlyphPreviewSettings::default()
            };

            assert!(settings.validate().is_err(), "{family:?}");
        }
    }

    #[test]
    fn unknown_setting_is_rejected() {
        let error = toml::from_str::<OptionalGlyphPreviewSettings>(
            r##"
                backdrop = "#000000"
            "##,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `backdrop`"));
    }
}
