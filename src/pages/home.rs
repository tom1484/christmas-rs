use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use derive_builder::Builder;
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Frame, Page, PageId};
use crate::{
    action::{act, Action, ActionState, Command, HomeAction},
    components::{
        background::{Background, BackgroundState},
        multiline::MultiLine,
    },
    config::{key_event_to_string, PageKeyBindings},
    constants::title::{TITLE_SHADOW, TITLE_TEXT},
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum OptionItem {
    Start,
}

pub struct HomePage {
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: PageKeyBindings,
    options: Vec<(OptionItem, &'static str)>,
    selected_option_index: usize,
    background_state: BackgroundState,
}

impl HomePage {
    pub fn new() -> Self {
        HomePage {
            action_tx: None,
            keymap: PageKeyBindings::default(),
            options: vec![(OptionItem::Start, "Start playing")],
            selected_option_index: 0,
            background_state: BackgroundState::new(2.0, 1.0 / 30.0).show_tree().show_snowman(),
        }
    }

    pub fn up(&mut self) {
        if self.selected_option_index < self.options.len() - 1 {
            self.selected_option_index += 1;
        }
    }

    pub fn down(&mut self) {
        if self.selected_option_index > 0 {
            self.selected_option_index -= 1;
        }
    }

    fn draw_options(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Draw options
        let option_titles = self.options.iter().map(|(_, title)| *title).collect::<Vec<_>>();
        let max_option_len = option_titles.iter().map(|title| title.len()).max().unwrap_or(0) as u16;
        let num_option_titles = option_titles.len();

        let option_titles = option_titles
            .into_iter()
            .map(|title| {
                let title = title.to_string();
                let pad_len = max_option_len as usize - title.len();
                let front_pad = vec![' '; 2].into_iter().collect::<String>();
                let back_pad = vec![' '; pad_len + 2].into_iter().collect::<String>();
                [front_pad, title, back_pad].into_iter().collect::<String>()
            })
            .collect::<Vec<_>>();

        let [option_area] = Layout::horizontal(vec![Constraint::Length(max_option_len + (2 * 2))])
            .flex(layout::Flex::SpaceAround)
            .areas(area);

        let lines = MultiLine::new(option_titles).line_padding(1).line_styles(
            (0..num_option_titles)
                .into_iter()
                .map(|index| {
                    let selected = index == self.selected_option_index;
                    match selected {
                        true => Style::default().fg(Color::Black).bg(Color::Blue),
                        false => Style::default().fg(Color::White).bg(Color::DarkGray),
                    }
                })
                .collect(),
        );
        f.render_widget(lines, option_area);

        Ok(())
    }
}

impl Page for HomePage {
    fn id(&self) -> PageId {
        PageId::Home
    }

    fn register_keymap(&mut self, keymaps: &HashMap<PageId, PageKeyBindings>) -> Result<()> {
        if let Some(keymap) = keymaps.get(&self.id()) {
            self.keymap = keymap.clone();
        }
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // TODO: Handle keymap
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Command::Home(command) = action.command {
            match command {
                HomeAction::Up => self.up(),
                HomeAction::Down => self.down(),
                HomeAction::Select => {
                    if let Some(action_tx) = &self.action_tx {
                        action_tx.send(act!(Command::StartGame))?;
                    }
                },
            }
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Draw background
        let background = Background::default();
        f.render_stateful_widget(background, area, &mut self.background_state);

        let title_lines: Vec<&str> = TITLE_TEXT.lines().filter(|s| s.len() != 0).collect();
        let shadow_lines: Vec<&str> = TITLE_SHADOW.lines().filter(|s| s.len() != 0).collect();

        let num_title_lines = title_lines.len().max(shadow_lines.len()) as u16;

        let num_options = self.options.len() as u16;
        let option_height = num_options * 2 - 1;

        let [title_area, option_area] =
            Layout::vertical(vec![Constraint::Length(num_title_lines), Constraint::Length(option_height)])
                .flex(layout::Flex::SpaceAround)
                .areas(area);

        // Draw title
        let title_width = title_lines.iter().map(|line| line.chars().count()).max().unwrap_or(0) as u16;
        let shadow_width = shadow_lines.iter().map(|line| line.chars().count()).max().unwrap_or(0) as u16;
        let width = title_width.max(shadow_width);

        let [_, title_area, _] =
            Layout::horizontal(vec![Constraint::Fill(1), Constraint::Length(width), Constraint::Fill(1)])
                .flex(layout::Flex::SpaceAround)
                .areas(title_area);

        let shadow_lines =
            MultiLine::new(shadow_lines).ignore_whitespace(true).pixel_mode().style(Style::default().fg(Color::Green));
        f.render_widget(shadow_lines, title_area);

        let title_lines =
            MultiLine::new(title_lines).ignore_whitespace(true).pixel_mode().style(Style::default().fg(Color::Red));
        f.render_widget(title_lines, title_area);

        // Draw options
        self.draw_options(f, option_area)?;

        Ok(())
    }
}
