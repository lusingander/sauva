use std::{
    cmp::Reverse,
    collections::HashSet,
    env,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use fs2::FileExt;
use jiff::Timestamp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    unicode::{CodePoint, parse_code_point_notation},
    viewport::ListViewport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bookmark {
    #[serde(
        serialize_with = "serialize_code_point",
        deserialize_with = "deserialize_code_point"
    )]
    code_point: CodePoint,
    added_at: Timestamp,
}

impl Bookmark {
    pub const fn new(code_point: CodePoint, added_at: Timestamp) -> Self {
        Self {
            code_point,
            added_at,
        }
    }

    pub const fn code_point(self) -> CodePoint {
        self.code_point
    }

    pub const fn added_at(self) -> Timestamp {
        self.added_at
    }
}

fn serialize_code_point<S: Serializer>(
    value: &CodePoint,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

fn deserialize_code_point<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<CodePoint, D::Error> {
    let notation = String::deserialize(deserializer)?;
    parse_code_point_notation(&notation).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkMove {
    Previous,
    Next,
    PageBackward,
    PageForward,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BookmarkSort {
    #[default]
    NewestFirst,
    OldestFirst,
    CodePointAscending,
    CodePointDescending,
}

impl BookmarkSort {
    const fn next(self) -> Self {
        match self {
            Self::NewestFirst => Self::OldestFirst,
            Self::OldestFirst => Self::CodePointAscending,
            Self::CodePointAscending => Self::CodePointDescending,
            Self::CodePointDescending => Self::NewestFirst,
        }
    }

    fn sort(self, entries: &mut [Bookmark]) {
        match self {
            Self::NewestFirst => {
                entries.sort_by_key(|entry| (Reverse(entry.added_at), entry.code_point));
            }
            Self::OldestFirst => {
                entries.sort_by_key(|entry| (entry.added_at, entry.code_point));
            }
            Self::CodePointAscending => entries.sort_by_key(|entry| entry.code_point),
            Self::CodePointDescending => entries.sort_by_key(|entry| Reverse(entry.code_point)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkState {
    entries: Vec<Bookmark>,
    sort: BookmarkSort,
    selected_index: usize,
    viewport: ListViewport,
}

impl BookmarkState {
    pub fn entries(&self) -> &[Bookmark] {
        &self.entries
    }

    pub fn contains(&self, code_point: CodePoint) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.code_point == code_point)
    }

    pub fn selected(&self) -> Option<Bookmark> {
        self.entries.get(self.selected_index).copied()
    }

    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn visible_range(&self) -> Range<usize> {
        self.viewport.visible_range(self.entries.len())
    }

    pub const fn sort(&self) -> BookmarkSort {
        self.sort
    }

    pub fn cycle_sort(&mut self) {
        let selected = self.selected().map(Bookmark::code_point);
        self.sort = self.sort.next();
        self.sort_and_select(selected);
    }

    pub fn replace(&mut self, entries: Vec<Bookmark>) {
        let selected = self.selected().map(Bookmark::code_point);
        self.entries = entries;
        self.sort_and_select(selected);
    }

    fn sort_and_select(&mut self, selected: Option<CodePoint>) {
        self.sort.sort(&mut self.entries);
        self.selected_index = self
            .entries
            .iter()
            .position(|entry| Some(entry.code_point) == selected)
            .unwrap_or(
                self.selected_index
                    .min(self.entries.len().saturating_sub(1)),
            );
        self.viewport
            .ensure_visible(self.selected_index, self.entries.len());
    }

    pub fn select(&mut self, code_point: CodePoint) {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.code_point == code_point)
        {
            self.selected_index = index;
            self.viewport.ensure_visible(index, self.entries.len());
        }
    }

    pub fn resize_viewport(&mut self, height: usize) {
        self.viewport
            .resize(height, self.selected_index, self.entries.len());
    }

    pub fn move_selection(&mut self, movement: BookmarkMove) {
        let last = self.entries.len().saturating_sub(1);
        let page = self.visible_range().len().max(1);
        self.selected_index = match movement {
            BookmarkMove::Previous => self.selected_index.saturating_sub(1),
            BookmarkMove::Next => self.selected_index.saturating_add(1).min(last),
            BookmarkMove::PageBackward => self.selected_index.saturating_sub(page),
            BookmarkMove::PageForward => self.selected_index.saturating_add(page).min(last),
            BookmarkMove::First => 0,
            BookmarkMove::Last => last,
        };
        self.viewport
            .ensure_visible(self.selected_index, self.entries.len());
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BookmarkFile {
    #[serde(default)]
    bookmarks: Vec<Bookmark>,
}

pub fn load() -> io::Result<Vec<Bookmark>> {
    let path = resolve_path(env::var_os("XDG_DATA_HOME"), env::var_os("HOME"))?;
    read_file(&path).map_err(|error| file_error("read", &path, error))
}

pub fn set_bookmarked(code_point: CodePoint, bookmarked: bool) -> io::Result<Vec<Bookmark>> {
    let path = resolve_path(env::var_os("XDG_DATA_HOME"), env::var_os("HOME"))?;
    update_file(&path, code_point, bookmarked, Timestamp::now())
        .map_err(|error| file_error("save", &path, error))
}

fn resolve_path(data_home: Option<OsString>, home: Option<OsString>) -> io::Result<PathBuf> {
    let directory = data_home
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            home.filter(|value| !value.is_empty())
                .map(|value| PathBuf::from(value).join(".local/share"))
        })
        .ok_or_else(|| io::Error::other("Cannot locate bookmarks: set XDG_DATA_HOME or HOME"))?;
    Ok(directory.join("sauva/bookmarks.toml"))
}

fn file_error(operation: &str, path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!(
            "Failed to {operation} bookmarks: {error} ({})",
            path.display()
        ),
    )
}

fn read_file(path: &Path) -> io::Result<Vec<Bookmark>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let file: BookmarkFile = toml::from_str(&text).map_err(io::Error::other)?;
    let mut seen = HashSet::new();
    for entry in &file.bookmarks {
        if !seen.insert(entry.code_point) {
            return Err(io::Error::other(format!(
                "Duplicate bookmark: {}",
                entry.code_point
            )));
        }
    }
    Ok(file.bookmarks)
}

fn update_file(
    path: &Path,
    code_point: CodePoint,
    bookmarked: bool,
    added_at: Timestamp,
) -> io::Result<Vec<Bookmark>> {
    let directory = path
        .parent()
        .expect("the bookmark file has a parent directory");
    fs::create_dir_all(directory)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))?;
    lock.try_lock_exclusive().map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            io::Error::new(error.kind(), "Bookmark file is busy; try again")
        } else {
            error
        }
    })?;
    // Reload while holding the lock so another running Sauva's changes are retained.
    let mut bookmarks = read_file(path)?;
    if bookmarked {
        if !bookmarks.iter().any(|entry| entry.code_point == code_point) {
            bookmarks.push(Bookmark::new(code_point, added_at));
        }
    } else {
        bookmarks.retain(|entry| entry.code_point != code_point);
    }
    BookmarkSort::NewestFirst.sort(&mut bookmarks);
    let text = toml::to_string(&BookmarkFile {
        bookmarks: bookmarks.clone(),
    })
    .map_err(io::Error::other)?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(text.as_bytes())?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(bookmarks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_data_home_and_falls_back_to_home() {
        assert_eq!(
            resolve_path(Some("/data".into()), Some("/home/user".into())).unwrap(),
            PathBuf::from("/data/sauva/bookmarks.toml")
        );
        for data in [None, Some("".into())] {
            assert_eq!(
                resolve_path(data, Some("/home/user".into())).unwrap(),
                PathBuf::from("/home/user/.local/share/sauva/bookmarks.toml")
            );
        }
        assert!(resolve_path(None, None).is_err());
    }

    #[test]
    fn persists_special_code_points_and_keeps_the_original_registration_time() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("data/bookmarks.toml");
        assert!(read_file(&path).unwrap().is_empty());
        let first: Timestamp = "2026-10-15T10:00:00Z".parse().unwrap();
        let later: Timestamp = "2026-10-16T10:00:00Z".parse().unwrap();
        let surrogate = CodePoint::new(0xd800).unwrap();
        let unassigned = CodePoint::new(0x0378).unwrap();
        update_file(&path, surrogate, true, first).unwrap();
        update_file(&path, unassigned, true, later).unwrap();
        update_file(&path, surrogate, true, later).unwrap();
        let entries = read_file(&path).unwrap();
        assert_eq!(
            entries,
            [
                Bookmark::new(unassigned, later),
                Bookmark::new(surrogate, first)
            ]
        );
        update_file(&path, unassigned, false, later).unwrap();
        assert_eq!(read_file(&path).unwrap(), [Bookmark::new(surrogate, first)]);
        update_file(&path, surrogate, false, later).unwrap();
        assert!(read_file(&path).unwrap().is_empty());
    }

    #[test]
    fn refuses_to_overwrite_invalid_files_or_write_while_locked() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("bookmarks.toml");
        let cp = CodePoint::from('A');
        for source in [
            "not valid TOML",
            r#"
                [[bookmarks]]
                code_point = 'U+110000'
                added_at = '2026-10-16T00:00:00Z'
            "#,
            r#"
                [[bookmarks]]
                code_point = 'U+0041'
                added_at = 'invalid'
            "#,
        ] {
            fs::write(&path, source).unwrap();
            assert!(update_file(&path, cp, true, Timestamp::UNIX_EPOCH).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), source);
        }
        fs::write(&path, "bookmarks = []").unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))
            .unwrap();
        lock.lock_exclusive().unwrap();
        assert_eq!(
            update_file(&path, cp, true, Timestamp::UNIX_EPOCH)
                .unwrap_err()
                .kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "bookmarks = []");
    }

    #[test]
    fn cycles_through_all_orders_and_breaks_date_ties_by_numeric_code_point() {
        let mut state = BookmarkState::default();
        state.replace(
            [
                (0x10000, 2),
                (0xe000, 1),
                (0xd800, 2),
                (0x41, 3),
                (0x378, 1),
            ]
            .map(|(value, seconds)| {
                Bookmark::new(
                    CodePoint::new(value).unwrap(),
                    Timestamp::from_second(seconds).unwrap(),
                )
            })
            .to_vec(),
        );
        for (sort, expected) in [
            (
                BookmarkSort::NewestFirst,
                [0x41, 0xd800, 0x10000, 0x378, 0xe000],
            ),
            (
                BookmarkSort::OldestFirst,
                [0x378, 0xe000, 0xd800, 0x10000, 0x41],
            ),
            (
                BookmarkSort::CodePointAscending,
                [0x41, 0x378, 0xd800, 0xe000, 0x10000],
            ),
            (
                BookmarkSort::CodePointDescending,
                [0x10000, 0xe000, 0xd800, 0x378, 0x41],
            ),
            (
                BookmarkSort::NewestFirst,
                [0x41, 0xd800, 0x10000, 0x378, 0xe000],
            ),
        ] {
            assert_eq!(state.sort(), sort);
            assert_eq!(
                state
                    .entries()
                    .iter()
                    .map(|entry| entry.code_point().value())
                    .collect::<Vec<_>>(),
                expected,
            );
            state.cycle_sort();
        }
    }

    #[rstest::rstest]
    #[case(0, 5, 0)]
    #[case(1, 7, 10)]
    #[case(2, 7, 10)]
    #[case(3, 5, 0)]
    fn selection_survives_sorting_and_removal_and_scrolls_with_the_viewport(
        #[case] cycles: usize,
        #[case] after_removal: u32,
        #[case] added_index: usize,
    ) {
        let mut state = BookmarkState::default();
        let entries = (0..10)
            .map(|value| {
                Bookmark::new(
                    CodePoint::new(value).unwrap(),
                    Timestamp::from_second(i64::from(value)).unwrap(),
                )
            })
            .collect();
        state.replace(entries);
        state.resize_viewport(3);
        state.move_selection(BookmarkMove::PageForward);
        assert_eq!(state.selected().unwrap().code_point().value(), 6);
        assert_eq!(state.visible_range(), 1..4);
        let selected = state.selected().unwrap();
        for _ in 0..cycles {
            state.cycle_sort();
            assert_eq!(state.selected(), Some(selected));
            assert!(state.visible_range().contains(&state.selected_index()));
        }
        let sort = state.sort();
        let mut entries = state.entries().to_vec();
        entries.push(Bookmark::new(
            CodePoint::from('A'),
            Timestamp::from_second(10).unwrap(),
        ));
        state.replace(entries);
        assert_eq!(state.sort(), sort);
        assert_eq!(
            state.entries()[added_index].code_point(),
            CodePoint::from('A')
        );
        assert_eq!(state.selected(), Some(selected));
        assert!(state.visible_range().contains(&state.selected_index()));
        let remaining = state
            .entries()
            .iter()
            .copied()
            .filter(|entry| *entry != selected)
            .collect();
        state.replace(remaining);
        assert_eq!(state.sort(), sort);
        assert_eq!(
            state.selected().unwrap().code_point().value(),
            after_removal
        );
        assert!(state.visible_range().contains(&state.selected_index()));
        state.replace(Vec::new());
        state.move_selection(BookmarkMove::Last);
        assert!(state.selected().is_none());
        assert_eq!(state.visible_range(), 0..0);
    }

    #[test]
    fn empty_and_single_bookmark_lists_can_cycle_sort() {
        let bookmark = Bookmark::new(CodePoint::from('A'), Timestamp::UNIX_EPOCH);
        for entries in [Vec::new(), vec![bookmark]] {
            let mut state = BookmarkState::default();
            state.replace(entries.clone());
            state.resize_viewport(3);
            for sort in [
                BookmarkSort::OldestFirst,
                BookmarkSort::CodePointAscending,
                BookmarkSort::CodePointDescending,
                BookmarkSort::NewestFirst,
            ] {
                state.cycle_sort();
                assert_eq!(state.sort(), sort);
                assert_eq!(state.entries(), entries);
                assert_eq!(state.selected(), entries.first().copied());
                assert_eq!(state.visible_range(), 0..entries.len());
            }
        }
    }
}
