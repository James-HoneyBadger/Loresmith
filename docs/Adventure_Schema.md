# Adventure Schema (v2)

Mellivorex reads two historical formats and writes a single canonical one:

- **Canonical integer-ID schema** (`schema_version: 2`) — documented here.
- **Legacy integer-ID schema** (4.0.x) — still readable; missing fields receive defaults.
- **Legacy string-ID schema** (the bundled demo) — still readable and converted to numeric IDs on load.

Unknown fields are preserved in memory and written back, so editor metadata survives round-trips. A document that uses only 4.0.x-era shapes can still be opened by 4.0.3; new shapes (locked exits, dialogue, shops, stages, endings) require 4.1+.

## Top-level object

```json
{
  "schema_version": 2,
  "id": "clockwork_citadel",
  "title": "Clockwork Citadel",
  "description": "A short summary for tooling.",
  "intro": "The gates groan shut behind you.",
  "author": "Your Name",
  "start_room": 1,
  "rooms": [],
  "items": [],
  "monsters": [],
  "quests": [],
  "shops": [],
  "ending": { },
  "player_start_inventory": [1, 2]
}
```

| Field | Required | Meaning |
|-------|----------|---------|
| `schema_version` | no | Written as `2`; legacy files default to it. |
| `id` | no | Stable identifier; generated from the title if missing. |
| `title` | yes | Shown in the opening banner. |
| `description` | no | Editor/tooling metadata. |
| `intro` | no | Opening text. |
| `author` | no | Preserved metadata. |
| `start_room` | yes | Room ID where the player starts. |
| `rooms` | yes | At least one room. |
| `items` | no | Item definitions. |
| `monsters` | no | NPCs and enemies. |
| `quests` | no | Quest definitions. |
| `shops` | no | Shop definitions. |
| `ending` | no | Optional victory/defeat text. |
| `player_start_inventory` | no | Item IDs or inline item objects. |
| `settings` | no | Preserved editor metadata (difficulty flags, etc.). |

## Rooms and exits

```json
{
  "id": 1,
  "name": "Village Square",
  "description": "A fountain murmurs in the dusk.",
  "is_dark": false,
  "exits": {
    "north": 2,
    "palace": 5,
    "east": {
      "to": 3,
      "locked": true,
      "key": "Brass Key",
      "consume_key": false,
      "message": "The iron door is barred."
    }
  },
  "items": [10, 11]
}
```

- Exit keys are case-insensitive; the loader lowercases them.
- A plain number is an unlocked exit. The object form adds a lock.
- `key` is matched case-insensitively against item names (or an item ID string).
- `consume_key` removes the key from the player's inventory when used.
- `message` is shown when the exit is locked and the key is missing.
- `items` lists item IDs placed in the room. These set each item's `location` and override the item's own `location`.
- `is_dark` hides the room description unless the player carries an item with `is_light_source: true`.

## Items

```json
{
  "id": 10,
  "name": "Brass Lantern",
  "description": "Warm light spills through the glass.",
  "type": "treasure",
  "weight": 2,
  "value": 25,
  "is_takeable": true,
  "is_light_source": true,
  "location": 1
}
```

Weapons:

```json
{
  "id": 11,
  "name": "Iron Sword",
  "description": "A dependable blade.",
  "type": "weapon",
  "weight": 3,
  "value": 100,
  "is_weapon": true,
  "weapon_type": 5,
  "weapon_dice": 1,
  "weapon_sides": 8,
  "location": 1
}
```

Armor:

```json
{
  "id": 12,
  "name": "Boiled Leather",
  "description": "Scarred but serviceable.",
  "type": "armor",
  "weight": 4,
  "value": 60,
  "is_armor": true,
  "is_wearable": true,
  "armor_value": 3,
  "location": 1
}
```

Consumables use `type: "edible"` or `"drinkable"` and `value` as the HP restored (clamped to 1–20 and capped at hardiness).

`location` semantics:

| Value | Meaning |
|-------|---------|
| `0` | Carried. Unplaced items at 0 become starting inventory unless reserved for monster loot/weapons or shop stock. |
| `-1` | Worn (runtime only). |
| `-2` | Stored in a shop's inventory (runtime only). |
| room ID | On the floor of that room. |
| monster ID | Held by that monster. |

Type values: `weapon`, `armor`, `treasure`, `readable`, `edible`, `drinkable`, `container`, `normal`.

## Monsters and dialogue

```json
{
  "id": 20,
  "name": "Harbormaster Mira",
  "description": "She watches the tide with tired eyes.",
  "room_id": 2,
  "hardiness": 10,
  "agility": 8,
  "friendliness": "friendly",
  "courage": 70,
  "gold": 5,
  "weapon_id": 11,
  "armor_worn": 1,
  "loot": [13],
  "dialogue": {
    "greeting": "The beacon has been dark for three nights.",
    "topics": {
      "beacon": "The lamp needs oil before the storm.",
      "storm": "It will be here by dusk."
    },
    "unknown": "I can't help with that."
  }
}
```

- `friendliness`: `friendly`, `neutral`, `hostile`. Only hostile monsters can be attacked.
- `weapon_id` must reference an item. On death the weapon and every `loot` item move into the monster's room and become takeable.
- `armor_worn` is flat damage reduction for the monster.
- `courage` above 50 slightly reduces the player's flee chance.
- `talk <npc>` uses `greeting`; `ask <npc> about <topic>` matches topic keys case-insensitively, then falls back to `unknown`.

## Quests

```json
{
  "id": 3,
  "title": "Break the Sable Heart",
  "description": "Reach the relay and destroy the core.",
  "giver_npc": "Quartermaster Elira",
  "difficulty": "challenging",
  "objectives": [
    { "type": "reach_room", "target_id": 13, "description": "Reach the relay", "count": 1 },
    { "type": "kill_monster", "target_id": "Core Revenant", "description": "Destroy the core", "count": 1 }
  ],
  "rewards": {
    "gold": 120,
    "xp": 160,
    "items": [40],
    "reputation": { "Citadel": 10 }
  }
}
```

Multi-stage form:

```json
{
  "id": 4,
  "title": "Two Steps",
  "stages": [
    {
      "stage_id": "reach",
      "stage_number": 1,
      "title": "Find the Gate",
      "objectives": [
        { "type": "reach_room", "target_id": 9, "count": 1, "description": "Reach the gate" }
      ],
      "stage_reward_xp": 25
    },
    {
      "stage_id": "talk",
      "stage_number": 2,
      "title": "Convince the Guard",
      "objectives": [
        { "type": "talk_to_npc", "target_id": "Gate Guard", "topic": "gate", "count": 1,
          "description": "Ask about the gate" }
      ]
    }
  ],
  "rewards": { "gold": 50, "xp": 75 }
}
```

Objective types:

| `type` | Progress trigger | `target_id` |
|--------|------------------|-------------|
| `kill_monster` | Killing a matching monster | monster name or ID |
| `collect_item` | Taking a matching item | item name or ID |
| `reach_room` | Entering a matching room | room ID or name |
| `talk_to_npc` | `talk`/`ask` with a matching NPC | NPC name or ID |
| `defend`/`deliver`/`discover`/`puzzle` | Reserved for custom systems | any |

- Objectives use case- and punctuation-insensitive matching on both sides.
- Stages advance automatically when every objective is complete; stage XP is granted immediately.
- Final rewards are paid only by `complete <id>`, which refuses while objectives remain.
- Quests without `stages` are wrapped in one stage. `objectives` mirrors the first stage for 4.0.x compatibility.
- `rewards.items` accepts item IDs or names; names resolve to the first matching item definition.

## Shops

```json
{
  "id": 1,
  "name": "Harbor Chandlery",
  "room_id": 2,
  "gold": 300,
  "stock": [
    { "item_id": 13, "quantity": 2 },
    { "item_id": 14, "quantity": 1 }
  ],
  "buy_multiplier": 1.0,
  "sell_multiplier": 0.5
}
```

- `shop`/`trade` lists every shop in the current room, its stock with buy prices, and the player's sell offers. `shop <name>` filters to one shop.
- Buy price = `ceil(value × buy_multiplier × (1 - 0.01 × charisma))`, minimum 1.
- Sell price = `floor(value × sell_multiplier × (0.5 + 0.01 × charisma))`, minimum 0.
- Charisma is clamped to `0..=50` for pricing, and non-finite or extreme multipliers are clamped to safe ranges.
- A shop never pays more than its own asking price for an item it stocks, which prevents buy/sell loops.
- Buying an item emits both `ItemPurchased` and `ItemCollected`, so collect objectives progress.
- Multiple shops can share a room. `buy` selects the cheapest in-stock offer; `sell` selects the highest offer the shop can afford.
- Shops have finite gold and stock. Selling adds the item back to stock.
- Only one copy of each item ID can be carried at a time.

## Endings

```json
{
  "victory_text": "The beacon burns again.",
  "defeat_text": "The dark takes the harbor.",
  "conditions": {
    "all_quests": true,
    "quests": [1, 2],
    "rooms": [14],
    "items": ["Eternal Flame"]
  }
}
```

All specified conditions are ANDed. Checking happens after movement, item pickup, combat, and quest completion. `all_quests` requires at least one quest and every quest completed. `rooms` requires every listed room to have been visited. `items` requires every named item to be carried.

## Starting inventory

```json
"player_start_inventory": [
  1,
  2,
  { "id": 99, "name": "Travel Ration", "type": "edible", "value": 5 }
]
```

Numeric IDs move existing definitions into the player's inventory. Inline objects create new items; missing IDs are assigned automatically.

## Validation

`AdventureDocument::report()` returns structured errors and warnings:

- Errors: missing start room, duplicate IDs, exits to unknown rooms, room item references, monster rooms/weapons/loot, shop rooms/stock, invalid starting inventory, negative shop gold, non-positive IDs, invalid or non-finite shop multipliers.
- Warnings include empty shop names, duplicate stock entries, non-positive stock quantities, and quest targets that do not match an item, room, or NPC.
- Warnings: unreachable rooms, locked exits without keys, empty quest titles/objectives, unmatched quest targets, endings without conditions, non-positive quest IDs.

Editors surface this report before saving and before starting a play session.

## Save format

Snapshot v2 contains:

```json
{
  "version": 2,
  "adventure_file": "examples/adventures/demo_adventure.json",
  "document": { "schema_version": 2, "...": "..." },
  "player": { },
  "items": { },
  "monsters": { },
  "shops": { },
  "system_states": { },
  "unlocked_exits": [[1, "east"]],
  "visited_rooms": [1, 2],
  "completed_quests": ["1"],
  "game_over": false,
  "game_won": false,
  "ending_text": "",
  "turn_count": 3,
  "autosave_enabled": false,
  "autosave_path": "autosave.json"
}
```

- Saves are portable: the full document is embedded, so the original adventure file is not required.
- v1 snapshots are migrated on load by reading the referenced adventure file.
- Writes are atomic (sibling temporary file + rename).
