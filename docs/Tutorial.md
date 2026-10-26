# Mellivorex Tutorial: Your First Adventure

This tutorial walks you through playing your first Mellivorex adventure from start to finish. You will learn every core mechanic in about 20 minutes.

---

## Before You Start

Make sure Mellivorex is compiled:

```bash
cd ~/Mellivorex
cargo build
```

---

## Step 1 — Launch the Demo

```bash
./Play.sh examples/adventures/demo_adventure.json
```

The standalone graphical Player opens directly into the demo. If you prefer the terminal:

```bash
./Console.sh examples/adventures/demo_adventure.json
# or
cargo run --bin mellivorex_console -- examples/adventures/demo_adventure.json
```

You will see:

```
============================================================
                     Demo Adventure
============================================================

Quiet Village
-------------
A small village with a single cobblestone path and a warm lantern glow.

Obvious exits: north

You see:
  - Ancient Key
```

**What you are looking at:**
- The title banner (`Demo Adventure`)
- The room name (`Quiet Village`)
- The room description
- The list of exits — only `north` is available
- An item in the room: `Ancient Key`

---

## Step 2 — Look Around

Type:
```
look
```

The room description is printed again. `look` (or its alias `l`) is your most-used command — use it whenever you want to refresh your view.

---

## Step 3 — Examine an Item

Before you pick anything up, inspect it:

```
examine ancient key
```

You will see the item's full description, type, weight, and gold value. It is always smart to `examine` before you `take`.

---

## Step 4 — Pick Up an Item

```
take ancient key
```

Response:
```
Taken: Ancient Key.
```

Now check your inventory:

```
inventory
```

```
Inventory (1/120 weight):
  - Ancient Key
```

The `1/120` shows you are carrying 1 weight unit out of a maximum of 120.

---

## Step 5 — Move to a New Room

```
north
```

```
Whispering Forest
-----------------
Tall pines sway as if sharing secrets. The village lies south.

Obvious exits: south
```

You moved north into the forest. The only exit back is south. This demo adventure has two rooms — you have seen both.

---

## Step 6 — Return South

```
south
```

You are back in the village.

---

## Step 7 — Drop an Item

If you want to leave something behind:

```
drop ancient key
```

```
Dropped: Ancient Key.
```

`look` to confirm the key is back in the room:

```
You see:
  - Ancient Key
```

---

## Step 8 — Play a Larger Adventure

Quit the demo:

```
quit
```

Now launch the Shattered Realms adventure, which has combat, items, quests, and 15 rooms:

```
cargo run --bin mellivorex_console -- examples/adventures/shattered_realms_demo.json
```

---

## Step 9 — Find and Equip a Weapon

After reading the opening room, search for items:

```
look
```

Find an item with `take`, then examine it:

```
take survivor's knife
examine survivor's knife
```

```
Survivor's Knife
A crude but sharp blade salvaged from the ruins.
Damage: 1d6
Weight: 1  Value: 5 gold
```

`Damage: 1d6` means the knife rolls 1 to 6 when you attack.

Equip it:

```
equip survivor's knife
```

```
You wield the Survivor's Knife.
```

---

## Step 10 — Check Your Status

```
status
```

```
Player: Adventurer
Health: 12/12
Level: 1  XP: 0
Gold: 200
Weapon: Survivor's Knife
Armor: none
Carrying: 1/120 weight
Location: Room 1
```

You start with full health (12) and 200 gold. Your knife is now showing as your weapon.

---

## Step 11 — Fight a Monster

Explore north to find the Kingsroad. Keep moving and you may encounter a bandit camp or hostile enemy. When enemies appear in a room, `look` will show them:

```
Present:
  - Bandit Raider (hostile)
```

Attack:

```
attack bandit
```

```
You attack the Bandit Raider for 5 damage. It has 3 health remaining.
The Bandit Raider strikes back for 2 damage. Your health: 10/12.
```

Attack again to finish the fight:

```
attack bandit
```

```
You defeat the Bandit Raider! (+8 gold)
```

---

## Step 12 — Heal With a Consumable

If you took damage in the fight, use a healing item. Look around the current room or earlier rooms for food or potions. When you find one:

```
take healing herbs
use healing herbs
```

```
You consume the Healing Herbs. Health: 12/12.
```

Consumables (edible/drinkable) heal HP equal to their **value** (clamped to 1–20, capped at max hardiness) and are then permanently removed.

---

## Step 13 — Try the Quest System

```
quests
```

If the adventure has quests loaded you will see them listed. To accept one:

```
accept 1
```

As you play — killing monsters and collecting items — quest objectives update automatically. When all objectives of a quest are filled, collect your reward:

```
complete 1
```

---

## Step 14 — Try the Workbench (TUI editor)

The terminal Workbench lets you edit adventures while viewing their structure. Quit the player first, then:

```
./Workbench.sh
```

**TUI key bindings:**

| Key | Action |
|-----|--------|
| `:` | Enter command mode |
| `s` | Save the current file |
| `j` / `k` | Navigate lists (vim-style up/down) |
| `q` | Quit |

---

## Step 15 — Try Studio (graphical editor)

```bash
./Studio.sh
# or
cargo run --bin mellivorex_studio
```

Studio provides a full visual editor with tabs for Rooms, Items, Monsters, Quests, Shops, endings, dialogue, and a live Play tab. You can open any `.json` adventure file, edit it, and save it without touching the JSON directly.

The standalone Player remains the focused way to play; Studio is for authoring.

---

## What to Try Next

- Read the **User Manual** for a complete command reference.
- Read the **Game Design Tips** to start building your own adventure.
- Read the **Technical Reference** to understand the JSON schema and system API.
- Create a new adventure file: start with a copy of examples/adventures/demo_adventure.json and expand it room by room.
