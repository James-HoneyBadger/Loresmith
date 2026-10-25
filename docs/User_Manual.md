# Saga Foundry User Manual

**Version 4.1 · Saga Foundry**

Saga Foundry is a text-based adventure game suite. You explore worlds, collect items, fight monsters, and complete quests by typing commands. This manual covers everything you need to play.

---

## Table of Contents

1. [Starting the Game](#starting-the-game)
2. [How to Play](#how-to-play)
3. [Complete Command Reference](#complete-command-reference)
4. [Movement and Navigation](#movement-and-navigation)
5. [Reading a Room](#reading-a-room)
6. [Inventory Management](#inventory-management)
7. [Combat System](#combat-system)
8. [Quests](#quests)
9. [Player Statistics](#player-statistics)
10. [Tips and Strategies](#tips-and-strategies)
11. [Troubleshooting](#troubleshooting)

---

## Starting the Game

### Using the provided scripts

```bash
# Standalone graphical Player (choose from the adventure library)
./Play.sh
./Play.sh path/to/adventure.json

# Terminal player
./Console.sh path/to/adventure.json

# Graphical editor (Studio)
./Studio.sh

# Terminal editor (Workbench)
./Saga.sh
```

The graphical Player autosaves after each turn. If you close it mid-adventure, the next launch offers **Resume** or **Discard** for that recovery save. The sidebar also shows a **Merchant** card when a shop is in your current room.

### Using Cargo directly

```bash
# Standalone graphical player
cargo run --bin sagacraft_play -- path/to/adventure.json

# Terminal player
cargo run --bin sagacraft_player -- path/to/adventure.json

# Graphical editor (Studio)
cargo run --bin sagacraft_ide_gui

# Terminal editor (Workbench)
cargo run --bin sagacraft_ide_tui
```

### Included adventures

| File | Description |
|------|-------------|
| examples/adventures/demo_adventure.json | Two-room introduction to Saga Foundry |
| examples/adventures/shattered_realms_demo.json | 15-room epic fantasy adventure with combat and quests |

---

## How to Play

Saga Foundry is played entirely by typing commands and reading the responses. Everything happens at the keyboard.

When you start a new game you will see the title banner, optional introduction text, and then the first room description:

```
============================================================
              The Shattered Realms
============================================================

An ancient evil stirs beneath the kingdom…

Waking Village - The Morning After
-----------------------------------
You awaken in the smoldering ruins of what was once a peaceful
farming village…

Obvious exits: north, east, south
```

Type a command and press **Enter**. The engine responds, and you continue.

---

## Complete Command Reference

### Movement

| Command | Aliases | Effect |
|---------|---------|--------|
| `north` | `n`, `go north`, `move north` | Move north |
| `south` | `s`, `go south`, `move south` | Move south |
| `east`  | `e`, `go east`,  `move east`  | Move east  |
| `west`  | `w`, `go west`,  `move west`  | Move west  |
| `up`    | `u`, `go up`,    `move up`    | Move up    |
| `down`  | `d`, `go down`,  `move down`  | Move down  |

### Information

| Command | Aliases | Effect |
|---------|---------|--------|
| `look` | `l` | Describe the current room |
| `inventory` | `inv`, `i` | List carried items and weight |
| `status` | `stats`, `score` | Show health, gold, level, and equipment |
| `quests` | `journal` | List active and available quests |
| `help` | `?` | Quick command reminder |

### Items

| Command | Aliases | Effect |
|---------|---------|--------|
| `take <item>` | `get <item>`, `grab <item>`, `pick <item>` | Pick up an item from the room |
| `drop <item>` | — | Drop a carried item into the room |
| `examine <item>` | `inspect <item>`, `x <item>` | See detailed item information |
| `use <item>` | `consume <item>`, `drink <item>`, `eat <item>` | Consume, read, or activate an item |
| `equip <item>` | `wield <item>`, `wear <item>` | Equip a weapon or armor |
| `unequip <slot>` | `remove <slot>` | Remove equipment (`weapon` or `armor`) |

### Combat

| Command | Aliases | Effect |
|---------|---------|--------|
| `attack <target>` | `fight <target>`, `kill <target>` | Attack a hostile monster in the room |
| `flee` | `run`, `escape` | Attempt to flee; chance is `agility/20` adjusted by the pursuer's courage (10–90%) |

### Quests

| Command | Effect |
|---------|--------|
| `quests` | Show active and available quests with objectives |
| `accept <quest_id>` | Accept an available quest |
| `complete <quest_id>` | Turn in a quest whose objectives are all complete |
| `abandon <quest_id>` | Abandon an active quest |

> Aliases: `finish` works the same as `complete`.

### Social and exploration

| Command | Aliases | Effect |
|---------|---------|--------|
| `talk <npc>` | `speak`, `greet`, `hello` | Speak to a nearby NPC (uses their greeting) |
| `ask <npc> about <topic>` | — | Ask about a topic; matches NPC dialogue topics |
| `say <text>` | `shout`, `yell` | Speak aloud; nearby non-hostile NPCs react |
| `unlock <direction>` | — | Unlock an exit with a carried key |
| `restart` | — | Start the adventure over |

### Shops

| Command | Aliases | Effect |
|---------|---------|--------|
| `shop [name]` | `trade`, `store` | List shops, stock, buy prices, and your sell offers in the current room |
| `buy <item>` | `purchase` | Buy one item if you can afford it and carry it |
| `sell <item>` | — | Sell a carried item if the shop can afford it |

Buy prices use the item value, the shop's `buy_multiplier` (default `1.0`), and your
charisma. Sell prices use `sell_multiplier` (default `0.5`) and charisma. Shops have finite
gold and stock, and buying an item counts as collecting it for quest objectives. A shop
never pays more than it charges for the same item. If several shops share a room, `buy`
picks the cheapest in-stock offer and `sell` picks the highest offer a shop can afford.

### Saves

| Command | Effect |
|---------|--------|
| `save [file]` | Save to `savegame.json` or the given path (atomic write) |
| `load [file]` | Load a v1 or v2 snapshot |
| `saves` | List JSON files in the current directory |
| `autosave on [file]` / `off` / `status` | Manage autosave |

### Game Control

| Command | Aliases | Effect |
|---------|---------|--------|
| `quit` | `exit`, `q` | Exit the game |

---

## Movement and Navigation

Each room description ends with a list of available exits:

```
Obvious exits: north, east, down
```

Type the direction (or its one-letter alias) to move. If a direction has no exit you will see:

```
You can't go that way.
```

Adventures can define exits with any name — you may encounter `gates`, `palace`, `oracle`, `secret`, `throne`, and similar in addition to cardinal directions. Always `look` after entering a new room to catch all available exits.

### Dark rooms

Some rooms are flagged `is_dark`. In a dark room you see:

```
It is pitch black. You can't see a thing.
```

Carry an item with `is_light_source: true` (for example a lantern) and the room description is revealed. Without one, items and NPCs stay hidden.

---

## Reading a Room

The `look` command (or just `l`) prints the full room description at any time:

```
Throne Room
-----------
The throne room is a cavernous hall of marble and gold,
lit by massive crystal chandeliers that cast eerie shadows.

Obvious exits: south, secret

You see:
  - Dragon Crown
  - Forbidden Tome

Present:
  - Palace Guard (hostile)
  - Court Advisor
```

- **You see** — items in this room that can be examined or taken.
- **Present** — creatures. Their disposition is shown in parentheses:
  - `(hostile)` — will fight back when attacked
  - `(friendly)` — cannot be attacked
  - *(none)* — neutral

---

## Inventory Management

### Viewing your inventory

```
> inventory
Inventory (7/120 weight):
  - Rusty Sword [wielded]
  - Leather Armor [worn]
  - Ancient Key
  - Health Potion
```

`[wielded]` and `[worn]` mark actively equipped items.

### Carry weight limit

Your maximum carrying capacity is **hardiness × 10**. The default player has hardiness 12, giving a capacity of 120. Each item has a weight value. If picking something up exceeds the limit you will see:

```
Too heavy to carry! (115/120 weight used, Plate Mail weighs 15.)
```

Drop or use something first to free capacity.

### Taking and dropping

```
> take ancient key
Taken: Ancient Key.

> drop ancient key
Dropped: Ancient Key.
```

Items must be present in the current room to take. You must be holding an item to drop it.

### Examining items

```
> examine rusty sword
Rusty Sword
A worn blade with a chipped edge.
Damage: 1d8
Weight: 3  Value: 10 gold
```

`examine` works on things in your **inventory** and on things **in the room** — you do not need to pick something up to inspect it.

### Equipping and unequipping

```
> equip rusty sword
You wield the Rusty Sword.

> equip leather armor
You wear the Leather Armor.

> unequip weapon
Weapon unequipped.

> unequip armor
Armor removed.
```

Only weapons (`is_weapon: true`) and wearable armor (`is_wearable: true` or `is_armor: true`) can be equipped. Unequipping keeps the item in your inventory.

### Using items

| Item type | What happens |
|-----------|--------------|
| Edible, Drinkable | Restores health equal to the item's **value**, clamped to [1, 20]. Health is capped at max hardiness. The item is consumed and removed. |
| Readable | Prints the item description. The item is kept. |
| Anything else | "You fiddle with it but nothing happens." |

```
> use health potion
You consume the Health Potion. Health: 12/12.
```

---

## Combat System

### Initiating combat

You start combat by attacking a monster explicitly:

```
> attack goblin
You attack the Goblin Warrior for 6 damage. It has 4 health remaining.
The Goblin Warrior strikes back for 2 damage. Your health: 10/12.
```

Attempting to attack a non-hostile creature is refused:

```
> attack merchant
You can't bring yourself to attack the friendly Merchant.
```

### How damage is calculated

**Your attack:**
- If you have a weapon equipped, the engine rolls that weapon's dice (e.g. `weapon_dice=1, weapon_sides=8` → 1–8 damage).
- If nothing is equipped, the engine uses your best weapon ability score (default 5) as the maximum of a random roll (1 to max).

**Monster counter-attack:**
- The monster rolls `1` to `(agility ÷ 3) + 1` damage (minimum 2).
- Your equipped armor's `armor_value` plus `armor_expertise / 5` is subtracted, with a minimum of 1 damage.

**Example:** A monster with agility 9 deals up to `(9÷3)+1 = 4` raw damage per counter-attack. With Leather Armor (armor_value 2) you absorb 2 points, reducing the hit to at most 2. A monster with agility 3 would deal `(3÷3)+1 = 2` raw damage.

### Victory and death

- A monster that reaches 0 health is killed. You automatically collect any gold it carried,
  and its loot and weapon fall into the room for you to take.
- If your health reaches 0 the game ends. Use `restart` to try again.
- Adventures may define victory conditions (rooms visited, quests completed, items carried).
  When they are all met, the ending text is shown and the adventure ends; `restart` starts over.

### After a fight

Use `status` to check your health. Heal with Edible or Drinkable items if available, then continue.

### Weapon type reference

| `weapon_type` value | Weapon category |
|---------------------|-----------------|
| 1 | Axe |
| 2 | Bow |
| 3 | Club |
| 4 | Spear |
| 5 | Sword |

---

## Quests

### Listing quests

```
> quests
Active Quests:
- [1] Slay the Goblin King (Moderate)
  Defeat the leader of the warband
  Stage 1/1: Main Objectives
    [ ] Kill Goblin King (0/1)

Available Quests:
- [2] Collect the Herb: Gather healing herbs from the forest
```

### Accepting a quest

```
> accept 3
Accepted quest: Collect the Herb
```

### Completing a quest

The journal shows quest IDs, stages, and objective progress. When every objective shows `[x]`, report completion:

```
> complete 1
Completed quest: Slay the Goblin King (1)
+50 gold
+100 XP
```

Gold and XP are added to your totals immediately.

> `complete` refuses to pay rewards while any objective is unfinished. The journal prints a
> "Ready to turn in" line as soon as the final objective is met.

### How objectives track automatically

| Objective type | When it advances |
|---------------|-----------------|
| Kill | Every time you kill a monster whose ID or name matches the quest target |
| Collect | Every time you `take` an item whose ID or name matches the quest target |
| Explore / Reach | Every time you enter the target room (ID or name) |
| Talk | When you `talk` to or `ask` a matching NPC (optionally requiring a topic) |

Quest progress notifications appear automatically after the triggering action:

```
> attack goblin king
You defeat the Goblin King! (+25 gold)
Quest update:
[Slay the Goblin King] Kill Goblin King (1/1)
```

---

## Player Statistics

The `status` command shows your full character sheet:

```
> status
Player: Adventurer
Health: 10/12
Level: 1  XP: 0
Gold: 200
Weapon: Rusty Sword
Armor: none
Carrying: 5/120 weight
Location: Room 3
```

| Stat | Starting value | Meaning |
|------|---------------|---------|
| Hardiness | 12 | Maximum health; also determines carry capacity (×10) |
| Agility | 12 | Speed; used in monster counter-attack formula |
| Charisma | 12 | NPC interaction modifier |
| Gold | 200 | Currency; gained from slain monsters and quests |
| Level | 1 | Increases with XP; levels up at level×100 XP |

---

## Tips and Strategies

### Exploration
- `look` every time you enter a room — exits and items reset on each `look`.
- Exits can have non-standard names. Read room descriptions for door/portal/passage hints.
- Keep track of visited rooms manually or with notes — there is no built-in map.

### Combat
- Check `status` before a fight. Heal first if your health is under half.
- `examine` a monster's weapon (visible in the room) before attacking heavily armed foes.
- Equip your best weapon and appropriate armor before engaging.
- If a fight is going badly, there is no automatic retreat — plan carefully.

### Inventory
- Consumables with higher `value` heal more HP. Prioritise high-value potions/food.
- Treasure items are mostly dead weight mid-adventure; drop them if encumbered.
- Use `examine` on room items before taking them — some carry crucial story clues in their description.

### Quests
- Accept quests the moment they appear; kill and collect objectives track automatically as you play.
- Complete finished quests promptly to bank rewards before dying.

---

## Troubleshooting

| Symptom | Solution |
|---------|----------|
| `Unknown command: …` | Check spelling; type `help` for valid commands |
| `You can't take that.` | Item is not in the current room, or is flagged non-takeable |
| `Too heavy to carry!` | Drop or consume items to free carry weight |
| `You can't bring yourself to attack…` | Target is not hostile; only attack hostile creatures |
| `There's no X here to attack.` | Name didn't match; try a short partial name |
| Adventure won't load | Verify the path is correct and the JSON is valid |
| Dark room with no light | Look for a torch or lantern item in earlier rooms |

**Bug reports:** https://github.com/James-HoneyBadger/SagaCraft/issues
