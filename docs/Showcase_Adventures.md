# Mellivorex Showcase Adventures

This workspace includes three large example adventures designed to demonstrate the Mellivorex engine and editing tools at a higher level than the basic demos.

## Included showcase files

### Clockwork Citadel: Siege of the Sable Core
Path: examples/adventures/clockwork_citadel_showcase.json

Highlights:
- dense steampunk-fantasy setting
- interconnected fortress map
- friendly, neutral, and hostile NPCs
- readable clues and healing items
- armor and weapon progression
- multi-step quests using collect, explore, and combat objectives

### Starfall Harbor: Beacon of the Deep
Path: examples/adventures/starfall_harbor_showcase.json

Highlights:
- coastal mystery and abyssal fantasy tone
- harbor, tower, and undersea ruin exploration
- story-rich NPC dialogue topics
- loot, consumables, and combat encounters
- layered quests that guide a long-form play session
- a shop in Saltwind Market
- a dark Smuggler Cave that requires the Lantern Charm
- a customs-sealed vault opened with the Customs Seal
- a three-quest victory ending

### The Iron Assembly: Crown of Ash and Oaths
Path: examples/adventures/iron_assembly_showcase.json

Highlights:
- political intrigue and faction conflict
- friendly, neutral, and hostile NPCs
- investigation-heavy quests
- apostrophe-bearing quest targets (regression coverage for match normalization)

## Quick start

Build and run either adventure with the CLI player:

cargo run --bin mellivorex_console -- examples/adventures/clockwork_citadel_showcase.json
cargo run --bin mellivorex_console -- examples/adventures/starfall_harbor_showcase.json
cargo run --bin mellivorex_console -- examples/adventures/iron_assembly_showcase.json

You can also open the same files in the GUI or TUI editors to inspect the structure and modify content.

## Recommended demonstration flow

1. Start in the CLI player and use `help`, `look`, `quests`, `talk`, and `ask`.
2. Accept a quest early and explore the main route.
3. Equip better gear, use consumables, and test combat and loot drops.
4. Try the shop, a locked exit, and a dark room in Starfall Harbor.
5. Enable autosave to show persistence during longer runs.
6. Open the file in the GUI editor to demonstrate validation, filtering, JSON diff, and play mode.
