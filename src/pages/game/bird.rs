use std::time::SystemTime;

use ratatui::style::Color;

use crate::pages::game::object::Object;

#[derive(Debug)]
pub struct Bird {
    width: u16,
    height: u16,
    x: f32,
    y: f32,
    layers: Vec<Vec<String>>,
    colors: Vec<Option<Color>>,
    velocity: f32,
    velocity_limit: f32,
    last_time: SystemTime,
    paused: bool,
}

impl Bird {
    pub fn new(layers: Vec<&str>, colors: Vec<Option<Color>>, x: u16, y: u16, velocity_limit: f32) -> Self {
        let layers: Vec<Vec<String>> = layers
            .into_iter()
            .map(|layer| layer.lines().filter(|line| !line.is_empty()).map(|line| line.to_string()).collect())
            .collect();

        let height = layers.iter().map(|layer| layer.len()).max().unwrap_or(0) as u16;
        let width =
            layers.iter().map(|layer| layer.iter().map(|line| line.len()).max().unwrap_or(0)).max().unwrap_or(0) as u16;
        Bird {
            width,
            height,
            x: x as f32,
            y: y as f32,
            layers,
            colors,
            velocity: 0.0,
            velocity_limit,
            last_time: SystemTime::now(),
            paused: false,
        }
    }

    pub fn reset_time(&mut self) {
        self.last_time = SystemTime::now();
    }

    fn get_delta_time(&mut self, now: SystemTime) -> f32 {
        let dt = now.duration_since(self.last_time).unwrap().as_secs_f32();
        dt
    }

    pub fn update(&mut self, gravity: f32) {
        if !self.paused {
            let now = SystemTime::now();
            let dt = self.get_delta_time(now);
            self.last_time = now;

            self.velocity -= gravity * dt;
            if self.velocity > self.velocity_limit {
                self.velocity = self.velocity_limit;
            }
            if self.velocity < -self.velocity_limit {
                self.velocity = -self.velocity_limit;
            }

            self.y += self.velocity * dt;
        }
    }

    pub fn up(&mut self, velocity: f32) {
        self.velocity = velocity.min(self.velocity_limit);
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.reset_time();
        self.paused = false;
    }
}

impl Object for Bird {
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
