# Saga Foundry Game Design Tips

**Audience:** Adventure designers creating `.json` files for the Saga Foundry engine.

This guide covers the craft of making text adventures people actually want to play: how to write memorable rooms, balance combat, pace a story, and avoid common pitfalls.

---

## Table of Contents

1. [Setting Up Your First Adventure](#setting-up-your-first-adventure)
2. [Writing Effective Room Descriptions](#writing-effective-room-descriptions)
3. [Designing Your Map](#designing-your-map)
4. [Items: Utility vs Atmosphere](#items-utility-vs-atmosphere)
5. [Combat Balancing](#combat-balancing)
6. [Quest Design](#quest-design)
7. [NPC Design](#npc-design)
8. [Pacing and Flow](#pacing-and-flow)
9. [Playtesting Checklist](#playtesting-checklist)
10. [Common Pitfalls](#common-pitfalls)
11. [Reference: JSON Quick-Start Template](#reference-json-quick-start-template)

---

## Setting Up Your First Adventure

Start small. A two-to-five room adventure you actually finish is more valuable than a sprawling hundred-room world left half complete.

### Minimum viable adventure

```json
{
  "id": "my_adventure",
  "title": "My First Adventure",
  "intro": "A short scene-setter sentence.",
  "start_room": 1,
  "rooms": [
    {
      "id": 1,
      "name": "Starting Room",
      "description": "Brief, vivid description. One exit leads forward.",
      "exits": { "north": 2 },
      "items": [101]
    },
    {
      "id": 2,
      "name": "Ending Room",
      "description": "The story concludes here.",
      "exits": { "south": 1 }
    }
  ],
  "items": [
    {
      "id": 101,
      "name": "Old Map",
      "description": "A tattered map with a red X marking the north cave.",
      "type": "readable",
      "weight": 0,
      "value": 0,
      "is_takeable": true,
      "location": 1
    }
  ],
  "monsters": [],
  "quests": []
}
```

Once this loads and plays correctly, expand by one room or one item at a time.

---

## Writing Effective Room Descriptions

The room description is the most important text in your adventure. Players will `look` repeatedly — make every word earn its place.

### The four-sentence structure

| Sentence | Job | Example |
|----------|-----|---------|
| 1 | Immediate sensory impression | "Smoke hangs thick in the air." |
| 2 | What the player sees | "Overturned tables and scattered papers fill the tavern floor." |
| 3 | Detail or atmosphere | "A mournful tune plays from somewhere deeper in the building." |
| 4 | Optional hint | "A glint of metal catches your eye beneath the nearest table." |

### Do
- Use concrete nouns and active verbs: *"A torch gutters in the iron bracket"* not *"There is some light."*
- Mark portable light sources with `is_light_source: true`; dark rooms are otherwise unreadable.
- Imply exits through description: *"A heavy oak door stands to the north"* before the exits list.
- Hint at items without name-dropping them: *"Something shiny"*, not *"You see the Golden Key (takes 0 weight)"*.
- Vary sentence rhythm — mix short punchy sentences with longer atmospheric ones.

### Don't
- Repeat information the exits list already provides: "You can go north, south, or east" is redundant.
- Write walls of text. Three to six sentences is ideal.
- Use generic filler: *"a room"*, *"some stuff"*, *"things"*.
- Put gameplay instructions in descriptions: *"Pick up the key to proceed"* breaks immersion.

---

## Designing Your Map

A good map has shape and purpose. Plan it before writing descriptions.

### Map topologies

| Shape | Good for | Risk |
|-------|----------|------|
| **Linear** (A→B→C→D) | Tight narrative, tutorial | Boring if too long |
| **Hub and spoke** (central room, radiating paths) | Exploration, choice | Confusing without landmarks |
| **Loop** (paths that circle back) | Replayability, shortcuts | Disorienting if rooms look alike |
| **Layer cake** (surface → dungeon → deeper dungeon) | Epic scale | Backtracking frustration |

### Practical rules

1. **Every room needs a purpose.** If a room exists solely to exist, cut it or merge it.
2. **Use non-cardinal exits sparingly.** `palace`, `oracle`, `gates` are memorable but players must discover them in text.
3. **Provide safe anchors.** Give players a clearly-described home base to return to.
4. **Avoid dead ends except deliberately.** If a corridor leads nowhere, players assume a bug.
5. **Distance matters.** Don't put an essential healing item one room away from the final boss with no backtrack option.

### Connections table

Draw your map as a table before writing JSON:

```
Room  Name                North  South  East  West  Up  Down
1     Village Square      2      –      3     –     –   –
2     Kingsroad           4      1      5     –     –   –
3     Forest Entrance     –      –      –     1     –   –
…
```

Then translate each row into a room JSON object. This prevents broken exit references.

---

## Items: Utility vs Atmosphere

Items serve two purposes: mechanical utility (weapons, armor, consumables) and environmental storytelling (atmosphere items).

### Utility items

| Use | Fields to set |
|-----|---------------|
| Weapon | `is_weapon:true`, `weapon_type`, `weapon_dice`, `weapon_sides` |
| Armor | `is_armor:true`, `is_wearable:true`, `armor_value` |
| Healing consumable | `type:"edible"` or `"drinkable"`, `value` = HP healed |
| Readable (lore/clue) | `type:"readable"`, description = the full text the player reads |

### Atmosphere items

Atmosphere items (`type:"normal"`, `is_takeable:false`) exist to make rooms feel lived-in. A half-eaten loaf of bread, a cracked mirror, a child's lost shoe — these cost nothing and add everything.

Set `"is_takeable": false` to keep them in place.

### Economy guidelines

- **Default player gold:** 200. Items with `value > 200` will feel rare and rewarding.
- **Weapon balance:** For a fresh player with no weapon, challenge monsters to about hardiness 6–8. With a 1d6 weapon, a monster with hardiness 10 takes 2–5 hits.
- **Consumable healing:** Set `value` between 4 and 8 for standard potions. A `value:12` potion is a full heal for a default player — make it rare.
- **Weight:** Most items should weigh 1–5. A suit of plate armor at weight 15 forces real inventory decisions.

---

## Combat Balancing

The default player has **hardiness 12** (12 HP), no weapon equipped, and no armor.

### Damage math

**Unarmed player:** rolls 1 to the best `weapon_ability` value (5 by default), and adds
`best_ability / 5` (so 1–6 damage at defaults).

**Example weapon progression:**

| Weapon | Dice | Average damage | Role |
|--------|------|---------------|------|
| Rusty Dagger | 1d4 | 2.5 | Starting weapon |
| Short Sword | 1d6 | 3.5 | Early dungeon |
| Longsword | 1d8 | 4.5 | Mid-game |
| Battleaxe | 1d10 | 5.5 | Late-game |
| Great Sword | 2d6 | 7.0 | Boss reward |

**Weapon damage:** weapon dice + `weapon_ability[weapon_type] / 5`, minus monster
`armor_worn`, floor 1.

**Monster counter-attack:** `max(1, agility÷3 + 1 - (player armor_value + armor_expertise÷5))`.
With armor_value 0 and armor_expertise 0 on the player:

| Monster agility | Max hits per round | Attacks to kill default player (no armor) |
|-----------------|-------------------|-------------------------------------------|
| 6 | 3 | 4–12 |
| 9 | 4 | 3–12 |
| 12 | 5 | 3–12 |
| 18 | 7 | 2–12 |

### Tiering monsters

- **Trash mobs** (hardiness 4–6, agility 4–6): Killed in 1–2 hits; reward learning combat.
- **Standard enemies** (hardiness 8–12, agility 8–10): Require 2–4 hits; require equipped gear.
- **Mini-bosses** (hardiness 16–24, agility 10–14): Require a consumable or two.
- **Final boss** (hardiness 30+, agility 16+): Should reward all prior gear and consumable collection.

### Healing availability

Place one mid-tier consumable (value 4–6) per 3–4 combat rooms. Place a high-value consumable (value 10–12) before the final encounter. Never make the adventure unsolvable if the player played reasonably.

---

## Quest Design

### What makes a good quest

1. **Clear start:** The player knows WHAT they need to do.
2. **Observable progress:** Kill and collect quests track automatically; players see the counter.
3. **Meaningful reward:** Gold and XP should feel proportional to effort.

### Current objective types

| JSON `type` | Tracks when |
|-------------|-------------|
| `kill_monster` | You kill a monster whose ID or name matches `target_id` |
| `collect_item` | You `take` an item whose ID or name matches `target_id` |
| `reach_room` | You enter a room whose ID or name matches `target_id` |
| `talk_to_npc` | You `talk`/`ask` a non-hostile NPC matching `target_id` |

> **Tip:** numeric `target_id` values match IDs directly. String values match names
> case-insensitively and ignore punctuation, so `"Usurper's Champion"` matches
> `"usurper s champion"` and `"Usurper's Champion"`. Prefer IDs for unambiguous targets
> and names for author-friendly content.

### Quest chain design

- Sequence quests so that earlier ones lead the player to areas where later ones become relevant.
- Do not create a quest that requires a monster to be killed that the player may not have triggered.
- Short chains (2–3 quests) work best in adventures under 20 rooms.

---

## NPC Design

In Saga Foundry, NPCs are monsters with `friendliness: "friendly"` or `"neutral"`. They cannot be attacked if friendly.

### Friendly NPC tips

- Give them a high `hardiness` so they appear robust (not that they fight, but `status` won't show them as frail).
- Write their `description` assuming the player will `examine` them.
- Hook dialogue into quests: the `talk_to_npc` objective type triggers on `talk` and `ask <npc> about <topic>` for a matching NPC. Use the objective's optional `topic` field to require a specific conversation topic.

### Neutral NPC tips

- Use neutral for ambiguous characters: merchants, strangers, thieves-in-waiting.
- A neutral monster can BECOME plot-relevant when the player says the right thing (future event hook support).

---

## Pacing and Flow

### The rhythm of a good adventure

```
1. Establish the world (one to two safe rooms, no combat)
2. Introduce low-stakes challenge (easy enemy or light puzzle)
3. Reward exploration (hidden item, story revelation)
4. Escalate tension (stronger enemy, moral choice)
5. Climax (hardest fight or key decision)
6. Resolution (clear ending room, full quest reward)
```

### Gating progression

Use items as soft gates. A door that needs a key, a guard that needs a bribe, a mechanism that needs a component – these force exploration without hard-coding impossible routes.

Avoid hard gates in early rooms. Being stuck in room 2 with no forward progress is the fastest way to lose a player.

### Information density

Space out lore. One piece of world-building per three rooms is a comfortable pace. Clues to puzzles should appear one to two rooms before the puzzle, not right next to it.

---

## Playtesting Checklist

Before sharing your adventure, verify:

- [ ] `start_room` id exists in `rooms`
- [ ] Every room referenced in any `exits` value exists
- [ ] Every item id referenced in a room's `items` array exists in the top-level `items` array, OR the item has its `location` field set to that room's id
- [ ] Every monster's `room_id` corresponds to an existing room
- [ ] JSON is valid (use `python3 -m json.tool my_adventure.json` to check)
- [ ] The adventure is completable: trace the critical path from `start_room` to ending room
- [ ] Combat is fair: verify the player can survive with default stats on the critical path
- [ ] All quest `target_id` values match their monsters/items (test with `quests` and then kill/collect)
- [ ] Consumables exist to recover from the hardest fight
- [ ] All room descriptions are 3–6 sentences and reference their key items/exits

### Quick validation command

```bash
python3 -m json.tool my_adventure.json > /dev/null && echo "JSON valid" || echo "JSON error"
```

---

## Common Pitfalls

### "My adventure just says Unknown command"

Players typed something you didn't expect. Saga Foundry handles a fixed set of verbs — if you want custom interactions, use the quest `talk_to_npc` objective or implement a custom system in Rust.

### "Monsters are too hard / too easy"

Apply the tiering table above. A monster should be beatable with the gear available at the point in the adventure where it appears. If it's the first enemy, make it beatable unarmed.

### "Players can't pick up an item"

Verify `"is_takeable": true` (or omit the field — it defaults to `true`). Also verify the item's `location` matches the room where it should appear.

### "Quest progress never updates"

The `target_id` partial-string match must work. Trace through: if `target_id` is `202` and the monster is named `"Goblin Warrior"`, they will not match. Rename the monster to include `"202"` or change the naming convention.

### "Room exits look circular or players get lost"

Draw the map first. Every exit must make geographic sense. Use `look` output to give strong directional cues in descriptions (*"The road continues north"*, *"To the east a cave entrance yawns open"*).

### "The adventure crashes on load"

Run `python3 -m json.tool my_adventure.json` to find JSON syntax errors. Common causes: trailing commas, unclosed brackets, unescaped quotes in strings.

---

## Reference: JSON Quick-Start Template

Copy this file and customise it to create a five-room adventure with one weapon, one healing item, one enemy, and one quest.

```json
{
  "id": "my_first_adventure",
  "title": "My First Adventure",
  "intro": "Write one sentence of scene-setting here.",
  "start_room": 1,

  "rooms": [
    {
      "id": 1,
      "name": "Opening Room",
      "description": "Describe what the player sees first. Hint at the item.",
      "exits": { "north": 2 },
      "items": [101, 102]
    },
    {
      "id": 2,
      "name": "Middle Room",
      "description": "A room with an enemy nearby.",
      "exits": { "south": 1, "north": 3 }
    },
    {
      "id": 3,
      "name": "Combat Room",
      "description": "An enemy guards the passage east.",
      "exits": { "south": 2, "east": 4 }
    },
    {
      "id": 4,
      "name": "Treasure Room",
      "description": "The prize is here.",
      "exits": { "west": 3, "north": 5 },
      "items": [103]
    },
    {
      "id": 5,
      "name": "Exit Room",
      "description": "You have escaped. The adventure ends here.",
      "exits": { "south": 4 }
    }
  ],

  "items": [
    {
      "id": 101,
      "name": "Short Sword",
      "description": "A plain iron blade, balance slightly off but usable.",
      "type": "weapon",
      "weight": 2,
      "value": 15,
      "is_weapon": true,
      "weapon_type": 5,
      "weapon_dice": 1,
      "weapon_sides": 6,
      "is_takeable": true,
      "location": 1
    },
    {
      "id": 102,
      "name": "Healing Potion",
      "description": "A cloudy liquid with a faint herbal scent.",
      "type": "drinkable",
      "weight": 1,
      "value": 6,
      "is_takeable": true,
      "location": 1
    },
    {
      "id": 103,
      "name": "Ancient Relic",
      "description": "What you came here for. Gold and jewels encrust the surface.",
      "type": "treasure",
      "weight": 2,
      "value": 200,
      "is_takeable": true,
      "location": 4
    }
  ],

  "monsters": [
    {
      "id": 201,
      "name": "Skeletal Guard 201",
      "description": "Bones held together by dark magic, wielding a rusted halberd.",
      "room_id": 3,
      "hardiness": 8,
      "agility": 6,
      "friendliness": "hostile",
      "courage": 100,
      "weapon_id": null,
      "armor_worn": 0,
      "gold": 15
    }
  ],

  "quests": [
    {
      "id": 1,
      "title": "Retrieve the Relic",
      "description": "Recover the Ancient Relic from the ruins.",
      "objectives": [
        {
          "type": "collect_item",
          "target_id": 103,
          "description": "Collect the Ancient Relic"
        },
        {
          "type": "kill_monster",
          "target_id": 201,
          "description": "Defeat the Skeletal Guard"
        }
      ]
    }
  ]
}
```

**Test your adventure:**

```bash
cargo run --bin sagacraft_player -- my_first_adventure.json
```

Good luck, and happy designing!
