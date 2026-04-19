# Quick Launch

SagaCraft includes two convenience scripts in the project root:

## IDE — Adventure Creator

```bash
./Saga.sh
```

Opens the SagaCraft TUI Adventure IDE for creating, editing, and testing adventures.

## Player — Adventure Player

```bash
./Play.sh
```

Launches the CLI player with the bundled demo in examples/adventures.

Pass a custom adventure file:

```bash
./Play.sh my_adventure.json
```

## Direct Commands (no scripts)

```bash
cargo run --bin sagacraft_player -- examples/adventures/shattered_realms_demo.json
cargo run --bin sagacraft_ide_tui
cargo run --bin sagacraft_ide_gui
```

## Documentation

- [User Manual](docs/User_Manual.md)
- [Game Designer Manual](docs/Game_Designer_Manual.md)
- [Technical Reference](docs/Technical_Reference.md)
