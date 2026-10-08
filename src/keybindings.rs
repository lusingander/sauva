use std::{collections::HashMap, error::Error, fmt};

use garde::Validate;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use tui_input::backend::crossterm::to_input_request;
use umbra::optional;

const ALL_COMMANDS: [Command; 28] = [
    Command::Quit,
    Command::Help,
    Command::Close,
    Command::PreviousCodePoint,
    Command::NextCodePoint,
    Command::PreviousGroup,
    Command::NextGroup,
    Command::MoveUp,
    Command::MoveDown,
    Command::MoveLeft,
    Command::MoveRight,
    Command::PageUp,
    Command::PageDown,
    Command::First,
    Command::Last,
    Command::CopyValue,
    Command::OpenCopyDialog,
    Command::OpenSearch,
    Command::BrowsePlanes,
    Command::BrowseRanges,
    Command::BrowseBlocks,
    Command::BrowseCodePoints,
    Command::PreviousResult,
    Command::NextResult,
    Command::InspectResult,
    Command::Activate,
    Command::Back,
    Command::Normalize,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    Global,
    Inspector,
    Search,
    Sequence,
    Normalization,
    NormalizationResult,
    CopyDialog,
    BrowsePlane,
    BrowseRange,
    BrowseBlock,
    BrowseCodePoints,
    Help,
}

impl Context {
    const fn config_name(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Inspector => "inspector",
            Self::Search => "search",
            Self::Sequence => "sequence",
            Self::Normalization => "normalization",
            Self::NormalizationResult => "normalization_result",
            Self::CopyDialog => "copy_dialog",
            Self::BrowsePlane => "browse_plane",
            Self::BrowseRange => "browse_range",
            Self::BrowseBlock => "browse_block",
            Self::BrowseCodePoints => "browse_code_points",
            Self::Help => "help",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    Quit,
    Help,
    Close,
    PreviousCodePoint,
    NextCodePoint,
    PreviousGroup,
    NextGroup,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    First,
    Last,
    CopyValue,
    OpenCopyDialog,
    OpenSearch,
    BrowsePlanes,
    BrowseRanges,
    BrowseBlocks,
    BrowseCodePoints,
    PreviousResult,
    NextResult,
    InspectResult,
    Activate,
    Back,
    Normalize,
}

impl Command {
    pub const fn is_repeatable(self) -> bool {
        matches!(
            self,
            Self::PreviousCodePoint
                | Self::NextCodePoint
                | Self::PreviousGroup
                | Self::NextGroup
                | Self::MoveUp
                | Self::MoveDown
                | Self::MoveLeft
                | Self::MoveRight
                | Self::PageUp
                | Self::PageDown
                | Self::First
                | Self::Last
                | Self::PreviousResult
                | Self::NextResult
        )
    }

    const fn config_name(self) -> &'static str {
        match self {
            Self::Quit => "quit",
            Self::Help => "help",
            Self::Close => "close",
            Self::PreviousCodePoint => "previous_code_point",
            Self::NextCodePoint => "next_code_point",
            Self::PreviousGroup => "previous_group",
            Self::NextGroup => "next_group",
            Self::MoveUp => "move_up",
            Self::MoveDown => "move_down",
            Self::MoveLeft => "move_left",
            Self::MoveRight => "move_right",
            Self::PageUp => "page_up",
            Self::PageDown => "page_down",
            Self::First => "first",
            Self::Last => "last",
            Self::CopyValue => "copy_value",
            Self::OpenCopyDialog => "open_copy_dialog",
            Self::OpenSearch => "search",
            Self::BrowsePlanes => "browse_planes",
            Self::BrowseRanges => "browse_ranges",
            Self::BrowseBlocks => "browse_blocks",
            Self::BrowseCodePoints => "browse_code_points",
            Self::PreviousResult => "previous_result",
            Self::NextResult => "next_result",
            Self::InspectResult => "inspect_result",
            Self::Activate => "activate",
            Self::Back => "back",
            Self::Normalize => "normalize",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyChord {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        let (code, modifiers) = normalize(code, modifiers);
        Self { code, modifiers }
    }

    pub fn from_event(event: KeyEvent) -> Self {
        Self::new(event.code, event.modifiers)
    }

    fn as_press_event(self) -> Event {
        Event::Key(KeyEvent::new(self.code, self.modifiers))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyParseError(String);

impl fmt::Display for KeyParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for KeyParseError {}

pub fn parse_key(value: &str) -> Result<KeyChord, KeyParseError> {
    if value.is_empty() {
        return Err(KeyParseError("key must not be empty".to_owned()));
    }

    if let Some(character) = single_bindable_character(value) {
        return Ok(plain(character));
    }

    let mut rest = value;
    let mut modifiers = KeyModifiers::NONE;
    while let Some((modifier, name, remaining)) = [
        (KeyModifiers::CONTROL, "ctrl", "ctrl-"),
        (KeyModifiers::ALT, "alt", "alt-"),
        (KeyModifiers::SHIFT, "shift", "shift-"),
    ]
    .into_iter()
    .find_map(|(modifier, name, prefix)| {
        rest.strip_prefix(prefix)
            .map(|remaining| (modifier, name, remaining))
    }) {
        if modifiers.contains(modifier) {
            return Err(KeyParseError(format!(
                "modifier `{name}` is specified more than once"
            )));
        }
        modifiers.insert(modifier);
        rest = remaining;
    }

    if !modifiers.is_empty() {
        let character = rest.chars().next().filter(|_| rest.chars().count() == 1);
        let Some(character) = character else {
            return Err(KeyParseError(
                "modifiers can only be combined with a single character".to_owned(),
            ));
        };
        if character.is_ascii_uppercase() {
            return Err(KeyParseError(
                "uppercase aliases cannot be combined with modifiers".to_owned(),
            ));
        }
        if !is_bindable_character(character) {
            return Err(KeyParseError(
                "whitespace and control characters must use a named key".to_owned(),
            ));
        }
        return Ok(KeyChord::new(KeyCode::Char(character), modifiers));
    }

    let code = match value {
        "space" => KeyCode::Char(' '),
        "enter" => KeyCode::Enter,
        "esc" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        _ => {
            if let Some(number) = value.strip_prefix('f').and_then(|value| value.parse().ok())
                && (1..=12).contains(&number)
            {
                KeyCode::F(number)
            } else {
                return Err(KeyParseError(format!("unknown key `{value}`")));
            }
        }
    };
    Ok(named(code))
}

fn single_bindable_character(value: &str) -> Option<char> {
    let mut characters = value.chars();
    let character = characters.next()?;
    (characters.next().is_none() && is_bindable_character(character)).then_some(character)
}

fn is_bindable_character(character: char) -> bool {
    !character.is_whitespace() && !character.is_control()
}

fn validate_config_key(value: &str, _: &()) -> garde::Result {
    parse_key(value)
        .map(|_| ())
        .map_err(|error| garde::Error::new(format!("invalid key `{value}`: {error}")))
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)],
    visibility = pub
)]
#[derive(Debug, Default, Serialize, Validate)]
pub struct Keybindings {
    #[garde(dive)]
    #[nested]
    global: GlobalKeybindings,
    #[garde(dive)]
    #[nested]
    inspector: InspectorKeybindings,
    #[garde(dive)]
    #[nested]
    search: SearchKeybindings,
    #[garde(dive)]
    #[nested]
    sequence: SequenceKeybindings,
    #[garde(dive)]
    #[nested]
    normalization: NormalizationKeybindings,
    #[garde(dive)]
    #[nested]
    normalization_result: NormalizationResultKeybindings,
    #[garde(dive)]
    #[nested]
    copy_dialog: CopyDialogKeybindings,
    #[garde(dive)]
    #[nested]
    browse_plane: BrowsePlaneKeybindings,
    #[garde(dive)]
    #[nested]
    browse_range: BrowseRangeKeybindings,
    #[garde(dive)]
    #[nested]
    browse_block: BrowseBlockKeybindings,
    #[garde(dive)]
    #[nested]
    browse_code_points: BrowseCodePointsKeybindings,
    #[garde(dive)]
    #[nested]
    help: HelpKeybindings,
}

impl Keybindings {
    fn into_bindings(self) -> Vec<Keybinding> {
        let mut bindings = Vec::new();
        self.global.append_bindings(&mut bindings);
        self.inspector.append_bindings(&mut bindings);
        self.search.append_bindings(&mut bindings);
        self.sequence.append_bindings(&mut bindings);
        self.normalization.append_bindings(&mut bindings);
        self.normalization_result.append_bindings(&mut bindings);
        self.copy_dialog.append_bindings(&mut bindings);
        self.browse_plane.append_bindings(&mut bindings);
        self.browse_range.append_bindings(&mut bindings);
        self.browse_block.append_bindings(&mut bindings);
        self.browse_code_points.append_bindings(&mut bindings);
        self.help.append_bindings(&mut bindings);
        bindings
    }
}

macro_rules! keybinding_context {
    ($name:ident, $context:expr, { $($field:ident => ($command:expr, [$($key:literal),* $(,)?])),* $(,)? }) => {
        #[optional(
            derives = [Debug, Deserialize],
            attrs = [serde(deny_unknown_fields)]
        )]
        #[derive(Debug, Serialize, Validate)]
        struct $name {
            $(
                #[garde(inner(custom(validate_config_key)))]
                $field: Vec<String>,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field: vec![$($key.to_owned()),*],)*
                }
            }
        }

        impl $name {
            fn append_bindings(self, bindings: &mut Vec<Keybinding>) {
                $(
                    bindings.push(Keybinding {
                        context: $context,
                        command: $command,
                        keys: self.$field,
                    });
                )*
            }
        }
    };
}

keybinding_context!(GlobalKeybindings, Context::Global, {
    quit => (Command::Quit, ["ctrl-c"]),
    help => (Command::Help, ["f1"]),
});

keybinding_context!(InspectorKeybindings, Context::Inspector, {
    quit => (Command::Quit, ["q", "esc"]),
    previous_code_point => (Command::PreviousCodePoint, ["h", "left"]),
    next_code_point => (Command::NextCodePoint, ["l", "right"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    previous_group => (Command::PreviousGroup, ["["]),
    next_group => (Command::NextGroup, ["]"]),
    page_up => (Command::PageUp, ["ctrl-u"]),
    page_down => (Command::PageDown, ["ctrl-d"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    copy_value => (Command::CopyValue, ["y"]),
    search => (Command::OpenSearch, ["/"]),
    browse_planes => (Command::BrowsePlanes, ["p"]),
    browse_ranges => (Command::BrowseRanges, ["r"]),
    browse_blocks => (Command::BrowseBlocks, ["b"]),
    browse_code_points => (Command::BrowseCodePoints, ["c"]),
    back => (Command::Back, ["backspace"]),
});

keybinding_context!(SearchKeybindings, Context::Search, {
    previous_result => (Command::PreviousResult, ["up", "ctrl-p"]),
    next_result => (Command::NextResult, ["down", "ctrl-n"]),
    inspect_result => (Command::InspectResult, ["enter"]),
    close => (Command::Close, ["esc"]),
});

keybinding_context!(SequenceKeybindings, Context::Sequence, {
    quit => (Command::Quit, ["q", "esc"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    previous_group => (Command::PreviousGroup, ["["]),
    next_group => (Command::NextGroup, ["]"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    activate => (Command::Activate, ["enter"]),
    normalize => (Command::Normalize, ["n"]),
    open_copy_dialog => (Command::OpenCopyDialog, ["Y"]),
});

keybinding_context!(NormalizationKeybindings, Context::Normalization, {
    quit => (Command::Quit, ["q"]),
    close => (Command::Close, ["esc"]),
    back => (Command::Back, ["backspace"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    activate => (Command::Activate, ["enter"]),
    copy_value => (Command::CopyValue, ["y"]),
});

keybinding_context!(NormalizationResultKeybindings, Context::NormalizationResult, {
    quit => (Command::Quit, ["q"]),
    close => (Command::Close, ["esc"]),
    back => (Command::Back, ["backspace"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    previous_group => (Command::PreviousGroup, ["["]),
    next_group => (Command::NextGroup, ["]"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    activate => (Command::Activate, ["enter"]),
    open_copy_dialog => (Command::OpenCopyDialog, ["Y"]),
});

keybinding_context!(CopyDialogKeybindings, Context::CopyDialog, {
    close => (Command::Close, ["esc"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    first => (Command::First, ["g", "home"]),
    last => (Command::Last, ["G", "end"]),
    activate => (Command::Activate, ["enter"]),
});

keybinding_context!(BrowsePlaneKeybindings, Context::BrowsePlane, {
    quit => (Command::Quit, ["q"]),
    close => (Command::Close, ["esc"]),
    back => (Command::Back, ["backspace"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    activate => (Command::Activate, ["enter"]),
});

keybinding_context!(BrowseRangeKeybindings, Context::BrowseRange, {
    quit => (Command::Quit, ["q"]),
    close => (Command::Close, ["esc"]),
    back => (Command::Back, ["backspace"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    page_up => (Command::PageUp, ["ctrl-u"]),
    page_down => (Command::PageDown, ["ctrl-d"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    activate => (Command::Activate, ["enter"]),
});

keybinding_context!(BrowseBlockKeybindings, Context::BrowseBlock, {
    quit => (Command::Quit, ["q"]),
    close => (Command::Close, ["esc"]),
    back => (Command::Back, ["backspace"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    page_up => (Command::PageUp, ["ctrl-u"]),
    page_down => (Command::PageDown, ["ctrl-d"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    activate => (Command::Activate, ["enter"]),
});

#[rustfmt::skip]
keybinding_context!(BrowseCodePointsKeybindings, Context::BrowseCodePoints, {
    quit => (Command::Quit, ["q"]),
    close => (Command::Close, ["esc"]),
    back => (Command::Back, ["backspace"]),
    move_left => (Command::MoveLeft, ["h", "left"]),
    move_right => (Command::MoveRight, ["l", "right"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    page_up => (Command::PageUp, ["ctrl-u"]),
    page_down => (Command::PageDown, ["ctrl-d"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
    activate => (Command::Activate, ["enter"]),
});

keybinding_context!(HelpKeybindings, Context::Help, {
    close => (Command::Close, ["esc"]),
    move_up => (Command::MoveUp, ["k", "up"]),
    move_down => (Command::MoveDown, ["j", "down"]),
    page_up => (Command::PageUp, ["ctrl-u"]),
    page_down => (Command::PageDown, ["ctrl-d"]),
    first => (Command::First, ["g"]),
    last => (Command::Last, ["G"]),
});

#[derive(Debug)]
struct Keybinding {
    context: Context,
    command: Command,
    keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeybindingError {
    InvalidKey {
        context: Context,
        command: Command,
        key: String,
        source: KeyParseError,
    },
    DuplicateKey {
        context: Context,
        command: Command,
        key: String,
    },
    ContextConflict {
        context: Context,
        command: Command,
        other_command: Command,
        key: String,
    },
    GlobalConflict {
        context: Context,
        command: Command,
        global_command: Command,
        key: String,
    },
    SearchInputConflict {
        context: Context,
        command: Command,
        key: String,
    },
}

impl fmt::Display for KeybindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey {
                context,
                command,
                key,
                source,
            } => write!(
                formatter,
                "invalid key `{key}` for keybindings.{}.{}: {source}",
                context.config_name(),
                command.config_name()
            ),
            Self::DuplicateKey {
                context,
                command,
                key,
            } => write!(
                formatter,
                "keybindings.{}.{} contains duplicate key `{key}` after normalization",
                context.config_name(),
                command.config_name()
            ),
            Self::ContextConflict {
                context,
                command,
                other_command,
                key,
            } => write!(
                formatter,
                "key `{key}` for keybindings.{}.{} conflicts with keybindings.{}.{}",
                context.config_name(),
                command.config_name(),
                context.config_name(),
                other_command.config_name()
            ),
            Self::GlobalConflict {
                context,
                command,
                global_command,
                key,
            } => write!(
                formatter,
                "key `{key}` for keybindings.{}.{} conflicts with keybindings.global.{}",
                context.config_name(),
                command.config_name(),
                global_command.config_name()
            ),
            Self::SearchInputConflict {
                context,
                command,
                key,
            } => write!(
                formatter,
                "key `{key}` for keybindings.{}.{} is reserved for search input",
                context.config_name(),
                command.config_name()
            ),
        }
    }
}

impl Error for KeybindingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidKey { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyLabelStyle {
    Full,
    Compact,
}

pub fn format_key(key: KeyChord, style: KeyLabelStyle) -> String {
    if key.modifiers == KeyModifiers::SHIFT
        && matches!(key.code, KeyCode::Char(character) if character.is_ascii_uppercase())
    {
        return match key.code {
            KeyCode::Char(character) => character.to_string(),
            _ => unreachable!(),
        };
    }

    let mut label = String::new();
    for (modifier, full, compact) in [
        (KeyModifiers::CONTROL, "Ctrl-", "C-"),
        (KeyModifiers::ALT, "Alt-", "A-"),
        (KeyModifiers::SHIFT, "Shift-", "S-"),
    ] {
        if key.modifiers.contains(modifier) {
            label.push_str(match style {
                KeyLabelStyle::Full => full,
                KeyLabelStyle::Compact => compact,
            });
        }
    }

    label.push_str(&code_label(key.code, style));
    label
}

fn code_label(code: KeyCode, style: KeyLabelStyle) -> String {
    match (code, style) {
        (KeyCode::Char(' '), _) => "Space".to_owned(),
        (KeyCode::Char(character), _) => character.to_string(),
        (KeyCode::Enter, KeyLabelStyle::Full) => "Enter".to_owned(),
        (KeyCode::Enter, KeyLabelStyle::Compact) => "Enter".to_owned(),
        (KeyCode::Esc, KeyLabelStyle::Full) => "Esc".to_owned(),
        (KeyCode::Esc, KeyLabelStyle::Compact) => "Esc".to_owned(),
        (KeyCode::Tab, KeyLabelStyle::Full) => "Tab".to_owned(),
        (KeyCode::Tab, KeyLabelStyle::Compact) => "Tab".to_owned(),
        (KeyCode::Backspace, KeyLabelStyle::Full) => "Backspace".to_owned(),
        (KeyCode::Backspace, KeyLabelStyle::Compact) => "BS".to_owned(),
        (KeyCode::Delete, KeyLabelStyle::Full) => "Delete".to_owned(),
        (KeyCode::Delete, KeyLabelStyle::Compact) => "Del".to_owned(),
        (KeyCode::Left, KeyLabelStyle::Full) => "Left".to_owned(),
        (KeyCode::Left, KeyLabelStyle::Compact) => "←".to_owned(),
        (KeyCode::Right, KeyLabelStyle::Full) => "Right".to_owned(),
        (KeyCode::Right, KeyLabelStyle::Compact) => "→".to_owned(),
        (KeyCode::Up, KeyLabelStyle::Full) => "Up".to_owned(),
        (KeyCode::Up, KeyLabelStyle::Compact) => "↑".to_owned(),
        (KeyCode::Down, KeyLabelStyle::Full) => "Down".to_owned(),
        (KeyCode::Down, KeyLabelStyle::Compact) => "↓".to_owned(),
        (KeyCode::Home, _) => "Home".to_owned(),
        (KeyCode::End, _) => "End".to_owned(),
        (KeyCode::PageUp, KeyLabelStyle::Full) => "PageUp".to_owned(),
        (KeyCode::PageUp, KeyLabelStyle::Compact) => "PgUp".to_owned(),
        (KeyCode::PageDown, KeyLabelStyle::Full) => "PageDown".to_owned(),
        (KeyCode::PageDown, KeyLabelStyle::Compact) => "PgDn".to_owned(),
        (KeyCode::F(number), _) => format!("F{number}"),
        (other, _) => format!("{other:?}"),
    }
}

fn normalize(code: KeyCode, mut modifiers: KeyModifiers) -> (KeyCode, KeyModifiers) {
    let KeyCode::Char(character) = code else {
        return (code, modifiers);
    };

    if !character.is_ascii_alphabetic() {
        return (code, modifiers);
    }

    if modifiers.contains(KeyModifiers::SHIFT) {
        (KeyCode::Char(character.to_ascii_uppercase()), modifiers)
    } else if character.is_ascii_uppercase() {
        modifiers.insert(KeyModifiers::SHIFT);
        (KeyCode::Char(character), modifiers)
    } else {
        (KeyCode::Char(character), modifiers)
    }
}

#[derive(Debug)]
pub struct ResolvedKeymap {
    by_key: HashMap<(Context, KeyChord), Command>,
    by_command: HashMap<(Context, Command), Vec<KeyChord>>,
}

impl Default for ResolvedKeymap {
    fn default() -> Self {
        Self::with_config(Keybindings::default()).expect("built-in keybindings must be valid")
    }
}

impl ResolvedKeymap {
    pub fn with_config(config: Keybindings) -> Result<Self, KeybindingError> {
        let bindings = config
            .into_bindings()
            .into_iter()
            .map(parse_binding)
            .collect::<Result<Vec<_>, _>>()?;
        let mut keymap = Self {
            by_key: HashMap::new(),
            by_command: HashMap::new(),
        };

        for binding in bindings {
            keymap.add_binding(binding)?;
        }
        keymap.validate_global_conflicts()?;
        keymap.validate_search_input_conflicts()?;
        Ok(keymap)
    }

    pub fn resolve(&self, context: Context, key: KeyChord) -> Option<Command> {
        self.by_key.get(&(context, key)).copied()
    }

    pub fn keys_for(&self, context: Context, command: Command) -> &[KeyChord] {
        self.by_command
            .get(&(context, command))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn keys_for_active(&self, context: Context, command: Command) -> Vec<KeyChord> {
        let mut keys = self.keys_for(context, command).to_vec();
        if context != Context::Global {
            for key in self.keys_for(Context::Global, command) {
                if !keys.contains(key) {
                    keys.push(*key);
                }
            }
        }
        keys
    }

    fn add_binding(&mut self, binding: ParsedKeybinding) -> Result<(), KeybindingError> {
        for key in &binding.keys {
            if let Some(other_command) = self.by_key.get(&(binding.context, *key)) {
                return Err(KeybindingError::ContextConflict {
                    context: binding.context,
                    command: binding.command,
                    other_command: *other_command,
                    key: format_key(*key, KeyLabelStyle::Full),
                });
            }
        }
        for key in &binding.keys {
            self.by_key.insert((binding.context, *key), binding.command);
        }
        self.by_command
            .insert((binding.context, binding.command), binding.keys);
        Ok(())
    }

    fn validate_global_conflicts(&self) -> Result<(), KeybindingError> {
        for context in [
            Context::Inspector,
            Context::Search,
            Context::Sequence,
            Context::Normalization,
            Context::NormalizationResult,
            Context::CopyDialog,
            Context::BrowsePlane,
            Context::BrowseRange,
            Context::BrowseBlock,
            Context::BrowseCodePoints,
            Context::Help,
        ] {
            for global_command in ALL_COMMANDS {
                for key in self.keys_for(Context::Global, global_command) {
                    if let Some(command) = self.resolve(context, *key)
                        && command != global_command
                    {
                        return Err(KeybindingError::GlobalConflict {
                            context,
                            command,
                            global_command,
                            key: format_key(*key, KeyLabelStyle::Full),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_search_input_conflicts(&self) -> Result<(), KeybindingError> {
        for context in [Context::Global, Context::Search] {
            for command in ALL_COMMANDS {
                for key in self.keys_for(context, command) {
                    if to_input_request(&key.as_press_event()).is_some() {
                        return Err(KeybindingError::SearchInputConflict {
                            context,
                            command,
                            key: format_key(*key, KeyLabelStyle::Full),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ParsedKeybinding {
    context: Context,
    command: Command,
    keys: Vec<KeyChord>,
}

fn parse_binding(binding: Keybinding) -> Result<ParsedKeybinding, KeybindingError> {
    let mut parsed = Vec::with_capacity(binding.keys.len());
    for value in binding.keys {
        let key = parse_key(&value).map_err(|source| KeybindingError::InvalidKey {
            context: binding.context,
            command: binding.command,
            key: value.clone(),
            source,
        })?;
        if parsed.contains(&key) {
            return Err(KeybindingError::DuplicateKey {
                context: binding.context,
                command: binding.command,
                key: value,
            });
        }
        parsed.push(key);
    }
    Ok(ParsedKeybinding {
        context: binding.context,
        command: binding.command,
        keys: parsed,
    })
}

fn plain(character: char) -> KeyChord {
    KeyChord::new(KeyCode::Char(character), KeyModifiers::NONE)
}

#[cfg(test)]
fn ctrl(character: char) -> KeyChord {
    KeyChord::new(KeyCode::Char(character), KeyModifiers::CONTROL)
}

#[cfg(test)]
fn shift(character: char) -> KeyChord {
    KeyChord::new(KeyCode::Char(character), KeyModifiers::SHIFT)
}

fn named(code: KeyCode) -> KeyChord {
    KeyChord::new(code, KeyModifiers::NONE)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn configured(toml: &str) -> Result<ResolvedKeymap, KeybindingError> {
        let config: OptionalKeybindings = toml::from_str(toml).unwrap();
        ResolvedKeymap::with_config(config.into())
    }

    #[test]
    fn copy_bindings_are_customizable_without_leaking_into_search() {
        let keymap =
            configured("[sequence]\nopen_copy_dialog = ['C']\n[copy_dialog]\nactivate = ['y']")
                .unwrap();
        assert_eq!(
            keymap.resolve(Context::Sequence, plain('C')),
            Some(Command::OpenCopyDialog)
        );
        assert_eq!(keymap.resolve(Context::Sequence, plain('Y')), None);
        assert_eq!(
            keymap.resolve(Context::NormalizationResult, plain('Y')),
            Some(Command::OpenCopyDialog)
        );
        assert_eq!(
            keymap.resolve(Context::CopyDialog, plain('y')),
            Some(Command::Activate)
        );
        assert_eq!(
            keymap.resolve(Context::CopyDialog, named(KeyCode::Enter)),
            None
        );
        assert_eq!(keymap.resolve(Context::Search, plain('Y')), None);
        let error = configured("[copy_dialog]\nmove_down = ['ctrl-c']").unwrap_err();
        assert!(matches!(
            error,
            KeybindingError::GlobalConflict {
                context: Context::CopyDialog,
                command: Command::MoveDown,
                global_command: Command::Quit,
                ..
            }
        ));
    }

    #[test]
    fn defaults_support_forward_and_reverse_lookup() {
        let keymap = ResolvedKeymap::default();
        let down = KeyChord::new(KeyCode::Down, KeyModifiers::NONE);

        assert_eq!(
            keymap.resolve(Context::Inspector, down),
            Some(Command::MoveDown)
        );
        assert_eq!(
            keymap.keys_for(Context::Inspector, Command::MoveDown),
            &[plain('j'), down]
        );
    }

    #[test]
    fn block_browser_bindings_can_be_customized() {
        let keymap =
            configured("[inspector]\nbrowse_blocks = ['B']\n[browse_block]\nmove_down = ['n']")
                .unwrap();

        assert_eq!(
            keymap.resolve(Context::Inspector, plain('B')),
            Some(Command::BrowseBlocks)
        );
        assert_eq!(
            keymap.resolve(Context::BrowseBlock, plain('n')),
            Some(Command::MoveDown)
        );
    }

    #[test]
    fn defaults_prefer_vim_keys_for_directional_commands() {
        let keymap = ResolvedKeymap::default();

        for context in [
            Context::Inspector,
            Context::BrowsePlane,
            Context::BrowseRange,
            Context::BrowseBlock,
            Context::BrowseCodePoints,
            Context::Help,
        ] {
            assert_eq!(
                keymap.keys_for(context, Command::MoveUp).first(),
                Some(&plain('k'))
            );
            assert_eq!(
                keymap.keys_for(context, Command::MoveDown).first(),
                Some(&plain('j'))
            );
        }

        for (context, command, key) in [
            (Context::Inspector, Command::PreviousCodePoint, plain('h')),
            (Context::Inspector, Command::NextCodePoint, plain('l')),
            (Context::BrowseCodePoints, Command::MoveLeft, plain('h')),
            (Context::BrowseCodePoints, Command::MoveRight, plain('l')),
        ] {
            assert_eq!(keymap.keys_for(context, command).first(), Some(&key));
        }
    }

    #[test]
    fn active_reverse_lookup_prefers_context_keys_then_global_keys() {
        let keymap = ResolvedKeymap::default();

        assert_eq!(
            keymap.keys_for_active(Context::Inspector, Command::Quit),
            vec![plain('q'), named(KeyCode::Esc), ctrl('c')]
        );
        assert_eq!(
            keymap.keys_for_active(Context::Search, Command::Quit),
            vec![ctrl('c')]
        );
    }

    #[test]
    fn uppercase_characters_normalize_to_shifted_keys() {
        assert_eq!(
            KeyChord::new(KeyCode::Char('G'), KeyModifiers::NONE),
            KeyChord::new(KeyCode::Char('g'), KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn full_and_compact_labels_are_distinct() {
        assert_eq!(
            format_key(ctrl('u'), KeyLabelStyle::Full),
            "Ctrl-u".to_owned()
        );
        assert_eq!(
            format_key(ctrl('u'), KeyLabelStyle::Compact),
            "C-u".to_owned()
        );
        assert_eq!(format_key(shift('g'), KeyLabelStyle::Full), "G".to_owned());
        assert_eq!(
            format_key(shift('g'), KeyLabelStyle::Compact),
            "G".to_owned()
        );
    }

    #[rstest]
    #[case("a", plain('a'))]
    #[case("G", shift('g'))]
    #[case("space", named(KeyCode::Char(' ')))]
    #[case("pageup", named(KeyCode::PageUp))]
    #[case("f12", named(KeyCode::F(12)))]
    #[case("ctrl-alt-shift-x", KeyChord::new(KeyCode::Char('x'), KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT))]
    fn parses_supported_key_syntax(#[case] value: &str, #[case] expected: KeyChord) {
        assert_eq!(parse_key(value), Ok(expected));
    }

    #[rstest]
    #[case("")]
    #[case("unknown")]
    #[case("f13")]
    #[case("ctrl-")]
    #[case("ctrl-G")]
    #[case("ctrl-ctrl-a")]
    #[case("ctrl-enter")]
    fn rejects_unsupported_key_syntax(#[case] value: &str) {
        assert!(parse_key(value).is_err());
    }

    #[test]
    fn partial_overrides_replace_only_the_requested_command_and_preserve_order() {
        let keymap = configured(
            r#"
                [inspector]
                next_code_point = ["right", "n"]
            "#,
        )
        .unwrap();

        assert_eq!(
            keymap.keys_for(Context::Inspector, Command::NextCodePoint),
            &[named(KeyCode::Right), plain('n')]
        );
        assert_eq!(
            keymap.keys_for(Context::Inspector, Command::PreviousCodePoint),
            &[plain('h'), named(KeyCode::Left)]
        );
        assert_eq!(keymap.resolve(Context::Inspector, plain('l')), None);
    }

    #[test]
    fn inspector_back_binding_can_be_configured() {
        let keymap = configured(
            r#"
                [inspector]
                back = ["x"]
            "#,
        )
        .unwrap();

        assert_eq!(
            keymap.resolve(Context::Inspector, plain('x')),
            Some(Command::Back)
        );
        assert_eq!(
            keymap.resolve(Context::Inspector, named(KeyCode::Backspace)),
            None
        );
    }

    #[rstest]
    #[case(Context::Inspector)]
    #[case(Context::Sequence)]
    #[case(Context::NormalizationResult)]
    fn group_bindings_can_be_remapped_or_disabled_per_context(#[case] context: Context) {
        let keymap = configured(&format!(
            "[{}]\nprevious_group = ['a']\nnext_group = []\nmove_down = [']']",
            context.config_name(),
        ))
        .unwrap();
        assert_eq!(
            keymap.resolve(context, plain('a')),
            Some(Command::PreviousGroup)
        );
        assert_eq!(keymap.resolve(context, plain('[')), None);
        assert_eq!(keymap.resolve(context, plain(']')), Some(Command::MoveDown));
        assert!(keymap.keys_for(context, Command::NextGroup).is_empty());
        for other in [
            Context::Inspector,
            Context::Sequence,
            Context::NormalizationResult,
        ] {
            if other != context {
                assert_eq!(
                    keymap.resolve(other, plain('[')),
                    Some(Command::PreviousGroup)
                );
                assert_eq!(keymap.resolve(other, plain(']')), Some(Command::NextGroup));
            }
        }
    }

    #[test]
    fn conflicts_with_default_group_keys_identify_both_commands() {
        let error = configured("[sequence]\nmove_down = [']']").unwrap_err();
        let message = error.to_string();
        assert!(message.contains("keybindings.sequence.move_down"));
        assert!(message.contains("keybindings.sequence.next_group"));
    }

    #[test]
    fn an_empty_array_disables_only_the_requested_context_binding() {
        let keymap = configured(
            r#"
                [inspector]
                quit = []
            "#,
        )
        .unwrap();

        assert!(
            keymap
                .keys_for(Context::Inspector, Command::Quit)
                .is_empty()
        );
        assert_eq!(
            keymap.keys_for(Context::Global, Command::Quit),
            &[ctrl('c')]
        );
    }

    #[test]
    fn multiple_overrides_can_swap_default_keys() {
        let keymap = configured(
            r#"
                [inspector]
                move_up = ["j"]
                move_down = ["k"]
            "#,
        )
        .unwrap();

        assert_eq!(
            keymap.resolve(Context::Inspector, plain('j')),
            Some(Command::MoveUp)
        );
        assert_eq!(
            keymap.resolve(Context::Inspector, plain('k')),
            Some(Command::MoveDown)
        );
    }

    #[test]
    fn normalized_duplicates_are_rejected() {
        let error = configured(
            r#"
                [inspector]
                last = ["G", "shift-g"]
            "#,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            KeybindingError::DuplicateKey {
                context: Context::Inspector,
                command: Command::Last,
                ..
            }
        ));
    }

    #[test]
    fn conflicts_with_another_command_in_the_same_context_are_rejected() {
        let error = configured(
            r#"
                [inspector]
                next_code_point = ["h"]
            "#,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            KeybindingError::ContextConflict {
                context: Context::Inspector,
                command: Command::NextCodePoint,
                other_command: Command::PreviousCodePoint,
                ..
            }
        ));
    }

    #[test]
    fn conflicts_between_global_and_reachable_contexts_are_rejected() {
        let error = configured(
            r#"
                [global]
                help = ["esc"]
            "#,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            KeybindingError::GlobalConflict {
                context: Context::Inspector,
                command: Command::Quit,
                global_command: Command::Help,
                ..
            }
        ));
    }

    #[test]
    fn keys_used_by_search_input_are_rejected_in_search_and_global() {
        for source in [
            "[search]\nnext_result = [\"x\"]\n",
            "[global]\nhelp = [\"delete\"]\n",
        ] {
            assert!(matches!(
                configured(source).unwrap_err(),
                KeybindingError::SearchInputConflict { .. }
            ));
        }
    }

    #[test]
    fn mutually_exclusive_contexts_can_reuse_a_key() {
        configured(
            r#"
                [inspector]
                next_code_point = ["x"]

                [browse_plane]
                activate = ["x"]
            "#,
        )
        .unwrap();
    }

    #[test]
    fn unknown_contexts_and_commands_are_rejected_during_deserialization() {
        for source in [
            "[unknown]\nquit = [\"q\"]\n",
            "[inspector]\nquitt = [\"q\"]\n",
        ] {
            let error = toml::from_str::<OptionalKeybindings>(source).unwrap_err();

            assert!(error.to_string().contains("unknown field"));
        }
    }
}
