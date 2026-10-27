//! Headless tests for the terminal editor.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use loresmith_workbench::{App, draw_main, handle_key, starter_document};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::path::PathBuf;

fn app() -> App {
    App::new(PathBuf::from("unused.json"), starter_document())
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Run an editor command through command mode, the way a user would.
fn run_command(app: &mut App, command: &str) {
    handle_key(app, key(KeyCode::Char(':')));
    for character in command.chars() {
        handle_key(app, key(KeyCode::Char(character)));
    }
    handle_key(app, key(KeyCode::Enter));
}

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

#[test]
fn renders_room_list_and_details() {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).expect("terminal");
    let app = app();

    terminal
        .draw(|frame| {
            let area = frame.area();
            draw_main(frame, area, &app)
        })
        .expect("draw");

    let text = buffer_text(&terminal);
    assert!(text.contains("Rooms"), "room list is missing:\n{text}");
    assert!(text.contains("Details"), "details pane is missing:\n{text}");
    assert!(text.contains("Exits:"), "details have no exits:\n{text}");
}

#[test]
fn arrow_keys_move_the_room_selection() {
    let mut app = app();
    run_command(&mut app, "room add 2");

    handle_key(&mut app, key(KeyCode::Down));
    assert_eq!(app.selected_room().map(|room| room.id), Some(2));

    handle_key(&mut app, key(KeyCode::Up));
    assert_eq!(app.selected_room().map(|room| room.id), Some(1));

    handle_key(&mut app, key(KeyCode::Up));
    assert_eq!(
        app.selected_room().map(|room| room.id),
        Some(1),
        "selection must not move above the first room"
    );
}

#[test]
fn typing_a_command_adds_a_room() {
    let mut app = app();
    let before = app.room_count();

    run_command(&mut app, "room add 99");

    assert_eq!(app.room_count(), before + 1);
    assert!(app.status().contains("Added room 99"), "{}", app.status());
    assert!(app.dirty(), "adding a room should mark the document dirty");
}

#[test]
fn quitting_with_unsaved_changes_asks_first() {
    let mut app = app();
    run_command(&mut app, "room add 7");
    assert!(app.dirty());

    assert!(
        !handle_key(&mut app, key(KeyCode::Char('q'))),
        "the first q must not quit while the document is dirty"
    );
    assert!(app.status().contains("Unsaved changes"), "{}", app.status());
    assert!(
        handle_key(&mut app, key(KeyCode::Char('q'))),
        "the second q should quit"
    );
}
