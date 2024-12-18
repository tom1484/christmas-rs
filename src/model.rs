use std::io;

use ratatui::Frame;

#[derive(PartialEq, Debug)]
pub enum ModelResponse {
    /// Check for another update from the screen model
    NoOp,
    /// Run the ui function on the screen model
    Refresh,
    /// Exit the application
    Exit,
    /// Trigger a new game by switching to the GameScreen model
    NewGame,
    /// Indicates user has quit a game by pressing escape
    QuitGame,
    /// Indicates that the user wants to come back to the title screen after achieving a high score
    ReturnToTitle,
    /// Indicates that the user has finished his game. The score will be checked against high scores
    EndGame,
}

pub trait Model {
    /// Called by the main program loop to look for an update message to act upon.
    /// This could be that an update the screen has performed something and wants a screen refresh,
    /// or that the program should switch to a different screen, etc...
    fn update(&mut self) -> io::Result<ModelResponse>;

    /// Called by the main program loop to refresh the current screen.
    fn ui(&mut self, frame: &mut Frame);
}

