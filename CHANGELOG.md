# Loresmith Changelog

All notable changes to Loresmith will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [6.0.0] - 2026-10-27

### Added
- `loresmith-update.sh` installs the latest release binaries for Linux x86_64 and macOS Apple silicon, with `--prefix`, `--tag`, `--archive`, and `--dry-run` options; it is uploaded as a release asset and included in the Unix archives

### Changed
- Renamed the project from Mellivorex to **Loresmith** to avoid confusion with the unrelated Mellivora OS; crates, binaries, launch scripts, release archives, window titles, banners, and docs use the new name
- Crate and binary identifiers: `mellivorex_rs` → `loresmith_rs`, `mellivorex_ui` → `loresmith_ui`, `mellivorex_studio` → `loresmith_studio`, `mellivorex_workbench` → `loresmith_workbench`, `mellivorex_player` → `loresmith_player`, `mellivorex_console` → `loresmith_console`
- Player state lives in `LORESMITH_STATE_DIR`, `~/.local/state/loresmith`, `%APPDATA%\Loresmith`, or `~/Library/Application Support/Loresmith`
- Release archives are named `Loresmith-v<version>-<target>`

Adventure files and portable save files are unchanged; only the application identifiers moved.

### Security
- Bumped `wayland-scanner` 0.31.8 → 0.31.11 so `quick-xml` resolves to 0.41.0, clearing `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195`; the matching `deny.toml` ignores are gone

## [5.0.0] - 2026-10-26

### Changed
- Renamed the project from SagaCraft / Saga Foundry to **Mellivorex**; crates, binaries, launch scripts, release archives, window titles, banners, and docs use the new name
- Crate and binary identifiers: `sagacraft_rs` → `mellivorex_rs`, `sagacraft_ui` → `mellivorex_ui`, `sagacraft_ide_gui` → `mellivorex_studio`, `sagacraft_ide_tui` → `mellivorex_workbench`, `sagacraft_play` → `mellivorex_player`, `sagacraft_player` → `mellivorex_console`
- Player state lives in `MELLIVOREX_STATE_DIR`, `~/.local/state/mellivorex`, `%APPDATA%\Mellivorex`, or `~/Library/Application Support/Mellivorex`
- Release archives are named `Mellivorex-v<version>-<target>`
- `Saga.sh` was replaced by `Workbench.sh`

### Removed
- Compatibility aliases for the old `saga-foundry` and `sagacraft` identifiers, state directories, and binary names

Adventure files and portable save files are unchanged; only the application identifiers moved.

## [4.2.0] - 2026-10-26

### Added
- Saga Foundry Player now autosaves after every turn and offers **Resume** / **Discard** for the last recovery save on the library screen
- Player detects merchants in the current room, shows their names in the sidebar, and adds a context-aware `shop` quick action
- Windows 2000 theme for Studio and Player: gradient caption bar with minimize/maximize/close, draggable undecorated window chrome, resize grips, classic menu highlights, recessed toolbar, 3-D beveled panels, sunken status bars, and Tahoma when available
- Player **Edit** menu with **Copy Story Log** and **Copy Last Command**; Studio gained an **Edit** menu with undo/redo and clipboard JSON copy
- Classic window chrome hides OS decorations only while the Windows 2000 theme is active and restores them for the other themes

### Fixed
- Studio and Player now enable `eframe` persistence, so the selected theme (and Player's recent files/zoom) survives a restart

## [4.1.0] - 2026-10-24

### Added
- Canonical, versioned `AdventureDocument` model (`schema_version: 2`) with lossless round-trips and legacy string-ID/integer-ID adapters
- Staged quests, verified turn-in, quest IDs in the journal, reward items, reputation, and `abandon`
- `TalkedTo` events and working `talk_to_npc` objectives with optional topics
- Per-NPC dialogue greetings, topic replies, and unknown-topic fallback
- Locked exits with keys, optional key consumption, `unlock`, and `ExitUnlocked` events
- Shops with finite gold/stock, `buy`/`sell`, and charisma-based pricing
- Light sources and dark-room visibility
- Victory/defeat endings, `game_won`, ending text, and `restart`
- Monster loot and weapon drops
- Numeric-schema `player_start_inventory` and corrected `location: 0` semantics
- Portable save v2 (embedded document, atomic writes) with v1 migration
- `Engine::from_document`, `load_from_str`, `restart`, `won`, and `ending`
- GUI editors for shops, dialogue, locks, light sources, staged objectives, starting inventory, and endings
- GUI validation panel, JSON diff, clipboard copy, undo/redo, keyboard shortcuts, and play-mode sidebar
- GUI visual redesign: branded shell, workspace navigation rail, themed cards and stat tiles, redesigned Play and Preview tabs, and full-height scrolling editors
- GUI dark and light themes selectable from View → Theme and persisted between sessions
- Standalone graphical **SagaCraft Player** (`sagacraft_play`) with adventure library, recent files, story log, character sidebar, journal shortcuts, native save/load, and dark/light themes
- Shared `sagacraft_ui` crate for the egui palette and common widgets used by Player and Studio
- Suite launchers (`Suite.sh`, `Play.sh`, `Studio.sh`, `Workbench.sh`, `Console.sh`) and [docs/Suite.md](docs/Suite.md)
- The suite is branded **Saga Foundry**; application titles, banners, About dialogs, package descriptions, and docs use the new name while `sagacraft_*` binary identifiers remain compatible in 4.1.x
- Release archives are named `SagaFoundry-v<version>-<target>` going forward
- TUI canonical-document editing for items, monsters, quests, shops, exits, and validation
- Integration tests for every bundled adventure and all new features
- `docs/Adventure_Schema.md`

### Changed
- GUI and TUI now edit the canonical document directly; saving no longer drops fields
- GUI Save & Exit only exits after a successful save; New/Open and window close protect unsaved work
- TUI fails loudly on load errors instead of silently replacing the file with a demo
- Combat uses `weapon_ability` and `armor_expertise`; flee picks a deterministic safe exit
- Quest targets match by ID or punctuation-insensitive name
- Level-ups loop for large XP awards
- Exit keys are lowercased on load
- Workspace metadata and MSRV (1.95) centralized; dependencies upgraded through 2026
- CI adds docs, all-target tests, MSRV, deny checks, and a multi-platform release workflow

### Fixed
- Shop pricing defaults corrected to buy `1.0` and sell `0.5`; charisma and multiplier values are clamped, and same-shop buy/sell arbitrage is blocked
- Buying an item now progresses collect objectives and reports sold-out stock correctly
- Shop listings now show shop gold, stock buy prices, and sell offers for carried items
- Multiple shops in one room are now all reachable: `shop` lists them, `shop <name>` filters, `buy` chooses the cheapest offer, and `sell` chooses the best affordable offer
- Iron Assembly quest 3 could never auto-complete because of apostrophe normalization
- `complete` paid rewards with unfinished objectives
- Quest XP never triggered level-ups
- `location: 0` items were invisible and starting inventory was unsupported
- `go N` and `unequip Weapon` were case-sensitive
- GUI could not open the bundled showcase adventures
- GUI "Copy JSON" faked success; Preview TODO buttons removed
- TUI `:q` skipped the unsaved-changes check; deleting a room left dangling references
- Autosave is atomic and no longer corruptible on crash

### Removed
- Phantom editor-only gameplay claims (unsupported traps, ambient audio, safe zones) from documentation; the fields are preserved as metadata
- Fictional community resources and unpublished install instructions

## [4.0.3] - 2026-04-18

### Added
- Save, load, and autosave support for longer play sessions and regression-tested persistence
- Direct NPC talk and ask interactions for richer non-combat gameplay
- Three large showcase adventures plus a dedicated examples folder for bundled content
- GitHub CI, CodeQL, Dependabot, cargo audit, and a repository security policy
- **Help command** (`help` / `?`) handled as a system command — works in CLI player, GUI Play tab, and any future frontend
- **Intro text** displayed on game start — `Engine::intro()` returns the adventure banner/intro string
- **GUI: Exit confirmation** — File → Exit now warns if there are unsaved changes
- **GUI: Quest objective editing** — objectives are now editable text fields with add/remove buttons
- **GUI: Add Exit direction picker** — new exit dialog uses a direction dropdown + room ID instead of always inserting "north → 1"
- **Item name matching helper** (`name_matches`) — deduplicated case-insensitive substring matching across game_state, combat, and quests

### Changed
- Bundled adventures now live under the cleaner examples/adventures layout
- GUI editor gained live search and filtering for rooms, items, monsters, and quests
- **Direction abbreviations** (`n`, `s`, `e`, `w`, `u`, `d`) now correctly expand to full words before room exit lookup, fixing silent navigation failures
- **Monster counter-attack** damage floor changed from 0 to 1, matching player attack floor (symmetric)
- **GUI: Modding tab removed** — it was entirely fake/hardcoded data
- **GUI: MonsterData.charisma removed** — field had no engine equivalent

### Removed
- **`command.rs` module** — `Command` enum, `Direction` enum, `ParseError`, and `parse()` were dead code (never called at runtime)
- **`GameState` struct** — dead alternative to `AdventureGame`; `into_game_state()` and tests removed from `adventure.rs`
- **Dead quest types** — `QuestStatus::Abandoned/Blocked`, `QuestDifficulty::Trivial/Legendary`
- **Dead quest fields** — `Quest.prerequisites`, `blocking_quests`, `time_limit_hours`, `is_radiant`, `chain_id`, `QuestObjective.is_optional`, `completion_reward`
- **Dead quest methods** — `advance_stage()`, `get_level_adjusted_rewards()`, `mark_failed()`, `fail_quest()`, `get_optional_completed()`
- **Dead config files** — `config/` directory (engine.json, game_settings.json, modding_state.json, recent_adventures.json — never loaded)
- **Dead GUI methods** — `refresh_mods()`, `open_mods_folder()`, `discover_mods()`

### Fixed
- Adventure loading now fails fast on missing start rooms, broken exits, and invalid references
- Quest parsing now accepts both structured objectives and GUI-authored string objectives
- **README.md** rewritten — removed 40+ non-existent feature claims, fixed Rust version badge (1.85+), fixed version (4.0.2), removed phantom directories
- **LAUNCH.md** rewritten — removed Python references
- **CHANGELOG.md** rewritten — removed phantom 1.0.0 entries with impossible features

## [4.0.2] - 2026-02-20

### Fixed
- Upgraded `eframe` 0.29 → 0.33 and `egui` 0.29 → 0.33 with `glow` renderer to resolve RUSTSEC-2024-0436
- Updated deprecated egui 0.33 menu APIs
- Updated removed egui 0.32 API (`selectable_label` → `Button::new().selected()`)
- Added `wayland` and `x11` feature flags for eframe 0.30+ Linux support
- Fixed `rfd 0.14` type mismatch in `add_filter` calls

## [4.0.1] - 2026-02-20

### Added
- `Tutorial.md` and `Game_Design_Tips.md` documentation

### Changed
- Rewrote User Manual and Technical Reference to match actual engine

### Fixed
- GUI IDE compilation errors (unicode escapes, ComboBox API, type annotations)

## [4.0.0] - 2025-12-29

### Added
- "The Shattered Realms" demo adventure (15 rooms, 17 items, 4 monster types, quests)
- Engine rewrite with System trait, event bus, and four built-in systems
- XP, level-up, and flee mechanics in combat
- Quest system with objective tracking and auto-advancement on events
- GUI IDE with Play, Info, Rooms, Items, Monsters, Quests, and Preview tabs
- TUI IDE for string-ID adventure format

### Changed
- Adventure format uses integer room/item/monster IDs
- Major version bump to 4.0.0
