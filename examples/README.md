# Examples

Mellivorex keeps its bundled playable content in the adventures folder. All files load through the canonical document model and are covered by integration tests.

## Included content

- adventures/demo_adventure.json — minimal two-room starter adventure
- adventures/shattered_realms_demo.json — larger fantasy demo with quests and combat
- adventures/clockwork_citadel_showcase.json — steampunk showcase
- adventures/starfall_harbor_showcase.json — maritime mystery showcase
- adventures/iron_assembly_showcase.json — political intrigue showcase

Run any of them with the standalone player:

cargo run --bin mellivorex_player -- examples/adventures/<file>.json

Or in the terminal:

cargo run --bin mellivorex_console -- examples/adventures/<file>.json
