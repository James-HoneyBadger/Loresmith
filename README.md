# SagaCraft

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-4.0.3-green.svg)]()

> A text-based adventure game engine and development suite written in Rust. Create, edit, and play interactive fiction adventures with a JSON-based format.

## Features

### Game Engine (`sagacraft_rs`)
- **Room navigation** with compass directions and named exits
- **Item system** — pick up, drop, equip weapons & armor, consume food/potions, examine
- **Turn-based combat** — attack, flee, XP & level-up, equipment bonuses
- **Quest framework** — accept, track objectives, complete for gold & XP rewards
- **Dialogue support** — talk to NPCs, ask about topics, and react to friendly/neutral/hostile behavior
- **Save, load, and autosave** — preserve long play sessions and showcase persistence features
- **Event system** — quest objectives auto-advance on kills, item pickups, and room exploration
- **JSON adventure format** — human-readable, easy to create by hand or with the editors

### Binaries
| Binary | Description |
|--------|-------------|
| `sagacraft_player` | CLI game player (REPL) |
| `sagacraft_ide_tui` | Terminal UI adventure editor (ratatui) |
| `sagacraft_ide_gui` | Graphical adventure editor (egui) with built-in Play tab |

## Quick Start

```bash
# Build
cargo build --release

# Play the included demo adventure
./target/release/sagacraft_player examples/adventures/shattered_realms_demo.json

# Or use the TUI editor
./target/release/sagacraft_ide_tui

# Or use the GUI editor
./target/release/sagacraft_ide_gui
```

## In-Game Commands

```
look / l                    Look around
inventory / i / inv         Show inventory
n/s/e/w/u/d                 Move in a direction
take <item>                 Pick up an item
drop <item>                 Drop an item
equip/wield/wear <item>     Equip a weapon or armor
unequip/remove <slot>       Unequip weapon or armor
use <item>                  Use/consume an item
examine / x <item>          Examine an item
attack / fight <monster>    Attack a monster
flee / run                  Attempt to flee combat
talk <npc>                  Speak directly to someone nearby
ask <npc> about <topic>     Ask an NPC about a subject
say / shout / yell <text>   Speak aloud
status / stats              Show player status & XP
quests / journal            Show quest journal
accept <quest_id>           Accept a quest
complete <quest_id>         Complete a quest
save [file]                 Save current progress
load [file]                 Load saved progress
autosave on [file]          Enable automatic saves
autosave off                Disable autosave
autosave status             Show autosave status
help / ?                    Show command help
quit / q / exit             Quit (CLI player only)
```

## Project Structure

```
SagaCraft/
├── sagacraft_rs/           # Core game engine (Rust library)
│   └── src/
│       ├── lib.rs          # Public API re-exports
│       ├── engine.rs       # High-level Engine wrapper
│       ├── adventure.rs    # String-ID adventure format (TUI)
│       ├── game_state.rs   # Runtime types: Room, Item, Monster, Player, AdventureGame
│       └── systems/        # Pluggable game systems
│           ├── basic_world.rs  # Navigation, look, help, say
│           ├── inventory.rs    # Take, drop, equip, use, examine
│           ├── combat.rs       # Attack, flee, XP, level-up
│           └── quests.rs       # Quest tracking & objectives
├── sagacraft_player/       # CLI game player
├── sagacraft_ide_tui/      # Terminal UI editor
├── sagacraft_ide_gui/      # GUI editor (egui/eframe)
├── docs/                   # Documentation
└── examples/
    └── adventures/         # Bundled demos and showcase adventures
```

## Adventure JSON Format

Adventures are JSON files loaded by the engine. Minimal example:

```json
{
  "title": "My Adventure",
  "intro": "Welcome, brave adventurer!",
  "start_room": 1,
  "rooms": [
    {
      "id": 1,
      "name": "Village Square",
      "description": "A quiet square with a fountain.",
      "exits": { "north": 2 }
    }
  ],
  "items": [],
  "monsters": [],
  "quests": []
}
```

See [examples/adventures/shattered_realms_demo.json](examples/adventures/shattered_realms_demo.json) for a complete example with items, monsters, and quests.

## Showcase Adventures

The workspace also includes larger adventure showcases for demonstrations and testing:

- examples/adventures/clockwork_citadel_showcase.json — steampunk fortress siege
- examples/adventures/starfall_harbor_showcase.json — maritime mystery and abyssal fantasy
- examples/adventures/iron_assembly_showcase.json — political intrigue and faction conflict

## Documentation

| Document | Description |
|----------|-------------|
| [User Manual](docs/User_Manual.md) | Gameplay guide |
| [Game Designer Manual](docs/Game_Designer_Manual.md) | Creating adventures |
| [Technical Reference](docs/Technical_Reference.md) | Architecture & systems |
| [API Reference](docs/API_Reference.md) | Library API docs |
| [Installation Guide](docs/Installation_Guide.md) | Setup instructions |
| [Development Guide](docs/Development_Guide.md) | Contributing |

## Building & Development

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all
```

## License

MIT — see [LICENSE](LICENSE).

---

*Made with Rust • MIT Licensed • Cross-Platform*
