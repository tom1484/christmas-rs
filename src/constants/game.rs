use ratatui::style::Color;

pub const GRAVITY: f32 = 70.0;
pub const UP_VELOCITY: f32 = 20.0;
pub const VELOCITY_LIMIT: f32 = 20.0;

pub const PIPE_WIDRH: u16 = 6;
pub const PIPE_VELOCITY: f32 = 12.0;
pub const PIPE_GAP_BASE: u16 = 10;
pub const PIPE_GAP_RANGE: u16 = 2;
pub const PIPE_MARGIN_BASE: u16 = 25;
pub const PIPR_MARGIN_RANGE: u16 = 2;
pub const PIPE_COLOR: Option<Color> = Some(Color::LightGreen);

pub const MAX_PIPE_NUM: u16 = 1;

pub const BIRD_INITIAL_X: u16 = 20;
pub const BIRD_TEXTS: [&str; 2] = [
    r#"
 ^ ^
(   )
(   )
- - -
"#,
    r#"
    
 O,O 
     
 " " 
"#,
];
pub const BIRD_COLORS: [Option<Color>; 2] = [Some(Color::LightBlue), Some(Color::Yellow)];
