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
    components::{
        background::{Background, BackgroundState},
        multiline::MultiLine,
    },
    config::{key_event_to_string, PageKeyBindings},
};

pub struct GamePage {
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: PageKeyBindings,
    // background_state: BackgroundState,
}

impl GamePage {
    pub fn new() -> Self {
        GamePage {
            action_tx: None,
            keymap: PageKeyBindings::default(),
            // background_state: BackgroundState::new(2.0, 1.0 / 30.0).show_tree().show_snowman(),
        }
    }
}

impl Page for GamePage {
    fn id(&self) -> PageId {
        PageId::Game
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
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action.command {
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Draw background
        // let background = Background::default();
        // f.render_stateful_widget(background, area, &mut self.background_state);

        Ok(())
    }
}
