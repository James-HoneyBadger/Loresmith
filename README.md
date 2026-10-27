# Loresmith

[![Rust](https://img.shields.io/badge/Rust-1.95%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-6.0.1-green.svg)]()

> A text-based adventure game engine and development suite written in Rust. Create, edit, and play interactive fiction adventures with a JSON-based format.

**Loresmith** is the suite name for this project. It is made of separate applications built on one engine: **Studio** for authoring, **Player** for playing, **Console** for terminal play, and **Workbench** for terminal authoring. See [The Loresmith Suite](docs/Suite.md) for the architecture and naming.

## Features

### Game Engine (`loresmith_rs`)
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
| `loresmith_player` | **Loresmith Player** | Standalone graphical game player — no editor UI |
| `loresmith_studio` | **Loresmith Studio** | Graphical adventure editor with a Play test tab |
| `loresmith_console` | **Loresmith Console** | CLI game player (REPL) |
| `loresmith_workbench` | **Loresmith Workbench** | Terminal UI adventure editor (ratatui) |

Studio and Player share three themes from **View → Theme**: Dark, Light, and **Windows 2000**. The Windows 2000 theme switches to classic desktop chrome — a gradient caption bar, menu highlights, recessed toolbar, beveled panels, and a sunken status bar — and persists between sessions.

## Quick Start

```bash
# Build every suite application
cargo build --release

# Standalone graphical player
./target/release/loresmith_player examples/adventures/shattered_realms_demo.json
# or: ./Play.sh examples/adventures/shattered_realms_demo.json

# Graphical editor
./target/release/loresmith_studio
# or: ./Studio.sh

# Terminal player and editor
./target/release/loresmith_console examples/adventures/shattered_realms_demo.json
./target/release/loresmith_workbench

# Suite launcher
./Suite.sh player      # also: studio, workbench, console
```

## In-Game Commands

Kept in sync with `loresmith_rs::commands::COMMANDS`, the table the in-game `help` command renders; `cargo test` fails if this section drifts from it.

| Command | Description |
| --- | --- |
| `look/l` | Look around |
| `inventory/i/inv` | Show inventory |
| `n/s/e/w/u/d` | Move in a compass direction |
| `north/south/east/west/up/down` | Move using a full direction word |
| `go/move <direction>` | Move in a direction |
| `take/get/grab/pick <item>` | Pick up an item |
| `drop <item>` | Drop an item |
| `equip/wield/wear <item>` | Equip a weapon or armor |
| `unequip/remove <slot>` | Unequip weapon or armor |
| `use/consume/eat/drink <item>` | Use, consume, or read an item |
| `examine/inspect/x <item>` | Examine an item |
| `attack/fight/kill <monster>` | Attack a hostile monster |
| `flee/run/escape` | Attempt to flee combat |
| `talk/speak/greet/hello <npc>` | Speak directly to someone nearby |
| `ask <npc> about <topic>` | Ask an NPC about a subject |
| `say/shout/yell <text>` | Speak aloud |
| `unlock <direction>` | Unlock an exit with a carried key |
| `shop/trade/store [name]` | Browse shops in this room, or one shop by name |
| `buy/purchase <item>` | Buy from the cheapest shop in this room |
| `sell <item>` | Sell to the best-paying shop in this room |
| `status/stats/score` | Show player status, XP, and reputation |
| `quests/journal` | Show quest journal |
| `accept <quest_id>` | Accept a quest |
| `complete/finish <quest_id>` | Turn in a completed quest |
| `abandon <quest_id>` | Abandon an active quest |
| `save [file]` | Save current progress |
| `load [file]` | Load saved progress |
| `saves` | List JSON saves in this directory |
| `autosave on [file]` | Enable automatic saving |
| `autosave off` | Disable automatic saving |
| `autosave status` | Show autosave status |
| `restart` | Start the adventure over |
| `help/?` | Show this help |

The Console also accepts `quit`/`q`/`exit` to close the game.


## Project Structure

```
Loresmith/
├── loresmith_rs/        # Core game engine (Rust library)
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
├── loresmith_player/    # Standalone graphical player (egui/eframe)
├── loresmith_studio/    # Studio — graphical editor (egui/eframe)
├── loresmith_console/   # Console — CLI game player
├── loresmith_workbench/ # Workbench — terminal UI editor
├── loresmith_ui/        # Shared egui theme and widgets
├── docs/                # Documentation
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
| [Suite Overview](docs/Suite.md) | Applications, architecture, and Loresmith naming |
| [User Manual](docs/User_Manual.md) | Gameplay guide |
| [Tutorial](docs/Tutorial.md) | Play your first adventure, mechanic by mechanic |
| [Game Designer Manual](docs/Game_Designer_Manual.md) | Creating adventures |
| [Game Design Tips](docs/Game_Design_Tips.md) | Craft advice for rooms, balance, and pacing |
| [Adventure Schema](docs/Adventure_Schema.md) | Canonical JSON schema and feature examples |
| [Showcase Adventures](docs/Showcase_Adventures.md) | Tour of the bundled large adventures, including The Iron Assembly |
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
