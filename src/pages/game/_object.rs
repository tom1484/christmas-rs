#[derive(Debug, Default)]
pub struct Object {
    pub width: u16,
    pub height: u16,
    pub x: f32,
    pub y: f32,
    pub lines: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CollisionType {
    None,
    Left,   // Collision on the left side of self
    Right,  // Collision on the right side of self
    Top,    // Collision on the top of self
    Bottom, // Collision on the bottom of self
}

impl Object {
    // Create a new Object with a string, calculating width and height
    pub fn new(lines: &str, x: u16, y: u16) -> Self {
        let lines: Vec<String> = lines.lines().filter(|line| !line.is_empty()).map(|line| line.to_string()).collect();
        let height = lines.len() as u16;
        let width = lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
        Object { width, height, x: x as f32, y: y as f32, lines }
    }

    pub fn set_pos(&mut self, x: u16, y: u16) {
        self.x = x as f32;
        self.y = y as f32;
    }

    pub fn get_pos(&self) -> (u16, u16) {
        (self.x as u16, self.y as u16)
    }

    // Check for collision with another object and return the collision type
    pub fn get_collision(&self, other: &Object) -> CollisionType {
        let self_x = self.x as u16;
        let self_y = (self.y - self.height as f32) as u16;
        let other_x = other.x as u16;
        let other_y = (other.y - other.height as f32) as u16;

        let self_right = self_x + self.width;
        let self_top = self_y + self.height;
        let other_right = other_x + other.width;
        let other_top = other_y + other.height;

        // First check if there's any collision at all
        if self_x > other_right || self_right < other_x || self_y > other_top || self_top < other_y {
            return CollisionType::None;
        }

        // Determine which side the collision occurred on
        // Compare the overlap on each axis to determine the most likely collision side
        let left_overlap = ((other_right as i32) - (self_x as i32)).abs() as u16;
        let right_overlap = ((self_right as i32) - (other_x as i32)).abs() as u16;
        let top_overlap = ((other_top as i32) - (self_y as i32)).abs() as u16;
        let bottom_overlap = ((self_top as i32) - (other_y as i32)).abs() as u16;

        let min_overlap = left_overlap.min(right_overlap).min(top_overlap).min(bottom_overlap);

        match min_overlap {
            x if x == left_overlap => CollisionType::Left,
            x if x == right_overlap => CollisionType::Right,
            x if x == top_overlap => CollisionType::Top,
            _ => CollisionType::Bottom,
        }
    }

    // Convenience method that returns true if there's any collision
    pub fn collides_with(&self, other: &Object) -> bool {
        self.get_collision(other) != CollisionType::None
    }
}
