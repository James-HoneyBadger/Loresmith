//! Headless UI tests for the graphical editor.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use loresmith_studio::LoresmithStudio;

fn harness() -> Harness<'static, LoresmithStudio> {
    let app = LoresmithStudio::with_storage(None);
    // Most tab fields are wired to their captions; the ones that are not are
    // tracked by `unlabelled_form_fields_stay_bounded` below, so the global
    // accessibility check stays off until that count reaches zero.
    let mut harness = Harness::builder()
        .with_accessibility_check(false)
        .build_ui_state(|ui, app: &mut LoresmithStudio| app.update_ui(ui), app);
    harness.run();
    harness
}

#[test]
fn starts_on_the_play_tab() {
    let harness = harness();
    assert_eq!(harness.state().active_tab_title(), "Play");
    harness.get_by_label_contains("Run and test the adventure from the current editor state");
}

#[test]
fn every_tab_renders_without_panicking() {
    let mut harness = harness();
    for title in LoresmithStudio::TAB_TITLES {
        assert!(
            harness.state_mut().select_tab(title),
            "{title} should be a known tab"
        );
        harness.run();
        assert_eq!(harness.state().active_tab_title(), *title);
    }
    assert!(
        !harness.state_mut().select_tab("Nope"),
        "unknown tabs are rejected"
    );
}

#[test]
fn rooms_tab_offers_room_editing() {
    let mut harness = harness();
    assert!(harness.state_mut().select_tab("Rooms"));
    harness.run();
    harness.get_by_label("Add Room");
    assert!(!harness.state().is_modified());
}

#[test]
fn adding_a_room_marks_the_document_modified() {
    let mut harness = harness();
    assert!(harness.state_mut().select_tab("Rooms"));
    harness.run();
    harness.get_by_label("Add Room").click();
    harness.run();
    assert!(
        harness.state().is_modified(),
        "adding a room should mark the document modified"
    );
}

/// Ratchet on unlabelled form fields.
///
/// Numeric and text fields inside the tab editors are not all wired to their
/// captions yet: the ones that remain are nested inside `if`/`horizontal`
/// blocks where the caption is a few lines above the widget. `labelled_row` /
/// `labelled_field` (and `combo_label` + `.labelled_by`) cover the rest, so
/// this bound only moves down.
#[test]
fn unlabelled_form_fields_stay_bounded() {
    use egui_kittest::kittest::NodeT;

    fn count_unnamed(node: &egui_kittest::Node<'_>) -> usize {
        let accesskit = node.accesskit_node();
        let role = format!("{:?}", accesskit.role());
        let named = accesskit.label().is_some_and(|label| !label.is_empty());
        let here = if !named
            && matches!(
                role.as_str(),
                "SpinButton" | "TextInput" | "ComboBox" | "Slider"
            ) {
            1
        } else {
            0
        };
        let children: Vec<_> = node.children().collect();
        here + children.iter().map(count_unnamed).sum::<usize>()
    }

    let mut harness = Harness::builder()
        .with_accessibility_check(false)
        .build_ui_state(
            |ui, app: &mut LoresmithStudio| app.update_ui(ui),
            LoresmithStudio::with_storage(None),
        );
    harness.run();

    let mut unnamed = 0;
    for title in LoresmithStudio::TAB_TITLES {
        harness.state_mut().select_tab(title);
        harness.run();
        let root = harness.root();
        unnamed += count_unnamed(&root);
    }

    assert!(
        unnamed <= 21,
        "{unnamed} form fields lack an accessible name (was 21); \n\
         wrap them with theme::labelled_row/labelled_field or .labelled_by(...)"
    );
}
