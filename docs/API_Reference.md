# Mellivorex API Reference

**Version 5.0.0 · mellivorex_rs crate**

Complete API documentation for the Mellivorex core library. All types are defined in the `mellivorex_rs` crate and re-exported from `lib.rs`.

---

## Table of Contents

1. [Public Re-exports](#public-re-exports)
2. [Engine](#engine)
3. [AdventureGame](#adventuregame)
4. [Room](#room)
5. [Item & ItemType](#item--itemtype)
6. [Monster & MonsterStatus](#monster--monsterstatus)
7. [Player](#player)
8. [GameEvent](#gameevent)
9. [System Trait](#system-trait)
10. [Built-in Systems](#built-in-systems)
11. [Adventure (Secondary Format)](#adventure-secondary-format)
12. [Error Types](#error-types)
13. [Usage Examples](#usage-examples)

---

## Public Re-exports

```rust
// mellivorex_rs/src/lib.rs
pub use adventure::{Adventure, AdventureError, AdventureItem, AdventureRoom};
pub use engine::Engine;
pub use game_state::{AdventureGame, GameEvent, Item, Monster, Player, Room, ItemType, MonsterStatus};
pub use systems::{BasicWorldSystem, InventorySystem, CombatSystem, QuestSystem, System};
```

---

## Engine

High-level wrapper that creates an `AdventureGame` with all five built-in systems pre-registered. This is the recommended entry point for playing adventures.

```rust
pub struct Engine {
    pub game: AdventureGame,
    // intro_text: String (private)
}
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new(adventure_path: impl Into<String>) -> Self` | Create engine with systems registered. Call `start()` to load. |
| `start` | `fn start(&mut self) -> Result<String, Box<dyn Error>>` | Load adventure from disk. Returns intro/banner text. |
| `load` | `fn load(path: impl Into<String>) -> Result<Self, Box<dyn Error>>` | Shorthand: `new()` + `start()`. |
| `intro` | `fn intro(&self) -> &str` | Return the intro text captured at load time. |
| `send` | `fn send(&mut self, input: &str) -> Vec<String>` | Process one line of player input. Returns response lines. |
| `look` | `fn look(&self) -> String` | Return current room description. |
| `is_over` | `fn is_over(&self) -> bool` | Whether the game has ended. |

### Example

```rust
use mellivorex_rs::Engine;

let mut engine = Engine::load("my_adventure.json")?;
println!("{}", engine.intro());
println!("{}", engine.look());

for line in engine.send("take sword") {
    println!("{}", line);
}
```

---

## AdventureGame

The core runtime struct holding all game state. Used directly by the GUI IDE's Play tab and indirectly through `Engine` by the CLI player.

```rust
pub struct AdventureGame {
    pub adventure_file: String,
    pub rooms: HashMap<i32, Room>,
    pub items: HashMap<i32, Item>,
    pub monsters: HashMap<i32, Monster>,
    pub player: Player,
    pub turn_count: i32,
    pub game_over: bool,
    pub adventure_title: String,
    pub adventure_intro: String,
    pub systems: Vec<Box<dyn System>>,
    pub quests: Vec<serde_json::Value>,
    pub events: Vec<GameEvent>,
}
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new(adventure_file: String) -> Self` | Create empty game with the given adventure path. |
| `load_adventure` | `fn load_adventure(&mut self) -> Result<String, Box<dyn Error>>` | Parse JSON, populate rooms/items/monsters/quests. Returns intro banner. |
| `get_current_room` | `fn get_current_room(&self) -> Option<&Room>` | Current room reference. |
| `get_items_in_room` | `fn get_items_in_room(&self, room_id: i32) -> Vec<&Item>` | Items located in the given room. |
| `get_monsters_in_room` | `fn get_monsters_in_room(&self, room_id: i32) -> Vec<&Monster>` | Living monsters in the given room. |
| `look` | `fn look(&self) -> String` | Full room description with exits, items, and monsters. |
| `move_player` | `fn move_player(&mut self, direction: &str) -> Option<String>` | Move via exit. Returns new room description or `None`. |
| `take_item` | `fn take_item(&mut self, name: &str) -> Result<String, String>` | Pick up item from room. Checks weight limit. |
| `drop_item` | `fn drop_item(&mut self, name: &str) -> Option<String>` | Drop item from inventory. Returns item name or `None`. |
| `equip_item` | `fn equip_item(&mut self, name: &str) -> Result<String, String>` | Equip weapon or armor from inventory. |
| `unequip_slot` | `fn unequip_slot(&mut self, slot: &str) -> Result<String, String>` | Unequip by slot: `"weapon"` or `"armor"`. |
| `use_item` | `fn use_item(&mut self, name: &str) -> Result<String, String>` | Consume edible/drinkable or read a readable. |
| `examine_item` | `fn examine_item(&self, name: &str) -> Option<String>` | Details for an item in inventory or room. |
| `carry_weight` | `fn carry_weight(&self) -> (i32, i32)` | (current weight, max weight). Max = hardiness × 10. |
| `add_system` | `fn add_system(&mut self, system: Box<dyn System>)` | Register a custom system. |
| `process_command` | `fn process_command(&mut self, input: &str) -> Vec<String>` | Dispatch input to systems, run event observers. |

---

## Room

```rust
pub struct Room {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, i32>,  // direction → room_id
    pub is_dark: bool,
}
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new(id: i32, name: String, description: String) -> Self` | Create room (exits empty, not dark). |
| `get_exit` | `fn get_exit(&self, direction: &str) -> Option<i32>` | Lookup exit (case-insensitive). |

---

## Item & ItemType

```rust
pub struct Item {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub weight: i32,
    pub value: i32,
    pub is_weapon: bool,
    pub weapon_type: i32,     // 1=axe, 2=bow, 3=club, 4=spear, 5=sword
    pub weapon_dice: i32,
    pub weapon_sides: i32,
    pub is_armor: bool,
    pub armor_value: i32,
    pub is_takeable: bool,    // default: true
    pub is_wearable: bool,
    pub is_light_source: bool, // reveals dark rooms when carried
    pub container_capacity: i32, // reserved
    pub location: i32,        // room_id, 0=carried, -1=worn, -2=shop stock
}

pub enum ItemType {
    Weapon, Armor, Treasure, Readable,
    Edible, Drinkable, Container, Normal,
}
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new(id, name, description, item_type, weight, value) -> Self` | Create item with defaults (not weapon/armor, takeable). |
| `get_damage` | `fn get_damage(&self) -> i32` | Roll `weapon_dice` d `weapon_sides`. Returns 0 if not a weapon. |

### JSON `type` values

`"weapon"`, `"armor"`, `"treasure"`, `"readable"`, `"edible"`, `"drinkable"`, `"container"`, `"normal"` (default).

---

## Monster & MonsterStatus

```rust
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub room_id: i32,
    pub hardiness: i32,
    pub agility: i32,
    pub friendliness: MonsterStatus,
    pub courage: i32,
    pub weapon_id: Option<i32>,
    pub armor_worn: i32,
    pub gold: i32,
    pub is_dead: bool,
    pub current_health: i32,   // initialized to hardiness
}

pub enum MonsterStatus {
    Friendly, Neutral, Hostile,
}
```

### JSON `friendliness` values

`"friendly"`, `"neutral"` (default), `"hostile"` — lowercase strings.

---

## Player

```rust
pub struct Player {
    pub name: String,                         // default: "Adventurer"
    pub hardiness: i32,                       // default: 12 (also max HP)
    pub agility: i32,                         // default: 12
    pub charisma: i32,                        // default: 12
    pub weapon_ability: HashMap<i32, i32>,    // weapon_type → skill (default: 5)
    pub armor_expertise: i32,                 // default: 0
    pub gold: i32,                            // default: 200
    pub current_room: i32,
    pub current_health: i32,                  // starts at hardiness
    pub inventory: Vec<i32>,                  // item IDs
    pub equipped_weapon: Option<i32>,
    pub equipped_armor: Option<i32>,
    pub experience_points: i32,
    pub level: i32,                           // default: 1
    pub reputation: HashMap<String, i32>,     // quest/faction reputation
}
```

### Carry weight

Max carry weight = `hardiness × 10`. Attempting to take an item that exceeds this fails.

### Level-up

Handled by `CombatSystem`. XP threshold = `level × 100`. On level-up: hardiness +2, agility +1, current_health restored.

---

## GameEvent

Events emitted during gameplay for cross-system communication (e.g. quest objective tracking).

```rust
pub enum GameEvent {
    MonsterKilled { monster_name: String, room_id: i32 },
    ItemCollected { item_name: String, item_id: i32 },
    RoomEntered { room_id: i32 },
    ItemUsed { item_name: String },
}
```

After each command, `process_command()` calls `on_events()` on every system with the pending events, then clears the event buffer.

---

## System Trait

```rust
pub trait System {
    /// Handle a player command. Return Some(output) to claim it; None to pass.
    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String>;

    /// React to game events (optional). Called after on_command for all systems.
    fn on_events(
        &mut self,
        _events: &[GameEvent],
        _game: &mut AdventureGame,
    ) -> Option<String> {
        None
    }
}
```

### Dispatch rules

1. `process_command()` lowercases the first word as the verb, remaining words as args.
2. Each system's `on_command()` is called in registration order. The **first** to return `Some` claims the command.
3. If any `GameEvent`s were emitted, `on_events()` is called on **all** systems (observer pattern).

---

## Built-in Systems

### BasicWorldSystem

Commands: `look`/`l`, `go`/`move <dir>`, `north`/`south`/`east`/`west`/`up`/`down` (and `n`/`s`/`e`/`w`/`u`/`d`), `say`/`shout`/`yell <text>`, `help`/`?`.

Direction abbreviations are expanded to full words before exit lookup.

### InventorySystem

Commands: `inventory`/`i`/`inv`, `take`/`get`/`grab`/`pick`, `drop`, `equip`/`wield`/`wear`, `unequip`/`remove`, `use`/`consume`/`drink`/`eat`, `examine`/`x`/`inspect`.

### CombatSystem

Commands: `attack`/`fight`/`kill <target>`, `flee`/`run`/`escape`, `status`/`stats`/`score`.

Combat resolution (since 4.1):
- Player damage: weapon dice roll + `weapon_ability[weapon_type] / 5`; unarmed uses the best ability value. Monster `armor_worn` reduces damage; floor 1.
- Monster counter-attack: `max(1, agility / 3 + 1 - (equipped armor_value + armor_expertise / 5))`.
- On monster death: gold and XP awarded, every level-up applied, loot and weapon dropped into the room.
- Flee chance: `(agility / 20 - highest hostile courage / 1000)`, clamped 10–90%. The destination prefers an exit with no hostiles and not the previous room.
- XP thresholds use cumulative `level * 100` and loop, so a large award can grant multiple levels.

### QuestSystem

Commands: `quests`/`journal`, `accept <quest_id>`, `complete`/`finish <quest_id>`, `abandon <quest_id>`.

Implements `on_events()` to auto-advance quest objectives on `MonsterKilled`, `ItemCollected`, `RoomEntered`, and `TalkedTo` events. Stages advance automatically; `complete` verifies every objective before paying gold, XP, item, and reputation rewards.

### ShopSystem

Commands: `shop`/`trade`/`store`, `buy`/`purchase <item>`, `sell <item>`.

Shops live in the current room, have finite gold and stock, and price items from `value` and the player's `charisma`.

---

## AdventureDocument (Canonical Format)

`AdventureDocument` is the single authoring model. See
[Adventure_Schema.md](Adventure_Schema.md) for every field and example.

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_json_str` | `fn from_json_str(&str) -> Result<Self, AdventureError>` | Parse either legacy format into a normalized document. |
| `from_value` | `fn from_value(Value) -> Result<Self, AdventureError>` | Same, from a parsed JSON value. |
| `load_json_file` | `fn load_json_file(path) -> Result<Self, AdventureError>` | Load and validate from disk. |
| `save_json_file` | `fn save_json_file(&self, path) -> Result<(), AdventureError>` | Validate, normalize, and write atomically. |
| `to_json_string` | `fn to_json_string(&self) -> Result<String, AdventureError>` | Serialize the normalized canonical form. |
| `report` | `fn report(&self) -> ValidationReport` | Structured errors and warnings. |
| `validate` | `fn validate(&self) -> Result<(), AdventureError>` | Fail on the first validation error. |

Runtime helpers:

| Method | Description |
|--------|-------------|
| `AdventureGame::load_from_str` | Load an adventure from a JSON string. |
| `AdventureGame::load_from_value` | Load an adventure from a JSON value. |
| `AdventureGame::load_document` | Build runtime state from an `AdventureDocument`. |
| `AdventureGame::restart` | Start the current adventure over. |
| `Engine::from_document` | Build an engine from a document. |
| `Engine::load_from_str` | Build an engine from JSON text. |
| `Engine::restart` | Restart and refresh the intro text. |
| `Engine::won` / `Engine::ending` | Victory state and ending text. |

`AdventureGame` exposes `document`, `shops`, `game_over`, `game_won`, `ending_text`,
`visited_rooms`, and `completed_quests` for tooling.

## Adventure (Secondary Format)

The `Adventure` struct is the **legacy string-ID format**. It is still readable (and converted to a canonical `AdventureDocument`) but the TUI now edits the canonical numeric format.

```rust
pub struct Adventure {
    pub id: String,
    pub title: String,
    pub start_room: String,
    pub rooms: Vec<AdventureRoom>,
    pub player_start_inventory: Vec<AdventureItem>,
}
```

### Validation

`Adventure::validate()` checks that `start_room` exists and all exit targets reference valid room IDs.

---

## Error Types

```rust
pub enum AdventureError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Validation(String),
}
```

`AdventureError` implements `From<std::io::Error>` and `From<serde_json::Error>`.

`Engine` and `AdventureGame` methods return `Box<dyn std::error::Error>` for flexibility.

---

## Usage Examples

### Minimal CLI player

```rust
use mellivorex_rs::Engine;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::load("examples/adventures/demo_adventure.json")?;
    let intro = engine.intro();
    if !intro.is_empty() {
        println!("{}", intro);
    }
    println!("{}", engine.look());

    let stdin = io::stdin();
    loop {
        if engine.is_over() { break; }
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();
        if input == "quit" { break; }
        for line in engine.send(input) {
            println!("{}", line);
        }
    }
    Ok(())
}
```

### Custom System

```rust
use mellivorex_rs::{System, AdventureGame, GameEvent};

pub struct GreeterSystem;

impl System for GreeterSystem {
    fn on_command(
        &mut self, command: &str, _args: &[&str], _game: &mut AdventureGame,
    ) -> Option<String> {
        match command {
            "hello" | "greet" => Some("Hello, adventurer!".to_string()),
            _ => None,
        }
    }

    fn on_events(
        &mut self, events: &[GameEvent], _game: &mut AdventureGame,
    ) -> Option<String> {
        for event in events {
            if let GameEvent::RoomEntered { room_id } = event {
                return Some(format!("Welcome to room {}!", room_id));
            }
        }
        None
    }
}
```

Register it before loading:

```rust
let mut engine = Engine::new("adventure.json");
engine.game.add_system(Box::new(GreeterSystem));
engine.start()?;
```
