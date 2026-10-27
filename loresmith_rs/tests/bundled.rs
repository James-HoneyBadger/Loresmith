use std::path::PathBuf;

fn adventure_files() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/adventures");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("adventure directory should exist")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no bundled adventures found");
    files
}

#[test]
fn all_bundled_adventures_load_and_validate() {
    for path in adventure_files() {
        let document = loresmith_rs::AdventureDocument::load_json_file(&path)
            .unwrap_or_else(|err| panic!("{} failed to load: {}", path.display(), err));
        let report = document.report();
        assert!(
            report.is_valid(),
            "{} has validation errors: {:?}",
            path.display(),
            report.errors
        );
        assert!(
            document.schema_version == loresmith_rs::document::SCHEMA_VERSION,
            "{} was not normalised",
            path.display()
        );
        assert!(
            !document.rooms.is_empty() && document.start_room != 0,
            "{} has no usable rooms",
            path.display()
        );

        // Editors must round-trip without losing data.
        let json = document.to_json_string().expect("serialise");
        let reparsed = loresmith_rs::AdventureDocument::from_json_str(&json)
            .unwrap_or_else(|err| panic!("{} failed to reparse: {}", path.display(), err));
        assert_eq!(
            document,
            reparsed,
            "{} did not survive a canonical round-trip",
            path.display()
        );
    }
}

#[test]
fn all_bundled_adventures_start_in_engine() {
    for path in adventure_files() {
        let mut engine = loresmith_rs::Engine::load(path.to_string_lossy().to_string())
            .unwrap_or_else(|err| panic!("{} failed to start: {}", path.display(), err));
        let look = engine.look();
        assert!(
            !look.is_empty(),
            "{} has an empty start room",
            path.display()
        );
        let _ = engine.send("status");
        let _ = engine.send("quests");
        let _ = engine.send("inventory");
    }
}

#[test]
fn every_room_is_reachable() {
    for path in adventure_files() {
        let document = loresmith_rs::AdventureDocument::load_json_file(&path).unwrap();
        assert!(
            !document
                .report()
                .warnings
                .iter()
                .any(|warning| warning.contains("not reachable")),
            "{} contains unreachable rooms: {:?}",
            path.display(),
            document.report().warnings
        );
    }
}
