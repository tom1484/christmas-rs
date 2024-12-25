use std::time::SystemTime;

use ratatui::style::Color;

use crate::pages::game::object::Object;

#[derive(Debug)]
pub struct Boundary {
    width: u16,
    height: u16,
    x: f32,
    y: f32,
    layers: Vec<Vec<String>>,
    colors: Vec<Option<Color>>,
}

impl Boundary {
    pub fn new(layers: Vec<&str>, colors: Vec<Option<Color>>, x: i16, y: i16) -> Self {
        let layers: Vec<Vec<String>> = layers
            .into_iter()
            .map(|layer| layer.lines().filter(|line| !line.is_empty()).map(|line| line.to_string()).collect())
            .collect();

        let height = layers.iter().map(|layer| layer.len()).max().unwrap_or(0) as u16;
        let width = layers
            .iter()
            .map(|layer| layer.iter().map(|line| line.chars().count()).max().unwrap_or(0))
            .max()
            .unwrap_or(0) as u16;
        Boundary { width, height, x: x as f32, y: y as f32, layers, colors }
    }

    pub fn move_left(&mut self, step: u16) {
        self.x -= step as f32;
    }
}

impl Object for Boundary {
    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    fn set_pos(&mut self, x: u16, y: u16) {
        self.x = x as f32;
        self.y = y as f32;
    }

    fn get_layers(&self) -> Vec<Vec<String>> {
        self.layers.clone()
    }

    fn get_colors(&self) -> Vec<Option<Color>> {
        self.colors.clone()
    }
}
