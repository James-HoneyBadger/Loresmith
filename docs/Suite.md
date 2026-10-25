# The Saga Foundry Suite

Saga Foundry is the public name of the suite. The original `SagaCraft` name remains on the repository and on 4.1.x binary identifiers for compatibility, but the products are branded **Saga Foundry**.

Saga Foundry is no longer just an editor with a play mode. It is a suite of separate applications that share one engine, one adventure schema, and one visual language.

## Applications

| Product | Binary / crate | Purpose |
|---------|----------------|---------|
| **Saga Foundry Engine** | `sagacraft_rs` | Runtime, canonical adventure schema, systems, saves, validation |
| **Saga Foundry Studio** | `sagacraft_ide_gui` | Full graphical adventure editor |
| **Saga Foundry Player** | `sagacraft_play` | Standalone graphical game player — no editor controls or authoring UI |
| **Saga Foundry Console** | `sagacraft_player` | Fast terminal/REPL player |
| **Saga Foundry Workbench** | `sagacraft_ide_tui` | Terminal adventure editor |
| **Saga Foundry UI** | `sagacraft_ui` | Shared egui theme, palette, and widgets used by Studio and Player |

The important separation is between **authoring** and **playing**:

- Studio and Workbench edit adventures.
- Player and Console only play them.
- The Player links the engine directly and cannot modify adventure files.

## Launching

```bash
./Suite.sh player [adventure.json]   # standalone graphical player (default)
./Suite.sh studio                    # graphical editor
./Suite.sh workbench                 # terminal editor
./Suite.sh console [adventure.json]  # terminal player
```

Individual launchers:

```bash
./Play.sh examples/adventures/starfall_harbor_showcase.json
./Studio.sh
./Workbench.sh
./Console.sh examples/adventures/demo_adventure.json
```

Direct binaries after `cargo build --release`:

```bash
./target/release/sagacraft_play
./target/release/sagacraft_ide_gui
./target/release/sagacraft_ide_tui
./target/release/sagacraft_player
```

## Player features

- Adventure library discovered from bundled and local adventure folders
- Recent adventures and persisted theme/text size
- Full story log with command input and command history
- Live vitals, gold, XP, location, exits, equipment, inventory, and reputation
- Journal/status shortcuts
- Save and load through native file dialogs
- Restart, victory/defeat presentation, and library return
- Dark and light themes
- Keyboard shortcuts: `Ctrl+O`, `Ctrl+S`, `Ctrl+L`, `F5`, `Esc`

The Player deliberately has no room editor, item editor, validation panel, or JSON tools. Those belong to Studio and Workbench.

## Suite name

**Saga Foundry** is the chosen umbrella name for the project.

Why:

- A *saga* is the content; a *foundry* is where many tools shape raw material into finished work.
- It reads naturally as a suite: **Saga Foundry Engine**, **Saga Foundry Studio**, **Saga Foundry Player**, **Saga Foundry Console**, **Saga Foundry Workbench**.
- It removes the ambiguity of "SagaCraft" meaning both the umbrella and one of its applications.

The 4.1.x crate and binary identifiers keep their `sagacraft_*` names for compatibility. A future breaking release can adopt the identifier mapping:

| Current identifier | Future identifier |
|--------------------|-------------------|
| `sagacraft_rs` | `saga_foundry_engine` |
| `sagacraft_ide_gui` | `saga_foundry_studio` |
| `sagacraft_play` | `saga_foundry_player` |
| `sagacraft_player` | `saga_foundry_console` |
| `sagacraft_ide_tui` | `saga_foundry_workbench` |
| `sagacraft_ui` | `saga_foundry_ui` |

User-facing window titles, banners, About dialogs, and documentation already use **Saga Foundry**.

## Architecture

```text
                    ┌────────────────────┐
                    │  sagacraft_rs      │
                    │  engine + schema   │
                    └─────────┬──────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
   ┌──────▼──────┐     ┌──────▼──────┐     ┌──────▼──────┐
   │ Studio GUI  │     │ Player GUI  │     │ Console CLI │
   │ authoring   │     │ play only   │     │ play only   │
   └──────┬──────┘     └──────┬──────┘     └─────────────┘
          │                   │
          └─────────┬─────────┘
             ┌──────▼──────┐
             │ sagacraft_ui│
             │ shared theme│
             └─────────────┘
```

The Workbench (TUI editor) also consumes `sagacraft_rs` directly and keeps its own terminal layout.
