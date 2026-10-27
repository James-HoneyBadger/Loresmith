# Loresmith

[![Rust](https://img.shields.io/badge/Rust-1.95%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-6.0.1-green.svg)]()

> A text-based adventure game engine and development suite written in Rust. Create, edit, and play interactive fiction adventures with a JSON-based format.

## Included Binaries

| Binary | Product | Description |
|--------|---------|-------------|
| `loresmith_player` | Player | Standalone graphical game player (egui) |
| `loresmith_studio` | Studio | Graphical adventure editor (egui) |
| `loresmith_console` | Console | CLI adventure player (REPL) |
| `loresmith_workbench` | Workbench | Terminal UI adventure editor (ratatui) |

## Quick Start

```bash
# Standalone graphical player
./loresmith_player examples/adventures/demo_adventure.json

# Graphical editor
./loresmith_studio

# Terminal player and editor
./loresmith_console examples/adventures/demo_adventure.json
./loresmith_workbench
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


## Features

- **Room navigation** with compass directions and custom exits
- **Item system** — pick up, drop, equip weapons & armor, consume food/potions
- **Turn-based combat** — attack, flee, XP & level-up, equipment bonuses
- **Quest framework** — accept, track objectives, complete for gold & XP
- **Event system** — objectives auto-advance on kills, pickups, and exploration
- **System trait** — extensible game logic via custom Rust systems
- **JSON adventure format** — human-readable, easy to author

## Creating Adventures

Adventures are JSON files with rooms, items, monsters, and quests using integer IDs. See the included demo files or the full documentation at:

<https://github.com/James-HoneyBadger/Loresmith>

## Documentation

- [User Manual](https://github.com/James-HoneyBadger/Loresmith/blob/main/docs/User_Manual.md)
- [Game Designer Manual](https://github.com/James-HoneyBadger/Loresmith/blob/main/docs/Game_Designer_Manual.md)
- [Technical Reference](https://github.com/James-HoneyBadger/Loresmith/blob/main/docs/Technical_Reference.md)
- [API Reference](https://github.com/James-HoneyBadger/Loresmith/blob/main/docs/API_Reference.md)

## Building from Source

```bash
git clone https://github.com/James-HoneyBadger/Loresmith.git
cd Loresmith
cargo build --release
```

Binaries are written to `target/release/`.

## License

MIT — see [LICENSE](LICENSE) for details.
