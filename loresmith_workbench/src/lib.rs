//! Terminal adventure editor.
//!
//! The binary is a thin wrapper around [`main_entry`]; the editor state and
//! drawing helpers are public so integration tests can drive them headlessly.

mod app;

pub use app::*;
