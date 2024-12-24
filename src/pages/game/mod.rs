mod bird;
mod boundary;
mod object;

use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, SystemTime},
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Frame, Page, PageId};
use crate::{
    action::{act, Action, ActionState, Command, GameAction},
    components::{
        background::{Background, BackgroundState},
        multiline::MultiLine,
    },
    config::{key_event_to_string, PageKeyBindings},
    constants::game,
    pages::game::{bird::Bird, boundary::Boundary, object::Object},
};

enum State {
    Idle,
    Ready,
    Dead,
}

pub struct GamePage {
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: PageKeyBindings,
    state: State,
    canvas: Rect,
    bird: Bird,
    boundaries: Vec<Boundary>,
    pipes: VecDeque<(Boundary, Boundary)>,
    pipespeed: f32,
    last_time: SystemTime,
    progress: u16,
    next_gap: u16,
    next_height: u16,
    next_margin: u16,
    generated: u16,
}

impl GamePage {
    pub fn new() -> Self {
        GamePage {
            action_tx: None,
            keymap: PageKeyBindings::default(),
            state: State::Idle,
            canvas: Rect::new(0, 0, 0, 0),
            bird: Bird::new(Vec::from(game::BIRD_TEXTS), Vec::from(game::BIRD_COLORS), 0, 0, game::VELOCITY_LIMIT),
            boundaries: Vec::new(),
            pipes: VecDeque::new(),
            pipespeed: game::PIPE_VELOCITY,
            last_time: SystemTime::now(),
            progress: 0,
            next_gap: 0,
            next_height: 0,
            next_margin: 0,
            generated: 0,
        }
    }

    fn set_canvas(&mut self, canvas: Rect) {
        self.canvas = canvas;
    }

    fn reset(&mut self) {
        self.state = State::Ready;

        self.bird.set_pos(game::BIRD_INITIAL_X, self.canvas.height / 2);
        self.bird.reset_time();

        // Floor and ceiling
        let boundary_string = std::iter::repeat_n('-', self.canvas.width as usize).collect::<String>();
        self.boundaries.push(Boundary::new(vec![boundary_string.as_str()], vec![None], 0, -1));
        self.boundaries.push(Boundary::new(vec![boundary_string.as_str()], vec![None], 0, self.canvas.height as i16));

        self.pipes = VecDeque::new();
        self.last_time = SystemTime::now();
        self.sample_next_pipe();
        self.progress = 100;
        self.generated = 0;
    }

    fn draw_object<T: Object>(&self, f: &mut Frame<'_>, area: Rect, object: &T, ignore_whitespace: bool) {
        let (mut width, mut height) = object.get_size();
        let (mut x, mut y) = object.transform_pos(area);

        let left = x as i16;
        let right = x + width as i16;
        let bottom = y as i16;
        let top = y + height as i16;

        let canvas_left = self.canvas.x as i16;
        let canvas_right = canvas_left + self.canvas.width as i16;
        let canvas_bottom = self.canvas.y as i16;
        let canvas_top = canvas_bottom + self.canvas.height as i16;

        let mut layers = object.get_layers();
        for lines in layers.iter_mut() {
            if left < canvas_left || right > canvas_right {
                let begin = (canvas_left - left).max(0) as usize;
                let end = ((width as i16) - (right - canvas_right).max(0)) as usize;
                *lines = lines.iter().map(|s| s.chars().skip(begin).take(end - begin).collect()).collect();

                x += begin as i16;
                width = (end - begin) as u16;
            }
            if bottom < canvas_bottom || top > canvas_top {
                let begin = (canvas_bottom - bottom).max(0) as usize;
                let end = ((height as i16) - (top - canvas_top).max(0)) as usize;
                *lines = lines[begin..end].into_iter().map(|s| s.clone()).collect();

                y += begin as i16;
                height = (end - begin) as u16;
            }
        }

        let x = x as u16;
        let y = y as u16;

        for (index, (lines, color)) in layers.into_iter().zip(object.get_colors().into_iter()).enumerate() {
            let lines = MultiLine::new(lines).ignore_whitespace(ignore_whitespace && index > 0);
            if let Some(color) = color {
                let lines = lines.style(Style::default().fg(color));
                f.render_widget(lines, Rect { x, y, width, height });
            } else {
                f.render_widget(lines, Rect { x, y, width, height });
            };
        }
    }

    fn rand_in(&self, base: u16, range: u16) -> u16 {
        base - range + rand::random::<u16>() % (2 * range)
    }

    fn generate_pipe_string(&self, width: u16, height: u16, reverse: bool) -> String {
        let row = std::iter::repeat_n('|', width as usize).collect::<String>();
        let edge_row = std::iter::repeat_n('█', width as usize).collect::<String>();

        let rows_iter = std::iter::repeat_with(|| row.clone()).take(height as usize - 2);
        let edge_rows_iter = std::iter::repeat_with(|| edge_row.clone()).take(2);
        let rows: Vec<String> =
            if reverse { edge_rows_iter.chain(rows_iter).collect() } else { rows_iter.chain(edge_rows_iter).collect() };

        rows.join("\n")
    }

    fn sample_next_pipe(&mut self) {
        self.next_gap = self.rand_in(game::PIPE_GAP_BASE, game::PIPE_GAP_RANGE);
        self.next_height = rand::random::<u16>() % (self.canvas.height - self.next_gap - 4) + 2;
        self.next_margin = self.rand_in(game::PIPE_MARGIN_BASE, game::PIPR_MARGIN_RANGE);
        self.progress = 0;
    }

    fn generate_pipe(&mut self) {
        let pipe_width = game::PIPE_WIDRH; // Example width
        let gap_height = self.next_gap; // Example gap height
        let pipe_x = self.canvas.width as i16; // Start at the right edge

        let lower_pipe_height = self.next_height;
        let upper_pipe_height = self.canvas.height - gap_height - lower_pipe_height;
        let upper_pipe_y = lower_pipe_height + gap_height;

        let lower_stirng = self.generate_pipe_string(pipe_width, lower_pipe_height, true);
        let upper_stirng = self.generate_pipe_string(pipe_width, upper_pipe_height, false);

        let lower_pipe = Boundary::new(vec![lower_stirng.as_str()], vec![game::PIPE_COLOR], pipe_x, 0);
        let upper_pipe =
            Boundary::new(vec![upper_stirng.as_str()], vec![game::PIPE_COLOR], pipe_x, upper_pipe_y as i16);

        self.pipes.push_back((lower_pipe, upper_pipe));
        self.generated += 1;
    }

    fn draw_pipes(&self, f: &mut Frame<'_>, area: Rect) {
        for (upper_pipe, lower_pipe) in &self.pipes {
            self.draw_object(f, area, upper_pipe, false);
            self.draw_object(f, area, lower_pipe, false);
        }
    }

    pub fn reset_time(&mut self) {
        self.last_time = SystemTime::now();
    }

    fn get_delta_time(&mut self, now: SystemTime) -> f32 {
        let dt = now.duration_since(self.last_time).unwrap().as_secs_f32();
        dt
    }

    fn update_pipes(&mut self) {
        let now = SystemTime::now();
        let dt = self.get_delta_time(now);

        if dt >= (1.0 / self.pipespeed) {
            self.last_time = now;
            self.progress += 1;

            // Move pipes to the left and remove those that are out of view
            for (upper, lower) in self.pipes.iter_mut() {
                upper.move_left(1);
                lower.move_left(1);
            }

            // self.pipes.retain(|(upper, lower)| upper.visible(self.canvas));
            self.pipes.retain(|(upper, lower)| upper.visible(self.canvas));

            // if self.progress >= self.next_margin + game::PIPE_WIDRH && self.generated < game::MAX_PIPE_NUM {
            if self.progress >= self.next_margin + game::PIPE_WIDRH && self.generated < game::MAX_PIPE_NUM {
                self.generate_pipe();
                self.sample_next_pipe();
            }
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
        if let Command::Game(command) = action.command {
            match command {
                GameAction::Up if action.state == ActionState::End => {
                    self.bird.up(game::UP_VELOCITY);
                },
                _ => {},
            }
        }

        match self.state {
            State::Idle => {},
            State::Ready => {
                self.bird.update(game::GRAVITY);
                self.update_pipes();

                if self.pipes.len() == 0 && self.generated >= game::MAX_PIPE_NUM {
                    if let Some(action_tx) = &self.action_tx {
                        action_tx.send(act!(Command::ShowCard))?;
                    }
                }

                let mut game_over = false;
                if self.bird.collides_with(&self.boundaries[0]) || self.bird.collides_with(&self.boundaries[1]) {
                    game_over = true;
                }
                for (lower_pipe, upper_pipe) in self.pipes.iter() {
                    if self.bird.collides_with(lower_pipe) || self.bird.collides_with(upper_pipe) {
                        game_over = true;
                        break;
                    }
                }

                if game_over {
                    self.reset();
                }
            },
            State::Dead => {},
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if let State::Idle = self.state {
            self.set_canvas(area);
            self.reset();
        }

        // Draw player
        self.draw_object(f, area, &self.bird, true);
        // Draw pipes
        self.draw_pipes(f, area);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_player() -> Result<()> {
    //     let bird = Bird::new(game::BIRD_TEXT, 0, 0, 10.0);
    //     println!("{:?}", bird);

    //     Ok(())
    // }

    // #[test]
    // fn test_collides() {
    //     let bird = Bird::new(game::BIRD_TEXT, 0, 10, 10.0);

    //     let boundary_string = std::iter::repeat_n('-', 80).collect::<String>();
    //     let bound = Boundary::new(boundary_string.as_str(), 0, 9);

    //     println!("{}", bird.collides_with(&bound));
    // }
}
