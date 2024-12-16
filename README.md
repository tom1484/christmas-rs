# Thardians!

A space invaders game you can play in the terminal, using the Ratatui terminal UI.
This is a Rust port of my [original game](https://gitlab.com/thustle/thardians) which was written in Go using BubbleTea.

The game is complete now. You can see the progress made in the checklist below.

Why Thardians? I think it was because these games make me think back to 
old 8-bit games I used to play on the Acorn Electron, such as Arcadians, and Elite (with the Thargoids).

## Building
You can build the app using the following command:
```shell
cargo build --release
```

## Screenshots
Demo
![Game screen layout (iTerm2)](readme/demo.gif)

Game on iTerm2
![Game screen layout (iTerm2)](readme/Screenshot1.png)

Title screen on iTerm2
![Game screen layout (iTerm2)](readme/Screenshot2.png)

## Milestones
- [X] Moving band of invaders
- [X] Ship movement
- [X] Invader animation
- [X] Shields
- [X] Missiles from defender
- [X] Invader speed-up
- [X] Collision detection of missiles on invaders
- [X] Title Screen
- [X] Pause Game
- [X] Collision detection of missiles on shields
- [X] Collision detection of invaders on shields
- [X] Missiles from invaders
- [X] Collision detection of missiles on defender
- [X] Start/End game scenarios
- [X] Manage lives
- [X] Levels
- [X] High-scores
- [X] High-score name entry
- [X] More Tests

## License
[MIT](LICENSE)
