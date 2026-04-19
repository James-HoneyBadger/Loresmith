# SagaCraft Changelog

All notable changes to SagaCraft will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
