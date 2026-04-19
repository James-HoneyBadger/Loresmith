# SagaCraft

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-4.0.3-green.svg)]()

> A text-based adventure game engine and development suite written in Rust. Create, edit, and play interactive fiction adventures with a JSON-based format.

## Included Binaries

| Binary | Description |
|--------|-------------|
| `sagacraft_player` | CLI adventure player (REPL) |
| `sagacraft_ide_tui` | Terminal UI adventure editor (ratatui) |
| `sagacraft_ide_gui` | Graphical adventure editor (egui) with built-in Play tab |

## Quick Start

```bash
# Play the included demo adventure
./sagacraft_player examples/adventures/demo_adventure.json

# Or use the TUI editor
./sagacraft_ide_tui

# Or use the GUI editor
./sagacraft_ide_gui
```

## In-Game Commands

```
look / l                    Look around
inventory / i / inv         Show inventory
n/s/e/w/u/d                 Move in a direction
take / get <item>           Pick up an item
drop <item>                 Drop an item
equip / wield / wear        Equip weapon or armor
unequip / remove <slot>     Unequip weapon or armor
use / eat / drink <item>    Consume or read an item
examine / x <item>          Inspect an item
attack / kill <target>      Attack a hostile monster
flee / run / escape         Attempt to flee combat
status / score              View player stats
quests / journal            View quest log
accept <quest_id>           Accept a quest
complete / finish <id>      Turn in a completed quest
say / shout <text>          Speak aloud
help / ?                    Show command list
quit / q                    Exit the game
```

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

<https://github.com/James-HoneyBadger/SagaCraft>

## Documentation

- [User Manual](https://github.com/James-HoneyBadger/SagaCraft/blob/main/docs/User_Manual.md)
- [Game Designer Manual](https://github.com/James-HoneyBadger/SagaCraft/blob/main/docs/Game_Designer_Manual.md)
- [Technical Reference](https://github.com/James-HoneyBadger/SagaCraft/blob/main/docs/Technical_Reference.md)
- [API Reference](https://github.com/James-HoneyBadger/SagaCraft/blob/main/docs/API_Reference.md)

## Building from Source

```bash
git clone https://github.com/James-HoneyBadger/SagaCraft.git
cd SagaCraft
cargo build --release
```

Binaries are written to `target/release/`.

## License

MIT — see [LICENSE](LICENSE) for details.
