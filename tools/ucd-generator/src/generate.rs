use std::{error::Error, fmt, fs, path::Path};

use crate::property_value_aliases::{PropertyValueAlias, parse as parse_property_value_aliases};
use crate::ucd::{UcdEntry, parse};
use crate::unicode_data::{DecompositionEntry, DecompositionKind, parse_decomposition_entries};

const UNICODE_VERSION: &str = "17.0.0";
const MAX_CODE_POINT: u32 = 0x10_ffff;

pub struct GenerationSummary {
    pub unicode_version: &'static str,
    pub name_count: usize,
    pub general_category_range_count: usize,
    pub block_range_count: usize,
    pub script_range_count: usize,
    pub age_range_count: usize,
    pub default_ignorable_range_count: usize,
    pub name_alias_count: usize,
    pub east_asian_width_range_count: usize,
    pub canonical_combining_class_range_count: usize,
    pub decomposition_mapping_count: usize,
    pub bidi_class_range_count: usize,
}

pub fn generate(
    input_directory: &Path,
    output: &Path,
) -> Result<GenerationSummary, GenerationError> {
    let readme = read(input_directory.join("ReadMe.txt"))?;
    validate_unicode_version(&readme)?;

    let names = build_names(&parse_file(input_directory.join("DerivedName.txt"))?)?;
    let categories = build_general_categories(&parse_file(
        input_directory.join("DerivedGeneralCategory.txt"),
    )?)?;
    let blocks = build_string_property(
        &parse_file(input_directory.join("Blocks.txt"))?,
        "Block",
        "No_Block",
    )?;
    let scripts = build_string_property(
        &parse_file(input_directory.join("Scripts.txt"))?,
        "Script",
        "Unknown",
    )?;
    let ages = build_string_property(
        &parse_file(input_directory.join("DerivedAge.txt"))?,
        "Age",
        "Unassigned",
    )?;
    let default_ignorables = build_binary_property(
        &parse_file(input_directory.join("DerivedCoreProperties.txt"))?,
        "Default_Ignorable_Code_Point",
    )?;
    let property_value_aliases =
        parse_property_value_alias_file(input_directory.join("PropertyValueAliases.txt"))?;
    let name_aliases = build_name_aliases(&parse_file(input_directory.join("NameAliases.txt"))?)?;
    let east_asian_widths = build_enum_property(
        &parse_file(input_directory.join("EastAsianWidth.txt"))?,
        &property_value_aliases,
        "ea",
        "East Asian Width",
        east_asian_width_variant,
    )?;
    let (canonical_combining_classes, canonical_combining_class_values) =
        build_canonical_combining_classes(
            &parse_file(input_directory.join("DerivedCombiningClass.txt"))?,
            &property_value_aliases,
        )?;
    let decompositions = parse_unicode_data_file(input_directory.join("UnicodeData.txt"))?;
    validate_decomposition_entries(&decompositions)?;
    let bidi_classes = build_enum_property(
        &parse_file(input_directory.join("DerivedBidiClass.txt"))?,
        &property_value_aliases,
        "bc",
        "Bidi Class",
        bidi_class_variant,
    )?;
    let generated = render(&RenderData {
        names: &names,
        categories: &categories,
        blocks: &blocks,
        scripts: &scripts,
        ages: &ages,
        default_ignorables: &default_ignorables,
        name_aliases: &name_aliases,
        east_asian_widths: &east_asian_widths,
        canonical_combining_classes: &canonical_combining_classes,
        canonical_combining_class_values: &canonical_combining_class_values,
        decompositions: &decompositions,
        bidi_classes: &bidi_classes,
    });

    fs::write(output, generated).map_err(|source| GenerationError::Io {
        path: output.to_owned(),
        source,
    })?;

    Ok(GenerationSummary {
        unicode_version: UNICODE_VERSION,
        name_count: names.len(),
        general_category_range_count: categories.len(),
        block_range_count: blocks.len(),
        script_range_count: scripts.len(),
        age_range_count: ages.len(),
        default_ignorable_range_count: default_ignorables.len(),
        name_alias_count: name_aliases.iter().map(|group| group.aliases.len()).sum(),
        east_asian_width_range_count: east_asian_widths.len(),
        canonical_combining_class_range_count: canonical_combining_classes.len(),
        decomposition_mapping_count: decompositions.len(),
        bidi_class_range_count: bidi_classes.len(),
    })
}

fn parse_file(path: impl AsRef<Path>) -> Result<Vec<UcdEntry>, GenerationError> {
    let path = path.as_ref();
    let source = read(path)?;
    parse(&source).map_err(|source| GenerationError::Parse {
        path: path.to_owned(),
        source,
    })
}

fn parse_unicode_data_file(
    path: impl AsRef<Path>,
) -> Result<Vec<DecompositionEntry>, GenerationError> {
    let path = path.as_ref();
    let source = read(path)?;
    parse_decomposition_entries(&source).map_err(|source| GenerationError::UnicodeDataParse {
        path: path.to_owned(),
        source,
    })
}

fn parse_property_value_alias_file(
    path: impl AsRef<Path>,
) -> Result<Vec<PropertyValueAlias>, GenerationError> {
    let path = path.as_ref();
    let source = read(path)?;
    parse_property_value_aliases(&source).map_err(|source| {
        GenerationError::PropertyValueAliasParse {
            path: path.to_owned(),
            source,
        }
    })
}

fn read(path: impl AsRef<Path>) -> Result<String, GenerationError> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|source| GenerationError::Io {
        path: path.to_owned(),
        source,
    })
}

fn validate_unicode_version(readme: &str) -> Result<(), GenerationError> {
    let expected = format!("for Version {UNICODE_VERSION} of the Unicode Standard.");
    if !readme.contains("This directory contains final data files") || !readme.contains(&expected) {
        return Err(GenerationError::Data(format!(
            "ReadMe.txt does not identify final Unicode {UNICODE_VERSION} data"
        )));
    }

    Ok(())
}

fn build_names(entries: &[UcdEntry]) -> Result<Vec<(u32, String)>, GenerationError> {
    let mut names = Vec::new();

    for entry in entries.iter().filter(|entry| !entry.is_missing()) {
        let [name] = entry.fields() else {
            return Err(GenerationError::Data(format!(
                "DerivedName entry U+{:04X} must have exactly one field",
                entry.range().start()
            )));
        };

        let range = entry.range();
        let pattern_count = name.matches('*').count();
        if pattern_count > 1 || (range.start() != range.end() && pattern_count != 1) {
            return Err(GenerationError::Data(format!(
                "DerivedName entry U+{:04X}..U+{:04X} must contain at most one `*`, and a range must contain exactly one",
                range.start(),
                range.end()
            )));
        }

        names.extend((range.start()..=range.end()).map(|code_point| {
            let name = if pattern_count == 1 {
                name.replace('*', &format!("{code_point:04X}"))
            } else {
                name.clone()
            };
            (code_point, name)
        }));
    }

    names.sort_unstable_by_key(|(code_point, _)| *code_point);
    for pair in names.windows(2) {
        if pair[0].0 == pair[1].0 {
            return Err(GenerationError::Data(format!(
                "duplicate DerivedName entry for U+{:04X}",
                pair[0].0
            )));
        }
    }

    Ok(names)
}

fn build_general_categories(
    entries: &[UcdEntry],
) -> Result<Vec<GeneralCategoryRange>, GenerationError> {
    let mut ranges = entries
        .iter()
        .filter(|entry| !entry.is_missing())
        .map(|entry| {
            let [category] = entry.fields() else {
                return Err(GenerationError::Data(format!(
                    "DerivedGeneralCategory entry U+{:04X} must have exactly one field",
                    entry.range().start()
                )));
            };

            Ok(GeneralCategoryRange {
                start: entry.range().start(),
                end: entry.range().end(),
                variant: general_category_variant(category)?,
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;

    ranges.sort_unstable_by_key(|range| range.start);
    validate_complete_coverage(&ranges)?;

    let mut merged: Vec<GeneralCategoryRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(previous) = merged.last_mut()
            && previous.end.checked_add(1) == Some(range.start)
            && previous.variant == range.variant
        {
            previous.end = range.end;
        } else {
            merged.push(range);
        }
    }

    Ok(merged)
}

fn build_name_aliases(entries: &[UcdEntry]) -> Result<Vec<NameAliasGroup>, GenerationError> {
    let mut aliases = entries
        .iter()
        .filter(|entry| !entry.is_missing())
        .map(|entry| {
            let range = entry.range();
            if range.start() != range.end() {
                return Err(GenerationError::Data(format!(
                    "NameAliases entry U+{:04X}..U+{:04X} must contain one code point",
                    range.start(),
                    range.end()
                )));
            }
            let [name, kind] = entry.fields() else {
                return Err(GenerationError::Data(format!(
                    "NameAliases entry U+{:04X} must have exactly two fields",
                    range.start()
                )));
            };
            Ok((
                range.start(),
                NameAliasValue {
                    name: name.clone(),
                    variant: name_alias_type_variant(kind)?,
                },
            ))
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;
    aliases.sort_by_key(|(code_point, _)| *code_point);

    let mut groups: Vec<NameAliasGroup> = Vec::new();
    for (code_point, alias) in aliases {
        if let Some(group) = groups.last_mut()
            && group.code_point == code_point
        {
            if group.aliases.iter().any(|existing| existing == &alias) {
                return Err(GenerationError::Data(format!(
                    "duplicate NameAliases entry for U+{code_point:04X}"
                )));
            }
            group.aliases.push(alias);
        } else {
            groups.push(NameAliasGroup {
                code_point,
                aliases: vec![alias],
            });
        }
    }
    Ok(groups)
}

fn build_enum_property(
    entries: &[UcdEntry],
    aliases: &[PropertyValueAlias],
    property: &str,
    property_name: &str,
    variant: fn(&PropertyValueAlias) -> Result<&'static str, GenerationError>,
) -> Result<Vec<EnumRange>, GenerationError> {
    let ranges = build_complete_string_property(entries, property_name)?;
    let mut typed: Vec<EnumRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        let value = resolve_property_value(aliases, property, &range.value, property_name)?;
        let variant = variant(value)?;
        if let Some(previous) = typed.last_mut()
            && previous.end + 1 == range.start
            && previous.variant == variant
        {
            previous.end = range.end;
        } else {
            typed.push(EnumRange {
                start: range.start,
                end: range.end,
                variant,
            });
        }
    }
    Ok(typed)
}

fn build_canonical_combining_classes(
    entries: &[UcdEntry],
    aliases: &[PropertyValueAlias],
) -> Result<(Vec<NumericRange>, Vec<CanonicalCombiningClassValue>), GenerationError> {
    let mut values = aliases
        .iter()
        .filter(|alias| alias.property() == "ccc")
        .map(|alias| {
            let value = alias.canonical().parse::<u8>().map_err(|_| {
                GenerationError::Data(format!(
                    "Canonical Combining Class value `{}` is not an unsigned byte",
                    alias.canonical()
                ))
            })?;
            Ok(CanonicalCombiningClassValue {
                value,
                name: display_property_name(alias.long_name()),
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;
    values.sort_unstable_by_key(|value| value.value);
    for pair in values.windows(2) {
        if pair[0].value == pair[1].value {
            return Err(GenerationError::Data(format!(
                "duplicate Canonical Combining Class value {}",
                pair[0].value
            )));
        }
    }

    let source_ranges = build_complete_string_property(entries, "Canonical Combining Class")?;
    let mut ranges: Vec<NumericRange> = Vec::with_capacity(source_ranges.len());
    for range in source_ranges {
        let alias =
            resolve_property_value(aliases, "ccc", &range.value, "Canonical Combining Class")?;
        let value = alias.canonical().parse::<u8>().map_err(|_| {
            GenerationError::Data(format!(
                "Canonical Combining Class value `{}` is not an unsigned byte",
                alias.canonical()
            ))
        })?;
        if let Some(previous) = ranges.last_mut()
            && previous.end + 1 == range.start
            && previous.value == value
        {
            previous.end = range.end;
        } else {
            ranges.push(NumericRange {
                start: range.start,
                end: range.end,
                value,
            });
        }
    }
    Ok((ranges, values))
}

fn resolve_property_value<'a>(
    aliases: &'a [PropertyValueAlias],
    property: &str,
    value: &str,
    property_name: &str,
) -> Result<&'a PropertyValueAlias, GenerationError> {
    aliases
        .iter()
        .find(|alias| alias.property() == property && alias.matches(value))
        .ok_or_else(|| {
            GenerationError::Data(format!(
                "{property_name} value `{value}` has no property value alias"
            ))
        })
}

fn display_property_name(value: &str) -> String {
    value.replace('_', " ")
}

fn name_alias_type_variant(value: &str) -> Result<&'static str, GenerationError> {
    match value {
        "correction" => Ok("Correction"),
        "control" => Ok("Control"),
        "alternate" => Ok("Alternate"),
        "figment" => Ok("Figment"),
        "abbreviation" => Ok("Abbreviation"),
        _ => Err(GenerationError::Data(format!(
            "unknown Name Alias type `{value}`"
        ))),
    }
}

fn east_asian_width_variant(value: &PropertyValueAlias) -> Result<&'static str, GenerationError> {
    match (value.abbreviation(), value.long_name()) {
        ("A", "Ambiguous") => Ok("Ambiguous"),
        ("F", "Fullwidth") => Ok("Fullwidth"),
        ("H", "Halfwidth") => Ok("Halfwidth"),
        ("N", "Neutral") => Ok("Neutral"),
        ("Na", "Narrow") => Ok("Narrow"),
        ("W", "Wide") => Ok("Wide"),
        (abbreviation, name) => Err(GenerationError::Data(format!(
            "unknown East Asian Width value `{abbreviation}` / `{name}`"
        ))),
    }
}

fn bidi_class_variant(value: &PropertyValueAlias) -> Result<&'static str, GenerationError> {
    let variant = match (value.abbreviation(), value.long_name()) {
        ("AL", "Arabic_Letter") => "ArabicLetter",
        ("AN", "Arabic_Number") => "ArabicNumber",
        ("B", "Paragraph_Separator") => "ParagraphSeparator",
        ("BN", "Boundary_Neutral") => "BoundaryNeutral",
        ("CS", "Common_Separator") => "CommonSeparator",
        ("EN", "European_Number") => "EuropeanNumber",
        ("ES", "European_Separator") => "EuropeanSeparator",
        ("ET", "European_Terminator") => "EuropeanTerminator",
        ("FSI", "First_Strong_Isolate") => "FirstStrongIsolate",
        ("L", "Left_To_Right") => "LeftToRight",
        ("LRE", "Left_To_Right_Embedding") => "LeftToRightEmbedding",
        ("LRI", "Left_To_Right_Isolate") => "LeftToRightIsolate",
        ("LRO", "Left_To_Right_Override") => "LeftToRightOverride",
        ("NSM", "Nonspacing_Mark") => "NonspacingMark",
        ("ON", "Other_Neutral") => "OtherNeutral",
        ("PDF", "Pop_Directional_Format") => "PopDirectionalFormat",
        ("PDI", "Pop_Directional_Isolate") => "PopDirectionalIsolate",
        ("R", "Right_To_Left") => "RightToLeft",
        ("RLE", "Right_To_Left_Embedding") => "RightToLeftEmbedding",
        ("RLI", "Right_To_Left_Isolate") => "RightToLeftIsolate",
        ("RLO", "Right_To_Left_Override") => "RightToLeftOverride",
        ("S", "Segment_Separator") => "SegmentSeparator",
        ("WS", "White_Space") => "WhiteSpace",
        (abbreviation, name) => {
            return Err(GenerationError::Data(format!(
                "unknown Bidi Class value `{abbreviation}` / `{name}`"
            )));
        }
    };
    Ok(variant)
}

fn build_string_property(
    entries: &[UcdEntry],
    property_name: &str,
    expected_missing: &str,
) -> Result<Vec<StringRange>, GenerationError> {
    validate_missing_entry(entries, property_name, expected_missing)?;

    let mut ranges = entries
        .iter()
        .filter(|entry| !entry.is_missing())
        .map(|entry| {
            let [value] = entry.fields() else {
                return Err(GenerationError::Data(format!(
                    "{property_name} entry U+{:04X} must have exactly one field",
                    entry.range().start()
                )));
            };

            Ok(StringRange {
                start: entry.range().start(),
                end: entry.range().end(),
                value: value.clone(),
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;

    ranges.sort_unstable_by_key(|range| range.start);
    validate_non_overlapping(&ranges, property_name, |range| (range.start, range.end))?;

    let mut merged: Vec<StringRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(previous) = merged.last_mut()
            && previous.end.checked_add(1) == Some(range.start)
            && previous.value == range.value
        {
            previous.end = range.end;
        } else {
            merged.push(range);
        }
    }

    Ok(merged)
}

fn build_complete_string_property(
    entries: &[UcdEntry],
    property_name: &str,
) -> Result<Vec<StringRange>, GenerationError> {
    let mut explicit = property_rules(entries, property_name, false)?;
    let missing = property_rules(entries, property_name, true)?;
    explicit.sort_unstable_by_key(|range| range.start);
    validate_non_overlapping(&explicit, property_name, |range| (range.start, range.end))?;
    validate_missing_rules(&missing, property_name)?;

    let mut boundaries = vec![0, MAX_CODE_POINT + 1];
    for range in explicit.iter().chain(&missing) {
        boundaries.push(range.start);
        boundaries.push(range.end + 1);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut ranges: Vec<StringRange> = Vec::with_capacity(boundaries.len());
    for bounds in boundaries.windows(2) {
        let start = bounds[0];
        let end = bounds[1] - 1;
        let value = explicit
            .iter()
            .find(|range| range.start <= start && start <= range.end)
            .or_else(|| {
                missing
                    .iter()
                    .filter(|range| range.start <= start && start <= range.end)
                    .min_by_key(|range| range.end - range.start)
            })
            .ok_or_else(|| {
                GenerationError::Data(format!("{property_name} has no value for U+{start:04X}"))
            })?
            .value
            .clone();

        if let Some(previous) = ranges.last_mut()
            && previous.end + 1 == start
            && previous.value == value
        {
            previous.end = end;
        } else {
            ranges.push(StringRange { start, end, value });
        }
    }

    Ok(ranges)
}

fn property_rules(
    entries: &[UcdEntry],
    property_name: &str,
    missing: bool,
) -> Result<Vec<StringRange>, GenerationError> {
    entries
        .iter()
        .filter(|entry| entry.is_missing() == missing)
        .map(|entry| {
            let [value] = entry.fields() else {
                let kind = if missing { "@missing" } else { "entry" };
                return Err(GenerationError::Data(format!(
                    "{property_name} {kind} U+{:04X} must have exactly one field",
                    entry.range().start()
                )));
            };
            Ok(StringRange {
                start: entry.range().start(),
                end: entry.range().end(),
                value: value.clone(),
            })
        })
        .collect()
}

fn validate_missing_rules(
    missing: &[StringRange],
    property_name: &str,
) -> Result<(), GenerationError> {
    if !missing
        .iter()
        .any(|range| range.start == 0 && range.end == MAX_CODE_POINT)
    {
        return Err(GenerationError::Data(format!(
            "{property_name} must contain an @missing rule for U+0000..U+10FFFF"
        )));
    }

    for (index, left) in missing.iter().enumerate() {
        for right in &missing[index + 1..] {
            let overlap = left.start <= right.end && right.start <= left.end;
            let nested = (left.start <= right.start && right.end <= left.end)
                || (right.start <= left.start && left.end <= right.end);
            if overlap && !nested {
                let start = left.start.max(right.start);
                return Err(GenerationError::Data(format!(
                    "{property_name} @missing rules overlap without nesting at U+{start:04X}"
                )));
            }
            if left.start == right.start && left.end == right.end {
                return Err(GenerationError::Data(format!(
                    "{property_name} has duplicate @missing rules for U+{:04X}..U+{:04X}",
                    left.start, left.end
                )));
            }
        }
    }

    Ok(())
}

fn validate_decomposition_entries(entries: &[DecompositionEntry]) -> Result<(), GenerationError> {
    for pair in entries.windows(2) {
        if pair[0].code_point() >= pair[1].code_point() {
            return Err(GenerationError::Data(format!(
                "Decomposition Mapping entries are not ordered at U+{:04X}",
                pair[1].code_point()
            )));
        }
    }
    for entry in entries {
        if entry.mapping().is_empty() {
            return Err(GenerationError::Data(format!(
                "Decomposition Mapping for U+{:04X} is empty",
                entry.code_point()
            )));
        }
        match entry.kind() {
            DecompositionKind::Canonical => {}
            DecompositionKind::Compatibility(tag) if !tag.is_empty() => {}
            DecompositionKind::Compatibility(_) => {
                return Err(GenerationError::Data(format!(
                    "Decomposition Mapping for U+{:04X} has an empty compatibility tag",
                    entry.code_point()
                )));
            }
        }
    }
    Ok(())
}

fn validate_missing_entry(
    entries: &[UcdEntry],
    property_name: &str,
    expected_value: &str,
) -> Result<(), GenerationError> {
    let missing = entries
        .iter()
        .filter(|entry| entry.is_missing())
        .collect::<Vec<_>>();
    let [entry] = missing.as_slice() else {
        return Err(GenerationError::Data(format!(
            "{property_name} must contain exactly one @missing entry"
        )));
    };

    if entry.range().start() != 0
        || entry.range().end() != MAX_CODE_POINT
        || entry.fields() != [expected_value]
    {
        return Err(GenerationError::Data(format!(
            "{property_name} @missing must be U+0000..U+10FFFF; {expected_value}"
        )));
    }

    Ok(())
}

fn build_binary_property(
    entries: &[UcdEntry],
    property_name: &str,
) -> Result<Vec<CodePointRange>, GenerationError> {
    let mut ranges = entries
        .iter()
        .filter(|entry| {
            !entry.is_missing()
                && entry
                    .fields()
                    .first()
                    .is_some_and(|value| value == property_name)
        })
        .map(|entry| {
            if entry.fields() != [property_name] {
                return Err(GenerationError::Data(format!(
                    "{property_name} entry U+{:04X} must have exactly one field",
                    entry.range().start()
                )));
            }

            Ok(CodePointRange {
                start: entry.range().start(),
                end: entry.range().end(),
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;

    if ranges.is_empty() {
        return Err(GenerationError::Data(format!(
            "{property_name} contains no entries"
        )));
    }

    ranges.sort_unstable_by_key(|range| range.start);
    validate_non_overlapping(&ranges, property_name, |range| (range.start, range.end))?;

    let mut merged: Vec<CodePointRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(previous) = merged.last_mut()
            && previous.end.checked_add(1) == Some(range.start)
        {
            previous.end = range.end;
        } else {
            merged.push(range);
        }
    }

    Ok(merged)
}

fn validate_non_overlapping<T>(
    ranges: &[T],
    property_name: &str,
    bounds: impl Fn(&T) -> (u32, u32),
) -> Result<(), GenerationError> {
    for pair in ranges.windows(2) {
        let (_, previous_end) = bounds(&pair[0]);
        let (next_start, _) = bounds(&pair[1]);
        if next_start <= previous_end {
            return Err(GenerationError::Data(format!(
                "{property_name} ranges overlap at U+{next_start:04X}"
            )));
        }
    }

    Ok(())
}

fn validate_complete_coverage(ranges: &[GeneralCategoryRange]) -> Result<(), GenerationError> {
    let Some(first) = ranges.first() else {
        return Err(GenerationError::Data(
            "DerivedGeneralCategory contains no entries".to_owned(),
        ));
    };

    if first.start != 0 {
        return Err(GenerationError::Data(format!(
            "General Category coverage starts at U+{:04X} instead of U+0000",
            first.start
        )));
    }

    for pair in ranges.windows(2) {
        let expected = pair[0].end.checked_add(1).ok_or_else(|| {
            GenerationError::Data("General Category range extends past U+10FFFF".to_owned())
        })?;
        if pair[1].start != expected {
            return Err(GenerationError::Data(format!(
                "General Category ranges overlap or leave a gap between U+{:04X} and U+{:04X}",
                pair[0].end, pair[1].start
            )));
        }
    }

    let end = ranges.last().expect("the first entry exists").end;
    if end != MAX_CODE_POINT {
        return Err(GenerationError::Data(format!(
            "General Category coverage ends at U+{end:04X} instead of U+10FFFF"
        )));
    }

    Ok(())
}

fn general_category_variant(value: &str) -> Result<&'static str, GenerationError> {
    let variant = match value {
        "Lu" => "UppercaseLetter",
        "Ll" => "LowercaseLetter",
        "Lt" => "TitlecaseLetter",
        "Lm" => "ModifierLetter",
        "Lo" => "OtherLetter",
        "Mn" => "NonspacingMark",
        "Mc" => "SpacingMark",
        "Me" => "EnclosingMark",
        "Nd" => "DecimalNumber",
        "Nl" => "LetterNumber",
        "No" => "OtherNumber",
        "Pc" => "ConnectorPunctuation",
        "Pd" => "DashPunctuation",
        "Ps" => "OpenPunctuation",
        "Pe" => "ClosePunctuation",
        "Pi" => "InitialPunctuation",
        "Pf" => "FinalPunctuation",
        "Po" => "OtherPunctuation",
        "Sm" => "MathSymbol",
        "Sc" => "CurrencySymbol",
        "Sk" => "ModifierSymbol",
        "So" => "OtherSymbol",
        "Zs" => "SpaceSeparator",
        "Zl" => "LineSeparator",
        "Zp" => "ParagraphSeparator",
        "Cc" => "Control",
        "Cf" => "Format",
        "Cs" => "Surrogate",
        "Co" => "PrivateUse",
        "Cn" => "Unassigned",
        _ => {
            return Err(GenerationError::Data(format!(
                "unknown General Category `{value}`"
            )));
        }
    };

    Ok(variant)
}

struct RenderData<'a> {
    names: &'a [(u32, String)],
    categories: &'a [GeneralCategoryRange],
    blocks: &'a [StringRange],
    scripts: &'a [StringRange],
    ages: &'a [StringRange],
    default_ignorables: &'a [CodePointRange],
    name_aliases: &'a [NameAliasGroup],
    east_asian_widths: &'a [EnumRange],
    canonical_combining_classes: &'a [NumericRange],
    canonical_combining_class_values: &'a [CanonicalCombiningClassValue],
    decompositions: &'a [DecompositionEntry],
    bidi_classes: &'a [EnumRange],
}

fn render(data: &RenderData<'_>) -> String {
    let mut output = String::with_capacity(
        data.names.len() * 48
            + data.categories.len() * 72
            + (data.blocks.len() + data.scripts.len() + data.ages.len()) * 48
            + data.default_ignorables.len() * 28
            + data
                .name_aliases
                .iter()
                .map(|group| group.aliases.len())
                .sum::<usize>()
                * 64
            + (data.east_asian_widths.len() + data.bidi_classes.len()) * 64
            + data.canonical_combining_classes.len() * 48
            + data
                .decompositions
                .iter()
                .map(|entry| 72 + entry.mapping().len() * 10)
                .sum::<usize>(),
    );
    render_header(&mut output);
    output.push_str(
        "use crate::unicode::{\n    BidiClass, Decomposition, EastAsianWidth, GeneralCategory, NameAlias, NameAliasType,\n};\n\n",
    );
    output.push_str(&format!(
        "pub(super) const UNICODE_VERSION: &str = {UNICODE_VERSION:?};\n\n"
    ));
    output.push_str("#[rustfmt::skip]\n");
    output.push_str("pub(super) const PRIMARY_NAMES: &[(u32, &str)] = &[\n");
    for (code_point, name) in data.names {
        output.push_str(&format!("    (0x{code_point:06X}, {name:?}),\n"));
    }
    output.push_str("];\n\n");
    output.push_str("#[rustfmt::skip]\n");
    output.push_str("pub(super) const GENERAL_CATEGORIES: &[(u32, u32, GeneralCategory)] = &[\n");
    for range in data.categories {
        output.push_str(&format!(
            "    (0x{:06X}, 0x{:06X}, GeneralCategory::{}),\n",
            range.start, range.end, range.variant
        ));
    }
    output.push_str("];\n\n");
    render_string_ranges(&mut output, "BLOCKS", data.blocks);
    output.push('\n');
    render_string_ranges(&mut output, "SCRIPTS", data.scripts);
    output.push('\n');
    render_string_ranges(&mut output, "AGES", data.ages);
    output.push('\n');
    output.push_str("#[rustfmt::skip]\n");
    output.push_str("pub(super) const DEFAULT_IGNORABLES: &[(u32, u32)] = &[\n");
    for range in data.default_ignorables {
        output.push_str(&format!(
            "    (0x{:06X}, 0x{:06X}),\n",
            range.start, range.end
        ));
    }
    output.push_str("];\n\n");
    output.push_str("#[rustfmt::skip]\n");
    output.push_str("pub(super) const NAME_ALIASES: &[(u32, &[NameAlias])] = &[\n");
    for group in data.name_aliases {
        output.push_str(&format!("    (0x{:06X}, &[\n", group.code_point));
        for alias in &group.aliases {
            output.push_str(&format!(
                "        NameAlias::new({:?}, NameAliasType::{}),\n",
                alias.name, alias.variant
            ));
        }
        output.push_str("    ]),\n");
    }
    output.push_str("];\n\n");
    render_enum_ranges(
        &mut output,
        "EAST_ASIAN_WIDTHS",
        "EastAsianWidth",
        data.east_asian_widths,
    );
    output.push('\n');
    output.push_str("#[rustfmt::skip]\n");
    output.push_str("pub(super) const CANONICAL_COMBINING_CLASS_VALUES: &[(u8, &str)] = &[\n");
    for value in data.canonical_combining_class_values {
        output.push_str(&format!("    ({}, {:?}),\n", value.value, value.name));
    }
    output.push_str("];\n\n");
    output.push_str("#[rustfmt::skip]\n");
    output.push_str("pub(super) const CANONICAL_COMBINING_CLASSES: &[(u32, u32, u8)] = &[\n");
    for range in data.canonical_combining_classes {
        output.push_str(&format!(
            "    (0x{:06X}, 0x{:06X}, {}),\n",
            range.start, range.end, range.value
        ));
    }
    output.push_str("];\n\n");
    render_decompositions(&mut output, data.decompositions);
    output.push('\n');
    render_enum_ranges(&mut output, "BIDI_CLASSES", "BidiClass", data.bidi_classes);
    output
}

fn render_header(output: &mut String) {
    output.push_str("// @generated by tools/ucd-generator. DO NOT EDIT.\n");
    output.push_str(&format!("// Unicode version: {UNICODE_VERSION}\n"));
    output.push_str("// Contains data derived from the Unicode Character Database.\n");
    output.push_str("// Copyright © 1991-2026 Unicode, Inc.\n");
    output.push_str("// Distributed under Unicode License v3; see UNICODE-LICENSE.txt.\n");
    output.push_str("// Third-party attribution details: THIRD_PARTY_NOTICES.\n");
    output.push_str(&format!(
        "// Sources: data/ucd/{UNICODE_VERSION}/DerivedName.txt, DerivedGeneralCategory.txt,\n"
    ));
    output.push_str("// Blocks.txt, Scripts.txt, DerivedAge.txt, DerivedCoreProperties.txt,\n");
    output.push_str("// NameAliases.txt, EastAsianWidth.txt, DerivedCombiningClass.txt,\n");
    output.push_str("// DerivedBidiClass.txt, PropertyValueAliases.txt, and UnicodeData.txt\n");
    output.push_str(&format!(
        "// Regenerate: cargo run --manifest-path tools/ucd-generator/Cargo.toml -- data/ucd/{UNICODE_VERSION} src/unicode/generated.rs\n\n"
    ));
}

fn render_decompositions(output: &mut String, decompositions: &[DecompositionEntry]) {
    output.push_str("#[rustfmt::skip]\n");
    output.push_str("pub(super) const DECOMPOSITIONS: &[(u32, Decomposition)] = &[\n");
    for decomposition in decompositions {
        let mapping = decomposition
            .mapping()
            .iter()
            .map(|value| format!("0x{value:06X}"))
            .collect::<Vec<_>>()
            .join(", ");
        match decomposition.kind() {
            DecompositionKind::Canonical => output.push_str(&format!(
                "    (0x{:06X}, Decomposition::canonical(&[{mapping}])),\n",
                decomposition.code_point()
            )),
            DecompositionKind::Compatibility(tag) => {
                output.push_str(&format!(
                    "    (0x{:06X}, Decomposition::compatibility({tag:?}, &[{mapping}])),\n",
                    decomposition.code_point()
                ));
            }
        }
    }
    output.push_str("];\n");
}

fn render_string_ranges(output: &mut String, constant_name: &str, ranges: &[StringRange]) {
    output.push_str("#[rustfmt::skip]\n");
    output.push_str(&format!(
        "pub(super) const {constant_name}: &[(u32, u32, &str)] = &[\n"
    ));
    for range in ranges {
        output.push_str(&format!(
            "    (0x{:06X}, 0x{:06X}, {:?}),\n",
            range.start, range.end, range.value
        ));
    }
    output.push_str("];\n");
}

fn render_enum_ranges(
    output: &mut String,
    constant_name: &str,
    enum_name: &str,
    ranges: &[EnumRange],
) {
    output.push_str("#[rustfmt::skip]\n");
    output.push_str(&format!(
        "pub(super) const {constant_name}: &[(u32, u32, {enum_name})] = &[\n"
    ));
    for range in ranges {
        output.push_str(&format!(
            "    (0x{:06X}, 0x{:06X}, {enum_name}::{}),\n",
            range.start, range.end, range.variant
        ));
    }
    output.push_str("];\n");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeneralCategoryRange {
    start: u32,
    end: u32,
    variant: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StringRange {
    start: u32,
    end: u32,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NameAliasGroup {
    code_point: u32,
    aliases: Vec<NameAliasValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NameAliasValue {
    name: String,
    variant: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EnumRange {
    start: u32,
    end: u32,
    variant: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NumericRange {
    start: u32,
    end: u32,
    value: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalCombiningClassValue {
    value: u8,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodePointRange {
    start: u32,
    end: u32,
}

#[derive(Debug)]
pub enum GenerationError {
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: std::path::PathBuf,
        source: crate::ucd::ParseError,
    },
    UnicodeDataParse {
        path: std::path::PathBuf,
        source: crate::unicode_data::ParseError,
    },
    PropertyValueAliasParse {
        path: std::path::PathBuf,
        source: crate::property_value_aliases::ParseError,
    },
    Data(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Parse { path, source } => write!(formatter, "{}:{source}", path.display()),
            Self::UnicodeDataParse { path, source } => {
                write!(formatter, "{}:{source}", path.display())
            }
            Self::PropertyValueAliasParse { path, source } => {
                write!(formatter, "{}:{source}", path.display())
            }
            Self::Data(message) => formatter.write_str(message),
        }
    }
}

impl Error for GenerationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::UnicodeDataParse { source, .. } => Some(source),
            Self::PropertyValueAliasParse { source, .. } => Some(source),
            Self::Data(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn generated_header_includes_unicode_license_attribution() {
        let mut output = String::new();

        render_header(&mut output);

        assert_eq!(
            output,
            concat!(
                "// @generated by tools/ucd-generator. DO NOT EDIT.\n",
                "// Unicode version: 17.0.0\n",
                "// Contains data derived from the Unicode Character Database.\n",
                "// Copyright © 1991-2026 Unicode, Inc.\n",
                "// Distributed under Unicode License v3; see UNICODE-LICENSE.txt.\n",
                "// Third-party attribution details: THIRD_PARTY_NOTICES.\n",
                "// Sources: data/ucd/17.0.0/DerivedName.txt, DerivedGeneralCategory.txt,\n",
                "// Blocks.txt, Scripts.txt, DerivedAge.txt, DerivedCoreProperties.txt,\n",
                "// NameAliases.txt, EastAsianWidth.txt, DerivedCombiningClass.txt,\n",
                "// DerivedBidiClass.txt, PropertyValueAliases.txt, and UnicodeData.txt\n",
                "// Regenerate: cargo run --manifest-path tools/ucd-generator/Cargo.toml -- data/ucd/17.0.0 src/unicode/generated.rs\n\n",
            )
        );
    }

    #[test]
    fn expands_pattern_names_and_sorts_them() {
        let entries = parse(
            "4E00..4E02 ; CJK UNIFIED IDEOGRAPH-*\n0041 ; LATIN CAPITAL LETTER A\n18CFF ; KHITAN SMALL SCRIPT CHARACTER-*",
        )
        .unwrap();

        assert_eq!(
            build_names(&entries).unwrap(),
            [
                (0x0041, "LATIN CAPITAL LETTER A".to_owned()),
                (0x4e00, "CJK UNIFIED IDEOGRAPH-4E00".to_owned()),
                (0x4e01, "CJK UNIFIED IDEOGRAPH-4E01".to_owned()),
                (0x4e02, "CJK UNIFIED IDEOGRAPH-4E02".to_owned()),
                (0x18cff, "KHITAN SMALL SCRIPT CHARACTER-18CFF".to_owned(),),
            ]
        );
    }

    #[test]
    fn renders_canonical_and_compatibility_decompositions() {
        let entries =
            parse_decomposition_entries(include_str!("../tests/fixtures/unicode-data-valid.txt"))
                .unwrap();
        let mut output = String::new();

        render_decompositions(&mut output, &entries);

        assert_eq!(
            output,
            concat!(
                "#[rustfmt::skip]\n",
                "pub(super) const DECOMPOSITIONS: &[(u32, Decomposition)] = &[\n",
                "    (0x0000A0, Decomposition::compatibility(\"noBreak\", &[0x000020])),\n",
                "    (0x0000E9, Decomposition::canonical(&[0x000065, 0x000301])),\n",
                "];\n",
            )
        );
    }

    #[test]
    fn groups_name_aliases_by_code_point_without_reordering_each_group() {
        let entries =
            parse("0001;START OF HEADING;control\n0000;NULL;control\n0000;NUL;abbreviation")
                .unwrap();

        assert_eq!(
            build_name_aliases(&entries).unwrap(),
            [
                NameAliasGroup {
                    code_point: 0,
                    aliases: vec![
                        NameAliasValue {
                            name: "NULL".to_owned(),
                            variant: "Control",
                        },
                        NameAliasValue {
                            name: "NUL".to_owned(),
                            variant: "Abbreviation",
                        },
                    ],
                },
                NameAliasGroup {
                    code_point: 1,
                    aliases: vec![NameAliasValue {
                        name: "START OF HEADING".to_owned(),
                        variant: "Control",
                    }],
                },
            ]
        );
    }

    #[rstest]
    #[case("correction", "Correction")]
    #[case("control", "Control")]
    #[case("alternate", "Alternate")]
    #[case("figment", "Figment")]
    #[case("abbreviation", "Abbreviation")]
    fn maps_all_formal_name_alias_types(#[case] value: &str, #[case] expected: &str) {
        assert_eq!(name_alias_type_variant(value).unwrap(), expected);
    }

    #[test]
    fn sorts_and_merges_complete_general_category_ranges() {
        let entries = parse("0002..10FFFF ; Cn\n0001 ; Cn\n0000 ; Cc").unwrap();

        assert_eq!(
            build_general_categories(&entries).unwrap(),
            [
                GeneralCategoryRange {
                    start: 0,
                    end: 0,
                    variant: "Control",
                },
                GeneralCategoryRange {
                    start: 1,
                    end: 0x10_ffff,
                    variant: "Unassigned",
                },
            ]
        );
    }

    #[test]
    fn rejects_general_category_gaps() {
        let entries = parse("0000 ; Cc\n0002..10FFFF ; Cn").unwrap();

        assert_eq!(
            build_general_categories(&entries).unwrap_err().to_string(),
            "General Category ranges overlap or leave a gap between U+0000 and U+0002"
        );
    }

    #[test]
    fn builds_sparse_string_properties_and_checks_the_default() {
        let entries =
            parse("# @missing: 0000..10FFFF; Unknown\n0042 ; Latin\n0041 ; Latin\n0061 ; Latin")
                .unwrap();

        assert_eq!(
            build_string_property(&entries, "Script", "Unknown").unwrap(),
            [
                StringRange {
                    start: 0x0041,
                    end: 0x0042,
                    value: "Latin".to_owned(),
                },
                StringRange {
                    start: 0x0061,
                    end: 0x0061,
                    value: "Latin".to_owned(),
                },
            ]
        );
        assert_eq!(
            build_string_property(&entries, "Script", "No_Script")
                .unwrap_err()
                .to_string(),
            "Script @missing must be U+0000..U+10FFFF; No_Script"
        );
    }

    #[test]
    fn composes_explicit_bidi_values_over_nested_missing_defaults() {
        let entries = parse(include_str!("../tests/fixtures/bidi-multiple-missing.txt")).unwrap();

        assert_eq!(
            build_complete_string_property(&entries, "Bidi Class").unwrap(),
            [
                StringRange {
                    start: 0x0000,
                    end: 0x058f,
                    value: "Left_To_Right".to_owned(),
                },
                StringRange {
                    start: 0x0590,
                    end: 0x0590,
                    value: "Right_To_Left".to_owned(),
                },
                StringRange {
                    start: 0x0591,
                    end: 0x0591,
                    value: "Nonspacing_Mark".to_owned(),
                },
                StringRange {
                    start: 0x0592,
                    end: 0x05ff,
                    value: "Right_To_Left".to_owned(),
                },
                StringRange {
                    start: 0x0600,
                    end: 0x0604,
                    value: "Arabic_Letter".to_owned(),
                },
                StringRange {
                    start: 0x0605,
                    end: 0x0605,
                    value: "Arabic_Number".to_owned(),
                },
                StringRange {
                    start: 0x0606,
                    end: 0x06ff,
                    value: "Arabic_Letter".to_owned(),
                },
                StringRange {
                    start: 0x0700,
                    end: 0x10_ffff,
                    value: "Left_To_Right".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn rejects_crossing_missing_defaults() {
        let entries = parse(
            "# @missing: 0000..10FFFF; Left_To_Right\n# @missing: 0100..02FF; Right_To_Left\n# @missing: 0200..03FF; Arabic_Letter",
        )
        .unwrap();

        assert_eq!(
            build_complete_string_property(&entries, "Bidi Class")
                .unwrap_err()
                .to_string(),
            "Bidi Class @missing rules overlap without nesting at U+0200"
        );
    }

    #[test]
    fn selects_and_merges_one_binary_property() {
        let entries = parse(
            "0041 ; Other_Property\n00AD ; Default_Ignorable_Code_Point\n00AE..00AF ; Default_Ignorable_Code_Point",
        )
        .unwrap();

        assert_eq!(
            build_binary_property(&entries, "Default_Ignorable_Code_Point").unwrap(),
            [CodePointRange {
                start: 0x00ad,
                end: 0x00af,
            }]
        );
    }

    #[test]
    fn rejects_overlapping_sparse_property_ranges() {
        let entries = parse(
            "# @missing: 0000..10FFFF; No_Block\n0000..007F ; Basic Latin\n007F..00FF ; Latin-1 Supplement",
        )
        .unwrap();

        assert_eq!(
            build_string_property(&entries, "Block", "No_Block")
                .unwrap_err()
                .to_string(),
            "Block ranges overlap at U+007F"
        );
    }

    #[test]
    fn requires_the_expected_final_unicode_version() {
        assert!(
            validate_unicode_version(
                "This directory contains final data files\nfor Version 17.0.0 of the Unicode Standard."
            )
            .is_ok()
        );
        assert_eq!(
            validate_unicode_version(
                "This directory contains final data files\nfor Version 18.0.0 of the Unicode Standard."
            )
            .unwrap_err()
            .to_string(),
            "ReadMe.txt does not identify final Unicode 17.0.0 data"
        );
    }
}
