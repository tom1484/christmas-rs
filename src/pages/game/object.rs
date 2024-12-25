use ratatui::{layout::Rect, style::Color};

#[derive(Debug, PartialEq, Eq)]
pub enum CollisionType {
    None,
    Left,   // Collision on the left side of self
    Right,  // Collision on the right side of self
    Top,    // Collision on the top of self
    Bottom, // Collision on the bottom of self
}

pub trait Object {
    fn get_size(&self) -> (u16, u16);
    fn get_pos(&self) -> (f32, f32);
    fn set_pos(&mut self, x: u16, y: u16);
    fn get_layers(&self) -> Vec<Vec<String>>;
    fn get_colors(&self) -> Vec<Option<Color>>;

    fn visible(&self, canvas: Rect) -> bool {
        let canvas_width = canvas.width as i16;
        let canvas_height = canvas.height as i16;

        let (width, height) = self.get_size();
        let width = width as i16;
        let height = height as i16;

        let (x, y) = self.get_pos();
        let (x, y) = (x as i16, y as i16);
        let right = x + width - 1;
        let top = y + height - 1;

        return right > 0 && x < canvas_width && top > 0 && y < canvas_height;
    }

    fn transform_pos(&self, canvas: Rect) -> (i16, i16) {
        let canvas_width = canvas.width as i16;
        let canvas_height = canvas.height as i16;
        let canvas_x = canvas.x as i16;
        let canvas_y = canvas.y as i16;

        let (width, height) = self.get_size();
        let width = width as i16;
        let height = height as i16;

        let (x, y) = self.get_pos();
        let (x, y) = (x as i16, y as i16);
        let y = height - 1 + y;

        let y = canvas_height - (1 + y);

        let x = x + canvas_x;
        let y = y + canvas_y;

        (x, y)
    }

    // fn get_collision<T: Object>(&self, other: &T) -> CollisionType {
    //     let (self_width, self_height) = self.get_size();
    //     let self_width = self_width as i16;
    //     let self_height = self_height as i16;

    //     let (self_left, self_bottom) = self.get_pos();
    //     let self_left = self_left as i16;
    //     let self_bottom = self_bottom as i16;

    //     let (other_width, other_height) = other.get_size();
    //     let other_width = other_width as i16;
    //     let other_height = other_height as i16;

    //     let (other_left, other_bottom) = other.get_pos();
    //     let other_left = other_left as i16;
    //     let other_bottom = other_bottom as i16;

    //     let self_right = self_left + self_width - 1;
    //     let self_top = self_bottom + self_height - 1;
    //     let other_right = other_left + other_width - 1;
    //     let other_top = other_bottom + other_height - 1;

    //     if self_left == other_right && interval_sec(self_bottom, self_top, other_bottom, other_top) {
    //         CollisionType::Left
    //     } else if self_right == other_left && interval_sec(self_bottom, self_top, other_bottom, other_top) {
    //         CollisionType::Right
    //     } else if self_bottom == other_top && interval_sec(self_left, self_right, other_left, other_right) {
    //         CollisionType::Bottom
    //     } else if self_top == other_bottom && interval_sec(self_left, self_right, other_left, other_right) {
    //         CollisionType::Top
    //     } else {
    //         CollisionType::None
    //     }
    // }

    // fn collides_with<T: Object>(&self, other: &T) -> bool {
    //     self.get_collision(other) != CollisionType::None
    // }

    fn collides_with<T: Object>(&self, other: &T) -> bool {
        let (self_width, self_height) = self.get_size();
        let self_width = self_width as i16;
        let self_height = self_height as i16;

        let (self_left, self_bottom) = self.get_pos();
        let self_left = self_left as i16;
        let self_bottom = self_bottom as i16;

        let (other_width, other_height) = other.get_size();
        let other_width = other_width as i16;
        let other_height = other_height as i16;

        let (other_left, other_bottom) = other.get_pos();
        let other_left = other_left as i16;
        let other_bottom = other_bottom as i16;

        let self_right = self_left + self_width - 1;
        let self_top = self_bottom + self_height - 1;
        let other_right = other_left + other_width - 1;
        let other_top = other_bottom + other_height - 1;

        interval_sec(self_bottom, self_top, other_bottom, other_top)
            && interval_sec(self_left, self_right, other_left, other_right)
    }
}

fn interval_sec(l1: i16, r1: i16, l2: i16, r2: i16) -> bool {
    !(r1 < l2 || l1 > r2)
}
