# The Mellivorex Suite

Mellivorex is the name of the suite and of every application in it. The tools are not one editor with a play mode; they are separate applications that share one engine, one adventure schema, and one visual language.

## Applications

| Product | Binary / crate | Purpose |
|---------|----------------|---------|
| **Mellivorex Engine** | `mellivorex_rs` | Runtime, canonical adventure schema, systems, saves, validation |
| **Mellivorex Studio** | `mellivorex_studio` | Full graphical adventure editor |
| **Mellivorex Player** | `mellivorex_player` | Standalone graphical game player — no editor controls or authoring UI |
| **Mellivorex Console** | `mellivorex_console` | Fast terminal/REPL player |
| **Mellivorex Workbench** | `mellivorex_workbench` | Terminal adventure editor |
| **Mellivorex UI** | `mellivorex_ui` | Shared egui theme, palette, and widgets used by Studio and Player |

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
./target/release/mellivorex_player
./target/release/mellivorex_studio
./target/release/mellivorex_workbench
./target/release/mellivorex_console
```

## Player features

- Adventure library discovered from bundled and local adventure folders
- Recent adventures and persisted theme/text size
- Full story log with command input and command history
- Live vitals, gold, XP, location, exits, equipment, inventory, and reputation
- Journal/status shortcuts
- Save and load through native file dialogs
- Automatic recovery saves after every turn, with Resume/Discard on the library screen
- Merchant detection, a sidebar shop card, and a context-aware shop quick action
- Restart, victory/defeat presentation, and library return
- Dark, light, and Windows 2000 themes
- Windows 2000 mode adds classic desktop chrome: gradient caption bar with window controls, menu bar, recessed toolbar, beveled panels, and a status bar
- Edit → Copy Story Log and Copy Last Command use the system clipboard
- Keyboard shortcuts: `Ctrl+O`, `Ctrl+S`, `Ctrl+L`, `F5`, `Esc`

The Player deliberately has no room editor, item editor, validation panel, or JSON tools. Those belong to Studio and Workbench.

## Suite name

**Mellivorex** is the umbrella name for the project, the engine, and each application. Crates, binaries, release archives, window titles, banners, About dialogs, documentation, and runtime state directories all use the `mellivorex` identifier:

| Layer | Identifier |
|-------|------------|
| Engine crate | `mellivorex_rs` |
| Shared UI crate | `mellivorex_ui` |
| Studio binary | `mellivorex_studio` |
| Player binary | `mellivorex_player` |
| Console binary | `mellivorex_console` |
| Workbench binary | `mellivorex_workbench` |
| Release archive | `Mellivorex-v<version>-<target>` |

## Architecture

```text
                           ┌─────────────────────┐
                           │    mellivorex_rs    │
                           │   engine + schema   │
                           └─────────────────────┘
                                      │
                ┌─────────────────────┼─────────────────────┐
                │                     │                     │
       ┌────────────────┐    ┌────────────────┐    ┌────────────────┐
       │   Studio GUI   │    │   Player GUI   │    │  Console CLI   │
       │   authoring    │    │   play only    │    │   play only    │
       └────────────────┘    └────────────────┘    └────────────────┘
                │                     │
                └─────────────────────┘
                                      │
                           ┌─────────────────────┐
                           │    mellivorex_ui    │
                           │     shared theme    │
                           └─────────────────────┘
```

The Workbench (TUI editor) also consumes `mellivorex_rs` directly and keeps its own terminal layout.
