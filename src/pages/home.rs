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
    config::{key_event_to_string, PageKeyBindings},
    constants::TITLE_TEXT,
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum OptionItem {
    Start,
}

#[derive(Builder)]
pub struct HomePage {
    #[builder(default)]
    pub action_tx: Option<UnboundedSender<Action>>,
    #[builder(default)]
    pub keymap: PageKeyBindings,
    options: Vec<(OptionItem, &'static str)>,
    selected_option_index: usize,
}

impl HomePage {
    pub fn new() -> Self {
        HomePageBuilder::default()
            .options(vec![(OptionItem::Start, "Start playing"), (OptionItem::Start, "Start playing")])
            .selected_option_index(0)
            .build()
            .unwrap()
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
            }
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        f.render_widget(Clear, rect);

        let title_lines: Vec<&str> = TITLE_TEXT.lines().filter(|s| s.len() != 0).collect();
        let num_title_lines = title_lines.len() as u16;

        let num_options = self.options.len() as u16;
        let option_height = num_options * 2 - 1;

        let [title_area, option_area] =
            Layout::vertical(vec![Constraint::Length(num_title_lines), Constraint::Length(option_height)])
                .flex(layout::Flex::SpaceAround)
                .areas(rect);

        // Draw title
        let lines = title_lines.iter().map(|line| Line::from(*line)).collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines).style(Style::default().fg(Color::Red)).alignment(Alignment::Center);
        f.render_widget(paragraph, title_area);

        // Draw options
        let option_titles = self.options.iter().map(|(_, title)| *title).collect::<Vec<_>>();
        let max_option_len = option_titles.iter().map(|title| title.len()).max().unwrap_or(0) as u16;

        // Pad option titles
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
            .areas(option_area);

        let lines = option_titles
            .iter()
            .enumerate()
            .map(|(index, title)| {
                Line::from(title.as_str()).style({
                    if index == self.selected_option_index {
                        Style::default().bg(Color::Cyan)
                    } else {
                        Style::default()
                    }
                })
            })
            .collect::<Vec<_>>();
        // Insert empty lines
        let lines = {
            let len = lines.len();
            let mut new_lines = vec![];
            for (index, line) in lines.into_iter().enumerate() {
                new_lines.push(line);
                if index < len - 1 {}
                new_lines.push(Line::from(""));
            }
            new_lines
        };

        let paragraph = Paragraph::new(lines).style(Style::default().fg(Color::White)).alignment(Alignment::Left);
        f.render_widget(paragraph, option_area);

        Ok(())
    }
}
