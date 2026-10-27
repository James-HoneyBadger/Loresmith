//! Guards the command reference against drift.
//!
//! `commands::COMMANDS` claims to be the single source of truth for the command
//! list, so every alias it documents must actually be routed by the engine, and
//! the README tables must match the Markdown rendering of the same table.

use loresmith_rs::Engine;
use loresmith_rs::commands::{COMMANDS, markdown_table};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn demo_engine() -> Engine {
    let path = repo_root().join("examples/adventures/demo_adventure.json");
    Engine::load(path.to_string_lossy().to_string()).expect("demo adventure should load")
}

fn scratch_dir(tag: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("loresmith_commands_{tag}_{unique}"));
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// Build a minimal invocation for an entry that does not touch the repository.
fn invocation(alias: &str, args: &str, scratch: &Path) -> String {
    match (args, alias) {
        ("on [file]", _) => format!("{alias} on {}", scratch.join("auto.json").display()),
        ("off", _) => format!("{alias} off"),
        ("status", _) => format!("{alias} status"),
        (_, "save" | "load") => format!("{alias} {}", scratch.join("save.json").display()),
        _ => alias.to_string(),
    }
}

#[test]
fn every_documented_alias_is_routed() {
    let scratch = scratch_dir("aliases");
    let mut engine = demo_engine();
    let mut checked = 0;

    for command in COMMANDS {
        for alias in command.aliases {
            let input = invocation(alias, command.args, &scratch);
            let response = engine.send(&input).join("\n");
            assert!(
                !response.contains("Unknown command"),
                "`{input}` is documented as {} but the engine does not handle it:\n{response}",
                command.usage()
            );
            checked += 1;
        }
    }

    assert!(
        checked >= 40,
        "expected the table to document at least 40 aliases, saw {checked}"
    );
    std::fs::remove_dir_all(&scratch).ok();
}

fn embedded_table(path: &Path) -> String {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
    let start = text
        .find("| Command | Description |")
        .unwrap_or_else(|| panic!("{} has no command table", path.display()));
    text[start..]
        .lines()
        .take_while(|line| line.starts_with('|'))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn readme_command_tables_match_the_commands_module() {
    let expected = markdown_table();
    for name in ["README.md", "release/README.md"] {
        let path = repo_root().join(name);
        assert_eq!(
            embedded_table(&path),
            expected,
            "{name} has drifted from commands::COMMANDS; regenerate it with \
             `cargo test -p loresmith_rs print_markdown_table -- --ignored --nocapture`"
        );
    }
}
