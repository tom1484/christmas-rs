use std::collections::HashMap;

use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};
use serde::Deserialize;
use strum::Display;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    config::{Config, PageKeyBindings},
    tui::Event,
};

pub mod card;
pub mod game;
pub mod home;

#[derive(Debug, Deserialize, Hash, Eq, PartialEq, Clone, Display)]
pub enum PageId {
    Home,
    Game,
    Card,
}

pub trait Page {
    fn id(&self) -> PageId;

    fn register_keymap(&mut self, keymaps: &HashMap<PageId, PageKeyBindings>) -> Result<()>;

    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        Ok(())
    }
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }

    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()>;
}
