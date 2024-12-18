use std::{collections::HashMap, fmt, fmt::Debug, path::PathBuf};

use color_eyre::eyre::Result;
use config::Value;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use ratatui::style::{Color, Modifier, Style};
use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize, Serialize,
};
use serde_json::Value as JsonValue;

use crate::{
    action::{self, Action, ActionState, CardAction, Command, GameAction, HomeAction},
    app::Mode,
    pages::{game, home, Page, PageId},
};

const CONFIG: &str = include_str!("../.config/config.yaml");

parse_and_map_actions![
    (PageId::Home, HomeAction, Command::Home),
    (PageId::Game, GameAction, Command::Game),
    (PageId::Card, CardAction, Command::Card)
];

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub _data_dir: PathBuf,
    #[serde(default)]
    pub _config_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
    #[serde(default)]
    pub keybindings: KeyBindings,
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        let default_config: Config = serde_yaml::from_str(CONFIG).unwrap();
        let data_dir = crate::utils::get_data_dir();
        let config_dir = crate::utils::get_config_dir();
        let mut builder = config::Config::builder()
            .set_default("_data_dir", data_dir.to_str().unwrap())?
            .set_default("_config_dir", config_dir.to_str().unwrap())?;

        let config_files = [("config.yaml", config::FileFormat::Yaml)];
        let mut found_config = false;
        for (file, format) in &config_files {
            builder = builder.add_source(config::File::from(config_dir.join(file)).format(*format).required(false));
            if config_dir.join(file).exists() {
                found_config = true
            }
        }
        if !found_config {
            log::error!("No configuration file found. Application may not behave as expected");
        }

        let mut cfg: Self = builder.build()?.try_deserialize()?;
        for (scope, default_bindings) in default_config.keybindings.pages.iter() {
            let user_bindings = cfg.keybindings.pages.entry(scope.clone()).or_default();
            for (key, cmd) in default_bindings.0.iter() {
                user_bindings.0.entry(key.clone()).or_insert_with(|| cmd.clone());
            }
        }
        let user_bindings = &mut cfg.keybindings.global;
        for (key, cmd) in default_config.keybindings.global.0.iter() {
            user_bindings.0.entry(key.clone()).or_insert_with(|| cmd.clone());
        }

        Ok(cfg)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct _RawPageKeyBindings {
    pub click: Option<HashMap<String, String>>,
    pub hold: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct _RawKeyBindings {
    pub global: _RawPageKeyBindings,
    pub pages: HashMap<String, _RawPageKeyBindings>,
}

#[derive(Clone, Debug, Default)]
pub struct PageKeyBindings(pub HashMap<KeyEvent, Action>);

#[derive(Clone, Debug, Default)]
pub struct KeyBindings {
    pub global: PageKeyBindings,
    pub pages: HashMap<PageId, PageKeyBindings>,
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_keybindings: _RawKeyBindings = _RawKeyBindings::deserialize(deserializer)?;

        let global_keybindings = PageKeyBindings(
            parse_global_keybindings(raw_keybindings.global)
                .into_iter()
                .map(|(event, (command, state))| (event, Action { command, state }))
                .collect(),
        );
        let page_keybindings = raw_keybindings
            .pages
            .into_iter()
            .map(|(page, keybindings)| match_page_keybindings(&page, keybindings))
            .collect();

        Ok(KeyBindings { global: global_keybindings, pages: page_keybindings })
    }
}

#[macro_export]
macro_rules! parse_and_map_actions {
    ( $( ( $page_id_variant:path, $action_type:ty, $action_variant:path ) ),* ) => {
        fn match_page_keybindings(page: &str, raw_page_keybindings: _RawPageKeyBindings) -> (PageId, PageKeyBindings) {
            let page_id: PageId = serde_yaml::from_str(page).unwrap();
            let map = match page_id {
                $(
                    $page_id_variant => {
                        parse_page_keybindings::<$action_type>(&raw_page_keybindings)
                            .into_iter()
                            .map(|(event, (command, state))| (event, Action{command: $action_variant(command), state}))
                            .collect()
                    },
                )*
            };

            (page_id, PageKeyBindings(map))
        }
    };
}

use parse_and_map_actions;

fn parse_binding_pairs(map: &Option<HashMap<String, String>>) -> HashMap<KeyEvent, Command> {
    if let Some(inner_map) = map {
        inner_map
            .iter()
            .map(|(key_str, action_str)| {
                let action: Command = serde_yaml::from_str(action_str).unwrap();
                (parse_key_event(key_str).unwrap(), action)
            })
            .collect()
    } else {
        HashMap::new()
    }
}

fn parse_global_keybindings(raw_keybindings: _RawPageKeyBindings) -> HashMap<KeyEvent, (Command, ActionState)> {
    let extract = |map: &Option<HashMap<String, String>>| {
        if let Some(inner_map) = map {
            inner_map
                .iter()
                .map(|(key_str, action_str)| {
                    let action: Command = serde_yaml::from_str(action_str).unwrap();
                    (parse_key_event(key_str).unwrap(), action)
                })
                .collect()
        } else {
            HashMap::new()
        }
    };

    let keybindings = parse_binding_pairs(&raw_keybindings.click);
    let hold_keybindings = parse_binding_pairs(&raw_keybindings.hold);

    merge_keybinding_maps(keybindings, hold_keybindings)
}

fn parse_page_keybindings<'de, T>(raw_page_keybindings: &'de _RawPageKeyBindings) -> HashMap<KeyEvent, (T, ActionState)>
where
    T: Deserialize<'de> + Clone + Debug,
{
    let extract = |map: &'de Option<HashMap<String, String>>| {
        if let Some(inner_map) = map {
            inner_map
                .iter()
                .map(|(key_str, action_str)| {
                    let action: T = serde_yaml::from_str(&action_str).unwrap();
                    (parse_key_event(key_str).unwrap(), action)
                })
                .collect()
        } else {
            HashMap::new()
        }
    };

    let click_keybindings = extract(&raw_page_keybindings.click);
    let hold_keybindings = extract(&raw_page_keybindings.hold);

    merge_keybinding_maps(click_keybindings, hold_keybindings)
}

fn merge_keybinding_maps<A: Clone>(
    click: HashMap<KeyEvent, A>,
    hold: HashMap<KeyEvent, A>,
) -> HashMap<KeyEvent, (A, ActionState)> {
    let mut click: HashMap<KeyEvent, (A, ActionState)> =
        click.into_iter().map(|(event, command)| (event, (command, ActionState::default()))).collect();
    for (mut key_event, action) in hold {
        click.insert(key_event.clone(), (action.clone(), ActionState::Start));
        key_event.kind = KeyEventKind::Release;
        click.insert(key_event.clone(), (action.clone(), ActionState::End));
        key_event.kind = KeyEventKind::Repeat;
        click.insert(key_event.clone(), (action.clone(), ActionState::Repeat));
    }

    click
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
        return Err(format!("Unable to parse `{}`", raw));
    }
    let raw = if !raw.contains("><") {
        let raw = raw.strip_prefix('<').unwrap_or(raw);
        let raw = raw.strip_prefix('>').unwrap_or(raw);
        raw
    } else {
        raw
    };
    let raw = raw.trim_start_matches('<').trim_end_matches('>');

    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            },
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            },
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            },
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(raw: &str, mut modifiers: KeyModifiers) -> Result<KeyEvent, String> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        },
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        },
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

pub fn key_event_to_string(key_event: &KeyEvent) -> String {
    let char;
    let key_code = match key_event.code {
        KeyCode::Backspace => "Backspace",
        KeyCode::Enter => "Enter",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Tab => "Tab",
        KeyCode::BackTab => "BackTab",
        KeyCode::Delete => "Delete",
        KeyCode::Insert => "Insert",
        KeyCode::F(c) => {
            char = format!("F({c})");
            &char
        },
        KeyCode::Char(c) if c == ' ' => "Space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        },
        KeyCode::Esc => "Esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        modifiers.push("Ctrl");
    }

    if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
        modifiers.push("Shift");
    }

    if key_event.modifiers.intersects(KeyModifiers::ALT) {
        modifiers.push("Alt");
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Styles(pub HashMap<Mode, HashMap<String, Style>>);

impl<'de> Deserialize<'de> for Styles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<Mode, HashMap<String, String>>::deserialize(deserializer)?;

        let styles = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map =
                    inner_map.into_iter().map(|(str, style)| (str, parse_style(&style))).collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(Styles(styles))
    }
}

pub fn parse_style(line: &str) -> Style {
    let (foreground, background) = line.split_at(line.to_lowercase().find("on ").unwrap_or(line.len()));
    let foreground = process_color_string(foreground);
    let background = process_color_string(&background.replace("on ", ""));

    let mut style = Style::default();
    if let Some(fg) = parse_color(&foreground.0) {
        style = style.fg(fg);
    }
    if let Some(bg) = parse_color(&background.0) {
        style = style.bg(bg);
    }
    style = style.add_modifier(foreground.1 | background.1);
    style
}

fn process_color_string(color_str: &str) -> (String, Modifier) {
    let color = color_str
        .replace("grey", "gray")
        .replace("bright ", "")
        .replace("bold ", "")
        .replace("underline ", "")
        .replace("inverse ", "");

    let mut modifiers = Modifier::empty();
    if color_str.contains("underline") {
        modifiers |= Modifier::UNDERLINED;
    }
    if color_str.contains("bold") {
        modifiers |= Modifier::BOLD;
    }
    if color_str.contains("inverse") {
        modifiers |= Modifier::REVERSED;
    }

    (color, modifiers)
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim_start();
    let s = s.trim_end();
    if s.contains("bright color") {
        let s = s.trim_start_matches("bright ");
        let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
        Some(Color::Indexed(c.wrapping_shl(8)))
    } else if s.contains("color") {
        let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
        Some(Color::Indexed(c))
    } else if s.contains("gray") {
        let c = 232 + s.trim_start_matches("gray").parse::<u8>().unwrap_or_default();
        Some(Color::Indexed(c))
    } else if s.contains("rgb") {
        let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
        let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
        let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
        let c = 16 + red * 36 + green * 6 + blue;
        Some(Color::Indexed(c))
    } else if s == "bold black" {
        Some(Color::Indexed(8))
    } else if s == "bold red" {
        Some(Color::Indexed(9))
    } else if s == "bold green" {
        Some(Color::Indexed(10))
    } else if s == "bold yellow" {
        Some(Color::Indexed(11))
    } else if s == "bold blue" {
        Some(Color::Indexed(12))
    } else if s == "bold magenta" {
        Some(Color::Indexed(13))
    } else if s == "bold cyan" {
        Some(Color::Indexed(14))
    } else if s == "bold white" {
        Some(Color::Indexed(15))
    } else if s == "black" {
        Some(Color::Indexed(0))
    } else if s == "red" {
        Some(Color::Indexed(1))
    } else if s == "green" {
        Some(Color::Indexed(2))
    } else if s == "yellow" {
        Some(Color::Indexed(3))
    } else if s == "blue" {
        Some(Color::Indexed(4))
    } else if s == "magenta" {
        Some(Color::Indexed(5))
    } else if s == "cyan" {
        Some(Color::Indexed(6))
    } else if s == "white" {
        Some(Color::Indexed(7))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_style_default() {
        let style = parse_style("");
        assert_eq!(style, Style::default());
    }

    #[test]
    fn test_parse_style_foreground() {
        let style = parse_style("red");
        assert_eq!(style.fg, Some(Color::Indexed(1)));
    }

    #[test]
    fn test_parse_style_background() {
        let style = parse_style("on blue");
        assert_eq!(style.bg, Some(Color::Indexed(4)));
    }

    #[test]
    fn test_parse_style_modifiers() {
        let style = parse_style("underline red on blue");
        assert_eq!(style.fg, Some(Color::Indexed(1)));
        assert_eq!(style.bg, Some(Color::Indexed(4)));
    }

    #[test]
    fn test_process_color_string() {
        let (color, modifiers) = process_color_string("underline bold inverse gray");
        assert_eq!(color, "gray");
        assert!(modifiers.contains(Modifier::UNDERLINED));
        assert!(modifiers.contains(Modifier::BOLD));
        assert!(modifiers.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_parse_color_rgb() {
        let color = parse_color("rgb123");
        let expected = 16 + 1 * 36 + 2 * 6 + 3;
        assert_eq!(color, Some(Color::Indexed(expected)));
    }

    #[test]
    fn test_parse_color_unknown() {
        let color = parse_color("unknown");
        assert_eq!(color, None);
    }

    #[test]
    fn test_config() -> Result<()> {
        let c = Config::new()?;
        println!("{:?}", c.keybindings.pages.get(&PageId::Home));
        // assert_eq!(
        //     c.keybindings.pages.get("Game").unwrap().0.get(&parse_key_event("<b>").unwrap()).unwrap(),
        //     &Action::Game(GameAction::Right)
        // );
        Ok(())
    }

    #[test]
    fn test_simple_keys() {
        assert_eq!(parse_key_event("a").unwrap(), KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

        assert_eq!(parse_key_event("enter").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

        assert_eq!(parse_key_event("esc").unwrap(), KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    }

    #[test]
    fn test_with_modifiers() {
        assert_eq!(parse_key_event("ctrl-a").unwrap(), KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));

        assert_eq!(parse_key_event("alt-enter").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));

        assert_eq!(parse_key_event("shift-esc").unwrap(), KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT));
    }

    #[test]
    fn test_multiple_modifiers() {
        assert_eq!(
            parse_key_event("ctrl-alt-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT)
        );

        assert_eq!(
            parse_key_event("ctrl-shift-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_reverse_multiple_modifiers() {
        assert_eq!(
            key_event_to_string(&KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT)),
            "ctrl-alt-a".to_string()
        );
    }

    #[test]
    fn test_invalid_keys() {
        assert!(parse_key_event("invalid-key").is_err());
        assert!(parse_key_event("ctrl-invalid-key").is_err());
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(parse_key_event("CTRL-a").unwrap(), KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));

        assert_eq!(parse_key_event("AlT-eNtEr").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    }
}
