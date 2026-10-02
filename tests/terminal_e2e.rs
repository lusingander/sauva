use std::fs;

use tempfile::tempdir;
use termlens::{Color, Key, Terminal};

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
    let help = terminal.snapshot_after(|screen| {
        screen.contains("sauva / Help")
            && screen.contains("Keybindings · Inspector")
            && screen.contains("x  Ctrl-c                Quit")
            && screen.contains("F2                       Open or close help")
    })?;
    let help = help
        .mask_matching(env!("CARGO_PKG_REPOSITORY"), '▒')
        .mask_matching(env!("CARGO_PKG_DESCRIPTION"), '▒')
        .mask_matching(env!("CARGO_PKG_NAME"), '▒')
        .mask_matching(env!("CARGO_PKG_VERSION"), '▒');
    insta::assert_snapshot!(help.with_styles());
    terminal.send(Key::F(2))?;
    terminal.wait_until(|screen| {
        screen.contains("Identity") && screen.contains("LATIN CAPITAL LETTER A")
    })?;

    terminal.send(Key::Char('q'))?;
    terminal.send(Key::F(2))?;
    terminal.wait_until(|screen| {
        screen.contains("sauva / Help") && screen.contains("Keybindings · Inspector")
    })?;
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
fn inspects_a_text_sequence_and_preserves_its_position_across_search() -> termlens::Result<()> {
    let mut terminal = termlens::bin!("sauva", size(100, 30), args(["A→B", "--graphics", "off"]))?;
    terminal.snapshot_after(|screen| {
        screen.contains("3 code points")
            && screen.contains("U+0041")
            && screen.contains("U+2192")
            && screen.contains("U+0042")
    })?;

    terminal.send(Key::Down)?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence 2/3 / Inspector")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;

    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| screen.contains("Search by name or alias"))?;
    terminal.send_str("Ω")?;
    terminal.wait_until(|screen| {
        screen.contains("U+03A9") && screen.contains("GREEK CAPITAL LETTER OMEGA")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence 2/3 / Inspector")
            && screen.contains("U+03A9")
            && screen.contains("GREEK CAPITAL LETTER OMEGA")
    })?;

    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| screen.contains("3 code points"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence 2/3 / Inspector")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn sequence_clusters_keep_the_cursor_and_highlight_in_the_row_body() -> termlens::Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("config.toml");
    fs::write(&path, "[ui]\nselection_cursor = \"▸\"\n")?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        env("SAUVA_CONFIG_FILE", &path),
        args(["--text", "🇯🇵X\r\n👨‍🔧", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| {
        screen.contains("8 code points · 4 graphemes") && screen.contains("1/4 · member 1/2")
    })?;

    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("2/8 · U+1F1F5") && screen.contains("1/4 · member 2/2")
    })?;
    assert_eq!(screen.cell(4, 4).unwrap().contents(), "└");
    assert_eq!(screen.cell(4, 4).unwrap().style().bg, Color::Default);
    assert_eq!(screen.cell(4, 4).unwrap().style().fg, Color::Indexed(8));
    assert_eq!(screen.cell(4, 6).unwrap().contents(), "▸");
    assert_eq!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    assert_eq!(screen.cell(4, 57).unwrap().style().bg, Color::Indexed(6));
    insta::assert_snapshot!(screen.with_styles());

    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence 2/8 / Inspector") && screen.contains("U+1F1F5")
    })?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| {
        screen.contains("2/8 · U+1F1F5") && screen.contains("1/4 · member 2/2")
    })?;

    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("3/8 · U+0058") && screen.contains("2/4 · member 1/1")
    })?;
    assert_eq!(screen.cell(5, 4).unwrap().contents(), "•");

    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| {
        screen.contains("4/8 · U+000D") && screen.contains("3/4 · member 1/2")
    })?;
    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("5/8 · U+000A") && screen.contains("3/4 · member 2/2")
    })?;
    assert!(screen.row_text(6).contains("<CONTROL>"));
    assert!(screen.row_text(7).contains("<CONTROL>"));
    assert_eq!(screen.cell(6, 4).unwrap().contents(), "┌");
    assert_eq!(screen.cell(7, 4).unwrap().contents(), "└");

    for _ in 0..2 {
        terminal.send(Key::Down)?;
    }
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("7/8 · U+200D") && screen.contains("4/4 · member 2/3")
    })?;
    assert_eq!(screen.cell(9, 4).unwrap().contents(), "│");
    assert_eq!(screen.cell(9, 4).unwrap().style().bg, Color::Default);
    assert_eq!(screen.cell(9, 6).unwrap().contents(), "▸");
    assert_eq!(screen.cell(9, 6).unwrap().style().bg, Color::Indexed(6));

    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence 7/8 / Inspector") && screen.contains("U+200D")
    })?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| screen.contains("4/4 · member 2/3"))?;
    terminal.resize(60, 16)?;
    terminal.wait_until(|screen| {
        screen.contains("7/8 · U+200D") && screen.contains("8 code points · 4 graphemes")
    })?;
    terminal.resize(100, 30)?;
    terminal.wait_until(|screen| {
        screen.contains("7/8 · U+200D") && screen.contains("4/4 · member 2/3")
    })?;

    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn searches_for_a_name_and_opens_the_result_in_the_inspector() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| {
        screen.contains("Search")
            && screen.contains("Search by name or alias, code point, or character")
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
fn searches_for_a_name_alias_and_explains_the_match() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| screen.contains("Search by name or alias"))?;
    terminal.send_str("latin capital letter gha")?;
    terminal.wait_until(|screen| {
        screen.contains("U+01A2")
            && screen.contains("LATIN CAPITAL LETTER GHA")
            && screen.contains("Matched Alias")
            && screen.contains("correction")
            && screen.contains("LATIN CAPITAL LETTER OI")
    })?;

    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Identity")
            && screen.contains("U+01A2")
            && screen.contains("LATIN CAPITAL LETTER OI")
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
    terminal.wait_until(|screen| screen.contains("Search by name or alias"))?;
    terminal.send_str("rightwards arrox")?;
    terminal.snapshot_after(|screen| {
        screen.contains("rightwards arrox") && screen.contains("No matching characters")
    })?;

    terminal.send(Key::Esc)?;
    terminal.snapshot_after(|screen| {
        screen.contains("Identity") && screen.contains("LATIN CAPITAL LETTER A")
    })?;
    terminal.send(Key::Char('/'))?;
    let reopened = terminal.snapshot_after(|screen| {
        screen.contains("Search")
            && screen.contains("rightwards arrox")
            && screen.contains("No matching characters")
    })?;
    insta::assert_snapshot!(reopened.with_styles());

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
fn keeps_the_search_result_selected_across_terminal_resizes() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| screen.contains("Search by name or alias"))?;
    terminal.send_str("U+2192")?;
    terminal.snapshot_after(|screen| {
        screen.contains("Search")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;

    terminal.resize(60, 16)?;
    let minimum = terminal.snapshot_after(|screen| {
        screen.size() == (60, 16)
            && screen.contains("Search")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;
    insta::assert_snapshot!(minimum.with_styles());

    terminal.resize(100, 30)?;
    terminal.snapshot_after(|screen| {
        screen.size() == (100, 30)
            && screen.contains("Selection")
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
fn search_help_does_not_edit_the_query_or_lose_the_result() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    terminal.send(Key::Char('/'))?;
    terminal.wait_until(|screen| screen.contains("Search by name or alias"))?;
    terminal.send_str("U+2192")?;
    terminal
        .wait_until(|screen| screen.contains("1 result") && screen.contains("RIGHTWARDS ARROW"))?;

    terminal.send(Key::F(1))?;
    terminal.snapshot_after(|screen| {
        screen.contains("sauva / Help")
            && screen.contains("Keybindings · Search")
            && screen.contains("Inspect the selected result")
    })?;
    terminal.send(Key::Char('x'))?;
    terminal.send(Key::F(1))?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("Search")
            && screen.contains("1 result")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;
    assert!(!screen.contains("U+2192x"), "{screen}");

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
        screen.contains("Browse / Planes") && screen.contains("Basic Multilingual Plane")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Browse / Ranges")
            && screen.contains("Plane 0")
            && screen.contains("U+0000–U+00FF")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Browse / Code Points") && screen.contains("Selection")
    })?;
    terminal.send(Key::Right)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("Browse / Code Points") && screen.contains("U+0042")
    })?;
    insta::assert_snapshot!(screen.with_styles());

    terminal.send(Key::Enter)?;
    let inspector = terminal.snapshot_after(|screen| {
        screen.contains("Identity")
            && screen.contains("U+0042")
            && screen.contains("LATIN CAPITAL LETTER B")
    })?;
    insta::assert_snapshot!("inspector_after_browse_selection", inspector.with_styles());

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
        screen.contains("Browse / Blocks") && screen.contains("Basic Latin")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("Block Code Points"))?;
    terminal.send(Key::Right)?;
    terminal.wait_until(|screen| screen.contains("U+0042"))?;

    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| screen.contains("Browse / Blocks"))?;
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
    terminal.wait_until(|screen| screen.contains("Browse / Code Points"))?;
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
