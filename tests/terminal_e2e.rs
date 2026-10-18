use std::fs;

use rstest::rstest;
use tempfile::tempdir;
use termlens::{Color, Key, Screen, Terminal};

fn spawn(graphics: &str) -> termlens::Result<Terminal> {
    termlens::bin!("sauva", size(100, 30), args(["--graphics", graphics]))
}

fn selected_property(screen: &Screen, label: &str) -> bool {
    (0..screen.rows()).any(|row| {
        screen.row_text(row).trim_start().starts_with(label)
            && screen.cell(row, 3).unwrap().style().bg == Color::Indexed(6)
    })
}

fn footer_contains(screen: &Screen, texts: &[&str]) -> bool {
    let footer = screen.row_text(screen.rows() - 1);
    texts.iter().all(|text| footer.contains(text))
}

fn resize_and_wait(
    terminal: &mut Terminal,
    columns: u16,
    rows: u16,
    mut ready: impl FnMut(&Screen) -> bool,
) -> termlens::Result<Screen> {
    // The grid resizes before the app repaints, so ready must identify the new layout.
    terminal.resize(columns, rows)?;
    terminal.snapshot_after(|screen| screen.size() == (columns, rows) && ready(screen))
}

fn quit(terminal: &mut Terminal) -> termlens::Result<()> {
    terminal.send(Key::Char('q'))?;
    let status = terminal.wait_exit()?;
    assert!(status.success(), "exit status: {status}");
    Ok(())
}

#[test]
fn property_page_keeps_reading_position_and_restores_the_source_inspector() -> termlens::Result<()>
{
    let mut terminal = termlens::bin!("sauva", size(100, 30), args(["éß", "--graphics", "off"]))?;
    terminal.wait_until(|screen| screen.row_text(0).contains("Sequence"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.row_text(0).contains("Inspector"))?;
    terminal.send(Key::Char('a'))?;
    for _ in 0..6 {
        terminal.send(Key::Char('j'))?;
    }
    terminal.wait_until(|screen| {
        screen.row_text(2).contains("All") && selected_property(screen, "General Category")
    })?;
    terminal.send(Key::Char('e'))?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("sauva / Property")
            && screen.row_text(0).trim_end().ends_with("General Category")
    })?;
    let minimum = resize_and_wait(&mut terminal, 60, 16, |screen| {
        screen.contains("Current value · U+00E9")
            && footer_contains(screen, &["Esc Back", "F1 Help"])
    })?;
    insta::assert_snapshot!("property_page_minimum", minimum.with_styles());
    terminal.send(Key::Char('G'))?;
    terminal
        .wait_until(|screen| screen.contains("Reference") && !screen.contains("Current value"))?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| screen.contains("Keybindings · Property"))?;
    terminal.send(Key::Esc)?;
    terminal.snapshot_after(|screen| {
        screen.row_text(0).contains("sauva / Property")
            && screen.contains("Reference")
            && !screen.contains("Current value")
    })?;
    resize_and_wait(&mut terminal, 100, 30, |screen| {
        screen.row_text(0).contains("General Category")
            && footer_contains(screen, &["Esc Back", "F1 Help"])
    })?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("Inspector")
            && screen.row_text(0).contains("U+00E9")
            && screen.row_text(2).contains("All")
            && selected_property(screen, "General Category")
    })?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("Sequence") && screen.contains("2 code points")
    })?;
    quit(&mut terminal)?;
    Ok(())
}

#[rstest]
#[case("U+00DF", 13, "Uppercase Mapping", "U+0053 LATIN CAPITAL LETTER S")]
#[case("U+00BC", 18, "Numeric Value", "1/4")]
#[case("U+D800", 19, "UTF-8", "Not available")]
fn property_page_opens_new_and_unavailable_fields_with_custom_bindings(
    #[case] code_point: &str,
    #[case] index: usize,
    #[case] label: &str,
    #[case] value: &str,
) -> termlens::Result<()> {
    let directory = tempdir()?;
    let config = directory.path().join("config.toml");
    fs::write(
        &config,
        r#"
            [keybindings.inspector]
            explain_property = ['x']

            [keybindings.property]
            close = ['backspace']
            move_down = ['n']
        "#,
    )?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        env("SAUVA_CONFIG_FILE", &config),
        args([code_point, "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.row_text(0).contains(code_point))?;
    terminal.send(Key::Char('a'))?;
    for _ in 0..index {
        terminal.send(Key::Char('j'))?;
    }
    terminal.send(Key::Char('e'))?;
    terminal.send(Key::Char('x'))?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("sauva / Property")
            && screen.row_text(0).contains(label)
            && screen.contains(value)
    })?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| {
        screen.contains("Backspace") && screen.contains("Scroll down one line")
    })?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.row_text(0).contains("sauva / Property"))?;
    terminal.send(Key::Char('n'))?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("Inspector") && screen.row_text(0).contains(code_point)
    })?;
    quit(&mut terminal)?;
    Ok(())
}

#[test]
fn bookmarks_persist_and_show_configured_dates_and_restore_the_source_inspector()
-> termlens::Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("config.toml");
    fs::write(
        &path,
        r#"
            [ui]
            bookmark_date_format = '%d/%m/%Y %H:%M:%S %Z'
        "#,
    )?;
    let data = directory.path().join("sauva");
    fs::create_dir(&data)?;
    fs::write(
        data.join("bookmarks.toml"),
        r#"
            [[bookmarks]]
            code_point = 'U+2192'
            added_at = '2020-01-02T03:04:05Z'
        "#,
    )?;
    let launch = || {
        termlens::bin!(
            "sauva",
            size(100, 30),
            env("SAUVA_CONFIG_FILE", &path),
            env("XDG_DATA_HOME", directory.path()),
            env("TZ", "Asia/Tokyo"),
            args(["--graphics", "off"])
        )
    };
    let mut terminal = launch()?;
    terminal.wait_until(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;
    terminal.send(Key::Char('m'))?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("U+0041  * Bookmarked")
            && footer_contains(screen, &["Bookmarked U+0041"])
    })?;
    terminal.send(Key::Char('M'))?;
    terminal.wait_until(|screen| screen.row_text(0).contains("2 bookmarks · U+0041"))?;
    terminal.send(Key::Char('j'))?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("2 bookmarks · U+2192")
            && screen.contains("02/01/2020 12:04:05 JST")
    })?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| screen.contains("Keybindings · Bookmarks"))?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| screen.row_text(0).contains("2 bookmarks · U+2192"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("Bookmarks / Inspector") && screen.contains("RIGHTWARDS ARROW")
    })?;
    terminal.send(Key::Backspace)?;
    terminal.wait_until(|screen| screen.row_text(0).contains("2 bookmarks · U+2192"))?;
    resize_and_wait(&mut terminal, 60, 16, |screen| {
        footer_contains(screen, &["Esc Close"]) && screen.contains("02/01/2020 12:04:05 JST")
    })?;
    terminal.send(Key::Char('m'))?;
    terminal.wait_until(|screen| {
        footer_contains(screen, &["Removed bookmark U+2192"])
            && screen.row_text(0).contains("1 bookmark · U+0041")
    })?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("Inspector")
            && screen.row_text(0).contains("U+0041  * Bookmarked")
            && !footer_contains(screen, &["Esc Close"])
    })?;
    quit(&mut terminal)?;

    let mut restored = launch()?;
    restored.wait_until(|screen| screen.row_text(0).contains("U+0041  * Bookmarked"))?;
    restored.send(Key::Char('M'))?;
    restored.wait_until(|screen| screen.row_text(0).contains("1 bookmark · U+0041"))?;
    restored.send(Key::Char('m'))?;
    restored.wait_until(|screen| {
        screen.contains("No bookmarks yet") && screen.contains("Removed bookmark U+0041")
    })?;
    restored.send(Key::Enter)?;
    restored.wait_until(|screen| screen.row_text(0).contains("0 bookmarks"))?;
    restored.send(Key::Esc)?;
    restored.wait_until(|screen| {
        screen.contains("Identity") && !screen.row_text(0).contains("Bookmarked")
    })?;
    quit(&mut restored)?;
    let file: toml::Value =
        toml::from_str(&fs::read_to_string(data.join("bookmarks.toml"))?).unwrap();
    assert!(file["bookmarks"].as_array().unwrap().is_empty());
    Ok(())
}

#[test]
fn invalid_bookmark_files_report_save_failure_without_changing_registration() -> termlens::Result<()>
{
    let directory = tempdir()?;
    let config = directory.path().join("config.toml");
    fs::write(&config, "")?;
    let data = directory.path().join("sauva");
    fs::create_dir(&data)?;
    let bookmarks = data.join("bookmarks.toml");
    let invalid = "invalid bookmark file";
    fs::write(&bookmarks, invalid)?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        env("SAUVA_CONFIG_FILE", &config),
        env("XDG_DATA_HOME", directory.path()),
        args(["--graphics", "off"])
    )?;
    terminal.wait_until(|screen| footer_contains(screen, &["Failed to read bookmarks"]))?;
    terminal.send(Key::Char('m'))?;
    let screen =
        terminal.snapshot_after(|screen| footer_contains(screen, &["Failed to save bookmarks"]))?;
    assert!(!screen.row_text(0).contains("* Bookmarked"), "{screen}");
    assert!(screen.contains("LATIN CAPITAL LETTER A"), "{screen}");
    assert_eq!(fs::read_to_string(&bookmarks)?, invalid);
    quit(&mut terminal)?;
    Ok(())
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
            toggle_properties = ["t"]
        "#,
    )?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        env("SAUVA_CONFIG_FILE", &path),
        args(["--graphics", "off"])
    )?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_until(|screen| screen.row_text(2).contains("All"))?;

    terminal.send(Key::F(2))?;
    terminal.send(Key::Char('G'))?;
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
        screen.contains("Identity")
            && screen.contains("LATIN CAPITAL LETTER A")
            && screen.row_text(2).contains("All")
    })?;
    terminal.send(Key::Char('t'))?;
    terminal.wait_until(|screen| screen.row_text(2).contains("Relevant"))?;

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

    quit(&mut terminal)?;
    assert!(!terminal.screen().alternate_screen());
    assert!(terminal.screen().cursor_visible());
    Ok(())
}

#[test]
fn inspector_follows_the_selected_property_across_code_point_changes() -> termlens::Result<()> {
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        args(["U+0030", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.row_text(0).contains("U+0030"))?;
    terminal.send(Key::Char(']'))?;
    terminal.send(Key::Char(']'))?;
    terminal.send(Key::Char('j'))?;
    terminal.wait_until(|screen| selected_property(screen, "Numeric Value"))?;
    terminal.send(Key::Char('l'))?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("U+0031") && selected_property(screen, "Numeric Value")
    })?;
    terminal.send(Key::Char('h'))?;
    terminal.send(Key::Char('h'))?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("U+002F") && selected_property(screen, "UTF-8")
    })?;
    quit(&mut terminal)?;
    Ok(())
}

#[test]
fn inspector_property_modes_show_optional_fields_and_preserve_the_selected_property()
-> termlens::Result<()> {
    let directory = tempdir()?;
    let config = directory.path().join("config.toml");
    fs::write(&config, "")?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 60),
        env("SAUVA_CONFIG_FILE", &config),
        env("XDG_DATA_HOME", directory.path()),
        args(["U+00C5", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| {
        screen.row_text(0).contains("U+00C5") && screen.row_text(2).contains("Relevant")
    })?;

    terminal.send(Key::Char('a'))?;
    for _ in 0..3 {
        terminal.send(Key::Char('j'))?;
    }
    let all = terminal.snapshot_after(|screen| {
        screen.row_text(2).contains("All") && selected_property(screen, "Aliases")
    })?;
    insta::assert_snapshot!(all.with_styles());

    terminal.send(Key::Char('a'))?;
    terminal.wait_until(|screen| {
        screen.row_text(2).contains("Relevant")
            && selected_property(screen, "Plane")
            && !screen.contains("Aliases")
    })?;

    terminal.send(Key::Char('a'))?;
    terminal.wait_until(|screen| {
        screen.row_text(2).contains("All") && selected_property(screen, "Plane")
    })?;
    quit(&mut terminal)?;
    Ok(())
}

#[test]
fn inspector_group_keys_show_section_headings_and_select_their_first_properties()
-> termlens::Result<()> {
    let mut terminal =
        termlens::bin!("sauva", size(60, 16), args(["U+D800", "--graphics", "off"]))?;
    terminal.wait_until(|screen| {
        screen.row_text(3).trim_start().starts_with("Identity")
            && screen.row_text(2).contains("Relevant")
    })?;
    terminal.send(Key::Char('a'))?;
    terminal.wait_until(|screen| screen.row_text(2).contains("All"))?;
    terminal.send(Key::Char('j'))?;
    terminal.send(Key::Char(']'))?;
    terminal.wait_until(|screen| {
        screen
            .row_text(3)
            .trim_start()
            .starts_with("Classification")
            && selected_property(screen, "General Category")
    })?;

    terminal.send(Key::Char('j'))?;
    terminal.send(Key::Char('['))?;
    terminal.wait_until(|screen| {
        screen.row_text(3).trim_start().starts_with("Identity")
            && selected_property(screen, "Character")
    })?;

    terminal.send(Key::Char(']'))?;
    terminal.send(Key::Char(']'))?;
    terminal.send(Key::Char(']'))?;
    terminal.send(Key::Char(']'))?;
    terminal.wait_until(|screen| {
        screen.row_text(3).trim_start().starts_with("Encoding")
            && selected_property(screen, "UTF-8")
    })?;
    terminal.send(Key::Char('a'))?;
    terminal.wait_until(|screen| screen.row_text(2).contains("Relevant"))?;
    quit(&mut terminal)?;
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
    quit(&mut terminal)?;
    Ok(())
}

#[test]
fn sequence_copy_dialog_preserves_source_and_candidate_across_help_and_resize()
-> termlens::Result<()> {
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        args(["A\u{0301}👩‍💻B", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("1/6 · U+0041"))?;
    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| screen.contains("2/6 · U+0301"))?;
    terminal.send(Key::Char('Y'))?;
    terminal.wait_until(|screen| {
        screen.contains("Sequence") && screen.contains("Preview · 1 CP · 2 bytes")
    })?;
    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| screen.contains("Preview · 2 CP · 3 bytes"))?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| screen.contains("Keybindings · Copy Dialog"))?;
    resize_and_wait(&mut terminal, 60, 16, |screen| {
        screen.contains("Keybindings · Copy Dialog") && footer_contains(screen, &["F1 Close"])
    })?;
    terminal.send(Key::Esc)?;
    let screen = terminal.snapshot_after(|screen| {
        screen.contains("Sequence") && screen.contains("Preview · 2 CP · 3 bytes")
    })?;
    insta::assert_snapshot!(screen.with_styles());

    terminal.send(Key::Char('G'))?;
    terminal.wait_until(|screen| screen.contains("Preview · 6 CP · 15 bytes"))?;
    resize_and_wait(&mut terminal, 100, 30, |screen| {
        screen.contains("Preview · 6 CP · 15 bytes")
            && footer_contains(screen, &["Enter Copy", "F1 Help"])
    })?;
    terminal.send(Key::Esc)?;
    // The source header is also visible behind the dialog. Its footer confirms
    // that Esc was handled before the next character can form an Alt chord.
    terminal.wait_until(|screen| {
        screen.contains("2/6 · U+0301") && footer_contains(screen, &["Y Copy..."])
    })?;
    terminal.send(Key::Char('Y'))?;
    terminal.wait_until(|screen| screen.contains("Preview · 1 CP · 2 bytes"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| {
        screen.contains("2/6 · U+0301") && footer_contains(screen, &["Y Copy..."])
    })?;
    quit(&mut terminal)?;
    Ok(())
}

#[test]
fn normalized_copy_dialog_uses_result_text_and_custom_bindings() -> termlens::Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("config.toml");
    fs::write(
        &path,
        r#"
            [keybindings.normalization_result]
            open_copy_dialog = ["C"]

            [keybindings.copy_dialog]
            move_down = ["l"]
            activate = ["y"]
            close = ["x"]
        "#,
    )?;
    let mut terminal = termlens::bin!(
        "sauva",
        size(60, 16),
        env("SAUVA_CONFIG_FILE", &path),
        args(["A\u{0301}ﬃ", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("1/3 · U+0041"))?;
    terminal.send(Key::Char('n'))?;
    terminal.send(Key::Down)?;
    terminal.send(Key::Down)?;
    terminal.wait_until(|screen| screen.contains("NFKC Result"))?;
    terminal.send(Key::Enter)?;
    terminal.wait_until(|screen| screen.contains("1/4 · U+00C1"))?;
    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| screen.contains("2/4 · U+0066"))?;
    assert!(screen.contains("C Copy..."), "{screen}");
    terminal.send(Key::Char('C'))?;
    terminal.wait_until(|screen| {
        screen.contains("NFKC Result") && screen.contains("Preview · 1 CP · 1 byte")
    })?;
    terminal.send(Key::Char('l'))?;
    terminal.send(Key::Char('l'))?;
    let screen = terminal.snapshot_after(|screen| screen.contains("Preview · 4 CP · 5 bytes"))?;
    assert!(screen.contains("Whole NFKC Result"), "{screen}");
    assert!(screen.contains("Áffi"), "{screen}");
    assert!(screen.contains("U+00C1 U+0066 U+0066 U+0069"), "{screen}");
    assert!(screen.contains("y Copy"), "{screen}");
    assert!(screen.contains("x Cancel"), "{screen}");
    resize_and_wait(&mut terminal, 100, 30, |screen| {
        screen.contains("Preview · 4 CP · 5 bytes")
            && footer_contains(screen, &["y Copy", "F1 Help"])
    })?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| {
        screen.contains("Keybindings · Copy Dialog")
            && screen.contains("Copy the selected candidate")
    })?;
    terminal.send(Key::F(1))?;
    terminal.wait_until(|screen| screen.contains("Preview · 4 CP · 5 bytes"))?;
    terminal.send(Key::Char('x'))?;
    terminal.wait_until(|screen| {
        screen.contains("2/4 · U+0066") && footer_contains(screen, &["C Copy..."])
    })?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("Sequence / Normalization"))?;
    terminal.send(Key::Esc)?;
    terminal.wait_until(|screen| screen.contains("1/3 · U+0041"))?;
    quit(&mut terminal)?;
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
    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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
    terminal.wait_until(|screen| {
        screen.contains("NFC Result") && screen.contains("1 changed region")
    })?;
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
    quit(&mut terminal)?;
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
    quit(&mut terminal)?;
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
    quit(&mut terminal)?;
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
    let screen = resize_and_wait(&mut terminal, 60, 16, |screen| {
        // The compact counts appear only after the app redraws at this width.
        screen.contains("150/150 · U+0069") && screen.contains("150 CP · 150 GC")
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
    quit(&mut terminal)?;
    Ok(())
}

#[test]
fn sequence_highlights_follow_selection_across_clusters() -> termlens::Result<()> {
    let mut terminal = termlens::bin!(
        "sauva",
        size(100, 30),
        args(["--text", "A\u{0301}B", "--graphics", "off"])
    )?;
    terminal.wait_until(|screen| screen.contains("1/3 · U+0041"))?;
    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| screen.contains("2/3 · U+0301"))?;
    assert_eq!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    assert_ne!(screen.cell(3, 6).unwrap().style().bg, Color::Indexed(6));
    assert_eq!(screen.cell(3, 2).unwrap().style().fg, Color::Indexed(6));

    terminal.send(Key::Down)?;
    let screen = terminal.snapshot_after(|screen| screen.contains("3/3 · U+0042"))?;
    assert_eq!(screen.cell(5, 6).unwrap().style().bg, Color::Indexed(6));
    assert_ne!(screen.cell(4, 6).unwrap().style().bg, Color::Indexed(6));
    assert_ne!(screen.cell(3, 2).unwrap().style().fg, Color::Indexed(6));

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    let minimum = resize_and_wait(&mut terminal, 60, 16, |screen| {
        screen.contains("Search")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
            && footer_contains(screen, &["Esc Close", "F1 Help"])
    })?;
    insta::assert_snapshot!(minimum.with_styles());

    resize_and_wait(&mut terminal, 100, 30, |screen| {
        screen.contains("Selection")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
            && footer_contains(screen, &["Esc Close", "F1 Help"])
    })?;
    terminal.send(Key::Enter)?;
    terminal.snapshot_after(|screen| {
        screen.contains("Identity")
            && screen.contains("U+2192")
            && screen.contains("RIGHTWARDS ARROW")
    })?;

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
    Ok(())
}

#[test]
fn redraws_across_the_minimum_terminal_size_boundary() -> termlens::Result<()> {
    let mut terminal = spawn("off")?;
    terminal.snapshot_after(|screen| screen.contains("LATIN CAPITAL LETTER A"))?;

    resize_and_wait(&mut terminal, 59, 15, |screen| {
        screen.contains("Terminal too small")
    })?;

    resize_and_wait(&mut terminal, 60, 16, |screen| {
        screen.contains("LATIN CAPITAL LETTER A")
    })?;

    quit(&mut terminal)?;
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

    quit(&mut terminal)?;
    assert_eq!(terminal.screen().graphics().deletes(), 1);
    Ok(())
}
