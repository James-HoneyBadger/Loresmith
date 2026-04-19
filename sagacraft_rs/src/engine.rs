use crate::game_state::AdventureGame;
use crate::systems::quests::QuestSystem;
use crate::systems::{BasicWorldSystem, CombatSystem, InventorySystem};
use std::path::Path;

/// High-level convenience wrapper that creates an `AdventureGame` with all four
/// built-in systems pre-registered.
///
/// # Example
/// ```no_run
/// use sagacraft_rs::Engine;
///
/// let mut engine = Engine::load("my_adventure.json").expect("failed to load");
/// println!("{}", engine.look());
/// for line in engine.send("north") {
///     println!("{}", line);
/// }
/// ```
pub struct Engine {
    pub game: AdventureGame,
    intro_text: String,
}

impl Engine {
    /// Create an `Engine` for the given adventure file path with all systems registered.
    /// Call [`Engine::start`] to load the adventure data from disk.
    pub fn new(adventure_path: impl Into<String>) -> Self {
        let mut game = AdventureGame::new(adventure_path.into());
        game.add_system(Box::new(BasicWorldSystem));
        game.add_system(Box::new(InventorySystem));
        game.add_system(Box::new(CombatSystem));
        game.add_system(Box::new(QuestSystem::new()));
        Self {
            game,
            intro_text: String::new(),
        }
    }

    /// Load the adventure file and return the opening banner/intro text.
    pub fn start(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let intro = self.game.load_adventure()?;
        self.intro_text = intro.clone();
        Ok(intro)
    }

    /// Create an `Engine`, load the adventure from disk, and return it.
    /// Combines [`Engine::new`] and [`Engine::start`].
    pub fn load(adventure_path: impl Into<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut engine = Self::new(adventure_path);
        engine.start()?;
        Ok(engine)
    }

    /// Return the intro/banner text captured at load time.
    pub fn intro(&self) -> &str {
        &self.intro_text
    }

    /// Process one line of player input and return the response lines.
    pub fn send(&mut self, input: &str) -> Vec<String> {
        self.game.process_command(input)
    }

    /// Return a description of the current room.
    pub fn look(&self) -> String {
        self.game.look()
    }

    /// Save the current runtime progress to a snapshot file.
    pub fn save_game(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        self.game.save_game(path)
    }

    /// Load a previously saved runtime snapshot.
    pub fn load_game(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let intro = self.game.load_game(path)?;
        self.intro_text = intro.clone();
        Ok(intro)
    }

    /// Whether the game has ended.
    pub fn is_over(&self) -> bool {
        self.game.game_over
    }
}
