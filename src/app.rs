use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Margin},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Row, Table},
    Frame,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::{
    action::{self, act, Action, ActionState, Command},
    components::{
        background::{Background, BackgroundState},
        help::Help,
    },
    config::Config,
    constants::{home, HEIGHT, WIDTH},
    pages::{card::CardPage, game::GamePage, home::HomePage, Page, PageId},
    tui,
};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    should_quit: bool,
    should_suspend: bool,
    show_help: bool,
    pages: Vec<Box<dyn Page>>,
    active_page_index: usize,
    background_state: BackgroundState,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let config = Config::new()?;

        let page_keybindings = &config.keybindings.pages;
        let home_page = HomePage::new();
        let game_page = GamePage::new();
        let card_page = CardPage::new();

        Ok(Self {
            tick_rate,
            frame_rate,
            should_quit: false,
            should_suspend: false,
            show_help: false,
            config,
            pages: vec![Box::new(home_page), Box::new(game_page), Box::new(card_page)],
            active_page_index: 0,
            background_state: BackgroundState::new(home::SNOWFLAKE_SPEED, home::SNOWFLAKE_DENSITY),
        })
    }

    fn get_active_page(&mut self) -> &mut Box<dyn Page> {
        self.pages.get_mut(self.active_page_index).unwrap()
    }

    fn set_active_page(&mut self, index: usize) {
        if index < self.pages.len() {
            self.active_page_index = index;
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()?;
        tui.tick_rate(self.tick_rate);
        tui.frame_rate(self.frame_rate);
        tui.enter()?;

        for page in self.pages.iter_mut() {
            page.register_keymap(&self.config.keybindings.pages)?;
        }

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
                    tui::Event::Quit => action_tx.send(act!(Command::Quit))?,
                    tui::Event::Tick => action_tx.send(act!(Command::Tick))?,
                    tui::Event::Render => action_tx.send(act!(Command::Render))?,
                    tui::Event::Resize(x, y) => action_tx.send(act!(Command::Resize(x, y)))?,
                    tui::Event::Key(key) => {
                        let mut action = None;

                        let activa_page_id = self.get_active_page().id();
                        let page_keymap = self.config.keybindings.pages.get(&activa_page_id);
                        if let Some(keymap) = self.config.keybindings.pages.get(&activa_page_id) {
                            action = keymap.0.get(&key.into());
                        };
                        if let Some(act) = self.config.keybindings.global.0.get(&key.into()) {
                            action = Some(act)
                        }

                        if let Some(action) = action {
                            log::info!("Got action: {action:?}");
                            action_tx.send(action.clone())?;
                        }
                    },
                    _ => {},
                }
                if let Some(action) = self.get_active_page().handle_events(Some(e))? {
                    action_tx.send(action)?;
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                let Action { command, state } = &action;
                if *command != Command::Tick && *command != Command::Render {
                    log::debug!("{command:?}");
                }
                match command {
                    Command::Tick => {},
                    Command::Quit => self.should_quit = true,
                    Command::Suspend => self.should_suspend = true,
                    Command::Resume => self.should_suspend = false,
                    Command::ToggleShowHelp => self.show_help = !self.show_help,
                    Command::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, *w, *h))?;
                        self.render(&mut tui, &action_tx)?;
                    },
                    Command::Render => {
                        self.render(&mut tui, &action_tx)?;
                    },
                    Command::StartGame => {
                        self.background_state.show_snowman = false;
                        self.background_state.show_tree = false;
                        self.set_active_page(1);
                    },
                    Command::ShowCard => {
                        self.set_active_page(2);
                    },
                    _ => {},
                }
                if !self.show_help {
                    if let Some(action) = self.get_active_page().update(action)? {
                        action_tx.send(action)?
                    }
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(act!(Command::Resume))?;
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

    fn render(&mut self, tui: &mut tui::Tui, action_tx: &UnboundedSender<Action>) -> Result<()> {
        tui.draw(|f| {
            let area = f.area();

            let [_, area, _] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(HEIGHT), Constraint::Fill(1)]).areas(area);
            let [_, area, _] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(WIDTH), Constraint::Fill(1)]).areas(area);

            let border = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(Color::Black));
            f.render_widget(border, area);

            let area = area.inner(Margin { horizontal: 1, vertical: 1 });

            let background = Background::default();
            f.render_stateful_widget(background, area, &mut self.background_state);

            let area = self.background_state.get_empty_area(area);

            if let Some(page) = self.pages.get_mut(self.active_page_index) {
                let r = page.draw(f, area);
                if let Err(e) = r {
                    action_tx.send(act!(Command::Error(format!("Failed to draw: {:?}", e)))).unwrap();
                }
            }

            if self.show_help {
                let r = self.draw_help(f, area);
                if let Err(e) = r {
                    action_tx.send(act!(Command::Error(format!("Failed to draw: {:?}", e)))).unwrap();
                }
            };
        })?;

        Ok(())
    }

    fn draw_help(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let activa_page_id = self.get_active_page().id();
        let help = Help::new(vec![
            ("System".to_string(), self.config.keybindings.global.clone()),
            (activa_page_id.to_string(), self.config.keybindings.pages.get(&activa_page_id).unwrap().clone()),
        ]);

        f.render_widget(help, rect);

        Ok(())
    }
}
