//! Headless UI tests for the graphical player.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use loresmith_player::LoresmithPlayer;

fn harness() -> Harness<'static, LoresmithPlayer> {
    let app = LoresmithPlayer::with_storage(None, None);
    let mut harness = Harness::new_ui_state(|ui, app: &mut LoresmithPlayer| app.update_ui(ui), app);
    harness.run();
    harness
}

#[test]
fn renders_the_library_screen() {
    let harness = harness();
    assert!(
        harness.get_all_by_label("Open Adventure").next().is_some(),
        "the library should offer an Open Adventure button"
    );
    assert!(
        harness
            .query_by_label_contains("Your next story starts here")
            .is_some(),
        "the empty library should invite the player in"
    );
}

#[test]
fn about_dialog_opens() {
    let mut harness = harness();
    harness.get_by_label("Help").click();
    harness.run();
    harness.get_by_label("About Loresmith Player").click();
    harness.run();
    harness.get_by_label_contains("A standalone game player for Loresmith adventures");
}

#[test]
fn controls_dialog_lists_the_canonical_commands() {
    let mut harness = harness();
    harness.get_by_label("Controls").click();
    harness.run();
    harness.get_by_label("Controls & Commands");

    let expected = loresmith_rs::commands::help_text();
    assert!(
        harness.query_by_value(&expected).is_some(),
        "the controls dialog should show the canonical help text"
    );
}
