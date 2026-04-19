# Showcase Adventures

This workspace now includes two large example adventures designed to demonstrate the SagaCraft engine and editing tools at a higher level than the basic demos.

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
- story-rich NPC interactions
- loot, consumables, and combat encounters
- layered quests that guide a long-form play session

## Quick start

Build and run either adventure with the CLI player:

cargo run --bin sagacraft_player -- examples/adventures/clockwork_citadel_showcase.json
cargo run --bin sagacraft_player -- examples/adventures/starfall_harbor_showcase.json

You can also open the same files in the GUI or TUI editors to inspect the structure and modify content.

## Recommended demonstration flow

1. Start in the CLI player and use help, look, quests, and talk commands.
2. Accept a quest early and explore the main route.
3. Equip better gear, use consumables, and test combat.
4. Enable autosave to show persistence during longer runs.
5. Open the file in the GUI editor to demonstrate live filtering and adventure editing.
