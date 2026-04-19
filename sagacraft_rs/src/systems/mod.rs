pub mod basic_world;
pub mod combat;
pub mod inventory;
pub mod quests;

pub use basic_world::BasicWorldSystem;
pub use combat::CombatSystem;
pub use inventory::InventorySystem;
pub use quests::QuestSystem;

use crate::game_state::{AdventureGame, GameEvent};

pub trait System {
    /// Handle a typed player command. Return `Some(output)` to claim the command;
    /// returning `None` passes the command on to the next system.
    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String>;

    /// Called after every command round when there are pending game events
    /// (monster kills, item pickups, room transitions, etc.).
    /// Return `Some(output)` to append an observer message (e.g. quest updates).
    /// The default implementation is a no-op.
    fn on_events(&mut self, _events: &[GameEvent], _game: &mut AdventureGame) -> Option<String> {
        None
    }

    /// Optional key used for save/load state snapshots.
    fn state_key(&self) -> Option<&'static str> {
        None
    }

    /// Serialize any system-specific state that should survive a save/load cycle.
    fn save_state(&self) -> Option<serde_json::Value> {
        None
    }

    /// Restore previously serialized state after the main game snapshot is loaded.
    fn load_state(&mut self, _state: Option<&serde_json::Value>, _game: &mut AdventureGame) {}
}
