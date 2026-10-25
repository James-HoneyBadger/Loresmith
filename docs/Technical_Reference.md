# Saga Foundry Technical Reference

**Version 4.1 · Audience: Developers & Advanced Modders**

This document describes the Saga Foundry engine internals: architecture, data structures, the adventure JSON schema, the system trait API, and build instructions. All information reflects the actual source at `sagacraft_rs/`.

---

## Table of Contents

1. [Project Structure](#project-structure)
2. [Crate Overview](#crate-overview)
3. [Core Types](#core-types)
4. [The System Trait](#the-system-trait)
5. [Built-in Systems](#built-in-systems)
6. [Engine and AdventureGame](#engine-and-adventuregame)
7. [Adventure JSON Schema](#adventure-json-schema)
8. [Game Events](#game-events)
9. [Quest System Internals](#quest-system-internals)
10. [Build and Test](#build-and-test)
11. [Contributing](#contributing)

---

## Project Structure

```
SagaCraft/
├── Cargo.toml                  # Workspace manifest
├── sagacraft_rs/               # Core engine library
│   └── src/
│       ├── lib.rs              # Public re-exports
│       ├── engine.rs           # High-level Engine wrapper
│       ├── document.rs         # Canonical AdventureDocument + legacy adapters
│       ├── game_state.rs       # AdventureGame, runtime types, loaders, saves
│       ├── adventure.rs        # Legacy string-ID format (read support)
│       ├── util.rs             # Atomic file writes
│       └── systems/
│           ├── mod.rs          # System trait definition
│           ├── basic_world.rs  # Movement, look, talk, locks, saves
│           ├── inventory.rs    # Item management
│           ├── combat.rs       # Combat, XP, loot, endings
│           ├── quests.rs       # Quest tracker, stages, rewards
│           └── shops.rs        # Shops and trading
├── sagacraft_play/             # Standalone egui game player
├── sagacraft_ide_gui/          # Studio — egui graphical editor
├── sagacraft_player/           # Console — CLI player
├── sagacraft_ide_tui/          # Workbench — ratatui terminal editor
└── sagacraft_ui/               # Shared egui theme and widgets
```

---

## Crate Overview

Every front-end depends on `sagacraft_rs` for loading, validation, runtime state, systems, and saves. The graphical applications (Player and Studio) also share `sagacraft_ui` for the theme and common widgets. Each application owns its layout and workflow; only the engine and UI foundation are shared.

### Public re-exports (`lib.rs`)

```rust
pub use adventure::{Adventure, AdventureError, AdventureItem, AdventureRoom};
pub use document::{
    AdventureDocument, AdventureSettings, DialogueDef, EndingConditions, EndingDef, ExitDef,
    InventoryRef, ItemDef, LockedExitDef, MonsterDef, ObjectiveDef, ObjectiveKind, QuestDef,
    QuestDifficultyDef, QuestRewardsDef, RewardItem, RoomDef, ShopDef, ShopStockDef, StageDef,
    StructuredObjective, ValidationReport,
};
pub use engine::Engine;
pub use game_state::{
    AdventureGame, Dialogue, GameEvent, Item, ItemType, Monster, MonsterStatus, Player, Room,
    Shop, ShopStock, award_experience,
};
pub use systems::{
    BasicWorldSystem, CombatSystem, InventorySystem, QuestSystem, ShopSystem, System,
};
```

---

## Core Types

### `Item`

Defined in `game_state.rs`.

```rust
pub struct Item {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,     // see ItemType enum
    pub weight: i32,
    pub value: i32,
    pub is_weapon: bool,
    pub weapon_type: i32,        // 1=axe 2=bow 3=club 4=spear 5=sword
    pub weapon_dice: i32,
    pub weapon_sides: i32,
    pub is_armor: bool,
    pub armor_value: i32,
    pub is_takeable: bool,       // default true
    pub is_wearable: bool,
    pub is_light_source: bool,   // reveals dark rooms when carried
    pub container_capacity: i32, // reserved
    pub location: i32,           // room_id | 0=carried | -1=worn | -2=shop stock
}
```

`Item::get_damage()` rolls `weapon_dice` d `weapon_sides` using `rand::rng()`.

**`ItemType` variants (JSON lowercase strings):**

| Rust variant | JSON string | Notes |
|---|---|---|
| `ItemType::Weapon` | `"weapon"` | use `is_weapon:true` for combat |
| `ItemType::Armor` | `"armor"` | use `is_armor:true` + `armor_value` |
| `ItemType::Treasure` | `"treasure"` | decorative value items |
| `ItemType::Readable` | `"readable"` | prints description on `use` |
| `ItemType::Edible` | `"edible"` | heals `value` HP on `use`, consumed |
| `ItemType::Drinkable` | `"drinkable"` | same as edible |
| `ItemType::Container` | `"container"` | no special engine logic yet |
| `ItemType::Normal` | `"normal"` / unrecognised | default |

### `Monster`

```rust
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub room_id: i32,
    pub hardiness: i32,          // max health
    pub agility: i32,            // counter-attack strength
    pub friendliness: MonsterStatus,
    pub courage: i32,
    pub weapon_id: Option<i32>,  // item ID of their weapon
    pub armor_worn: i32,         // static armor reduction (unused by player)
    pub gold: i32,               // dropped on death
    pub is_dead: bool,
    pub current_health: i32,          // initialized to hardiness
    pub dialogue: Option<Dialogue>,   // greeting, topics, unknown fallback
    pub loot: Vec<i32>,               // item IDs dropped on death
}
```

`MonsterStatus` variants: `Friendly`, `Neutral`, `Hostile`. Only `Hostile` monsters can be attacked.
Friendly and neutral NPCs respond to `talk` and `ask`.

Counter-attack damage formula:
`max(1, agility / 3 + 1 - (player armor_value + armor_expertise / 5))`.
On death, `loot` items and the monster's `weapon_id` move into its room and become takeable.

### `Room`

```rust
pub struct Room {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, i32>,  // direction string -> room id
    pub is_dark: bool,
}
```

Items are not stored in `Room`; they are found via `Item::location == room.id`.

### `Player`

```rust
pub struct Player {
    pub name: String,
    pub hardiness: i32,           // default 12; health pool & carry multiplier
    pub agility: i32,             // default 12
    pub charisma: i32,            // default 12
    pub weapon_ability: HashMap<i32, i32>, // weapon_type -> ability (1-5 → 5)
    pub armor_expertise: i32,
    pub gold: i32,                // default 200
    pub current_room: i32,
    pub current_health: i32,          // starts at hardiness
    pub inventory: Vec<i32>,      // item IDs
    pub equipped_weapon: Option<i32>,
    pub equipped_armor: Option<i32>,
    pub experience_points: i32,
    pub level: i32,               // default 1
}
```

Max carry weight = `player.hardiness * 10`.

### `AdventureGame`

The central mutable game object. Holds all state and the list of active systems.

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

**Key methods:**

| Method | Description |
|--------|-------------|
| `load_adventure() -> Result<String, _>` | Parses JSON; returns opening header string |
| `process_command(&str) -> Vec<String>` | Main dispatch; runs primary + observer passes |
| `look() -> String` | Renders current room |
| `move_player(&str) -> Option<String>` | Moves player, emits `RoomEntered` event |
| `take_item(&str) -> Result<String, String>` | Picks up item, checks weight, emits `ItemCollected` |
| `drop_item(&str) -> Option<String>` | Drops item, clears equip slots if needed |
| `equip_item(&str) -> Result<String, String>` | Equips weapon or wearable armor |
| `unequip_slot(&str) -> Result<String, String>` | Unequips `"weapon"` or `"armor"` slot |
| `use_item(&str) -> Result<String, String>` | Consumes, reads, or activates item |
| `examine_item(&str) -> Option<String>` | Returns full item details from inventory or room |
| `carry_weight() -> (i32, i32)` | Returns `(current, max)` carry weight |
| `get_items_in_room(room_id) -> Vec<&Item>` | Items whose `location == room_id` |
| `get_monsters_in_room(room_id) -> Vec<&Monster>` | Alive monsters in room |

---

## The System Trait

All game logic is plugged in via the `System` trait (`systems/mod.rs`):

```rust
pub trait System {
    /// Primary handler: return Some(output) to claim the command; None to pass.
    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String>;

    /// Called after every command round when there are pending game events.
    /// Return Some(output) to append an observer message (e.g. quest updates).
    /// The default implementation is a no-op.
    fn on_events(
        &mut self,
        _events: &[GameEvent],
        _game: &mut AdventureGame,
    ) -> Option<String> {
        None
    }
}
```

**Dispatch in `process_command`:**

1. The verb is extracted and lowercased.
2. Systems are iterated in registration order. The **first** system that returns `Some(...)` claims the command.
3. After the primary pass, if any `GameEvent`s were emitted, `on_events()` is called on **all** systems. This is the observer pass — systems react to pending events without owning the command.
4. The `events` buffer is cleared after the observer pass.

To add a custom system:

```rust
game.add_system(Box::new(MySystem::default()));
```

Register before `load_adventure` to ensure `init`-style behaviour in the first observer pass.

---

## Built-in Systems

### `BasicWorldSystem`

Handles: `look`/`l`, direction movement (all six cardinal directions plus aliases `n`/`s`/`e`/`w`/`u`/`d`), `go`/`move <dir>`, `talk`/`speak`/`greet`/`hello`, `ask <npc> about <topic>`, `say`/`shout`/`yell`, `unlock <dir>`, `save`, `load`, `saves`, `autosave`, `restart`, `help`/`?`.

Direction abbreviations are expanded to full words before exit lookup. `talk` and `ask` use NPC dialogue data and push `TalkedTo` events. Moving into a locked exit attempts to unlock it with a carried key.

### `InventorySystem`

Handles: `inventory`/`inv`/`i`, `take`/`get`/`grab`/`pick`, `drop`, `equip`/`wield`/`wear`, `unequip`/`remove`, `use`/`consume`/`drink`/`eat`, `examine`/`inspect`/`x`.

### `CombatSystem`

Handles: `attack`/`fight`/`kill <target>`, `flee`/`run`/`escape`, `status`/`stats`/`score`.

Combat flow per `attack` call:
1. Find matching monster in current room (case-insensitive partial name match).
2. Refuse if monster is not `Hostile`.
3. Calculate player damage and apply to monster HP.
4. If monster dies: set `is_dead = true`, transfer gold, award XP (looping level-ups), drop loot/weapon into the room, and push `MonsterKilled`.
5. If monster survives: calculate monster counter-attack with armor mitigation, apply to player HP. If player HP ≤ 0, set `game_over = true`.

### `QuestSystem`

Handles: `quests`/`journal`, `accept <id>`, `complete`/`finish <id>`, `abandon <id>`.

Implements `on_events()` to auto-advance quest objectives on `MonsterKilled`, `ItemCollected`, `RoomEntered`, and `TalkedTo` events. It supports Kill, Collect, Explore, Talk, and the reserved custom types. Objectives match by ID or by a case- and punctuation-insensitive name. Stages advance automatically; `complete` verifies the final stage and pays gold, XP, item, and reputation rewards.

### `ShopSystem`

Handles: `shop`/`trade`/`store`, `buy`/`purchase <item>`, `sell <item>`. Shops are selected by the current room (lowest shop ID if several), have finite gold and stock, and price items from `value` and the player's `charisma`.

---

## Engine and AdventureGame

`Engine` (`engine.rs`) is a thin wrapper that pre-wires all five default systems (world, inventory, combat, quests, shops):

```rust
impl Engine {
    pub fn new(adventure_path: impl Into<String>) -> Self;
    pub fn start(&mut self) -> Result<String, Box<dyn Error>>;
    pub fn load(path: impl Into<String>) -> Result<Self, Box<dyn Error>>;
    pub fn intro(&self) -> &str;
    pub fn send(&mut self, input: &str) -> Vec<String>;
    pub fn look(&self) -> String;
    pub fn is_over(&self) -> bool;
}
```

- `new()` creates the engine with all five systems registered.
- `start()` calls `load_adventure()` and captures the intro text.
- `load()` is a shorthand for `new()` + `start()`.
- `intro()` returns the intro/banner text captured at load time.
- `send()` passes one line of input to `process_command()` and returns the output lines.

Example:
```rust
let mut engine = Engine::load("examples/adventures/demo_adventure.json")?;
println!("{}", engine.intro());
println!("{}", engine.look());
for line in engine.send("north") {
    println!("{}", line);
}
```

---

## Adventure JSON Schema

This section documents the 4.0.x-compatible integer-ID shape. The canonical
`schema_version: 2` format adds locked exits, dialogue, shops, quest stages, endings, and
starting inventory; see [Adventure_Schema.md](Adventure_Schema.md) for those fields.

Every example here is still accepted by `AdventureDocument::from_json_str` and
`AdventureGame::load_adventure`.

### Top-level object

```json
{
  "id": "my_adventure",          // string; adventure identifier
  "title": "My Adventure",       // string; shown in banner
  "intro": "Once upon a time…",  // string; optional intro text
  "description": "…",            // string; metadata only
  "start_room": 1,               // integer; starting room id
  "rooms":    [ … ],
  "items":    [ … ],
  "monsters": [ … ],
  "quests":   [ … ]
}
```

### Room object

```json
{
  "id": 1,                           // integer, must be unique
  "name": "Village Square",          // string
  "description": "Cobblestones…",    // string
  "is_dark": false,                  // boolean, optional (default false)
  "exits": {                         // object: direction string -> room id int
    "north": 2,
    "east": 3,
    "palace": 5
  },
  "items": [1, 2, 3]                 // array of item ids starting in this room
}
```

`exits` keys can be any string; standard values are `north`, `south`, `east`, `west`, `up`, `down`.

Rooms referenced in `exits` must exist; the loader reports a validation error otherwise. Exit keys are lowercased on load.

The `items` array in a room lists item IDs whose `location` field will be interpreted as that room's ID. Alternatively, set `"location": <room_id>` on each item directly.

### Item object

```json
{
  "id": 101,
  "name": "Iron Sword",
  "description": "A dependable one-handed blade.",
  "type": "weapon",          // see ItemType table; key is "type" not "item_type"
  "weight": 3,
  "value": 25,
  "is_weapon": true,
  "weapon_type": 5,          // 1=axe 2=bow 3=club 4=spear 5=sword
  "weapon_dice": 1,
  "weapon_sides": 8,
  "is_armor": false,
  "armor_value": 0,
  "is_takeable": true,       // default true
  "is_wearable": false,
  "location": 1              // room_id where item starts; 0 = player inventory
}
```

For **armor** items set `"is_armor": true` and `"is_wearable": true`.

For **consumables** set `type` to `"edible"` or `"drinkable"` and set `value` to the HP restored.

### Monster object

```json
{
  "id": 201,
  "name": "Goblin Warrior",
  "description": "A small but fierce goblin.",
  "room_id": 3,
  "hardiness": 8,            // max health
  "agility": 6,              // counter-attack strength
  "friendliness": "hostile", // "friendly" | "neutral" | "hostile"
  "courage": 80,             // unused by default combat logic (future)
  "weapon_id": 301,          // optional; item id of monster's weapon
  "armor_worn": 1,           // static armor reduction for monster (future)
  "gold": 10
}
```

### Quest object

```json
{
  "id": 1,
  "title": "Slay the Goblin King",
  "description": "The Goblin King terrorises the village.",
  "objectives": [
    {
      "type": "kill_monster",   // "kill_monster"|"collect_item"|"reach_room"|"talk_to_npc"
      "target_id": 202,         // monster ID (names are also accepted)
      "description": "Kill the Goblin King"
    },
    {
      "type": "collect_item",
      "target_id": 105,
      "description": "Recover the stolen crown"
    }
  ]
}
```

> **Note:** Quest `id` must be an integer. Use the integer (e.g. `accept 1`, `complete 1`) in-game.

---

## Game Events

`GameEvent` (`game_state.rs`) is used as an inter-system bus:

```rust
pub enum GameEvent {
    MonsterKilled { monster_name: String, room_id: i32 },
    ItemCollected { item_name: String, item_id: i32 },
    RoomEntered   { room_id: i32 },
    ItemUsed      { item_name: String },
    TalkedTo      { npc_name: String, topic: Option<String> },
    ItemPurchased { item_name: String, item_id: i32 },
    ItemSold      { item_name: String, item_id: i32 },
    ExitUnlocked  { room_id: i32, direction: String },
    QuestCompleted { quest_id: String, title: String },
    GameWon       { text: String },
}
```

Events are pushed to `game.events` during the primary pass and consumed in the observer pass (`on_events()` call). Custom systems can subscribe to events by implementing `on_events()` and reading the events slice.

---

## Quest System Internals

`QuestSystem` (`systems/quests.rs`) manages `QuestTracker` and a pool of available quests.

**Key types:**

| Type | Purpose |
|------|---------|
| `Quest` | Named quest with stages and rewards |
| `QuestStage` | One phase of a quest; holds objectives |
| `QuestObjective` | Single goal with a progress counter |
| `QuestReward` | XP, gold, and items |
| `QuestTracker` | Owns active/completed/failed sets and history |

**Status lifecycle:**

```
Available → Active → Completed
                  → Abandoned (removed from the tracker)
```

Objectives advance on `MonsterKilled`, `ItemCollected`, `RoomEntered`, and `TalkedTo`.
Stages advance automatically; final rewards are paid only by `complete`, which verifies
that every objective is finished.

---

## Shops, Locks, Light, and Endings

These systems are documented field-by-field in [Adventure_Schema.md](Adventure_Schema.md).

- **Locks:** `Room.locked_exits` holds runtime lock state; `move_player` unlocks with a
  carried key and emits `ExitUnlocked`.
- **Light:** `Item.is_light_source` lets `look` reveal dark rooms.
- **Shops:** `AdventureGame.shops` holds runtime stock/gold; `ShopSystem` handles pricing.
- **Endings:** `AdventureGame.check_ending` evaluates the document's conditions after
  movement, pickups, combat, and quest completion and sets `game_won`.
- **Saves:** snapshot v2 embeds the full document, writes atomically, and migrates v1.

---

## Build and Test

### Prerequisites

- Rust 1.95+ (`rustup update stable`)
- GUI build additionally requires system OpenGL/EGL headers

### Compile and run

```bash
# Debug build (faster compile, slower runtime)
cargo build

# Release build
cargo build --release

# Run standalone graphical player
cargo run --bin sagacraft_play -- examples/adventures/demo_adventure.json

# Run CLI player
cargo run --bin sagacraft_player -- examples/adventures/demo_adventure.json

# Run terminal editor
cargo run --bin sagacraft_ide_tui

# Run graphical editor
cargo run --bin sagacraft_ide_gui

# Run tests
cargo test

# Run tests for the core library only
cargo test -p sagacraft_rs

# Generate API documentation
cargo doc --open
```

### Compile-time warnings

`ashpd` (Linux portal integration used by egui) may emit future-incompatibility warnings as of Rust 1.95+. These are upstream issues and do not affect functionality.

---

## Contributing

1. Fork the repository: https://github.com/James-HoneyBadger/SagaCraft
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Implement and test your changes
4. Run `cargo fmt && cargo clippy` before committing
5. Open a pull request against `main`

**Adding a new system:**
1. Create `sagacraft_rs/src/systems/my_system.rs`
2. Implement `System` trait
3. Register in `engine.rs` inside `Engine::new`
4. Add `pub mod my_system;` to `systems/mod.rs`
5. Re-export from `lib.rs` if needed by front-ends
