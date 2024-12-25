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
    constants::{card, title},
};

pub struct CardPage {
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: PageKeyBindings,
}

impl CardPage {
    pub fn new() -> Self {
        CardPage { action_tx: None, keymap: PageKeyBindings::default() }
    }

    pub fn up(&mut self) {
    }

    pub fn down(&mut self) {
    }

    pub fn draw_card(&self, f: &mut Frame<'_>, area: Rect, lines: Vec<&str>) -> Result<()> {
        let width = lines.iter().map(|s| s.chars().count() * 2).max().unwrap_or(0) as u16 + card::CARD_HPADDING * 2;
        let height = lines.len() as u16 + card::CARD_VPADDING * 2;

        let lines = lines.into_iter().map(|s| Line::from(s)).collect::<Vec<_>>();
        let [_, area, _] =
            Layout::horizontal(vec![Constraint::Fill(1), Constraint::Length(width), Constraint::Fill(1)]).areas(area);
        let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded);
        f.render_widget(block, area);

        let area = area.inner(Margin { horizontal: card::CARD_HPADDING, vertical: card::CARD_VPADDING });

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(paragraph, area);

        Ok(())
    }
}

impl Page for CardPage {
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
        let title_lines: Vec<&str> = card::CONGRAT_TEXT.lines().filter(|s| s.len() != 0).collect();
        let shadow_lines: Vec<&str> = card::CONGRAT_SHADOW.lines().filter(|s| s.len() != 0).collect();
        let num_title_lines = title_lines.len().max(shadow_lines.len()) as u16;

        // let num_options = self.options.len() as u16;
        // let option_height = num_options * 2 - 1;
        let card_lines: Vec<&str> = card::CARD_TEXT.lines().filter(|s| s.len() != 0).collect();
        let num_card_lines = card_lines.len().max(shadow_lines.len()) as u16 + card::CARD_VPADDING * 2;

        let [title_area, card_area] =
            Layout::vertical(vec![Constraint::Length(num_title_lines), Constraint::Length(num_card_lines)])
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

        // Draw card
        self.draw_card(f, card_area, card_lines)?;

        Ok(())
    }

    fn pause(&mut self) {
    }

    fn resume(&mut self) {
    }
}
