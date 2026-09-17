mod generate;
mod property_value_aliases;
mod ucd;
mod unicode_data;

use std::{env, error::Error, fmt, path::PathBuf, process::ExitCode};

use crate::generate::generate;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ucd-generator: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let (input_directory, output) = paths(env::args_os().skip(1))?;
    let summary = generate(&input_directory, &output)?;

    println!("generated {}", output.display());
    println!("Unicode version: {}", summary.unicode_version);
    println!("primary names: {}", summary.name_count);
    println!(
        "General Category ranges: {}",
        summary.general_category_range_count
    );
    println!("Block ranges: {}", summary.block_range_count);
    println!("Script ranges: {}", summary.script_range_count);
    println!("Age ranges: {}", summary.age_range_count);
    println!(
        "Default Ignorable ranges: {}",
        summary.default_ignorable_range_count
    );
    println!("name aliases: {}", summary.name_alias_count);
    println!(
        "East Asian Width ranges: {}",
        summary.east_asian_width_range_count
    );
    println!(
        "Canonical Combining Class ranges: {}",
        summary.canonical_combining_class_range_count
    );
    println!(
        "decomposition mappings: {}",
        summary.decomposition_mapping_count
    );
    println!("Bidi Class ranges: {}", summary.bidi_class_range_count);

    Ok(())
}

fn paths(
    mut arguments: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(PathBuf, PathBuf), UsageError> {
    let Some(input_directory) = arguments.next() else {
        return Err(UsageError);
    };
    let Some(output) = arguments.next() else {
        return Err(UsageError);
    };

    if arguments.next().is_some() {
        return Err(UsageError);
    }

    Ok((input_directory.into(), output.into()))
}

#[derive(Debug)]
struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("usage: ucd-generator <ucd-directory> <output-file>")
    }
}

impl Error for UsageError {}
