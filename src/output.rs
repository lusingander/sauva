use std::{
    borrow::Cow,
    error::Error,
    fmt,
    io::{self, BufWriter, Write},
};

use clap::ValueEnum;

use crate::{
    cli::LaunchTarget,
    report,
    unicode::{CodePoint, representation::TextFormat},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    JsonString,
    RustString,
}

#[derive(Debug)]
pub enum OutputError {
    NonScalar(CodePoint),
    Write(io::Error),
}

impl fmt::Display for OutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonScalar(point) => write!(
                formatter,
                "cannot output {point} as a string: not a Unicode scalar value"
            ),
            Self::Write(error) => write!(formatter, "failed to write stdout: {error}"),
        }
    }
}

impl Error for OutputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NonScalar(_) => None,
            Self::Write(error) => Some(error),
        }
    }
}

pub fn write(
    format: OutputFormat,
    target: &LaunchTarget,
    writer: impl Write,
) -> Result<(), OutputError> {
    let mut writer = BufWriter::new(writer);
    let result = match format {
        OutputFormat::Json => report::write(target, &mut writer),
        OutputFormat::JsonString | OutputFormat::RustString => {
            let text = match target {
                LaunchTarget::CodePoint(point) => Cow::Owned(
                    point
                        .to_char()
                        .ok_or(OutputError::NonScalar(*point))?
                        .to_string(),
                ),
                LaunchTarget::Sequence(text) => Cow::Borrowed(text.as_str()),
            };
            let format = match format {
                OutputFormat::JsonString => TextFormat::JsonString,
                OutputFormat::RustString => TextFormat::RustString,
                OutputFormat::Json => unreachable!(),
            };
            write_literal(format, &text, &mut writer)
        }
    }
    .and_then(|()| {
        writer.write_all(b"\n")?;
        writer.flush()
    });
    match result {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(OutputError::Write),
    }
}

fn write_literal(format: TextFormat, text: &str, mut writer: impl Write) -> io::Result<()> {
    for atom in format.atoms(text) {
        writer.write_all(atom.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    struct FailingWriter {
        kind: io::ErrorKind,
        on_flush: bool,
    }

    impl Write for FailingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.on_flush {
                Ok(bytes.len())
            } else {
                Err(self.kind.into())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(self.kind.into())
        }
    }

    #[rstest]
    #[case(OutputFormat::Json, false)]
    #[case(OutputFormat::Json, true)]
    #[case(OutputFormat::JsonString, false)]
    #[case(OutputFormat::JsonString, true)]
    #[case(OutputFormat::RustString, false)]
    #[case(OutputFormat::RustString, true)]
    fn handles_write_and_flush_failures(#[case] format: OutputFormat, #[case] on_flush: bool) {
        for kind in [io::ErrorKind::Other, io::ErrorKind::BrokenPipe] {
            let result = write(
                format,
                &LaunchTarget::Sequence("A".to_owned()),
                FailingWriter { kind, on_flush },
            );

            if kind == io::ErrorKind::BrokenPipe {
                assert!(result.is_ok());
            } else {
                assert!(matches!(result, Err(OutputError::Write(error)) if error.kind() == kind));
            }
        }
    }
}
