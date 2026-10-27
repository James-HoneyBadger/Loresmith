# Loresmith

[![Rust](https://img.shields.io/badge/Rust-1.95%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-6.0.0-green.svg)]()

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

```
look / l                    Look around
inventory / i / inv         Show inventory
n/s/e/w/u/d                 Move in a direction
take / get <item>           Pick up an item
drop <item>                 Drop an item
equip / wield / wear        Equip weapon or armor
unequip / remove <slot>     Unequip weapon or armor
use / consume / eat / drink <item>  Consume or read an item
examine / x <item>          Inspect an item
attack / kill <target>      Attack a hostile monster
flee / run / escape         Attempt to flee combat
status / score              View player stats
quests / journal            View quest log (with IDs)
shop / trade                Browse a shop
buy / sell <item>           Trade with a shop
unlock <direction>          Unlock an exit with a key
restart                     Restart the adventure
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
