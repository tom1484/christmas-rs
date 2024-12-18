use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Frame, Page, PageId};
use crate::{
    action::{act, Action, ActionState, Command},
    config::{key_event_to_string, PageKeyBindings},
};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Processing,
}

#[derive(Default)]
pub struct HomePage {
    pub show_help: bool,
    pub counter: usize,
    pub app_ticker: usize,
    pub render_ticker: usize,
    pub mode: Mode,
    pub input: Input,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: PageKeyBindings,
}

impl HomePage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        log::info!("Tick");
        self.app_ticker = self.app_ticker.saturating_add(1);
    }

    pub fn render_tick(&mut self) {
        log::debug!("Render Tick");
        self.render_ticker = self.render_ticker.saturating_add(1);
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
        // let command = match self.mode {
        // };
        // Ok(Some(act!(command)))
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action.command {
            Command::Tick => self.tick(),
            Command::Render => self.render_tick(),
            Command::ToggleShowHelp => self.show_help = !self.show_help,
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let rects =
            Layout::default().constraints([Constraint::Percentage(100), Constraint::Min(3)].as_ref()).split(rect);

        let width = rects[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(match self.mode {
                Mode::Insert => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, scroll as u16))
            .block(Block::default().borders(Borders::ALL).title(Line::from(vec![
                Span::raw("Enter Input Mode "),
                Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
                Span::styled("/", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
                Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
                Span::styled(" to finish)", Style::default().fg(Color::DarkGray)),
            ])));
        f.render_widget(input, rects[1]);
        if self.mode == Mode::Insert {
            f.set_cursor_position((
                (rects[1].x + 1 + self.input.cursor() as u16).min(rects[1].x + rects[1].width - 2),
                rects[1].y + 1,
            ))
        }

        // if self.show_help {
        //     let rect = rect.inner(Margin { horizontal: 4, vertical: 2 });
        //     f.render_widget(Clear, rect);
        //     let block = Block::default()
        //         .title(Line::from(vec![Span::styled("Key Bindings", Style::default().add_modifier(Modifier::BOLD))]))
        //         .borders(Borders::ALL)
        //         .border_style(Style::default().fg(Color::Yellow));
        //     f.render_widget(block, rect);
        //     let rows = vec![
        //         Row::new(vec!["j", "Increment"]),
        //         Row::new(vec!["k", "Decrement"]),
        //         Row::new(vec!["/", "Enter Input"]),
        //         Row::new(vec!["ESC", "Exit Input"]),
        //         Row::new(vec!["Enter", "Submit Input"]),
        //         Row::new(vec!["q", "Quit"]),
        //         Row::new(vec!["?", "Open Help"]),
        //     ];
        //     let widths = vec![Constraint::Percentage(100); rows.len()];
        //     let table = Table::new(rows, widths)
        //         .header(
        //             Row::new(vec!["Key", "Action"])
        //                 .bottom_margin(1)
        //                 .style(Style::default().add_modifier(Modifier::BOLD)),
        //         )
        //         .widths(&[Constraint::Percentage(10), Constraint::Percentage(90)])
        //         .column_spacing(1);
        //     f.render_widget(table, rect.inner(Margin { vertical: 4, horizontal: 2 }));
        // };

        Ok(())
    }
}
