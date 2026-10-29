use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

use rstest::rstest;

#[test]
fn prints_usage_and_command_name_for_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("sauva"));
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
    let output = Command::new(env!("CARGO_BIN_EXE_sauva"))
        .args(["--output", format, "--text", "A\u{301}😀"])
        .env("SAUVA_CONFIG_FILE", "")
        .output()
        .unwrap();

    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
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
