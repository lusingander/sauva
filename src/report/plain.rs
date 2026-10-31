use std::io::{self, Write};

use crate::{
    cli::LaunchTarget,
    inspector::{
        InspectorDisplayMode, InspectorField, InspectorFieldId, InspectorSection,
        UNAVAILABLE_ENCODING,
    },
    unicode::{
        CodePoint, UnicodeDatabase,
        representation::TextFormat,
        text::{NormalizationForm, TextAnalysis, TextMetrics},
    },
};

pub fn write(target: &LaunchTarget, mut writer: impl Write) -> io::Result<()> {
    match target {
        LaunchTarget::CodePoint(point) => write_code_point(*point, &mut writer),
        LaunchTarget::Sequence(text) => {
            write_sequence(&TextAnalysis::new(text.clone()), &mut writer)
        }
    }
}

fn write_code_point(point: CodePoint, writer: &mut impl Write) -> io::Result<()> {
    let character = point.to_char();
    let mut section = None;
    for field in InspectorField::for_code_point(point, InspectorDisplayMode::Relevant) {
        if section != Some(field.section()) {
            if section.is_some() {
                write!(writer, "\n\n")?;
            }
            // Relevant omits unavailable encoding fields; reports explain why.
            if character.is_none() && field.section() == InspectorSection::Data {
                write!(writer, "Encoding\n  {UNAVAILABLE_ENCODING}\n\n")?;
            }
            write!(writer, "{}", field.section().label())?;
            section = Some(field.section());
        }

        write!(writer, "\n  {}:", field.label())?;
        if field.id() == InspectorFieldId::Character {
            write!(writer, " ")?;
            if let Some(character) = character {
                write_text(&character.to_string(), writer)?;
            } else {
                write!(writer, "{UNAVAILABLE_ENCODING}")?;
            }
        } else if field.values().len() == 1 && field.id() != InspectorFieldId::Decomposition {
            write!(writer, " {}", field.values()[0])?;
        } else {
            for value in field.values() {
                write!(writer, "\n    {value}")?;
            }
        }
    }

    write!(writer, "\n\nText Analysis\n  ")?;
    if let Some(character) = character {
        let analysis = TextAnalysis::new(character.to_string());
        write_metrics(analysis.metrics(), writer)?;
        write_normalization(&analysis, writer)
    } else {
        write!(
            writer,
            "{UNAVAILABLE_ENCODING}\n\nNormalization Forms\n  {UNAVAILABLE_ENCODING}"
        )
    }
}

fn write_sequence(analysis: &TextAnalysis, writer: &mut impl Write) -> io::Result<()> {
    write!(writer, "Input: ")?;
    write_text(analysis.source(), writer)?;
    writeln!(writer)?;
    let metrics = analysis.metrics();
    write_metrics(metrics, writer)?;
    write!(
        writer,
        "\n\nPositions: CP# / GC# are 1-based; byte / unit ranges are 0-based, end-exclusive.\n"
    )?;

    let cp_width = metrics.code_points.to_string().len().max(3);
    let gc_width = metrics.graphemes.to_string().len().max(3);
    let utf8_width = (metrics.utf8_bytes.to_string().len() * 2 + 3).max(11);
    let utf16_width = (metrics.utf16_units.to_string().len() * 2 + 3).max(12);
    write!(
        writer,
        "{:<cp_width$}  {:<gc_width$}  {:<10}  {:<utf8_width$}  {:<utf16_width$}  Name",
        "CP#", "GC#", "Code Point", "UTF-8 Bytes", "UTF-16 Units"
    )?;
    for (index, entry) in analysis.code_points().iter().enumerate() {
        let point = entry.code_point();
        let utf8 = entry.byte_range();
        let utf16 = entry.utf16_range();
        write!(
            writer,
            "\n{:<cp_width$}  {:<gc_width$}  {:<10}  {:<utf8_width$}  {:<utf16_width$}  {}",
            index + 1,
            entry.grapheme_index() + 1,
            point.to_string(),
            format!("[{},{})", utf8.start, utf8.end),
            format!("[{},{})", utf16.start, utf16.end),
            UnicodeDatabase::primary_name_or_fallback(point),
        )?;
    }
    write_normalization(analysis, writer)?;
    write!(
        writer,
        "\n\nUnicode Version: {}",
        UnicodeDatabase::version()
    )
}

fn write_metrics(metrics: TextMetrics, writer: &mut impl Write) -> io::Result<()> {
    write!(
        writer,
        "CP: {}  UTF-8 Bytes: {}  UTF-16 Units: {}  GC: {}",
        metrics.code_points, metrics.utf8_bytes, metrics.utf16_units, metrics.graphemes
    )
}

fn write_normalization(analysis: &TextAnalysis, writer: &mut impl Write) -> io::Result<()> {
    let mut widths = [2, 11, 12, 2];
    for form in NormalizationForm::ALL {
        let metrics = analysis.normalization().get(form).metrics();
        for (width, count) in widths.iter_mut().zip([
            metrics.code_points,
            metrics.utf8_bytes,
            metrics.utf16_units,
            metrics.graphemes,
        ]) {
            *width = (*width).max(count.to_string().len());
        }
    }
    let [cp_width, utf8_width, utf16_width, gc_width] = widths;
    write!(
        writer,
        "\n\nNormalization Forms\n  {:<4}  {:<9}  {:<cp_width$}  {:<utf8_width$}  {:<utf16_width$}  {:<gc_width$}  Result",
        "Form", "State", "CP", "UTF-8 Bytes", "UTF-16 Units", "GC"
    )?;
    for form in NormalizationForm::ALL {
        let result = analysis.normalization().get(form);
        let metrics = result.metrics();
        write!(
            writer,
            "\n  {:<4}  {:<9}  {:<cp_width$}  {:<utf8_width$}  {:<utf16_width$}  {:<gc_width$}  ",
            form.label(),
            if result.is_changed() {
                "changed"
            } else {
                "unchanged"
            },
            metrics.code_points,
            metrics.utf8_bytes,
            metrics.utf16_units,
            metrics.graphemes,
        )?;
        write_text(analysis.normalized_text(form), writer)?;
    }
    Ok(())
}

fn write_text(text: &str, writer: &mut impl Write) -> io::Result<()> {
    for atom in TextFormat::JsonString.atoms(text) {
        writer.write_all(atom.as_bytes())?;
    }
    Ok(())
}
