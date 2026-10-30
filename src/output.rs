use std::{
    borrow::Cow,
    error::Error,
    fmt,
    io::{self, Write},
};

use clap::ValueEnum;

use crate::{
    cli::LaunchTarget,
    unicode::{CodePoint, representation::TextFormat},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
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
    };
    match write_literal(format, &text, writer) {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(OutputError::Write),
    }
}

fn write_literal(format: TextFormat, text: &str, mut writer: impl Write) -> io::Result<()> {
    for atom in format.atoms(text) {
        writer.write_all(atom.as_bytes())?;
    }
    writer.write_all(b"\n")?;
    writer.flush()
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
    #[case(false)]
    #[case(true)]
    fn handles_write_and_flush_failures(#[case] on_flush: bool) {
        for kind in [io::ErrorKind::Other, io::ErrorKind::BrokenPipe] {
            let result = write(
                OutputFormat::JsonString,
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
