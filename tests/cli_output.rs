use std::process::Command;

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
