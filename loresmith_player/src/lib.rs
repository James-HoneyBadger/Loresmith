//! Graphical adventure player.
//!
//! The binary is a thin wrapper around [`main_entry`]; the player state and
//! rendering entry point are public so integration tests can drive them
//! headlessly.

mod app;

pub use app::*;
