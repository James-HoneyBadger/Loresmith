# Saga Foundry Development Guide

## Contributing to Saga Foundry

Welcome to the Saga Foundry development community! This guide will help you get started with contributing to the project, whether you're fixing bugs, adding features, or creating documentation.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Environment](#development-environment)
3. [Code Style and Conventions](#code-style-and-conventions)
4. [Project Structure](#project-structure)
5. [Testing](#testing)
6. [Submitting Changes](#submitting-changes)
7. [Advanced Topics](#advanced-topics)
8. [Community Guidelines](#community-guidelines)

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust**: Version 1.95 or later (the workspace uses edition 2024)
- **Git**: Version control system
- **Text Editor**: VS Code, Vim, Emacs, or your preferred editor
- **Terminal**: Bash, Zsh, or compatible shell

### Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/SagaCraft.git
cd SagaCraft

# Add upstream remote
git remote add upstream https://github.com/James-HoneyBadger/SagaCraft.git

# Create a development branch
git checkout -b feature/your-feature-name
```

### Initial Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-expand cargo-flamegraph cargo-tarpaulin

# Build the project
cargo build

# Run tests
cargo test

# Verify everything works
cargo run --bin sagacraft_play
cargo run --bin sagacraft_player
```

## Development Environment

### Recommended Tools

#### VS Code Setup
```json
// .vscode/settings.json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll": true,
    "source.organizeImports": true
  },
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### Essential Extensions
- **Rust Analyzer**: Language server for Rust
- **CodeLLDB**: Debugger for Rust
- **Even Better TOML**: TOML file support
- **GitLens**: Enhanced Git capabilities

### Development Workflow

```bash
# Start development server with auto-reload
cargo watch -x 'run --bin sagacraft_player'

# In another terminal, run tests on changes
cargo watch -x test

# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open
```

## Code Style and Conventions

### Rust Style Guidelines

We follow the official Rust style guidelines:

```rust
// ✅ Good: Clear naming, proper formatting
pub fn calculate_damage(attacker: &Player, defender: &Monster) -> i32 {
    let base_damage = attacker.weapon_damage();
    let defense = defender.armor_value();
    let final_damage = (base_damage - defense).max(1);

    final_damage
}

// ❌ Bad: Unclear naming, poor formatting
pub fn dmg_calc(a: &Player, d: &Monster) -> i32 {
    let dmg = a.weapon_damage(); let def = d.armor_value();
    let fd = (dmg - def).max(1); fd
}
```

### Naming Conventions

- **Functions**: `snake_case` (e.g., `calculate_damage`)
- **Types**: `PascalCase` (e.g., `GameState`, `Adventure`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_HEALTH`)
- **Modules**: `snake_case` (e.g., `game_state.rs`)

### Documentation Standards

```rust
/// Calculates combat damage between two entities.
///
/// This function takes into account weapon damage, armor values,
/// and any special modifiers that might affect the final damage.
///
/// # Arguments
///
/// * `attacker` - The entity performing the attack
/// * `defender` - The entity being attacked
///
/// # Returns
///
/// The final damage amount (minimum 1)
///
/// # Examples
///
/// ```
/// let damage = calculate_damage(&player, &monster);
/// assert!(damage >= 1);
/// ```
pub fn calculate_damage(attacker: &Player, defender: &Monster) -> i32 {
    // Implementation...
}
```

### Error Handling

Use appropriate error types and provide context:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn process_command(command: &str) -> Result<(), GameError> {
    match command {
        "invalid" => Err(GameError::InvalidCommand(command.to_string())),
        _ => Ok(()),
    }
}
```

## Project Structure

### Core Library (`sagacraft_rs`)

```
src/
├── lib.rs              # Main library interface
├── document.rs         # Canonical AdventureDocument + legacy adapters
├── adventure.rs        # Legacy string-ID format (read support)
├── engine.rs           # High-level Engine wrapper
├── game_state.rs       # Runtime state, loaders, saves, events
├── util.rs             # Atomic file writes
└── systems/            # Game systems
    ├── mod.rs
    ├── basic_world.rs  # Navigation, look, talk, locks, saves
    ├── combat.rs       # Combat, XP, loot, endings
    ├── inventory.rs    # Item management
    ├── quests.rs       # Quests, stages, rewards
    └── shops.rs        # Shops and trading
```

### Applications

```
sagacraft_play/         # Standalone graphical player
├── src/main.rs
└── Cargo.toml

sagacraft_ide_gui/      # Graphical editor (Studio)
├── src/main.rs
└── Cargo.toml

sagacraft_player/       # CLI player (Console)
├── src/main.rs
└── Cargo.toml

sagacraft_ide_tui/      # Terminal editor (Workbench)
├── src/main.rs
└── Cargo.toml

sagacraft_ui/           # Shared egui theme and widgets
├── src/lib.rs
└── Cargo.toml
```

### Supporting Files

```
docs/                   # Documentation
Cargo.toml              # Workspace configuration
```

## Testing

### Unit Tests

Test document parsing, validation, and runtime helpers:

```rust
#[cfg(test)]
mod tests {
    use sagacraft_rs::AdventureDocument;

    #[test]
    fn legacy_string_schema_is_normalised() {
        let document = AdventureDocument::from_json_str(
            r#"{
                "id": "demo",
                "title": "Demo",
                "start_room": "village",
                "rooms": [
                    {"id": "village", "title": "Village", "description": "Home", "exits": {}},
                    {"id": "forest", "title": "Forest", "description": "Trees", "exits": {}}
                ]
            }"#,
        )
        .expect("document should parse");

        assert_eq!(document.start_room, 1);
        document.validate().expect("document should validate");
    }

    #[test]
    fn broken_exits_are_rejected() {
        let document = AdventureDocument::from_json_str(
            r#"{"title":"Broken","start_room":1,
                "rooms":[{"id":1,"name":"A","description":"A","exits":{"north":99}}]}"#,
        )
        .expect("document should parse");
        assert!(document.report().errors.iter().any(|error| error.contains("unknown room")));
    }
}
```

### Integration Tests

The repository has real integration tests in `sagacraft_rs/tests/`:

- `bundled.rs` loads, validates, and starts every adventure in `examples/adventures/`.
- `features.rs` covers quest matching, talk objectives, locks, light, shops, victory, and XP.

Add a new end-to-end test with the high-level engine:

```rust
use sagacraft_rs::Engine;

#[test]
fn player_can_walk_and_check_status() {
    let mut engine = Engine::load("../examples/adventures/shattered_realms_demo.json")
        .expect("adventure should load");
    assert!(!engine.look().is_empty());
    let status = engine.send("status").join("\n");
    assert!(status.contains("Health:"));
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test document::

# Run with coverage
cargo tarpaulin --out Html

# Run integration tests only
cargo test --test integration

# Run tests in release mode
cargo test --release
```

### Test Organization

- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test component interactions
- **End-to-end tests**: Test complete user workflows
- **Performance tests**: Benchmark critical paths

## Submitting Changes

### Commit Guidelines

Follow conventional commit format:

```bash
# Good commits
git commit -m "feat: add magic system with spell casting"
git commit -m "fix: resolve crash when loading corrupted save files"
git commit -m "docs: update installation guide for Windows"
git commit -m "refactor: simplify command parsing logic"

# Bad commits
git commit -m "fixed stuff"
git commit -m "changes"
git commit -m "update"
```

### Pull Request Process

1. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/add-magic-system
   ```

2. **Make your changes** and commit them:
   ```bash
   git add .
   git commit -m "feat: implement basic magic system"
   ```

3. **Push to your fork**:
   ```bash
   git push origin feature/add-magic-system
   ```

4. **Create a Pull Request** on GitHub with:
   - Clear title describing the change
   - Detailed description of what was changed and why
   - Screenshots/videos for UI changes
   - Links to related issues

5. **Address review feedback**:
   - Make requested changes
   - Add additional commits or amend existing ones
   - Rebase if needed to keep history clean

### Code Review Checklist

**For reviewers:**
- [ ] Code compiles without warnings
- [ ] Tests pass
- [ ] Documentation is updated
- [ ] Code follows style guidelines
- [ ] No security vulnerabilities
- [ ] Performance impact is acceptable

**For contributors:**
- [ ] All tests pass locally
- [ ] Code is formatted with `cargo fmt`
- [ ] Clippy warnings are resolved
- [ ] Documentation is updated
- [ ] Commit messages are clear

## Advanced Topics

### Performance Optimization

```rust
// Use efficient data structures
use std::collections::HashMap;

// Cache expensive operations
lazy_static::lazy_static! {
    static ref COMMAND_CACHE: Mutex<HashMap<String, Command>> = Mutex::new(HashMap::new());
}

// Profile performance-critical code
#[cfg(feature = "profiling")]
fn expensive_operation() {
    // Implementation with profiling
}
```

### Memory Management

```rust
// Use arena allocation for game objects
use typed_arena::Arena;

struct GameArena {
    players: Arena<Player>,
    monsters: Arena<Monster>,
    items: Arena<Item>,
}

// Avoid unnecessary allocations
impl GameState {
    pub fn get_player(&self, id: i32) -> Option<&Player> {
        self.players.get(&id)
    }
}
```

### Concurrency

```rust
use tokio::sync::RwLock;

// Thread-safe game state
pub struct ThreadSafeGameState {
    state: RwLock<GameState>,
}

impl ThreadSafeGameState {
    pub async fn process_command(&self, command: Command) -> Vec<String> {
        let mut state = self.state.write().await;
        // Process command with exclusive access
        process_command_impl(&mut state, command)
    }
}
```

### Extending the Engine

The primary extension point is the `System` trait. To add custom game logic:

```rust
use sagacraft_rs::{System, AdventureGame, GameEvent};

pub struct MySystem;

impl System for MySystem {
    fn on_command(&mut self, command: &str, args: &[&str], game: &mut AdventureGame) -> Option<String> {
        match command {
            "my_command" => Some("Custom response!".to_string()),
            _ => None,
        }
    }

    fn on_events(&mut self, events: &[GameEvent], _game: &mut AdventureGame) -> Option<String> {
        // React to game events (optional)
        None
    }
}
```

Register custom systems on the engine before loading:

```rust
let mut engine = Engine::new("adventure.json");
engine.game.add_system(Box::new(MySystem));
engine.start()?;
```

See the [API Reference](API_Reference.md) for the full `System` trait and `GameEvent` documentation.

### Serialization Strategies

```rust
use serde::{Serialize, Deserialize};

// Versioned serialization
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum SaveFormat {
    #[serde(rename = "1.0")]
    V1_0(GameStateV1),
    #[serde(rename = "2.0")]
    V2_0(GameStateV2),
}

// Migration support
impl SaveFormat {
    fn migrate_to_latest(self) -> GameState {
        match self {
            SaveFormat::V1_0(old) => old.migrate(),
            SaveFormat::V2_0(state) => state,
        }
    }
}
```

## Community Guidelines

### Code of Conduct

We follow a code of conduct to ensure a welcoming environment:

- **Be respectful**: Treat all contributors with respect
- **Be collaborative**: Work together to improve the project
- **Be patient**: Everyone is learning and growing
- **Be inclusive**: Welcome contributors from all backgrounds

### Communication

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For general questions and ideas
- **Pull Request Comments**: For code review discussions
- **Discord/Slack**: For real-time communication (when available)

### Getting Help

- **Documentation**: Check existing docs first
- **Search Issues**: Look for similar problems
- **Ask Questions**: Don't hesitate to ask for clarification
- **Share Knowledge**: Help others when you can

### Recognition

Contributors are recognized through:
- **GitHub Contributors**: Listed in repository contributors
- **Changelog**: Mentioned in release notes
- **Credits**: Special thanks in documentation
- **Badges**: Recognition for significant contributions

## Development Roadmap

### Current Priorities

1. **Performance**: Optimize memory usage and loading times
2. **Features**: Add requested community features
3. **Platforms**: Expand platform support
4. **Documentation**: Improve and expand documentation
5. **Testing**: Increase test coverage

### Contributing to the Roadmap

- **Propose Features**: Use GitHub Discussions
- **Vote on Issues**: Use reactions to show support
- **Submit PRs**: Implement features you care about
- **Report Bugs**: Help improve stability

### Long-term Vision

- **Multiplayer**: Real-time collaborative adventures
- **3D Support**: Integration with 3D rendering
- **Mobile Apps**: iOS and Android versions
- **Web Version**: Browser-based adventures
- **Mod Ecosystem**: Community plugin marketplace

---

**Thank you for contributing to Saga Foundry!** Your efforts help make text-based adventures more accessible and enjoyable for everyone. Whether you're fixing a small bug or implementing a major feature, every contribution matters.

For questions or help getting started, don't hesitate to reach out through GitHub Issues or Discussions.