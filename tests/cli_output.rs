use std::{
    io::Write,
    process::{Command, Stdio},
};

use rstest::rstest;

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
    assert!(stdout.starts_with("[color]\n"));
    assert!(stdout.contains("[glyph_preview]\n"));
    assert!(stdout.contains("[ui]\n"));
    assert!(stdout.contains("[keybindings.inspector]\n"));
}

#[rstest]
#[case(&["-"], b"", "invalid input from stdin")]
#[case(&["--text", "-"], b"", "text must not be empty")]
#[case(&["-"], b"U+2192\n", "invalid input from stdin")]
#[case(&["-"], &[0xff], "failed to read stdin")]
#[case(&["--text", "-"], &[0xff], "failed to read stdin")]
fn rejects_invalid_stdin_before_starting_the_terminal(
    #[case] arguments: &[&str],
    #[case] input: &[u8],
    #[case] expected_error: &str,
) {
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
    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success(), "status: {}", output.status);
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("sauva: "), "{stderr}");
    assert!(stderr.contains(expected_error), "{stderr}");
}
