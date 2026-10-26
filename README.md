# Mellivorex

[![Rust](https://img.shields.io/badge/Rust-1.95%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-5.0.0-green.svg)]()

> A text-based adventure game engine and development suite written in Rust. Create, edit, and play interactive fiction adventures with a JSON-based format.

**Mellivorex** is the suite name for this project. It is made of separate applications built on one engine: **Studio** for authoring, **Player** for playing, **Console** for terminal play, and **Workbench** for terminal authoring. See [The Mellivorex Suite](docs/Suite.md) for the architecture and naming.

## Features

### Game Engine (`mellivorex_rs`)
- **Room navigation** with compass directions and named exits
- **Item system** — pick up, drop, equip weapons & armor, consume food/potions, examine
- **Turn-based combat** — attack, flee, XP & level-up, equipment bonuses
- **Quest framework** — staged objectives, automatic progress on kills, pickups, exploration, and conversation, verified turn-in, gold/XP/item/reputation rewards
- **Dialogue support** — per-NPC greetings and topic replies, with friendly/neutral/hostile behavior
- **Locks and keys** — locked exits that open with carried key items (optionally consuming them)
- **Economy** — shops with finite stock and gold; buy and sell with charisma-based prices
- **Light and darkness** — dark rooms require a carried light source
- **Combat and loot** — weapon/armor bonuses, XP and multi-level progression, monster loot drops
- **Endings** — configurable victory conditions and defeat text, plus restart
- **Portable saves** — snapshot v2 embeds the adventure, writes atomically, and migrates v1 saves
- **Canonical JSON schema** — one versioned document model for both editors and the runtime
- **Event system** — systems observe kills, pickups, movement, conversation, purchases, and endings

### Applications
| Binary | Product | Description |
|--------|---------|-------------|
| `mellivorex_player` | **Mellivorex Player** | Standalone graphical game player — no editor UI |
| `mellivorex_studio` | **Mellivorex Studio** | Graphical adventure editor with a Play test tab |
| `mellivorex_console` | **Mellivorex Console** | CLI game player (REPL) |
| `mellivorex_workbench` | **Mellivorex Workbench** | Terminal UI adventure editor (ratatui) |

Studio and Player share three themes from **View → Theme**: Dark, Light, and **Windows 2000**. The Windows 2000 theme switches to classic desktop chrome — a gradient caption bar, menu highlights, recessed toolbar, beveled panels, and a sunken status bar — and persists between sessions.

## Quick Start

```bash
# Build every suite application
cargo build --release

# Standalone graphical player
./target/release/mellivorex_player examples/adventures/shattered_realms_demo.json
# or: ./Play.sh examples/adventures/shattered_realms_demo.json

# Graphical editor
./target/release/mellivorex_studio
# or: ./Studio.sh

# Terminal player and editor
./target/release/mellivorex_console examples/adventures/shattered_realms_demo.json
./target/release/mellivorex_workbench

# Suite launcher
./Suite.sh player      # also: studio, workbench, console
```

## In-Game Commands

```
look / l                    Look around
inventory / i / inv         Show inventory
n/s/e/w/u/d                 Move in a direction
go / move <direction>       Move using a direction word
take/get/grab/pick <item>   Pick up an item
drop <item>                 Drop an item
equip/wield/wear <item>     Equip a weapon or armor
unequip/remove <slot>       Unequip weapon or armor
use/consume/eat/drink <item> Use, consume, or read an item
examine / x <item>          Examine an item
attack / fight <monster>    Attack a hostile monster
flee / run / escape         Attempt to flee combat
talk <npc>                  Speak to someone nearby
ask <npc> about <topic>     Ask an NPC about a subject
say / shout / yell <text>   Speak aloud
unlock <direction>          Unlock an exit with a carried key
shop / trade                Browse the shop in this room
buy <item>                  Buy from the shop
sell <item>                 Sell to the shop
status / stats / score      Show player status, XP, and reputation
quests / journal            Show quest journal (with quest IDs)
accept <quest_id>           Accept a quest
complete <quest_id>         Turn in a completed quest
abandon <quest_id>          Abandon an active quest
save [file]                 Save current progress
load [file]                 Load saved progress
saves                       List JSON files in the current directory
autosave on [file]          Enable automatic saves
autosave off                Disable autosave
autosave status             Show autosave status
restart                     Restart the adventure
help / ?                    Show command help
quit / q / exit             Quit (CLI player only)
```

## Project Structure

```
Mellivorex/
├── mellivorex_rs/           # Core game engine (Rust library)
│   └── src/
│       ├── lib.rs          # Public API re-exports
│       ├── engine.rs       # High-level Engine wrapper
│       ├── document.rs     # Canonical adventure document + legacy adapters
│       ├── adventure.rs    # Legacy string-ID format (read support)
│       ├── game_state.rs   # Runtime types, loaders, saves, events
│       ├── util.rs         # Atomic file writes
│       └── systems/        # Pluggable game systems
│           ├── basic_world.rs  # Navigation, look, talk, locks, saves
│           ├── inventory.rs    # Take, drop, equip, use, examine
│           ├── combat.rs       # Attack, flee, XP, loot, endings
│           ├── quests.rs       # Quest tracking, stages, rewards
│           └── shops.rs        # Shops, buying, selling
├── mellivorex_player/         # Standalone graphical player (egui/eframe)
├── mellivorex_studio/      # Studio — graphical editor (egui/eframe)
├── mellivorex_console/       # Console — CLI game player
├── mellivorex_workbench/      # Workbench — terminal UI editor
├── mellivorex_ui/           # Shared egui theme and widgets
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
| [Suite Overview](docs/Suite.md) | Applications, architecture, and Mellivorex naming |
| [User Manual](docs/User_Manual.md) | Gameplay guide |
| [Game Designer Manual](docs/Game_Designer_Manual.md) | Creating adventures |
| [Technical Reference](docs/Technical_Reference.md) | Architecture & systems |
| [Adventure Schema](docs/Adventure_Schema.md) | Canonical JSON schema and feature examples |
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
