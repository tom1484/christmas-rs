use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    action::Action,
    config::Config,
    pages::{game::GamePage, title::TitlePage, Page},
    tui,
};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

pub struct App {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub pages: Vec<Box<dyn Page>>,
    pub active_page: usize,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub mode: Mode,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let title_page = TitlePage::new();
        let game_page = GamePage::new();

        let config = Config::new()?;
        let mode = Mode::Home;
        Ok(Self {
            tick_rate,
            frame_rate,
            // pages: vec![Box::new(title_page), Box::new(game_page)],
            pages: vec![Box::new(game_page)],
            active_page: 0,
            should_quit: false,
            should_suspend: false,
            config,
            mode,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()?;
        tui.tick_rate(self.tick_rate);
        tui.frame_rate(self.frame_rate);
        tui.enter()?;

        for page in self.pages.iter_mut() {
            page.register_action_handler(action_tx.clone())?;
        }

        for page in self.pages.iter_mut() {
            page.register_config_handler(self.config.clone())?;
        }

        for page in self.pages.iter_mut() {
            page.init()?;
        }

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        // if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                        if let Some(keymap) = self.config.keybindings.pages.get("HOME") {
                            // if let Some(action) = keymap.get(&key.into()) {
                            //     log::info!("Got action: {action:?}");
                            //     action_tx.send(action.clone())?;
                            // }
                        };
                    },
                    _ => {},
                }
                if let Some(page) = self.pages.get_mut(self.active_page) {
                    if let Some(action) = page.handle_events(Some(e.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {},
                    Action::Quit => self.should_quit = true,
                    // Action::SwitchPage => self.active_page = 1 - self.active_page,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            if let Some(page) = self.pages.get_mut(self.active_page) {
                                let r = page.draw(f, f.area());
                                if let Err(e) = r {
                                    action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                                }
                            }
                        })?;
                    },
                    Action::Render => {
                        tui.draw(|f| {
                            if let Some(page) = self.pages.get_mut(self.active_page) {
                                let r = page.draw(f, f.area());
                                if let Err(e) = r {
                                    action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                                }
                            }
                        })?;
                    },
                    _ => {},
                }
                if let Some(page) = self.pages.get_mut(self.active_page) {
                    if let Some(action) = page.update(action.clone())? {
                        action_tx.send(action)?
                    };
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new()?;
                tui.tick_rate(self.tick_rate);
                tui.frame_rate(self.frame_rate);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
