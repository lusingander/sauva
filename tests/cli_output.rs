use std::{
    io::Write,
    process::{Command, Output, Stdio},
    sync::LazyLock,
};

use rstest::rstest;
use serde_json::Value;

#[rstest]
#[case::fraction("json_fraction", &["U+00BD"])]
#[case::sequence("json_sequence", &["--text", "A\u{301}😀"])]
#[case::surrogate("json_surrogate", &["U+D800"])]
#[case::unassigned("json_unassigned", &["U+2FE0"])]
#[case::control("json_control", &["U+0000"])]
fn prints_complete_json_report_snapshots(#[case] name: &str, #[case] arguments: &[&str]) {
    static SCHEMA: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
        let schema: Value = serde_json::from_str(include_str!("../output.schema.json"))
            .expect("output.schema.json must contain valid JSON");
        jsonschema::validator_for(&schema).expect("output.schema.json must be a valid schema")
    });
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", "json"])
        .args(arguments)
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_ascii());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: Value = serde_json::from_str(&stdout).unwrap();
    SCHEMA.validate(&report).unwrap();
    assert!(stdout.ends_with("}\n"));
    assert!(!stdout.ends_with("\n\n"));
    // Keep snapshots useful across releases without pinning the binary version.
    let snapshot = stdout.replacen(
        &format!(r#""sauva_version": "{}""#, env!("CARGO_PKG_VERSION")),
        r#""sauva_version": "<version>""#,
        1,
    );
    insta::assert_snapshot!(name, snapshot);
}

#[test]
fn json_stdin_round_trips_controls_and_literal_escapes_without_normalizing() {
    let source = "\u{feff} A\u{301}\r\n👩‍💻\0\u{1b}[31m\u{202e}\"\\u0041\n";
    let output = run_with_stdin(&["--output", "json", "--text", "-"], source.as_bytes());
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_ascii());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["text"], source);
    assert_ne!(report["normalization"]["nfc"]["text"], source);
}

#[test]
fn prints_long_json_reports_without_truncating_occurrences_or_normalized_text() {
    let count = 4096;
    let source = "あ".repeat(count);
    let output = run_with_stdin(&["--output", "json", "--text", "-"], source.as_bytes());
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["text"], source);
    assert_eq!(report["code_points"].as_array().unwrap().len(), count);
    assert_eq!(report["graphemes"].as_array().unwrap().len(), count);
    assert_eq!(report["code_points"][count - 1]["index"], count - 1);
    for form in ["nfc", "nfd", "nfkc", "nfkd"] {
        assert_eq!(report["normalization"][form]["text"], source);
    }
}

#[rstest]
#[case("--help")]
#[case("--version")]
fn informational_options_bypass_input_and_configuration(#[case] option: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", "json-string", option])
        .env("SAUVA_CONFIG_FILE", "")
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());
    assert!(!output.stdout.is_empty());
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

#[test]
fn prints_output_without_loading_config() {
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
        .args(["--output", "json", "A"])
        .env("SAUVA_CONFIG_FILE", config.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());
    assert!(!output.stdout.is_empty());
}

#[rstest]
#[case(&["41"], r#""A""#, r#""A""#)]
#[case(&["--text", "41"], r#""41""#, r#""41""#)]
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
#[case(&["--text", "-"], "\u{feff} A\u{301}\r\n😀\0\n".as_bytes(), r#""\uFEFF A\u0301\r\n\uD83D\uDE00\u0000\n""#, r#""\u{FEFF} A\u{301}\r\n\u{1F600}\0\n""#)]
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
#[case::tui_empty_input(&["-"], b"", 1)]
#[case::tui_empty_text(&["--text", "-"], b"", 1)]
#[case::tui_invalid_notation(&["-"], b"U+2192\n", 1)]
#[case::tui_invalid_utf8(&["-"], &[0xff], 1)]
#[case::tui_invalid_literal_utf8(&["--text", "-"], &[0xff], 1)]
#[case::empty_input(&["--output", "json-string", "-"], b"", 2)]
#[case::empty_text(&["--output", "rust-string", "--text", "-"], b"", 2)]
#[case::invalid_notation(&["--output", "json-string", "-"], b"U+2192\n", 2)]
#[case::invalid_utf8(&["--output", "rust-string", "-"], &[0xff], 1)]
#[case::invalid_literal_utf8(&["--output", "json-string", "--text", "-"], &[0xff], 1)]
#[case::surrogate(&["--output", "json-string", "-"], b"U+D800", 1)]
fn rejects_invalid_stdin_without_printing_a_result(
    #[case] arguments: &[&str],
    #[case] input: &[u8],
    #[case] expected_code: i32,
) {
    let output = run_with_stdin(arguments, input);

    assert_failure(&output, expected_code);
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

#[test]
fn treats_a_closed_stdout_pipe_as_success() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", "json", "--text", "-"])
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
        .args(["--output", "json", "A"])
        .stdout(
            std::fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .output()
        .unwrap();

    assert_failure(&output, 1);
}

fn assert_failure(output: &Output, expected_code: i32) {
    assert_eq!(output.status.code(), Some(expected_code));
    assert!(output.stdout.is_empty());

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.starts_with("sauva: "), "{stderr}");
}

#[rstest]
#[case::missing_input(&["--output", "json-string"], 2)]
#[case::empty_text(&["--output", "json-string", "--text", ""], 2)]
#[case::empty_input(&["--output", "rust-string", ""], 2)]
#[case::out_of_range(&["--output", "json-string", "U+110000"], 2)]
#[case::graphics_conflict(&["--output", "json-string", "A", "--graphics", "auto"], 2)]
#[case::unknown_format(&["--output", "unknown", "A"], 2)]
#[case::surrogate(&["--output", "rust-string", "U+D800"], 1)]
fn rejects_invalid_output_requests(#[case] arguments: &[&str], #[case] expected_code: i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(arguments)
        .output()
        .unwrap();

    assert_failure(&output, expected_code);
}

#[rstest]
#[case(&["--output", "json-string", "U+\u{1b}[31m\n"], None)]
#[case(&["--output", "\u{202e}bad\r\n"], None)]
#[case(&["--output", "json-string", "--bad\u{1b}[31m\t"], Some("-- --bad"))]
#[case(&["--bad\u{1b}[31m\t"], Some("-- --bad"))]
fn escapes_untrusted_arguments_in_diagnostics(
    #[case] arguments: &[&str],
    #[case] suggestion: Option<&str>,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(arguments)
        .output()
        .unwrap();

    assert_failure(&output, 2);
    assert!(
        output
            .stderr
            .iter()
            .all(|byte| *byte == b'\n' || (b' '..=b'~').contains(byte))
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(r"\u{"));
    if let Some(suggestion) = suggestion {
        assert!(stderr.contains(suggestion), "{stderr}");
    }
}

#[cfg(unix)]
#[test]
fn rejects_invalid_utf8_arguments_with_usage_exit_code() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", "json", "--text"])
        .arg(OsString::from_vec(vec![0xff]))
        .output()
        .unwrap();
    assert_failure(&output, 2);
}
