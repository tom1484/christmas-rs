use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Row, Table, Widget},
};

use crate::{
    action::{ActionState, Command},
    config::{key_event_to_string, Config, PageKeyBindings},
    pages::PageId,
};

#[derive(Debug)]
pub struct Help {
    keybinding_groups: Vec<(String, Vec<(String, String)>)>,
    column_spacing: u16,
    margin_vertical: u16,
    margin_horizantal: u16,
}

impl Help {
    pub fn new(keybinding_groups: Vec<(String, PageKeyBindings)>) -> Self {
        let groups = keybinding_groups
            .into_iter()
            .map(|(group_name, keybindings)| {
                let mut keybindings: Vec<(String, String)> = keybindings
                    .0
                    .into_iter()
                    .filter(|(_, action)| action.state == ActionState::Start)
                    .map(|(event, action)| (key_event_to_string(&event), action.command.string()))
                    .collect();
                keybindings.sort_by_key(|(key, _)| key.clone());
                (group_name, keybindings)
            })
            .collect();

        Self { keybinding_groups: groups, column_spacing: 5, margin_vertical: 1, margin_horizantal: 2 }
    }

    fn render_group(
        &self,
        area: Rect,
        buf: &mut Buffer,
        group_name: &String,
        keybindings: &Vec<(String, String)>,
        key_length: u16,
        val_length: u16,
    ) {
        let rows: Vec<Row> =
            keybindings.into_iter().map(|(key, val)| Row::new(vec![key.clone(), val.clone()])).collect();
        let widths = vec![Constraint::Length(key_length), Constraint::Min(val_length)];

        let table = Table::new(rows, widths)
            .widths(&[Constraint::Length(key_length), Constraint::Min(val_length)])
            .column_spacing(5)
            .header(Row::new(vec!["Key", "Command"]).style(Style::new().bold()).bottom_margin(1))
            .block(
                Block::new()
                    .title(group_name.clone())
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().bold().fg(Color::Cyan))
                    .padding(ratatui::widgets::Padding::symmetric(2, 1)),
            );

        Clear::default().render(area, buf);
        table.render(area, buf);
    }
}

impl Widget for Help {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let group_names: Vec<&String> = self.keybinding_groups.iter().map(|(name, _)| name).collect();
        let group_keybindings: Vec<&Vec<(String, String)>> =
            self.keybinding_groups.iter().map(|(_, bindings)| bindings).collect();

        let key_length = group_keybindings
            .iter()
            .map(|keybindings| keybindings.iter().map(|(key, _)| key.len()).max().unwrap_or(0))
            .max()
            .unwrap_or(0) as u16;
        let val_length = group_keybindings
            .iter()
            .map(|keybindings| keybindings.iter().map(|(_, val)| val.len()).max().unwrap_or(0))
            .max()
            .unwrap_or(0) as u16;

        // + column_spacing + margin*2 + border 
        let width = key_length + val_length + self.column_spacing + (self.margin_horizantal * 2) + 2;
        // let width = key_length + val_length;
        let heights: Vec<u16> = group_keybindings
            .iter()
            .map(|keybindings| {
                let len = keybindings.len() as u16;
                // + margin*2 + header + border
                let height = len + (self.margin_vertical * 2) + 2 + 2;
                height
            })
            .collect();

        let area = Layout::default()
            .flex(ratatui::layout::Flex::Center)
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Length(width)])
            .split(area)[0];

        let areas = Layout::default()
            .flex(ratatui::layout::Flex::Center)
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(heights.iter().map(|h| Constraint::Length(*h)).collect::<Vec<_>>())
            .split(area);

        for i in 0..self.keybinding_groups.len() {
            self.render_group(areas[i], buf, &group_names[i], &group_keybindings[i], key_length, val_length);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_help() {
        let config = Config::new().unwrap();
        let keybinding_groups = vec![
            ("System".to_string(), config.keybindings.global),
            ("Home".to_string(), config.keybindings.pages.get(&PageId::Home).unwrap().clone()),
            // ("Game".to_string(), config.keybindings.pages.get(&PageId::Game).unwrap().clone()),
        ];
        let help = Help::new(keybinding_groups);

        println!("{help:?}");
    }
}
