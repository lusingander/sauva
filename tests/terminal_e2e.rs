use std::fs;

use tempfile::tempdir;
use termlens::{Key, Terminal};

fn spawn(graphics: &str) -> termlens::Result<Terminal> {
    termlens::bin!("sauva", size(100, 30), args(["--graphics", graphics]))
}

#[test]
fn custom_keybindings_appear_in_help_and_drive_the_app() -> termlens::Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("config.toml");
    fs::write(
        &path,
        r#"
            [keybindings.global]
            help = ["f2"]

            [keybindings.inspector]
            quit = ["x"]
        "#,
    )?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        env("SAUVA_CONFIG_FILE", &path),
        args(["--graphics", "off"])
    )?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::F(2))?;
    terminal.snapshot_after(|screen| {
        screen.contains("Help · Inspector") && screen.contains("<x>") && screen.contains("<F2>")
    })?;
    terminal.send(Key::F(2))?;
    terminal.wait_until(|screen| {
        screen.contains("Identity") && screen.contains("LATIN CAPITAL LETTER A")
    })?;

    terminal.send(Key::Char('q'))?;
    terminal.send(Key::F(2))?;
    terminal.wait_until(|screen| screen.contains("Help · Inspector"))?;
    terminal.send(Key::F(2))?;
    terminal.wait_until(|screen| screen.contains("Identity"))?;
    terminal.send(Key::Char('x'))?;
    let status = terminal.wait_exit()?;

    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn starts_and_restores_the_terminal_on_quit() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    let screen = terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    assert!(screen.alternate_screen(), "{screen}");
    assert!(screen.contains("Graphics disabled"), "{screen}");

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;

    assert!(status.success(), "exit status: {status}");
    assert!(!terminal.screen().alternate_screen());
    assert!(terminal.screen().cursor_visible());
    Ok(())
}

#[test]
fn starts_at_the_code_point_from_the_command_line() -> termlens::Result<()> {
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        args(["U+2192", "--graphics", "off"])
    )?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("U+2192") && screen.contains("RIGHTWARDS ARROW")
    })?;

    assert!(screen.contains("Identity"), "{screen}");

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn searches_for_a_name_and_opens_the_result_in_the_inspector() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| {
        screen.contains("Search")
            && screen.contains("Search by Unicode name, code point, or character")
    })?;
    terminal.send_str("rightwards arrow")?;
    terminal.wait_until(|screen| {
        screen.contains("rightwards arrow")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Identity")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn reopens_search_and_corrects_a_query_with_no_results() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| screen.contains("Search by Unicode name"))?;
    terminal.send_str("rightwards arrox")?;
    terminal.snapshot_after(|screen| {
        screen.contains("rightwards arrox") && screen.contains("No matching characters")
    })?;

    terminal.send(Key::Esc)?;
    terminal.snapshot_after(|screen| {
        screen.contains("Identity") && screen.contains("LATIN CAPITAL LETTER A")
    })?;
    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| {
        screen.contains("Search")
            && screen.contains("rightwards arrox")
            && screen.contains("No matching characters")
    })?;

    terminal.send(Key::Backspace)?;
    terminal.send(Key::Char('w'))?;
    terminal.snapshot_after(|screen| {
        screen.contains("rightwards arrow")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;
    terminal.send(Key::Enter)?;
    terminal.snapshot_after(|screen| {
        screen.contains("Identity")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn browses_through_planes_and_ranges_to_inspect_a_code_point() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('p'))?;
    terminal.wait_until(|screen| {
        screen.contains("Browse · Planes") && screen.contains("Basic Multilingual Plane")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Browse · Ranges · Plane 0") && screen.contains("U+0000–U+00FF")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Browse · Code Points") && screen.contains("Selection Preview")
    })?;
    terminal.send(Key::Right)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("Browse · Code Points") && screen.contains("U+0042")
    })?;
    insta::assert_snapshot!(screen.with_styles());

    terminal.send(Key::Enter)?;
    terminal.snapshot_after(|screen| {
        screen.contains("Identity")
            && screen.contains("U+0042")
            && screen.contains("LATIN CAPITAL LETTER B")
    })?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn backs_out_of_a_block_table_without_changing_the_inspected_code_point() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('b'))?;
    terminal.wait_until(|screen| {
        screen.contains("Browse · Blocks") && screen.contains("Basic Latin")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("Browse · Block Code Points"))?;
    terminal.send(Key::Right)?;
    terminal.wait_until(|screen| screen.contains("U+0042"))?;

    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| screen.contains("Browse · Blocks"))?;
    terminal.send(Key::Esc)?;
    terminal.snapshot_after(|screen| {
        screen.contains("Identity")
            && screen.contains("U+0041")
            && screen.contains("LATIN CAPITAL LETTER A")
    })?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn cancels_direct_code_point_browsing_without_changing_the_inspector() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('c'))?;
    terminal.wait_until(|screen| screen.contains("Browse · Code Points"))?;
    terminal.send(Key::Right)?;
    terminal.wait_until(|screen| screen.contains("U+0042"))?;
    terminal.send(Key::Esc)?;
    terminal.snapshot_after(|screen| {
        screen.contains("Identity")
            && screen.contains("U+0041")
            && screen.contains("LATIN CAPITAL LETTER A")
    })?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn redraws_across_the_minimum_terminal_size_boundary() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.resize(59, 15)?;
    terminal.snapshot_after(|screen| {
        screen.size() == (59, 15) && screen.contains("Terminal too small")
    })?;

    terminal.resize(60, 16)?;
    terminal.snapshot_after(|screen| {
        screen.size() == (60, 16) && screen.contains("LATIN CAPITAL LETTER A")
    })?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
#[cfg_attr(
    windows,
    ignore = "ConPTY does not preserve Kitty graphics transmissions"
)]
fn transmits_and_deletes_the_kitty_glyph_preview() -> termlens::Result<()> {
    let mut terminal = spawn("force")?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("Font") && screen.contains("Regular") && screen.graphics().kitty() == 1
    })?;

    assert_eq!(screen.graphics().kitty(), 1, "{screen}");

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;

    assert!(status.success(), "exit status: {status}");
    assert_eq!(terminal.screen().graphics().deletes(), 1);
    Ok(())
}
