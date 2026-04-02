# 1VI1

Lose the round. Pick a card. Come back stronger. Repeat until chaos.


<p align="center">
  <img src="demo/demo.gif" alt="gameplay">
</p>

## Features

- **Lose rounds, gain power** - After each round, the loser drafts a new ability card. Dash, reflect, gravity flip, shotgun blast — they all stack, and they all fire at once
- **Customize your glorpy!** - Pick your name, pick your color, pick your loadout. Make your little guy *yours*
- **Up to 4 players** - LAN, internet, couch-adjacent-laptops, whatever works
- **16 visual themes** - Neon, pastel, monochrome, CRT — the whole arena changes
- **Level editor** - Build and share your own arenas

## How to play

Grab the latest build from [Releases](https://github.com/xavierhampton/1vi1/releases) — Windows and Linux. Extract, run, shoot.

### Controls

| Action | Key |
|--------|-----|
| Move | A / D |
| Jump | Space |
| Shoot | Left Click |
| Unleash everything | Right Click |
| Aim | Mouse |

### Multiplayer

1. One player hosts
2. Others join with the host's IP
3. Fight

### Building from source

Requires Rust and raylib 5.5 system dependencies.

```bash
# Linux: install deps first
sudo apt install libgl1-mesa-dev libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev cmake

# Build and run
cargo run --release
```

## Attribution
- @keemcc for some map development
- Raylib's powerful and easy game engine
- Development assisted with Claude Code

## License

MIT
