//! Graphical adventure editor.
//!
//! The binary is a thin wrapper around [`main_entry`]; the editor state and
//! rendering entry point are public so integration tests can drive them
//! headlessly.

mod app;
mod tabs;

pub use app::*;
