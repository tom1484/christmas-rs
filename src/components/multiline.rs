use std::marker::PhantomData;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, StatefulWidget, Widget},
};

#[derive(Default)]
pub struct SetStyle(Style);
#[derive(Default)]
pub struct NoStyle;

#[derive(Default)]
pub struct SetLineStyles(Vec<Style>);
#[derive(Default)]
pub struct NoLineStyles;

#[derive(Default)]
pub struct LineMode;
#[derive(Default)]
pub struct PixelMode;

#[derive(Debug, Default)]
pub struct MultiLine<U, V, M, T: ToString> {
    lines: Vec<T>,
    line_padding: u16,
    ignore_whitespace: bool,
    style: U,
    line_styles: V,
    mode: M,
}

impl<T> MultiLine<SetStyle, SetLineStyles, LineMode, T>
where
    T: ToString,
{
    pub fn new(lines: Vec<T>) -> MultiLine<NoStyle, NoLineStyles, LineMode, T> {
        MultiLine::<NoStyle, NoLineStyles, LineMode, T> {
            lines,
            line_padding: 0,
            ignore_whitespace: false,
            style: NoStyle,
            line_styles: NoLineStyles,
            mode: LineMode,
        }
    }
}

impl<U, V, M, T> MultiLine<U, V, M, T>
where
    T: ToString,
{
    pub fn line_padding(self, line_padding: u16) -> MultiLine<U, V, M, T> {
        Self { line_padding, ..self }
    }

    pub fn ignore_whitespace(self, ignore_whitespace: bool) -> MultiLine<U, V, M, T> {
        Self { ignore_whitespace, ..self }
    }
}

impl<T> MultiLine<NoStyle, NoLineStyles, LineMode, T>
where
    T: ToString,
{
    pub fn style(self, style: Style) -> MultiLine<SetStyle, NoLineStyles, LineMode, T> {
        MultiLine {
            lines: self.lines,
            line_padding: self.line_padding,
            ignore_whitespace: self.ignore_whitespace,
            mode: self.mode,
            style: SetStyle(style),
            line_styles: NoLineStyles,
        }
    }

    pub fn line_styles(self, styles: Vec<Style>) -> MultiLine<NoStyle, SetLineStyles, LineMode, T> {
        assert!(styles.len() == self.lines.len(), "styles is not as long as lines");
        MultiLine {
            lines: self.lines,
            line_padding: self.line_padding,
            ignore_whitespace: self.ignore_whitespace,
            mode: self.mode,
            style: NoStyle,
            line_styles: SetLineStyles(styles),
        }
    }

    pub fn pixel_mode(self) -> MultiLine<NoStyle, NoLineStyles, PixelMode, T> {
        MultiLine {
            lines: self.lines,
            line_padding: self.line_padding,
            ignore_whitespace: self.ignore_whitespace,
            mode: PixelMode,
            style: self.style,
            line_styles: self.line_styles,
        }
    }
}

impl<T> MultiLine<NoStyle, NoLineStyles, PixelMode, T>
where
    T: ToString,
{
    pub fn style(self, style: Style) -> MultiLine<SetStyle, NoLineStyles, PixelMode, T> {
        MultiLine {
            lines: self.lines,
            line_padding: self.line_padding,
            ignore_whitespace: self.ignore_whitespace,
            mode: self.mode,
            style: SetStyle(style),
            line_styles: NoLineStyles,
        }
    }

    pub fn line_mode(self) -> MultiLine<NoStyle, NoLineStyles, LineMode, T> {
        MultiLine {
            lines: self.lines,
            line_padding: self.line_padding,
            ignore_whitespace: self.ignore_whitespace,
            mode: LineMode,
            style: self.style,
            line_styles: self.line_styles,
        }
    }
}

fn preprocess_lines<T: ToString>(
    lines: Vec<T>,
    line_padding: u16,
    ignore_whitespace: bool,
    area: Rect,
) -> (Vec<String>, Vec<Rect>) {
    let lines: Vec<String> = lines.into_iter().map(|s| s.to_string()).collect();
    let prefix_lens = if ignore_whitespace {
        lines.iter().map(|s| s.chars().take_while(|c| c.is_whitespace()).count()).collect::<Vec<_>>()
    } else {
        std::iter::repeat_n(0_usize, lines.len()).collect::<Vec<_>>()
    };

    let lines = if ignore_whitespace {
        lines.into_iter().map(|line| line.trim().to_string()).collect::<Vec<_>>()
    } else {
        lines
    };

    let x = area.x;
    let y = area.y;
    let y_offset = |index: u16| index * (1 + line_padding);
    let areas = lines
        .iter()
        .zip(prefix_lens.into_iter())
        .enumerate()
        .map(|(index, (s, p))| {
            Rect { x: x + (p as u16), y: y + y_offset(index as u16), width: s.chars().count() as u16, height: 1 }
        })
        .collect::<Vec<Rect>>();

    (lines, areas)
}

impl<'a, T: ToString> Widget for MultiLine<SetStyle, NoLineStyles, LineMode, T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (lines, areas) = preprocess_lines(self.lines, self.line_padding, self.ignore_whitespace, area);
        lines
            .into_iter()
            .zip(areas.into_iter())
            .for_each(|(line, area)| Text::from(line).style(self.style.0).render(area, buf));
    }
}

impl<'a, T: ToString> Widget for MultiLine<NoStyle, SetLineStyles, LineMode, T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (lines, areas) = preprocess_lines(self.lines, self.line_padding, self.ignore_whitespace, area);
        lines
            .into_iter()
            .zip(areas.into_iter())
            .zip(self.line_styles.0.into_iter())
            .for_each(|((line, area), style)| Text::from(line).style(style).render(area, buf));
    }
}

impl<'a, T: ToString> Widget for MultiLine<NoStyle, NoLineStyles, LineMode, T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (lines, areas) = preprocess_lines(self.lines, self.line_padding, self.ignore_whitespace, area);
        lines.into_iter().zip(areas.into_iter()).for_each(|(line, area)| Text::from(line).render(area, buf));
    }
}

fn preprocess_pixels<T: ToString>(
    lines: Vec<T>,
    line_padding: u16,
    ignore_whitespace: bool,
    area: Rect,
) -> (Vec<String>, Vec<Rect>) {
    let pixels = lines
        .into_iter()
        .map(|s| s.to_string().chars().map(|c| String::from(c)).collect::<Vec<String>>())
        .collect::<Vec<_>>();

    let y_offset = |index: u16| index * (1 + line_padding);
    let pixels_and_pos: Vec<(String, u16, u16)> = pixels
        .into_iter()
        .enumerate()
        .flat_map(|(r, row)| row.into_iter().enumerate().map(move |(c, pixel)| (pixel, r as u16, c as u16)))
        .filter(|(pixel, _, _)| {
            match ignore_whitespace {
                false => true,
                true => {
                    let c = pixel.chars().last().unwrap_or(' ');
                    !c.is_whitespace()
                },
            }
        })
        .collect();

    let x = area.x;
    let y = area.y;
    // let y_offset = |index: u16| index * (1 + margin);

    let areas = pixels_and_pos
        .iter()
        .map(|(_, r, c)| Rect { x: x + c, y: y + y_offset(*r as u16), width: 1, height: 1 })
        .collect();

    let pixels = pixels_and_pos.into_iter().map(|(pixel, _, _)| pixel).collect();

    (pixels, areas)
}

impl<'a, T: ToString> Widget for MultiLine<SetStyle, NoLineStyles, PixelMode, T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (pixels, areas) = preprocess_pixels(self.lines, self.line_padding, self.ignore_whitespace, area);
        pixels
            .into_iter()
            .zip(areas.into_iter())
            .for_each(|(pixel, area)| Span::from(pixel).style(self.style.0).render(area, buf));
    }
}

impl<'a, T: ToString> Widget for MultiLine<NoStyle, NoLineStyles, PixelMode, T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (pixels, areas) = preprocess_pixels(self.lines, self.line_padding, self.ignore_whitespace, area);
        pixels.into_iter().zip(areas.into_iter()).for_each(|(pixel, area)| Span::from(pixel).render(area, buf));
    }
}
