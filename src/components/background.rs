use std::{collections::HashMap, time::SystemTime};

use rand::prelude::*;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Alignment, Constraint, Layout, Margin, Offset, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, StatefulWidget, Widget},
};

use crate::{
    action::{ActionState, Command},
    components::multiline::MultiLine,
    config::{key_event_to_string, Config, PageKeyBindings},
    constants::background,
    pages::PageId,
};

#[derive(Debug)]
pub struct BackgroundState {
    speed: f32, // Snowflake drop speed: rows per second
    density: f32,
    last_time: SystemTime,
    snowflakes: Vec<Vec<usize>>,
    width: usize,
    height: usize,
    current: usize,
    pub show_snowman: bool,
    pub show_tree: bool,
}

impl BackgroundState {
    pub fn new(speed: f32, density: f32) -> Self {
        Self {
            speed,
            density,
            last_time: SystemTime::now(),
            snowflakes: Vec::new(),
            width: 0,
            height: 0,
            current: 0,
            show_snowman: true,
            show_tree: true,
        }
    }

    fn get_delta_time(&self, now: SystemTime) -> f32 {
        let dt = now.duration_since(self.last_time).unwrap().as_secs_f32();
        dt
    }

    fn sample(&self, rng: &mut ThreadRng) -> usize {
        let u: f32 = rng.gen();
        if u > self.density {
            background::SNOWFLAKES.len()
        } else {
            rng.gen_range(0..background::SNOWFLAKES.len())
        }
    }

    fn update(&mut self, area: Rect) -> Vec<String> {
        let width = area.width as usize;
        let height = area.height as usize;

        let mut rng = thread_rng();

        // Adjust size if size changed
        if width < self.width {
            // Trim out of bound
            self.snowflakes = self
                .snowflakes
                .iter()
                .map(|row| row.clone().into_iter().take(width).collect::<Vec<_>>())
                .collect::<Vec<_>>();
        } else if width > self.width {
            // Pad new space
            self.snowflakes = self
                .snowflakes
                .iter()
                .map(|row| {
                    let mut row = row.clone();
                    row.extend(std::iter::repeat_with(|| self.sample(&mut rng)).take(width - self.width));
                    row
                })
                .collect::<Vec<_>>();
        }

        if height < self.height {
            for _ in 0..(self.height - height) {
                self.snowflakes.pop();
            }
        } else {
            for _ in 0..(height - self.height) {
                let new_row = std::iter::repeat_n(background::SNOWFLAKES.len(), width).collect::<Vec<_>>();
                // let new_row = std::iter::repeat_n(0, width).collect::<Vec<_>>();
                self.snowflakes.push(new_row);
            }
        }

        self.width = width;
        self.height = height;

        let now = SystemTime::now();
        let dt = self.get_delta_time(now);

        if dt >= 1.0 / self.speed {
            self.last_time = now;

            let new_row = std::iter::repeat_with(|| self.sample(&mut rng)).take(width).collect();
            self.snowflakes = {
                let mut snowflakes = vec![new_row];
                snowflakes.extend(self.snowflakes.iter().map(|row| row.clone()).take(height - 1));
                snowflakes
            };
        }

        self.snowflakes
            .iter()
            .map(|row| {
                row.clone()
                    .into_iter()
                    .map(
                        |index| if index == background::SNOWFLAKES.len() { ' ' } else { background::SNOWFLAKES[index] },
                    )
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
    }

    pub fn get_empty_area(&self, area: Rect) -> Rect {
        let height = area.height as u16;
        let sky_height = height - background::GROUND_HEIGHT;
        Rect { height: sky_height, ..area }
    }
}

#[derive(Debug, Default)]
pub struct Background;

impl Background {
    pub fn new() -> Self {
        Self::default()
    }

    // fn render_snowman(&self, area: Rect, buf: &mut Buffer, snowman_lines: Vec<&str>) {
    fn render_snowman(&self, area: Rect, buf: &mut Buffer) {
        let (snowman_lines, num_snowman_lines) = filter_text(background::SNOWMAN);
        let max_width = snowman_lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;

        let [_, area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(num_snowman_lines)]).areas(area);
        let [_, area, _] = Layout::horizontal([
            Constraint::Length(background::SNOWMAN_MARGIN),
            Constraint::Length(max_width),
            Constraint::Fill(1),
        ])
        .areas(area);
        let area = area.offset(Offset { x: 0, y: 1 });

        MultiLine::new(snowman_lines)
            .style(Style::default().fg(Color::White))
            .ignore_whitespace(true)
            .render(area, buf);
    }

    fn render_ground(&self, area: Rect, buf: &mut Buffer) {
        let width = area.width;
        let height = area.height;

        let ground_string = std::iter::repeat_n('#', width as usize).collect::<String>();
        let ground_lines = std::iter::repeat_with(|| ground_string.clone())
            .enumerate()
            .map(|(index, s)| {
                // Line::from(s).style(Style::default().fg(match index {
                //     0 => Color::White,
                //     _ => Color::Green,
                // }))
                Line::from(s).style(Style::default().fg(Color::White))
            })
            .take(height as usize)
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(ground_lines);

        paragraph.render(area, buf);
    }

    fn render_snowflakes(&self, area: Rect, buf: &mut Buffer, state: &mut BackgroundState) {
        let lines = state.update(area).into_iter().map(|s| Line::from(s)).collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines);
        paragraph.render(area, buf);
    }

    fn render_tree(&self, area: Rect, buf: &mut Buffer) {
        let (treetop_lines, num_treetop_lines) = filter_text(background::TREETOP_TEXT);
        let (treebody_lines, num_treebody_lines) = filter_text(background::TREEBODY_TEXT);
        let (treebottom_lines, num_treebottom_lines) = filter_text(background::TREEBOTTOM_TEXT);
        let num_tree_lines = num_treetop_lines.max(num_treebody_lines).max(num_treebottom_lines);

        let max_width = treetop_lines
            .iter()
            .chain(treebody_lines.iter())
            .chain(treebottom_lines.iter())
            .map(|line| line.len())
            .max()
            .unwrap_or(0) as u16;

        let [_, area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(num_tree_lines)]).areas(area);
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(max_width),
            Constraint::Length(background::TREE_MARGIN),
        ])
        .areas(area);
        let area = area.offset(Offset { x: 0, y: 1 });

        MultiLine::new(treetop_lines)
            .style(Style::default().fg(Color::Yellow))
            .ignore_whitespace(true)
            .render(area, buf);
        MultiLine::new(treebody_lines)
            .style(Style::default().fg(Color::Green))
            .ignore_whitespace(true)
            .render(area, buf);
        MultiLine::new(treebottom_lines)
            .style(Style::default().fg(Color::Rgb(102, 56, 21)))
            .ignore_whitespace(true)
            .render(area, buf);
    }
}

fn filter_text(text: &'static str) -> (Vec<&str>, u16) {
    let lines: Vec<&str> = text.lines().filter(|s| s.len() != 0).collect();
    let num_lines = lines.len() as u16;

    (lines, num_lines)
}

impl StatefulWidget for Background {
    type State = BackgroundState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut BackgroundState)
    where
        Self: Sized,
    {
        let [area, ground_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(background::GROUND_HEIGHT)]).areas(area);

        self.render_snowflakes(area, buf, state);
        self.render_ground(ground_area, buf);

        if state.show_snowman {
            self.render_snowman(area, buf);
        }
        if state.show_tree {
            self.render_tree(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_new_help() {
    // }
}
