use loresmith_rs::{AdventureDocument, Engine, award_experience};

fn engine(json: &str) -> Engine {
    let document = AdventureDocument::from_json_str(json).expect("document should parse");
    Engine::from_document(document).expect("engine should load")
}

#[test]
fn quest_kill_target_with_apostrophe_progresses() {
    let mut engine = engine(
        r#"{
            "title": "Punctuation", "start_room": 1,
            "rooms": [{"id": 1, "name": "Arena", "description": "Dust.", "exits": {}}],
            "monsters": [{"id": 1, "name": "Usurper's Champion", "room_id": 1,
                          "friendliness": "hostile", "hardiness": 3, "agility": 1}],
            "quests": [{"id": 1, "title": "End the Usurper", "description": "Fight.",
                        "objectives": [{"type": "kill_monster",
                                        "target_id": "Usurper's Champion", "count": 1}],
                        "rewards": {"gold": 5, "xp": 10}}]
        }"#,
    );

    engine.send("accept 1");
    engine.game.monsters.get_mut(&1).unwrap().current_health = 1;
    let output = engine.send("attack usurper").join("\n");
    assert!(
        output.contains("You defeat the Usurper's Champion"),
        "{output}"
    );

    let journal = engine.send("quests").join("\n");
    assert!(journal.contains("(1/1)"), "{journal}");
    assert!(journal.contains("Ready to turn in"), "{journal}");

    let completed = engine.send("complete 1").join("\n");
    assert!(completed.contains("Completed quest"), "{completed}");
}

#[test]
fn quest_talk_objective_and_rewards() {
    let mut engine = engine(
        r#"{
            "title": "Talk", "start_room": 1,
            "rooms": [{"id": 1, "name": "Hall", "description": "Echoes.", "exits": {}}],
            "monsters": [{"id": 1, "name": "Elder", "room_id": 1, "friendliness": "friendly",
                          "dialogue": {"greeting": "Welcome.", "topics": {"gate": "It is barred."}}}],
            "items": [{"id": 7, "name": "Silver Coin", "description": "A coin.", "value": 10}],
            "quests": [{"id": 1, "title": "Ask the Elder", "description": "Speak.",
                        "objectives": [{"type": "talk_to_npc", "target_id": "Elder", "count": 1}],
                        "rewards": {"gold": 5, "xp": 10, "items": [7],
                                    "reputation": {"Village": 2}}}]
        }"#,
    );

    engine.send("accept 1");
    let talk = engine.send("talk elder").join("\n");
    assert!(talk.contains("Welcome."), "{talk}");
    let ask = engine.send("ask elder about gate").join("\n");
    assert!(ask.contains("It is barred."), "{ask}");

    let journal = engine.send("quests").join("\n");
    assert!(journal.contains("(1/1)"), "{journal}");

    let completed = engine.send("complete 1").join("\n");
    assert!(completed.contains("Received: Silver Coin"), "{completed}");
    assert_eq!(engine.game.player.reputation.get("Village"), Some(&2));
}

#[test]
fn complete_rejects_unfinished_quest() {
    let mut engine = engine(
        r#"{
            "title": "Blocked", "start_room": 1,
            "rooms": [{"id": 1, "name": "Camp", "description": "Camp.", "exits": {}}],
            "quests": [{"id": 1, "title": "Dragon", "description": "Kill it.",
                        "objectives": [{"type": "kill_monster", "target_id": "dragon"}],
                        "rewards": {"gold": 100, "xp": 100}}]
        }"#,
    );

    engine.send("accept 1");
    let output = engine.send("complete 1").join("\n");
    assert!(output.contains("cannot be completed yet"), "{output}");
    assert_eq!(engine.game.player.gold, 200, "reward must not be paid");
}

#[test]
fn locked_exit_requires_and_consumes_key() {
    let mut engine = engine(
        r#"{
            "title": "Locks", "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Hall", "description": "A hall.",
                 "exits": {"north": {"to": 2, "locked": true, "key": "Brass Key",
                                      "consume_key": true, "message": "The door is barred."}}},
                {"id": 2, "name": "Vault", "description": "Treasure.", "exits": {}}
            ],
            "items": [{"id": 1, "name": "Brass Key", "description": "A key.", "location": 0}]
        }"#,
    );

    assert!(
        engine.game.player.inventory.contains(&1),
        "location 0 should start carried"
    );
    let output = engine.send("north").join("\n");
    assert!(output.contains("unlock"), "{output}");
    assert_eq!(engine.game.player.current_room, 2);
    assert!(
        !engine.game.player.inventory.contains(&1),
        "key should be consumed"
    );
}

#[test]
fn locked_exit_without_key_is_refused() {
    let mut engine = engine(
        r#"{
            "title": "Locks", "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Hall", "description": "A hall.",
                 "exits": {"north": {"to": 2, "locked": true, "key": "Brass Key",
                                      "message": "The door is barred."}}},
                {"id": 2, "name": "Vault", "description": "Treasure.", "exits": {}}
            ],
            "items": [{"id": 1, "name": "Brass Key", "description": "A key.", "location": 2}]
        }"#,
    );

    let output = engine.send("north").join("\n");
    assert!(output.contains("barred"), "{output}");
    assert_eq!(engine.game.player.current_room, 1);
}

#[test]
fn dark_room_requires_light_source() {
    let with_light = engine(
        r#"{
            "title": "Dark", "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Hall", "description": "A hall.", "exits": {"north": 2}},
                {"id": 2, "name": "Crypt", "description": "Cold stone.", "is_dark": true, "exits": {}}
            ],
            "items": [{"id": 1, "name": "Lantern", "description": "It glows.",
                       "is_light_source": true, "location": 0}]
        }"#,
    );
    let mut with_light = with_light;
    let output = with_light.send("north").join("\n");
    assert!(output.contains("Cold stone."), "{output}");

    let mut without_light = engine(
        r#"{
            "title": "Dark", "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Hall", "description": "A hall.", "exits": {"north": 2}},
                {"id": 2, "name": "Crypt", "description": "Cold stone.", "is_dark": true, "exits": {}}
            ]
        }"#,
    );
    let output = without_light.send("north").join("\n");
    assert!(output.contains("pitch black"), "{output}");
}

#[test]
fn shops_buy_and_sell() {
    let mut engine = engine(
        r#"{
            "title": "Shop", "start_room": 1,
            "rooms": [{"id": 1, "name": "Market", "description": "Stalls.", "exits": {}}],
            "items": [{"id": 5, "name": "Iron Sword", "description": "A blade.",
                       "type": "weapon", "is_weapon": true, "value": 100, "weight": 2}],
            "shops": [{"id": 1, "name": "Smithy", "room_id": 1, "gold": 500,
                       "stock": [{"item_id": 5, "quantity": 1}]}]
        }"#,
    );

    let listing = engine.send("shop").join("\n");
    assert!(listing.contains("Iron Sword"), "{listing}");

    let gold_before = engine.game.player.gold;
    let bought = engine.send("buy iron sword").join("\n");
    assert!(bought.contains("Bought the Iron Sword"), "{bought}");
    assert!(engine.game.player.gold < gold_before);

    let sold = engine.send("sell iron sword").join("\n");
    assert!(sold.contains("Sold the Iron Sword"), "{sold}");
}

#[test]
fn shop_purchase_counts_for_collect_quests_and_cannot_be_arbitraged() {
    let mut engine = engine(
        r#"{
            "title": "Market Quest", "start_room": 1,
            "rooms": [{"id": 1, "name": "Market", "description": "Stalls.", "exits": {}}],
            "items": [{"id": 5, "name": "Iron Sword", "description": "A blade.",
                       "type": "weapon", "is_weapon": true, "value": 100, "weight": 2}],
            "shops": [{"id": 1, "name": "Smithy", "room_id": 1, "gold": 500,
                       "stock": [{"item_id": 5, "quantity": 1}]}],
            "quests": [{"id": 1, "title": "Buy a Blade", "description": "Purchase one.",
                        "objectives": [{"type": "collect_item",
                                        "target_id": "Iron Sword", "count": 1}],
                        "rewards": {"gold": 5, "xp": 10}}]
        }"#,
    );
    engine.send("accept 1");
    let gold_before = engine.game.player.gold;
    let bought = engine.send("buy iron sword").join("\n");
    assert!(bought.contains("Bought the Iron Sword"), "{bought}");

    let journal = engine.send("quests").join("\n");
    assert!(journal.contains("(1/1)"), "{journal}");
    assert!(journal.contains("Ready to turn in"), "{journal}");

    let sold_out = engine.send("buy iron sword").join("\n");
    assert!(sold_out.contains("sold out"), "{sold_out}");

    engine.game.player.charisma = 100;
    let listing = engine.send("shop").join("\n");
    assert!(listing.contains("You can sell:"), "{listing}");
    assert!(listing.contains("Iron Sword — 49 gold"), "{listing}");

    let sold = engine.send("sell iron sword").join("\n");
    assert!(sold.contains("Sold the Iron Sword"), "{sold}");
    assert!(
        engine.game.player.gold < gold_before,
        "buying and selling back must not create gold: {} -> {}",
        gold_before,
        engine.game.player.gold
    );
}

#[test]
fn multiple_shops_in_one_room_are_listed_and_compared() {
    let mut engine = engine(
        r#"{
            "title": "Two Shops", "start_room": 1,
            "rooms": [{"id": 1, "name": "Market", "description": "Stalls.", "exits": {}}],
            "items": [{"id": 5, "name": "Iron Sword", "description": "A blade.",
                       "type": "weapon", "is_weapon": true, "value": 100, "weight": 2}],
            "shops": [
                {"id": 1, "name": "Cheap Market", "room_id": 1, "gold": 500,
                 "buy_multiplier": 1.0, "sell_multiplier": 0.5,
                 "stock": [{"item_id": 5, "quantity": 1}]},
                {"id": 2, "name": "Expensive Emporium", "room_id": 1, "gold": 500,
                 "buy_multiplier": 2.0, "sell_multiplier": 0.5,
                 "stock": [{"item_id": 5, "quantity": 1}]}
            ]
        }"#,
    );

    let listing = engine.send("shop").join("\n");
    assert!(listing.contains("Cheap Market"), "{listing}");
    assert!(listing.contains("Expensive Emporium"), "{listing}");
    assert!(listing.contains("Your gold: 200"), "{listing}");

    let filtered = engine.send("shop expensive").join("\n");
    assert!(filtered.contains("Expensive Emporium"), "{filtered}");
    assert!(!filtered.contains("Cheap Market"), "{filtered}");

    let bought = engine.send("buy iron sword").join("\n");
    assert!(bought.contains("at Cheap Market"), "{bought}");
    assert!(bought.contains("88 gold"), "{bought}");
}

#[test]
fn victory_condition_ends_the_game() {
    let mut engine = engine(
        r#"{
            "title": "Win", "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Hall", "description": "A hall.", "exits": {"north": 2}},
                {"id": 2, "name": "Throne", "description": "A throne.", "exits": {}}
            ],
            "ending": {"victory_text": "The realm is saved.",
                       "conditions": {"rooms": [2]}}
        }"#,
    );

    let output = engine.send("north").join("\n");
    assert!(output.contains("The realm is saved."), "{output}");
    assert!(engine.won());
    assert!(engine.is_over());
}

#[test]
fn experience_awards_multiple_levels() {
    let mut engine = engine(
        r#"{"title": "XP", "start_room": 1,
            "rooms": [{"id": 1, "name": "A", "description": "A", "exits": {}}]}"#,
    );
    let messages = award_experience(&mut engine.game, 1000);
    // Thresholds are `level * 100` against cumulative XP, so 1000 XP reaches
    // level 11 (ten level-ups).
    assert_eq!(engine.game.player.level, 11);
    assert_eq!(messages.len(), 10);
}

#[test]
fn loader_rejects_duplicate_and_unknown_references() {
    let duplicate = AdventureDocument::from_json_str(
        r#"{"title": "Dup", "start_room": 1,
            "rooms": [{"id": 1, "name": "A", "description": "A"},
                      {"id": 1, "name": "B", "description": "B"}]}"#,
    )
    .expect("document should parse");
    assert!(!duplicate.report().is_valid());

    let unknown_exit = AdventureDocument::from_json_str(
        r#"{"title": "Bad", "start_room": 1,
            "rooms": [{"id": 1, "name": "A", "description": "A", "exits": {"north": 99}}]}"#,
    )
    .expect("document should parse");
    assert!(!unknown_exit.report().is_valid());
}

#[test]
fn save_v2_is_portable_and_v1_migrates() {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir();
    let adventure_path: PathBuf = dir.join(format!("loresmith_adventure_{unique}.json"));
    let save_path: PathBuf = dir.join(format!("loresmith_save_{unique}.json"));
    let v1_path: PathBuf = dir.join(format!("loresmith_v1_{unique}.json"));

    let document = AdventureDocument::from_json_str(
        r#"{
            "title": "Portable", "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Hall", "description": "A hall.", "exits": {"north": 2}},
                {"id": 2, "name": "Vault", "description": "A vault.", "exits": {}}
            ],
            "items": [{"id": 1, "name": "Coin", "description": "A coin.", "value": 5, "location": 0}]
        }"#,
    )
    .expect("document");
    document
        .save_json_file(&adventure_path)
        .expect("adventure save");

    let mut engine = Engine::load(adventure_path.to_string_lossy().to_string()).expect("engine");
    engine.send("north");
    engine.save_game(&save_path).expect("save v2");

    // v2 saves carry the document; the adventure file is no longer needed.
    fs::remove_file(&adventure_path).expect("remove adventure");
    let mut restored = Engine::new("missing.json");
    restored.load_game(&save_path).expect("v2 save should load");
    assert_eq!(restored.game.player.current_room, 2);

    // Simulate a v1 snapshot by removing the embedded document.
    let mut snapshot: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&save_path).expect("read save")).expect("json");
    snapshot["version"] = serde_json::Value::from(1);
    snapshot.as_object_mut().expect("object").remove("document");
    fs::write(
        &v1_path,
        serde_json::to_string_pretty(&snapshot).expect("json"),
    )
    .expect("write v1");

    // Recreate the adventure file for the v1 fallback path.
    document
        .save_json_file(&adventure_path)
        .expect("adventure save again");
    let mut migrated = Engine::new("missing.json");
    migrated
        .load_game(&v1_path)
        .expect("v1 save should migrate");
    assert_eq!(migrated.game.player.current_room, 2);

    let _ = fs::remove_file(&save_path);
    let _ = fs::remove_file(&v1_path);
    let _ = fs::remove_file(&adventure_path);
}
