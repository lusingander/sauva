use std::fs;

use rstest::rstest;
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
fn inspector_group_keys_show_section_headings_and_select_their_first_properties()
-> termlens::Result<()> {
    let mut terminal =
        termlens::bin!("sauva", size(60, 16), args(["U+D800", "--graphics", "off"]))?;
    terminal.wait_until(|screen| screen.row_text(2).trim_start().starts_with("Identity"))?;
    terminal.send(Key::Char('j'))?;
    terminal.send(Key::Char(']'))?;
    let screen = terminal.snapshot_after(|screen| {
        screen
            .row_text(2)
            .trim_start()
            .starts_with("Classification")
    })?;
    assert!(screen.row_text(3).contains("General Category"), "{screen}");
    assert_eq!(screen.cell(3, 3).unwrap().style().bg, Color::Indexed(6));

    terminal.send(Key::Char('j'))?;
    terminal.send(Key::Char('['))?;
    let screen = terminal
        .snapshot_after(|screen| screen.row_text(2).trim_start().starts_with("Identity"))?;
    assert!(screen.row_text(3).contains("Character"), "{screen}");
    assert_eq!(screen.cell(3, 3).unwrap().style().bg, Color::Indexed(6));

    terminal.send(Key::Char(']'))?;
    terminal.send(Key::Char(']'))?;
    let screen = terminal
        .snapshot_after(|screen| screen.row_text(2).trim_start().starts_with("Encoding"))?;
    assert!(screen.row_text(3).contains("UTF-8"), "{screen}");
    assert!(screen.contains("Not available"), "{screen}");
    assert_eq!(screen.cell(3, 3).unwrap().style().bg, Color::Indexed(6));
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn sequence_group_keys_skip_cluster_members_and_preserve_selection_across_inspection()
-> termlens::Result<()> {
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        args(["A\u{0301}👩‍💻B", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("1/6 · U+0041"))?;
    terminal.send(Key::Char('j'))?;
    terminal.wait_until(|screen| screen.contains("2/6 · U+0301"))?;
    terminal.send(Key::Char(']'))?;
    terminal.wait_until(|screen| screen.contains("3/6 · U+1F469"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("Sequence 3/6 / Inspector"))?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| screen.contains("3/6 · U+1F469"))?;
    terminal.send(Key::Char(']'))?;
    terminal.wait_until(|screen| screen.contains("6/6 · U+0042"))?;
    terminal.send(Key::Char('['))?;
    terminal.wait_until(|screen| screen.contains("3/6 · U+1F469"))?;
    terminal.send(Key::Char('j'))?;
    terminal.wait_until(|screen| screen.contains("4/6 · U+200D"))?;
    terminal.send(Key::Char('['))?;
    terminal.wait_until(|screen| screen.contains("1/6 · U+0041"))?;
    terminal.send(Key::F(1))?;
    terminal
        .wait_until(|screen| screen.contains("Select the first code point of the next grapheme"))?;
    terminal.send(Key::F(1))?;
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn normalization_group_keys_follow_result_clusters_and_update_the_original_highlight()
-> termlens::Result<()> {
    let mut terminal = termlens::bin!("sauva", size(100, 30), args(["ﬃ👩‍💻B", "--graphics", "off"]))?;
    terminal.wait_until(|screen| screen.contains("5 code points"))?;
    terminal.send(Key::Char('n'))?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| screen.contains("NFKC Result"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("1/7 · U+0066"))?;
    for position in ["2/7 · U+0066", "3/7 · U+0069"] {
        terminal.send(Key::Char(']'))?;
        let screen = terminal.snapshot_after(|screen| screen.contains(position))?;
        assert_eq!(screen.cell(3, 6).unwrap().style().bg, Color::Indexed(6));
    }
    terminal.send(Key::Char(']'))?;
    let screen = terminal.snapshot_after(|screen| screen.contains("4/7 · U+1F469"))?;
    assert_ne!(screen.cell(3, 6).unwrap().style().bg, Color::Indexed(6));
    assert_eq!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    terminal.send(Key::Char('j'))?;
    terminal.wait_until(|screen| screen.contains("5/7 · U+200D"))?;
    terminal.send(Key::Char('['))?;
    let screen = terminal.snapshot_after(|screen| screen.contains("3/7 · U+0069"))?;
    assert_eq!(screen.cell(3, 6).unwrap().style().bg, Color::Indexed(6));
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| {
        screen.contains("Keybindings · Normalization Result")
            && screen.contains("Select the first code point of the next grapheme")
    })?;
    terminal.send(Key::F(1))?;
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
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

#[rstest]
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[case(
    r#"printf 'U+2192' | "$SAUVA_BIN" - --graphics off"#,
    &["Identity", "U+2192", "RIGHTWARDS ARROW"]
)]
#[case(
    r#"printf '41' | "$SAUVA_BIN" --text - --graphics off"#,
    &["2 code points", "U+0034", "U+0031"]
)]
#[case(
    r#"printf 'A\nB\n' | "$SAUVA_BIN" - --graphics off"#,
    &["4 code points", "U+0041", "U+000A", "U+0042"]
)]
fn starts_from_piped_stdin_and_still_accepts_keys(
    #[case] command: &str,
    #[case] expected: &[&str],
) -> termlens::Result<()> {
    let mut terminal = Terminal::builder()
        .size(100, 30)
        .env_clear()
        .env("SAUVA_BIN", env!("CARGO_BIN_EXE_sauva"))
        .args(["-c", command])
        .spawn("/bin/sh")?;
    terminal.snapshot_after(|screen| expected.iter().all(|text| screen.contains(text)))?;

    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;

    assert!(status.success(), "exit status: {status}");
    assert!(!terminal.screen().alternate_screen());
    assert!(terminal.screen().cursor_visible());
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
fn normalization_comparison_highlights_changes_and_preserves_navigation() -> termlens::Result<()> {
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        args(["A\u{0301} ①👩‍💻", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("7 code points"))?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Char('n'))?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("NFC Result") && screen.contains("1 changed region")
    })?;
    assert!(screen.contains("7 CP · 18 Bytes · 4 GC"), "{screen}");
    assert_eq!(screen.cell(3, 2).unwrap().style().bg, Color::Indexed(3));
    assert_eq!(screen.cell(4, 2).unwrap().style().bg, Color::Indexed(3));
    assert_eq!(screen.cell(7, 1).unwrap().style().bg, Color::Indexed(6));
    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| {
        screen.contains("NFD Result") && screen.contains("Same as Original")
    })?;
    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("NFKC Result") && screen.contains("2 changed regions")
    })?;
    // vt100 measures emoji scalars separately, so this snapshot is for layout
    // and styles. TestBackend snapshots verify the complete ZWJ cluster.
    insta::assert_snapshot!(screen.with_styles());
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence / NFKC Result") && screen.contains("6 code points")
    })?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("NFKC Result 3/6 / Inspector") && screen.contains("DIGIT ONE")
    })?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| screen.contains("Sequence / NFKC Result"))?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence / Normalization") && screen.contains("NFKC Result")
    })?;
    terminal.send(Key::Enter)?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("NFKC Result 3/6 / Inspector"))?;
    terminal.send(Key::Backspace)?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("Sequence / Normalization"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("7 code points"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence 2/7 / Inspector") && screen.contains("COMBINING ACUTE ACCENT")
    })?;
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn normalization_fits_the_minimum_terminal_and_opens_single_point_results() -> termlens::Result<()>
{
    let mut terminal = termlens::bin!(
        "sauva",
        size(60, 16),
        args(["A\u{0301}", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("2 code points"))?;
    terminal.send(Key::Char('n'))?;
    let screen = terminal
        .snapshot_after(|screen| screen.contains("NFC Result") && screen.contains("U+00C1"))?;
    assert!(screen.contains("NFKD"), "{screen}");
    assert!(screen.contains("2 CP · 3 Bytes · 1 GC"), "{screen}");
    insta::assert_snapshot!(screen.with_styles());
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence / NFC Result") && screen.contains("1 CP · 1 GC")
    })?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("NFC Result 1/1 / Inspector"))?;
    terminal.send(Key::Backspace)?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("Sequence / Normalization"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("2 code points"))?;
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn normalization_respects_custom_keys_colors_and_contextual_help() -> termlens::Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("config.toml");
    fs::write(
        &path,
        r#"
        [color.difference]
        fg = "white"
        bg = "magenta"
        [keybindings.global]
        help = ["f2"]
        [keybindings.sequence]
        normalize = ["a"]
        [keybindings.normalization]
        move_down = ["l"]
        activate = ["i"]
        [keybindings.normalization_result]
        back = ["b"]
    "#,
    )?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        env("SAUVA_CONFIG_FILE", &path),
        args(["A\u{0301} ①", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("4 code points"))?;
    terminal.send(Key::Char('a'))?;
    let screen = terminal.snapshot_after(|screen| screen.contains("NFC Result"))?;
    assert_eq!(screen.cell(3, 2).unwrap().style().bg, Color::Indexed(5));
    assert_eq!(screen.cell(3, 2).unwrap().style().fg, Color::Indexed(15));
    terminal.send(Key::Char('l'))?;
    terminal.wait_until(|screen| screen.contains("NFD Result"))?;
    terminal.send(Key::F(2))?;
    terminal.wait_until(|screen| {
        screen.contains("Keybindings · Normalization")
            && screen.contains("Copy the exact normalized text")
    })?;
    terminal.send(Key::F(2))?;
    terminal.wait_until(|screen| screen.contains("NFD Result"))?;
    terminal.send(Key::Char('i'))?;
    terminal.wait_until(|screen| screen.contains("Sequence / NFD Result"))?;
    terminal.send(Key::Char('b'))?;
    terminal.wait_until(|screen| screen.contains("Sequence / Normalization"))?;
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn normalization_result_links_expansions_and_adjacent_changes_to_separate_original_rows()
-> termlens::Result<()> {
    let mut terminal = termlens::bin!("sauva", size(100, 30), args(["ﬃ①②", "--graphics", "off"]))?;
    terminal.wait_until(|screen| screen.contains("3 code points"))?;
    terminal.send(Key::Char('n'))?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| screen.contains("NFKC Result"))?;
    terminal.send(Key::Enter)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("Sequence / NFKC Result") && screen.contains("1/5 · U+0066")
    })?;
    assert_eq!(screen.cell(3, 6).unwrap().style().bg, Color::Indexed(6));
    assert_ne!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    for (position, point) in [("2/5", "U+0066"), ("3/5", "U+0069")] {
        terminal.send(Key::Down)?;
        let screen =
            terminal.snapshot_after(|screen| screen.contains(&format!("{position} · {point}")))?;
        assert_eq!(screen.cell(3, 6).unwrap().style().bg, Color::Indexed(6));
        assert_ne!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    }
    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| screen.contains("4/5 · U+0031"))?;
    assert_ne!(screen.cell(3, 6).unwrap().style().bg, Color::Indexed(6));
    assert_eq!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    assert_ne!(screen.cell(5, 6).unwrap().style().bg, Color::Indexed(6));
    insta::assert_snapshot!(screen.with_styles());
    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| screen.contains("5/5 · U+0032"))?;
    assert_ne!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    assert_eq!(screen.cell(5, 6).unwrap().style().bg, Color::Indexed(6));
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("Sequence / NFKC Result 5/5 / Inspector"))?;
    terminal.send(Key::Backspace)?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| screen.contains("Keybindings · Normalization Result"))?;
    terminal.send(Key::F(1))?;
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn normalization_reference_scrolls_on_jumps_and_resizes_without_moving_the_original_selection()
-> termlens::Result<()> {
    let source = "ﬃ".repeat(50);
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 16),
        args([source.as_str(), "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("50 code points"))?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Char('n'))?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| screen.contains("NFKC Result"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("Sequence / NFKC Result"))?;
    terminal.send(Key::Char('G'))?;
    let screen = terminal.snapshot_after(|screen| screen.contains("150/150 · U+0069"))?;
    assert_eq!(screen.cell(13, 7).unwrap().style().bg, Color::Indexed(6));
    terminal.resize(60, 16)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.size() == (60, 16)
            && screen.contains("150/150 · U+0069")
            && screen.contains("150 CP · 150 GC")
    })?;
    assert_eq!(screen.cell(13, 7).unwrap().style().bg, Color::Indexed(6));
    insta::assert_snapshot!(screen.with_styles());
    terminal.send(Key::Char('g'))?;
    let screen = terminal.snapshot_after(|screen| screen.contains("1/150 · U+0066"))?;
    assert_eq!(screen.cell(3, 7).unwrap().style().bg, Color::Indexed(6));
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("Sequence / Normalization"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("3/50 · U+FB03"))?;
    terminal.send(Key::Char('q'))?;
    assert!(terminal.wait_exit()?.success());
    Ok(())
}

#[test]
fn sequence_highlights_follow_selection_across_clusters() -> termlens::Result<()> {
    let assert_highlights =
        |screen: &termlens::Screen, selected_row: u16, selected_cluster: std::ops::Range<u16>| {
            for row in 3..6 {
                let cluster_selected = selected_cluster.contains(&row);
                for column in 2..6 {
                    let style = screen.cell(row, column).unwrap().style();
                    assert_eq!(
                        style.fg,
                        Color::Indexed(if cluster_selected { 6 } else { 8 })
                    );
                    assert_eq!(style.bold, cluster_selected);
                    assert_eq!(style.bg, Color::Default);
                }
                for column in 6..58 {
                    let expected = if row == selected_row {
                        Color::Indexed(6)
                    } else {
                        Color::Default
                    };
                    assert_eq!(screen.cell(row, column).unwrap().style().bg, expected);
                }
            }
        };
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        args(["--text", "A\u{0301}B", "--graphics", "off"])
    )?;
    let screen = terminal.snapshot_after(|screen| screen.contains("1/3 · U+0041"))?;
    assert_highlights(&screen, 3, 3..5);

    for (status, selected_row, selected_cluster) in
        [("2/3 · U+0301", 4, 3..5), ("3/3 · U+0042", 5, 5..6)]
    {
        terminal.send(Key::Down)?;
        let screen = terminal.snapshot_after(|screen| screen.contains(status))?;
        assert_highlights(&screen, selected_row, selected_cluster);
    }

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
