use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

use rstest::rstest;

#[rstest]
#[case(&["--help"])]
#[case(&["--output", "json-string", "--help"])]
fn prints_usage_and_supported_output_formats_for_help(#[case] arguments: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(arguments)
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("sauva"));
    assert!(stdout.contains("[possible values: json-string, rust-string]"));
    assert!(!stdout.contains("possible values: plain"));
    assert!(!stdout.contains("--format"));
}

#[rstest]
#[case(&["--version"])]
#[case(&["--output", "rust-string", "--version"])]
fn prints_version_without_requiring_input(#[case] arguments: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(arguments)
        .env("SAUVA_CONFIG_FILE", "")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        concat!("sauva ", env!("CARGO_PKG_VERSION"), "\n").as_bytes()
    );
}

#[test]
fn prints_default_config_without_loading_local_config() {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .arg("--print-default-config")
        .env("SAUVA_CONFIG_FILE", "")
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let config: toml::Table = toml::from_str(&stdout).unwrap();
    assert!(!config.is_empty());
}

#[rstest]
#[case("json-string", r#""A\u0301\uD83D\uDE00""#)]
#[case("rust-string", r#""A\u{301}\u{1F600}""#)]
fn prints_string_literals_without_loading_config(#[case] format: &str, #[case] expected: &str) {
    let mut config = tempfile::NamedTempFile::new().unwrap();
    config
        .write_all(
            r#"[glyph_preview
not valid TOML
"#
            .as_bytes(),
        )
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", format, "--text", "A\u{301}😀"])
        .env("SAUVA_CONFIG_FILE", config.path())
        .env("XDG_DATA_HOME", config.path())
        .env("TERM", "dumb")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
}

#[rstest]
#[case(&["41"], r#""A""#, r#""A""#)]
#[case(&["1"], r#""1""#, r#""1""#)]
#[case(&["--text", "41"], r#""41""#, r#""41""#)]
#[case(&["U+2192"], r#""\u2192""#, r#""\u{2192}""#)]
#[case(&["--text", "U+2192"], r#""U+2192""#, r#""U+2192""#)]
#[case(&["--text", r"\u0041"], r#""\\u0041""#, r#""\\u0041""#)]
#[case(&["U+002D"], r#""-""#, r#""-""#)]
#[case(&["input.txt"], r#""input.txt""#, r#""input.txt""#)]
fn preserves_existing_argument_interpretation(
    #[case] arguments: &[&str],
    #[case] json: &str,
    #[case] rust: &str,
) {
    for (format, expected) in [("json-string", json), ("rust-string", rust)] {
        let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
            .args(["--output", format])
            .args(arguments)
            .output()
            .unwrap();

        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
    }
}

#[rstest]
#[case(&["-"], b"41", r#""A""#, r#""A""#)]
#[case(&["--text", "-"], b"41", r#""41""#, r#""41""#)]
#[case(&["--text", "-"], b"U+D800", r#""U+D800""#, r#""U+D800""#)]
#[case(&["-"], b"-", r#""-""#, r#""-""#)]
#[case(&["-"], "\u{feff} A\u{301}\r\n😀\0\n".as_bytes(), r#""\uFEFF A\u0301\r\n\uD83D\uDE00\u0000\n""#, r#""\u{FEFF} A\u{301}\r\n\u{1F600}\0\n""#)]
#[case(&["--text", "-"], "\u{feff} A\u{301}\r\n😀\0\n".as_bytes(), r#""\uFEFF A\u0301\r\n\uD83D\uDE00\u0000\n""#, r#""\u{FEFF} A\u{301}\r\n\u{1F600}\0\n""#)]
#[case(&["--text", "-"], "\u{378}\u{e000}\u{ffff}".as_bytes(), r#""\u0378\uE000\uFFFF""#, r#""\u{378}\u{E000}\u{FFFF}""#)]
fn preserves_stdin_interpretation_and_contents(
    #[case] arguments: &[&str],
    #[case] input: &[u8],
    #[case] json: &str,
    #[case] rust: &str,
) {
    for (format, expected) in [("json-string", json), ("rust-string", rust)] {
        let mut command = vec!["--output", format];
        command.extend_from_slice(arguments);
        let output = run_with_stdin(&command, input);

        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
    }
}

#[rstest]
#[case("json-string", r"\u3042")]
#[case("rust-string", r"\u{3042}")]
fn prints_long_inputs_in_full(#[case] format: &str, #[case] escaped: &str) {
    let count = 32 * 1024;
    let output = run_with_stdin(
        &["--output", format, "--text", "-"],
        "あ".repeat(count).as_bytes(),
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        format!("\"{}\"\n", escaped.repeat(count)).as_bytes()
    );
}

#[rstest]
#[case(&["-"], b"", 1, "invalid input from stdin")]
#[case(&["--text", "-"], b"", 1, "text must not be empty")]
#[case(&["-"], b"U+2192\n", 1, "invalid input from stdin")]
#[case(&["-"], &[0xff], 1, "failed to read stdin")]
#[case(&["--text", "-"], &[0xff], 1, "failed to read stdin")]
#[case(&["--output", "json-string", "-"], b"", 2, "invalid input from stdin")]
#[case(&["--output", "rust-string", "--text", "-"], b"", 2, "text must not be empty")]
#[case(&["--output", "json-string", "-"], b"U+2192\n", 2, "invalid input from stdin")]
#[case(&["--output", "rust-string", "-"], &[0xff], 1, "failed to read stdin")]
#[case(&["--output", "json-string", "--text", "-"], &[0xff], 1, "failed to read stdin")]
#[case(&["--output", "json-string", "-"], b"U+D800", 1, "not a Unicode scalar value")]
fn rejects_invalid_stdin_without_printing_a_result(
    #[case] arguments: &[&str],
    #[case] input: &[u8],
    #[case] expected_code: i32,
    #[case] expected_error: &str,
) {
    let output = run_with_stdin(arguments, input);

    assert_failure(&output, expected_code, expected_error);
}

fn run_with_stdin(arguments: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(input).unwrap();
    drop(stdin);
    child.wait_with_output().unwrap()
}

#[rstest]
#[case("json-string")]
#[case("rust-string")]
fn treats_a_closed_stdout_pipe_as_success(#[case] format: &str) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", format, "--text", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Close the reader before allowing stdin to reach EOF and output to start.
    drop(child.stdout.take().unwrap());
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"A").unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn reports_stdout_write_failures() {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", "json-string", "A"])
        .stdout(
            std::fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .output()
        .unwrap();

    assert_failure(&output, 1, "failed to write stdout");
}

fn assert_failure(output: &Output, expected_code: i32, expected_error: &str) {
    assert_eq!(output.status.code(), Some(expected_code));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(stderr.starts_with("sauva: "), "{stderr}");
    assert!(stderr.contains(expected_error), "{stderr}");
}

#[rstest]
#[case(&["--output", "json-string"], 2, "required")]
#[case(&["--output", "json-string", "--text", ""], 2, "text must not be empty")]
#[case(&["--output", "rust-string", ""], 2, "invalid value")]
#[case(&["--output", "json-string", "U+110000"], 2, "between")]
#[case(&["--output", "json-string", "A", "--graphics", "auto"], 2, "cannot be used")]
#[case(&["--output", "json", "A"], 2, "invalid value")]
#[case(&["--output", "rust-string", "U+D800"], 1, "not a Unicode scalar value")]
fn rejects_invalid_output_requests(
    #[case] arguments: &[&str],
    #[case] expected_code: i32,
    #[case] expected_error: &str,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(arguments)
        .output()
        .unwrap();

    assert_failure(&output, expected_code, expected_error);
}

#[rstest]
#[case(&["--output", "json-string", "U+\u{1b}[31m\n"])]
#[case(&["--output", "\u{202e}bad\r\n"])]
#[case(&["--output", "json-string", "--bad\u{1b}[31m\t"])]
fn escapes_untrusted_arguments_in_diagnostics(#[case] arguments: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(arguments)
        .output()
        .unwrap();

    assert_failure(&output, 2, "error:");
    assert!(
        output
            .stderr
            .iter()
            .all(|byte| *byte == b'\n' || (b' '..=b'~').contains(byte))
    );
    assert!(String::from_utf8(output.stderr).unwrap().contains(r"\u{"));
}
