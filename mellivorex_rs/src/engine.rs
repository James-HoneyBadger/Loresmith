use crate::document::AdventureDocument;
use crate::game_state::AdventureGame;
use crate::systems::quests::QuestSystem;
use crate::systems::{BasicWorldSystem, CombatSystem, InventorySystem, ShopSystem};
use std::path::Path;

/// High-level convenience wrapper that creates an `AdventureGame` with all four
/// built-in systems pre-registered.
///
/// # Example
/// ```no_run
/// use mellivorex_rs::Engine;
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
        game.add_system(Box::new(ShopSystem));
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

    /// Create an engine from an in-memory document (used by editors and tests).
    pub fn from_document(document: AdventureDocument) -> Result<Self, Box<dyn std::error::Error>> {
        let mut engine = Self::new("");
        let intro = engine.game.load_document(document)?;
        engine.intro_text = intro;
        Ok(engine)
    }

    /// Create an engine from a JSON string.
    pub fn load_from_str(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let document = AdventureDocument::from_json_str(json)?;
        Self::from_document(document)
    }

    /// Restart the current adventure from its opening state.
    pub fn restart(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let intro = self.game.restart()?;
        self.intro_text = intro.clone();
        Ok(intro)
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

    /// Whether the game has ended, either by death or victory.
    pub fn is_over(&self) -> bool {
        self.game.game_over || self.game.game_won
    }

    /// Whether the player has won.
    pub fn won(&self) -> bool {
        self.game.game_won
    }

    /// The victory or defeat text, if an ending has been reached.
    pub fn ending(&self) -> Option<&str> {
        if self.is_over() && !self.game.ending_text.is_empty() {
            Some(&self.game.ending_text)
        } else {
            None
        }
    }

    /// Borrow the canonical document for the loaded adventure.
    pub fn document(&self) -> &AdventureDocument {
        &self.game.document
    }
}
