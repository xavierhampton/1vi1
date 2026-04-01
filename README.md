# 1VI1

A fast-paced arena shooter where every round lost makes you stronger. Up to 4 players duke it out in quick rounds of platforming and shooting.

<!-- ![gameplay](screenshots/gameplay.png) -->

## Features

- **Lose rounds, gain power** - Losers draft from 50+ abilities and powerups that stack as the match goes on
- **Multiplayer** - Up to 4 players over LAN or internet
- **Level editor** - Build and share your own arenas
- **16 themes** - Pick your vibe

## Getting started

Grab the latest build from [Releases](https://github.com/xavierhampton/1vi1/releases) - available for Windows and Linux. Extract and run, everything's included.

### Building from source

Requires Rust and raylib 5.5 system dependencies.

```bash
# Linux: install deps first
sudo apt install libgl1-mesa-dev libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev cmake

# Build and run
cargo run --release
```

### Multiplayer

1. One player hosts (starts a game from the menu)
2. Others join by entering the host's IP address and port
3. That's it

## Controls

| Action | Key |
|--------|-----|
| Move | A / D |
| Jump | Space |
| Shoot | Left Click |
| Use abilities | Right Click |
| Aim | Mouse |

## Attribution
- @keemcc for some map development
- Raylib's powerful and easy game engine
- Development assisted with Claude Code

## License

Do whatever you want with it. Have fun.
