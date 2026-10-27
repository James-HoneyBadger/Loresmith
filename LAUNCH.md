# Loresmith — Quick Launch

Loresmith is a suite of separate applications. Use the suite launcher or run an app directly.

## Suite launcher

```bash
./Suite.sh                 # standalone graphical player (default)
./Suite.sh player          # standalone graphical player
./Suite.sh studio          # graphical adventure editor
./Suite.sh workbench       # terminal adventure editor
./Suite.sh console         # terminal player
```

Pass an adventure to either player:

```bash
./Suite.sh player examples/adventures/starfall_harbor_showcase.json
./Suite.sh console examples/adventures/demo_adventure.json
```

## Player — standalone game application

```bash
./Play.sh
./Play.sh my_adventure.json
```

The Player contains no editor or authoring tools. It discovers bundled adventures, remembers recent files and theme choice, and provides story, character, inventory, journal, save/load, restart, and victory/defeat views.

## Studio — graphical adventure editor

```bash
./Studio.sh
```

Full room, item, monster, quest, shop, ending, starting-inventory, lock, light, dialogue, JSON diff, validation, undo/redo, and Play test tooling.

## Workbench — terminal adventure editor

```bash
./Workbench.sh
# or
./Workbench.sh
```

## Console — terminal player

```bash
./Console.sh
./Console.sh my_adventure.json
```

## Direct binaries

```bash
cargo run --bin loresmith_player -- examples/adventures/starfall_harbor_showcase.json
cargo run --bin loresmith_studio
cargo run --bin loresmith_console -- examples/adventures/shattered_realms_demo.json
cargo run --bin loresmith_workbench
```

## Documentation

- [Suite Overview](docs/Suite.md)
- [Adventure Schema](docs/Adventure_Schema.md)
- [User Manual](docs/User_Manual.md)
- [Game Designer Manual](docs/Game_Designer_Manual.md)
- [Technical Reference](docs/Technical_Reference.md)
