# 1VI1

Retro-style multiplayer shooter. 
Lose the round. Pick a card. Repeat until chaos.


<p align="center">
  <img src="demo/demo.gif" alt="gameplay">
</p>

## How to play

Grab the latest build from [Releases](https://github.com/xavierhampton/1vi1/releases) — Windows and Linux (Latest Arch glibc). Extract & Run.

## Features

- **Lose rounds, gain power** - After each round, the losers draft a new ability card. They all stack.
- **Customize your glorpy!** - Make your little guy *yours*.
- **Up to 4 players** - Connect via LAN or Internet.
- **Level editor** - Build and share arenas.
-  **16 visual themes** - Theme virtually everything.



### Controls

| Action | Key |
|--------|-----|
| Move | W / A / S / D |
| Jump | Space |
| Shoot | Left Click |
| Abilities | Right Click |
| Aim | Mouse |



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
