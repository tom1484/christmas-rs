use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use crate::constants::TITLE_TEXT;

pub const WIDTH: u16 = 140;
pub(crate) const HEIGHT: u16 = 50;

pub const BORDER: Color = Color::from_u32(6315991);
pub const DYING: Color = Color::from_u32(8847535);

pub fn playable_screen(r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(HEIGHT)])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(WIDTH)])
        .split(layout[0])[0]
}

pub fn render_border(frame: &mut Frame, rect: Rect) {
    let border_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(BORDER));
    let border = Paragraph::new(Text::default())
        .alignment(Alignment::Center)
        .block(border_block);
    frame.render_widget(border, rect);
}

pub fn show_resize_screen_message(frame: &mut Frame) -> bool {
    if frame.area().width < WIDTH || frame.area().height < HEIGHT {
        let chunks = Layout::default()
            .constraints([Constraint::Length(HEIGHT)])
            .split(frame.area());
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let title = Paragraph::new(Text::styled(
            "Please resize the screen to at least 80x25",
            Style::default().fg(Color::Cyan),
        ))
            .block(title_block);
        frame.render_widget(title, chunks[0]);
        false
    } else {
        true
    }
}

pub fn render_title(frame: &mut Frame, rect: Rect) {
    let title_block = Block::default().style(Style::default())
        .padding(Padding::new(0, 0, 1, 0));
    let title = Paragraph::new(Text::styled(
        TITLE_TEXT,
        Style::default().fg(Color::Gray),
    ))
        .alignment(Alignment::Center)
        .block(title_block);
    frame.render_widget(title, rect);
}
