use crate::document::{AdventureDocument, EndingConditions, InventoryRef, ItemDef, ShopDef};
use crate::systems::System;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub use crate::document::DialogueDef as Dialogue;

/// Case-insensitive substring match for item/monster names.
pub(crate) fn name_matches(name: &str, query: &str) -> bool {
    name.to_lowercase().contains(&query.to_lowercase())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Weapon,
    Armor,
    Treasure,
    Readable,
    Edible,
    Drinkable,
    Container,
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonsterStatus {
    Friendly,
    Neutral,
    Hostile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub weight: i32,
    pub value: i32,
    pub is_weapon: bool,
    pub weapon_type: i32, // 1=axe, 2=bow, 3=club, 4=spear, 5=sword
    pub weapon_dice: i32,
    pub weapon_sides: i32,
    pub is_armor: bool,
    pub armor_value: i32,
    pub is_takeable: bool,
    pub is_wearable: bool,
    /// Carrying this item reveals dark rooms.
    #[serde(default)]
    pub is_light_source: bool,
    /// Reserved capacity for container items.
    #[serde(default)]
    pub container_capacity: i32,
    /// 0 = carried (when in `Player::inventory`), -1 = worn, room ID, or monster ID.
    pub location: i32,
}

impl Item {
    pub fn new(
        id: i32,
        name: String,
        description: String,
        item_type: ItemType,
        weight: i32,
        value: i32,
    ) -> Self {
        Self {
            id,
            name,
            description,
            item_type,
            weight,
            value,
            is_weapon: false,
            weapon_type: 0,
            weapon_dice: 1,
            weapon_sides: 6,
            is_armor: false,
            armor_value: 0,
            is_takeable: true,
            is_wearable: false,
            is_light_source: false,
            container_capacity: 0,
            location: 0,
        }
    }

    pub fn get_damage(&self) -> i32 {
        if !self.is_weapon {
            return 0;
        }
        use rand::Rng;
        let mut rng = rand::rng();
        (0..self.weapon_dice)
            .map(|_| rng.random_range(1..=self.weapon_sides))
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub room_id: i32,
    pub hardiness: i32,
    pub agility: i32,
    pub friendliness: MonsterStatus,
    pub courage: i32,
    pub weapon_id: Option<i32>,
    pub armor_worn: i32,
    pub gold: i32,
    pub is_dead: bool,
    pub current_health: i32,
    #[serde(default)]
    pub dialogue: Option<Dialogue>,
    #[serde(default)]
    pub loot: Vec<i32>,
}

impl Monster {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i32,
        name: String,
        description: String,
        room_id: i32,
        hardiness: i32,
        agility: i32,
        friendliness: MonsterStatus,
        courage: i32,
    ) -> Self {
        Self {
            id,
            name,
            description,
            room_id,
            hardiness,
            agility,
            friendliness,
            courage,
            weapon_id: None,
            armor_worn: 0,
            gold: 0,
            is_dead: false,
            current_health: hardiness,
            dialogue: None,
            loot: Vec::new(),
        }
    }
}

/// Runtime lock state for an exit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedExit {
    pub to: i32,
    pub key: Option<String>,
    pub consume_key: bool,
    pub message: String,
    pub unlocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, i32>, // direction -> room_id
    pub is_dark: bool,
    pub locked_exits: HashMap<String, LockedExit>,
}

impl Room {
    pub fn new(id: i32, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            exits: HashMap::new(),
            is_dark: false,
            locked_exits: HashMap::new(),
        }
    }

    pub fn get_exit(&self, direction: &str) -> Option<i32> {
        self.exits.get(&direction.to_lowercase()).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub hardiness: i32,
    pub agility: i32,
    pub charisma: i32,
    pub weapon_ability: HashMap<i32, i32>, // weapon_type -> ability
    pub armor_expertise: i32,
    pub gold: i32,
    pub current_room: i32,
    pub current_health: i32,
    pub inventory: Vec<i32>, // item IDs
    pub equipped_weapon: Option<i32>,
    pub equipped_armor: Option<i32>,
    pub experience_points: i32,
    pub level: i32,
    #[serde(default)]
    pub reputation: HashMap<String, i32>,
}

impl Player {
    pub fn new() -> Self {
        let mut weapon_ability = HashMap::new();
        for i in 1..=5 {
            weapon_ability.insert(i, 5);
        }
        Self {
            name: "Adventurer".to_string(),
            hardiness: 12,
            agility: 12,
            charisma: 12,
            weapon_ability,
            armor_expertise: 0,
            gold: 200,
            current_room: 1,
            current_health: 12,
            inventory: Vec::new(),
            equipped_weapon: None,
            equipped_armor: None,
            experience_points: 0,
            level: 1,
            reputation: HashMap::new(),
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by systems so other systems can react (quest tracking, etc.).
#[derive(Debug, Clone)]
pub enum GameEvent {
    MonsterKilled {
        monster_name: String,
        monster_id: i32,
        room_id: i32,
    },
    ItemCollected {
        item_name: String,
        item_id: i32,
    },
    RoomEntered {
        room_id: i32,
    },
    ItemUsed {
        item_name: String,
    },
    TalkedTo {
        npc_name: String,
        npc_id: i32,
        topic: Option<String>,
    },
    ItemPurchased {
        item_name: String,
        item_id: i32,
    },
    ItemSold {
        item_name: String,
        item_id: i32,
    },
    ExitUnlocked {
        room_id: i32,
        direction: String,
    },
    QuestCompleted {
        quest_id: String,
        title: String,
    },
    GameWon {
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopStock {
    pub item_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub id: i32,
    pub name: String,
    pub room_id: i32,
    pub gold: i32,
    pub stock: Vec<ShopStock>,
    pub buy_multiplier: f32,
    pub sell_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameSnapshot {
    version: u32,
    adventure_file: String,
    player: Player,
    items: HashMap<i32, Item>,
    monsters: HashMap<i32, Monster>,
    turn_count: i32,
    game_over: bool,
    autosave_enabled: bool,
    autosave_path: String,
    system_states: HashMap<String, serde_json::Value>,
    /// Snapshot v2+: the full document makes saves portable.
    #[serde(default)]
    document: Option<AdventureDocument>,
    #[serde(default)]
    shops: HashMap<i32, Shop>,
    #[serde(default)]
    game_won: bool,
    #[serde(default)]
    ending_text: String,
    #[serde(default)]
    unlocked_exits: Vec<(i32, String)>,
    #[serde(default)]
    last_room: i32,
    #[serde(default)]
    visited_rooms: Vec<i32>,
    #[serde(default)]
    completed_quests: Vec<String>,
}

pub struct AdventureGame {
    pub adventure_file: String,
    pub rooms: HashMap<i32, Room>,
    pub items: HashMap<i32, Item>,
    pub monsters: HashMap<i32, Monster>,
    pub player: Player,
    pub turn_count: i32,
    pub game_over: bool,
    pub adventure_title: String,
    pub adventure_intro: String,
    pub autosave_enabled: bool,
    pub autosave_path: String,
    pub systems: Vec<Box<dyn System>>,
    pub quests: Vec<serde_json::Value>, // Quest definitions (derived from `document`)
    pub events: Vec<GameEvent>,         // Inter-system event bus
    pub document: AdventureDocument,
    pub shops: HashMap<i32, Shop>,
    pub game_won: bool,
    pub ending_text: String,
    pub ending_conditions: Option<EndingConditions>,
    pub last_room: i32,
    pub visited_rooms: std::collections::HashSet<i32>,
    pub completed_quests: std::collections::HashSet<String>,
}

fn item_from_def(def: &ItemDef) -> Item {
    Item {
        id: def.id,
        name: def.name.clone(),
        description: def.description.clone(),
        item_type: def.item_type.clone(),
        weight: def.weight,
        value: def.value,
        is_weapon: def.is_weapon,
        weapon_type: def.weapon_type,
        weapon_dice: def.weapon_dice,
        weapon_sides: def.weapon_sides,
        is_armor: def.is_armor,
        armor_value: def.armor_value,
        is_takeable: def.is_takeable,
        is_wearable: def.is_wearable,
        is_light_source: def.is_light_source,
        container_capacity: def.container_capacity,
        location: def.location,
    }
}

fn shop_from_def(def: &ShopDef) -> Shop {
    Shop {
        id: def.id,
        name: def.name.clone(),
        room_id: def.room_id,
        gold: def.gold,
        stock: def
            .stock
            .iter()
            .map(|stock| ShopStock {
                item_id: stock.item_id,
                quantity: stock.quantity,
            })
            .collect(),
        buy_multiplier: def.buy_multiplier,
        sell_multiplier: def.sell_multiplier,
    }
}

/// Award XP and apply every level-up the total qualifies for.
pub fn award_experience(game: &mut AdventureGame, amount: i32) -> Vec<String> {
    if amount <= 0 {
        return Vec::new();
    }
    game.player.experience_points = game.player.experience_points.saturating_add(amount);
    let mut messages = Vec::new();
    loop {
        let threshold = game.player.level.saturating_mul(100);
        if game.player.experience_points < threshold {
            break;
        }
        game.player.level += 1;
        game.player.hardiness += 2;
        game.player.agility += 1;
        game.player.current_health = game.player.hardiness;
        messages.push(format!(
            "*** Level Up! You are now level {}. Hardiness +2, Agility +1. Health restored to {}. ***",
            game.player.level, game.player.hardiness
        ));
        // Defensive bound for malformed saves with absurd XP.
        if game.player.level >= 10_000 {
            break;
        }
    }
    messages
}

impl AdventureGame {
    pub fn new(adventure_file: String) -> Self {
        Self {
            adventure_file,
            rooms: HashMap::new(),
            items: HashMap::new(),
            monsters: HashMap::new(),
            player: Player::new(),
            turn_count: 0,
            game_over: false,
            adventure_title: String::new(),
            adventure_intro: String::new(),
            autosave_enabled: false,
            autosave_path: "autosave.json".to_string(),
            systems: Vec::new(),
            quests: Vec::new(),
            events: Vec::new(),
            document: AdventureDocument::default(),
            shops: HashMap::new(),
            game_won: false,
            ending_text: String::new(),
            ending_conditions: None,
            last_room: 0,
            visited_rooms: std::collections::HashSet::new(),
            completed_quests: std::collections::HashSet::new(),
        }
    }

    fn reset_runtime_state(&mut self) {
        self.rooms.clear();
        self.items.clear();
        self.monsters.clear();
        self.shops.clear();
        self.quests.clear();
        self.events.clear();
        self.turn_count = 0;
        self.game_over = false;
        self.game_won = false;
        self.ending_text.clear();
        self.ending_conditions = None;
        self.last_room = 0;
        self.visited_rooms.clear();
        self.completed_quests.clear();
        self.player = Player::new();
    }

    fn build_header(&self) -> String {
        let mut header = format!("\n{:=^60}\n{:^60}\n{:=^60}\n", "", self.adventure_title, "");
        if !self.adventure_intro.is_empty() {
            header.push('\n');
            header.push_str(&self.adventure_intro);
            header.push('\n');
        }
        header
    }

    pub fn load_adventure(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let document = AdventureDocument::load_json_file(&self.adventure_file)?;
        self.load_document(document)
    }

    /// Load an adventure from a JSON string (used by editors and tests).
    pub fn load_from_str(&mut self, json: &str) -> Result<String, Box<dyn std::error::Error>> {
        let document = AdventureDocument::from_json_str(json)?;
        self.load_document(document)
    }

    /// Load an adventure from an already-parsed JSON value.
    pub fn load_from_value(
        &mut self,
        value: serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let document = AdventureDocument::from_value(value)?;
        self.load_document(document)
    }

    /// Build runtime state from a canonical document.
    pub fn load_document(
        &mut self,
        document: AdventureDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let document = document.normalized();
        document.validate()?;

        self.reset_runtime_state();
        self.document = document.clone();
        self.adventure_title = document.title.clone();
        self.adventure_intro = document.intro.clone();
        self.quests = document
            .quests
            .iter()
            .filter_map(|quest| serde_json::to_value(quest).ok())
            .collect();

        for room_def in &document.rooms {
            let mut exits = HashMap::new();
            let mut locked_exits = HashMap::new();
            for (direction, exit) in &room_def.exits {
                exits.insert(direction.clone(), exit.destination());
                if let Some(lock) = exit.lock() {
                    locked_exits.insert(
                        direction.clone(),
                        LockedExit {
                            to: lock.to,
                            key: lock.key.clone(),
                            consume_key: lock.consume_key,
                            message: lock
                                .message
                                .clone()
                                .unwrap_or_else(|| "It is locked.".to_string()),
                            unlocked: !lock.locked,
                        },
                    );
                }
            }
            self.rooms.insert(
                room_def.id,
                Room {
                    id: room_def.id,
                    name: if room_def.name.trim().is_empty() {
                        format!("Room {}", room_def.id)
                    } else {
                        room_def.name.clone()
                    },
                    description: room_def.description.clone(),
                    exits,
                    is_dark: room_def.is_dark,
                    locked_exits,
                },
            );
        }

        for item_def in &document.items {
            self.items.insert(item_def.id, item_from_def(item_def));
        }

        // Room `items` lists define placement, matching historical behavior.
        for room in &document.rooms {
            for item_id in &room.items {
                if let Some(item) = self.items.get_mut(item_id) {
                    item.location = room.id;
                }
            }
        }

        for entry in &document.player_start_inventory {
            match entry {
                InventoryRef::Id(id) => {
                    if let Some(item) = self.items.get_mut(id) {
                        item.location = 0;
                    }
                    if !self.player.inventory.contains(id) {
                        self.player.inventory.push(*id);
                    }
                }
                InventoryRef::Item(item_def) => {
                    let item = item_from_def(item_def);
                    if !self.player.inventory.contains(&item.id) {
                        self.player.inventory.push(item.id);
                    }
                    self.items.insert(item.id, item);
                }
            }
        }

        for monster_def in &document.monsters {
            let mut monster = Monster::new(
                monster_def.id,
                monster_def.name.clone(),
                monster_def.description.clone(),
                monster_def.room_id,
                monster_def.hardiness,
                monster_def.agility,
                monster_def.friendliness.clone(),
                monster_def.courage,
            );
            monster.weapon_id = monster_def.weapon_id;
            monster.armor_worn = monster_def.armor_worn;
            monster.gold = monster_def.gold;
            monster.dialogue = monster_def.dialogue.clone();
            monster.loot = monster_def.loot.clone();
            self.monsters.insert(monster.id, monster);
        }

        for shop_def in &document.shops {
            self.shops.insert(shop_def.id, shop_from_def(shop_def));
        }

        if let Some(ending) = &document.ending {
            self.ending_conditions = Some(ending.conditions.clone());
            self.ending_text = ending.victory_text.clone();
        }

        self.player.current_room = document.start_room;
        self.last_room = document.start_room;
        self.visited_rooms.insert(document.start_room);

        self.validate_loaded_state()
            .map_err(|msg| std::io::Error::new(std::io::ErrorKind::InvalidData, msg))?;
        self.notify_adventure_loaded();

        Ok(self.build_header())
    }

    fn notify_adventure_loaded(&mut self) {
        let mut systems = std::mem::take(&mut self.systems);
        for system in &mut systems {
            system.on_adventure_loaded(self);
        }
        self.systems = systems;
    }

    fn validate_loaded_state(&self) -> Result<(), String> {
        if self.rooms.is_empty() {
            return Err("adventure must define at least one room".to_string());
        }

        if !self.rooms.contains_key(&self.player.current_room) {
            return Err(format!(
                "start_room {} does not exist in the loaded adventure",
                self.player.current_room
            ));
        }

        for room in self.rooms.values() {
            for (direction, dest_id) in &room.exits {
                if !self.rooms.contains_key(dest_id) {
                    return Err(format!(
                        "room {} exit '{}' points to unknown room {}",
                        room.id, direction, dest_id
                    ));
                }
            }
        }

        for item in self.items.values() {
            // -2 means "stored in a shop's inventory".
            let valid_location = matches!(item.location, -2..=0)
                || self.rooms.contains_key(&item.location)
                || self.monsters.contains_key(&item.location);
            if !valid_location {
                return Err(format!(
                    "item '{}' references unknown location {}",
                    item.name, item.location
                ));
            }
        }

        for monster in self.monsters.values() {
            if !self.rooms.contains_key(&monster.room_id) {
                return Err(format!(
                    "monster '{}' references unknown room {}",
                    monster.name, monster.room_id
                ));
            }
            if let Some(weapon_id) = monster.weapon_id
                && !self.items.contains_key(&weapon_id)
            {
                return Err(format!(
                    "monster '{}' references unknown weapon item {}",
                    monster.name, weapon_id
                ));
            }
        }

        Ok(())
    }

    fn collect_system_states(&self) -> HashMap<String, serde_json::Value> {
        let mut states = HashMap::new();
        for system in &self.systems {
            if let Some(key) = system.state_key()
                && let Some(value) = system.save_state()
            {
                states.insert(key.to_string(), value);
            }
        }
        states
    }

    fn restore_system_states(&mut self, states: &HashMap<String, serde_json::Value>) {
        let mut systems = std::mem::take(&mut self.systems);
        for system in &mut systems {
            let state = system.state_key().and_then(|key| states.get(key));
            system.load_state(state, self);
        }
        self.systems = systems;
    }

    pub fn save_game(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let unlocked_exits = self
            .rooms
            .values()
            .flat_map(|room| {
                room.locked_exits
                    .iter()
                    .filter(|(_, lock)| lock.unlocked)
                    .map(move |(direction, _)| (room.id, direction.clone()))
            })
            .collect();

        let snapshot = GameSnapshot {
            version: 2,
            adventure_file: self.adventure_file.clone(),
            player: self.player.clone(),
            items: self.items.clone(),
            monsters: self.monsters.clone(),
            turn_count: self.turn_count,
            game_over: self.game_over,
            autosave_enabled: self.autosave_enabled,
            autosave_path: self.autosave_path.clone(),
            system_states: self.collect_system_states(),
            document: Some(self.document.clone()),
            shops: self.shops.clone(),
            game_won: self.game_won,
            ending_text: self.ending_text.clone(),
            unlocked_exits,
            last_room: self.last_room,
            visited_rooms: self.visited_rooms.iter().copied().collect(),
            completed_quests: self.completed_quests.iter().cloned().collect(),
        };

        let json = serde_json::to_string_pretty(&snapshot)?;
        crate::util::write_atomic(path, json.as_bytes())?;
        Ok(())
    }

    pub fn load_game(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let snapshot: GameSnapshot = serde_json::from_str(&fs::read_to_string(path)?)?;
        if snapshot.version > 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "save version {} is newer than this build supports (2)",
                    snapshot.version
                ),
            )
            .into());
        }

        self.adventure_file = snapshot.adventure_file.clone();
        let intro = if let Some(document) = snapshot.document.clone() {
            self.load_document(document)?
        } else {
            // v1 snapshots did not embed the adventure; fall back to the file.
            self.load_adventure()?
        };

        self.player = snapshot.player;
        self.items = snapshot.items;
        self.monsters = snapshot.monsters;
        self.turn_count = snapshot.turn_count;
        self.game_over = snapshot.game_over;
        self.autosave_enabled = snapshot.autosave_enabled;
        self.autosave_path = snapshot.autosave_path;
        self.shops = snapshot.shops;
        self.game_won = snapshot.game_won;
        self.ending_text = snapshot.ending_text;
        self.last_room = snapshot.last_room;
        self.visited_rooms = snapshot.visited_rooms.into_iter().collect();
        self.completed_quests = snapshot.completed_quests.into_iter().collect();
        self.events.clear();

        for (room_id, direction) in snapshot.unlocked_exits {
            if let Some(room) = self.rooms.get_mut(&room_id)
                && let Some(lock) = room.locked_exits.get_mut(&direction)
            {
                lock.unlocked = true;
            }
        }

        self.validate_loaded_state()
            .map_err(|msg| std::io::Error::new(std::io::ErrorKind::InvalidData, msg))?;
        self.restore_system_states(&snapshot.system_states);

        Ok(intro)
    }

    pub fn set_autosave(
        &mut self,
        enabled: bool,
        path: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(path) = path
            && !path.trim().is_empty()
        {
            self.autosave_path = path;
        }

        self.autosave_enabled = enabled;
        if enabled {
            self.save_game(&self.autosave_path)?;
            Ok(format!("Autosave enabled: {}", self.autosave_path))
        } else {
            Ok("Autosave disabled.".to_string())
        }
    }

    pub fn autosave_status(&self) -> String {
        if self.autosave_enabled {
            format!("Autosave is ON ({})", self.autosave_path)
        } else {
            format!("Autosave is OFF (next file: {})", self.autosave_path)
        }
    }

    /// Start the current adventure over from its opening state.
    pub fn restart(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.document.rooms.is_empty() {
            self.load_adventure()
        } else {
            let document = self.document.clone();
            self.load_document(document)
        }
    }

    pub fn get_current_room(&self) -> Option<&Room> {
        self.rooms.get(&self.player.current_room)
    }

    pub fn get_items_in_room(&self, room_id: i32) -> Vec<&Item> {
        self.items
            .values()
            .filter(|item| item.location == room_id)
            .collect()
    }

    pub fn get_monsters_in_room(&self, room_id: i32) -> Vec<&Monster> {
        self.monsters
            .values()
            .filter(|m| m.room_id == room_id && !m.is_dead)
            .collect()
    }

    pub fn player_has_light(&self) -> bool {
        self.player
            .inventory
            .iter()
            .filter_map(|id| self.items.get(id))
            .any(|item| item.is_light_source)
    }

    pub fn find_inventory_item(&self, query: &str) -> Option<i32> {
        let query = query.trim();
        if query.is_empty() {
            return None;
        }
        self.player.inventory.iter().copied().find(|id| {
            id.to_string() == query
                || self
                    .items
                    .get(id)
                    .is_some_and(|item| name_matches(&item.name, query))
        })
    }

    fn player_has_item_name(&self, name: &str) -> bool {
        self.player
            .inventory
            .iter()
            .filter_map(|id| self.items.get(id))
            .any(|item| name_matches(&item.name, name))
    }

    pub fn look(&self) -> String {
        let mut out = String::new();

        if let Some(room) = self.get_current_room() {
            let has_light = self.player_has_light();
            if room.is_dark && !has_light {
                return "It is pitch black. You can't see a thing.".to_string();
            }

            out.push('\n');
            out.push_str(&room.name);
            out.push('\n');
            out.push_str(&"-".repeat(room.name.len()));
            out.push('\n');
            out.push_str(&room.description);
            if room.is_dark && has_light {
                out.push_str("\nYour light pushes back the darkness.");
            }

            // Show exits
            if !room.exits.is_empty() {
                let mut exits: Vec<String> = room.exits.keys().cloned().collect();
                exits.sort();
                let described: Vec<String> = exits
                    .into_iter()
                    .map(|direction| {
                        if room
                            .locked_exits
                            .get(&direction)
                            .is_some_and(|lock| !lock.unlocked)
                        {
                            format!("{} (locked)", direction)
                        } else {
                            direction
                        }
                    })
                    .collect();
                out.push_str(&format!("\n\nObvious exits: {}", described.join(", ")));
            } else {
                out.push_str("\n\nNo obvious exits.");
            }
        } else {
            out.push_str("You are in a void.");
        }

        // Show items
        let items = self.get_items_in_room(self.player.current_room);
        if !items.is_empty() {
            out.push_str("\n\nYou see:");
            for item in items {
                out.push_str(&format!("\n  - {}", item.name));
            }
        }

        // Show monsters
        let monsters = self.get_monsters_in_room(self.player.current_room);
        if !monsters.is_empty() {
            out.push_str("\n\nPresent:");
            for monster in monsters {
                let status = match monster.friendliness {
                    MonsterStatus::Friendly => " (friendly)",
                    MonsterStatus::Hostile => " (hostile)",
                    MonsterStatus::Neutral => "",
                };
                out.push_str(&format!("\n  - {}{}", monster.name, status));
            }
        }

        out
    }

    pub fn move_player(&mut self, direction: &str) -> Option<String> {
        let direction = direction.to_lowercase();
        let room = self.get_current_room()?.clone();
        let new_room_id = room.get_exit(&direction)?;
        if !self.rooms.contains_key(&new_room_id) {
            return None;
        }

        let mut prefix = String::new();
        if let Some(lock) = room.locked_exits.get(&direction)
            && !lock.unlocked
        {
            match self.unlock_exit(&direction) {
                Ok(message) => prefix = format!("{}\n", message),
                Err(message) => return Some(message),
            }
        }

        self.last_room = self.player.current_room;
        self.player.current_room = new_room_id;
        self.turn_count += 1;
        self.visited_rooms.insert(new_room_id);
        self.events.push(GameEvent::RoomEntered {
            room_id: new_room_id,
        });

        let mut out = prefix;
        out.push_str(&self.look());
        if let Some(ending) = self.check_ending() {
            out.push('\n');
            out.push_str(&ending);
        }
        Some(out)
    }

    /// Unlock the exit in the current room. Returns a success message or the
    /// lock's refusal message.
    pub fn unlock_exit(&mut self, direction: &str) -> Result<String, String> {
        let direction = direction.to_lowercase();
        let room_id = self.player.current_room;
        let lock = self
            .rooms
            .get(&room_id)
            .and_then(|room| room.locked_exits.get(&direction))
            .cloned()
            .ok_or_else(|| format!("There is no locked exit to the {}.", direction))?;
        if lock.unlocked {
            return Err(format!("The {} exit is already unlocked.", direction));
        }

        let key_query = match &lock.key {
            Some(key) if !key.trim().is_empty() => key.clone(),
            _ => return Err(lock.message.clone()),
        };
        let Some(key_id) = self.find_inventory_item(&key_query) else {
            return Err(lock.message.clone());
        };
        let key_name = self
            .items
            .get(&key_id)
            .map(|item| item.name.clone())
            .unwrap_or(key_query);

        if let Some(room) = self.rooms.get_mut(&room_id)
            && let Some(lock) = room.locked_exits.get_mut(&direction)
        {
            lock.unlocked = true;
        }

        if lock.consume_key {
            self.player.inventory.retain(|id| *id != key_id);
            if self.player.equipped_weapon == Some(key_id) {
                self.player.equipped_weapon = None;
            }
            if self.player.equipped_armor == Some(key_id) {
                self.player.equipped_armor = None;
            }
            self.items.remove(&key_id);
        }

        self.events.push(GameEvent::ExitUnlocked {
            room_id,
            direction: direction.clone(),
        });

        Ok(if lock.consume_key {
            format!(
                "You use the {} to unlock the {} exit; the key crumbles away.",
                key_name, direction
            )
        } else {
            format!("You unlock the {} exit with the {}.", direction, key_name)
        })
    }

    /// Evaluate the adventure's victory conditions and return an ending message
    /// the first time they are satisfied.
    pub fn check_ending(&mut self) -> Option<String> {
        if self.game_won || self.game_over {
            return None;
        }
        let conditions = self.ending_conditions.clone()?;
        let has_condition = conditions.all_quests
            || !conditions.quests.is_empty()
            || !conditions.rooms.is_empty()
            || !conditions.items.is_empty();
        if !has_condition {
            return None;
        }

        let mut satisfied = true;
        if conditions.all_quests {
            satisfied &= !self.document.quests.is_empty()
                && self.completed_quests.len() >= self.document.quests.len();
        }
        if !conditions.quests.is_empty() {
            satisfied &= conditions
                .quests
                .iter()
                .all(|id| self.completed_quests.contains(&id.to_string()));
        }
        if !conditions.rooms.is_empty() {
            satisfied &= conditions
                .rooms
                .iter()
                .all(|id| self.visited_rooms.contains(id));
        }
        if !conditions.items.is_empty() {
            satisfied &= conditions
                .items
                .iter()
                .all(|name| self.player_has_item_name(name));
        }

        if !satisfied {
            return None;
        }

        self.game_won = true;
        let text = if self.ending_text.trim().is_empty() {
            "Victory! Your adventure is complete.".to_string()
        } else {
            self.ending_text.clone()
        };
        self.ending_text = text.clone();
        self.events.push(GameEvent::GameWon { text: text.clone() });
        Some(format!("*** {} ***", text))
    }

    pub fn take_item(&mut self, item_name: &str) -> Result<String, String> {
        const MAX_WEIGHT_PER_HARDINESS: i32 = 10;
        let max_carry = self.player.hardiness * MAX_WEIGHT_PER_HARDINESS;
        let current_weight: i32 = self
            .player
            .inventory
            .iter()
            .filter_map(|id| self.items.get(id))
            .map(|i| i.weight)
            .sum();

        let matched = self
            .get_items_in_room(self.player.current_room)
            .into_iter()
            .find(|i| name_matches(&i.name, item_name) && i.is_takeable)
            .map(|i| (i.id, i.name.clone(), i.weight));

        match matched {
            None => Err("You can't take that.".to_string()),
            Some((id, name, weight)) => {
                if current_weight + weight > max_carry {
                    return Err(format!(
                        "Too heavy to carry! ({}/{} weight used, {} weighs {}.)",
                        current_weight, max_carry, name, weight
                    ));
                }
                self.player.inventory.push(id);
                if let Some(item_ref) = self.items.get_mut(&id) {
                    item_ref.location = 0;
                }
                self.events.push(GameEvent::ItemCollected {
                    item_name: name.clone(),
                    item_id: id,
                });
                self.turn_count += 1;
                let mut message = format!("Taken: {}.", name);
                if let Some(ending) = self.check_ending() {
                    message.push('\n');
                    message.push_str(&ending);
                }
                Ok(message)
            }
        }
    }

    /// Drop an item from inventory onto the floor. Returns the item name on success, or `None`.
    pub fn drop_item(&mut self, item_name: &str) -> Option<String> {
        let matched = self.player.inventory.iter().copied().find_map(|id| {
            self.items
                .get(&id)
                .filter(|i| name_matches(&i.name, item_name))
                .map(|i| (id, i.name.clone()))
        });
        if let Some((item_id, name)) = matched {
            self.player.inventory.retain(|&id| id != item_id);
            if self.player.equipped_weapon == Some(item_id) {
                self.player.equipped_weapon = None;
            }
            if self.player.equipped_armor == Some(item_id) {
                self.player.equipped_armor = None;
            }
            if let Some(item_ref) = self.items.get_mut(&item_id) {
                item_ref.location = self.player.current_room;
            }
            self.turn_count += 1;
            Some(name)
        } else {
            None
        }
    }

    /// Equip a weapon or wearable armor from inventory.
    pub fn equip_item(&mut self, item_name: &str) -> Result<String, String> {
        let matched = self.player.inventory.iter().copied().find_map(|id| {
            self.items
                .get(&id)
                .filter(|i| {
                    name_matches(&i.name, item_name) && (i.is_weapon || i.is_wearable || i.is_armor)
                })
                .map(|i| (i.id, i.name.clone(), i.is_weapon))
        });
        match matched {
            None => Err(format!(
                "You don't have a weapon or armor called '{}'.",
                item_name
            )),
            Some((id, name, is_weapon)) => {
                if is_weapon {
                    self.player.equipped_weapon = Some(id);
                    Ok(format!("You wield the {}.", name))
                } else {
                    self.player.equipped_armor = Some(id);
                    Ok(format!("You wear the {}.", name))
                }
            }
        }
    }

    /// Unequip by slot name: "weapon" or "armor".
    pub fn unequip_slot(&mut self, slot: &str) -> Result<String, String> {
        match slot {
            "weapon" => {
                if self.player.equipped_weapon.take().is_some() {
                    Ok("Weapon unequipped.".to_string())
                } else {
                    Err("No weapon equipped.".to_string())
                }
            }
            "armor" => {
                if self.player.equipped_armor.take().is_some() {
                    Ok("Armor removed.".to_string())
                } else {
                    Err("No armor equipped.".to_string())
                }
            }
            _ => Err("Specify 'weapon' or 'armor'.".to_string()),
        }
    }

    /// Use a consumable or readable item from inventory.
    pub fn use_item(&mut self, item_name: &str) -> Result<String, String> {
        let matched = self.player.inventory.iter().copied().find_map(|id| {
            self.items
                .get(&id)
                .filter(|i| name_matches(&i.name, item_name))
                .map(|i| {
                    (
                        i.id,
                        i.name.clone(),
                        i.item_type.clone(),
                        i.description.clone(),
                        i.value,
                    )
                })
        });
        match matched {
            None => Err(format!("You don't have '{}'.", item_name)),
            Some((id, name, item_type, description, value)) => {
                let msg = match item_type {
                    ItemType::Edible | ItemType::Drinkable => {
                        let heal = value.clamp(1, 20);
                        let after = (self.player.current_health + heal).min(self.player.hardiness);
                        self.player.current_health = after;
                        self.player.inventory.retain(|&i| i != id);
                        // Remove consumed item from the world entirely
                        self.items.remove(&id);
                        self.events.push(GameEvent::ItemUsed {
                            item_name: name.clone(),
                        });
                        self.turn_count += 1;
                        format!(
                            "You consume the {}. Health: {}/{}.",
                            name, after, self.player.hardiness
                        )
                    }
                    ItemType::Readable => {
                        format!("You read the {}:\n{}", name, description)
                    }
                    _ => {
                        format!("You fiddle with the {} but nothing happens.", name)
                    }
                };
                Ok(msg)
            }
        }
    }

    /// Return details about an item in inventory or current room.
    pub fn examine_item(&self, item_name: &str) -> Option<String> {
        let in_inventory = self.player.inventory.iter().copied().find_map(|id| {
            self.items
                .get(&id)
                .filter(|i| name_matches(&i.name, item_name))
        });
        let in_room = self
            .get_items_in_room(self.player.current_room)
            .into_iter()
            .find(|i| name_matches(&i.name, item_name));
        let item = in_inventory.or(in_room)?;

        let mut msg = format!("{}\n{}", item.name, item.description);
        if item.is_weapon {
            msg.push_str(&format!(
                "\nDamage: {}d{}",
                item.weapon_dice, item.weapon_sides
            ));
        }
        if item.is_armor {
            msg.push_str(&format!("\nArmor value: {}", item.armor_value));
        }
        msg.push_str(&format!(
            "\nWeight: {}  Value: {} gold",
            item.weight, item.value
        ));
        Some(msg)
    }

    /// (current carried weight, max carry weight)
    pub fn carry_weight(&self) -> (i32, i32) {
        let current: i32 = self
            .player
            .inventory
            .iter()
            .filter_map(|id| self.items.get(id))
            .map(|i| i.weight)
            .sum();
        (current, self.player.hardiness * 10)
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn process_command(&mut self, command: &str) -> Vec<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        // Lowercase the verb so "Look", "ATTACK", etc. work regardless of caller.
        let cmd_lower = parts.first().unwrap_or(&"").to_lowercase();
        let cmd: &str = &cmd_lower;
        let args: Vec<&str> = parts.iter().skip(1).cloned().collect();

        if (self.game_over || self.game_won)
            && !matches!(cmd, "restart" | "help" | "?" | "status" | "stats" | "score")
        {
            return vec![if self.game_won {
                "The adventure is complete. Use 'restart' to play again.".to_string()
            } else {
                "You have been slain. Use 'restart' to try again.".to_string()
            }];
        }

        let turn_count_before = self.turn_count;
        let mut systems = std::mem::take(&mut self.systems);
        let mut results: Vec<String> = Vec::new();

        // Primary handler: first system that claims the command.
        for system in &mut systems {
            if let Some(output) = system.on_command(cmd, &args, self) {
                results.push(output);
                break;
            }
        }

        // Observer pass: systems react to pending game events via on_events().
        if !self.events.is_empty() {
            let events = std::mem::take(&mut self.events);
            for system in &mut systems {
                if let Some(side) = system.on_events(&events, self) {
                    results.push(side);
                }
            }
            // events is dropped here; self.events is already empty from the take()
        }

        self.systems = systems;

        if self.autosave_enabled
            && self.turn_count > turn_count_before
            && let Err(err) = self.save_game(&self.autosave_path)
        {
            results.push(format!("Warning: autosave failed: {}", err));
        }

        if results.is_empty() {
            vec![format!("Unknown command: {}", command)]
        } else {
            results
        }
    }
}

impl Default for AdventureGame {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_adventure(json: &str) -> PathBuf {
        let file_name = format!(
            "mellivorex_test_{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(file_name);
        fs::write(&path, json).expect("temp adventure should be writable");
        path
    }

    #[test]
    fn load_adventure_rejects_missing_start_room() {
        let json = r#"{
            "title": "Broken Start",
            "intro": "",
            "start_room": 99,
            "rooms": [
                {"id": 1, "name": "Village", "description": "Start", "exits": {}}
            ],
            "items": [],
            "monsters": [],
            "quests": []
        }"#;
        let path = write_temp_adventure(json);
        let mut game = AdventureGame::new(path.to_string_lossy().to_string());

        let err = game.load_adventure().expect_err("load should fail");
        assert!(err.to_string().contains("start_room"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_adventure_rejects_exit_to_unknown_room() {
        let json = r#"{
            "title": "Broken Exit",
            "intro": "",
            "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Village", "description": "Start", "exits": {"north": 2}}
            ],
            "items": [],
            "monsters": [],
            "quests": []
        }"#;
        let path = write_temp_adventure(json);
        let mut game = AdventureGame::new(path.to_string_lossy().to_string());

        let err = game.load_adventure().expect_err("load should fail");
        assert!(err.to_string().contains("unknown room"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_adventure_supports_string_id_demo_schema() {
        let json = r#"{
            "id": "demo",
            "title": "Demo Adventure",
            "start_room": "village",
            "rooms": [
                {
                    "id": "village",
                    "title": "Quiet Village",
                    "description": "Lanterns glow softly.",
                    "exits": {"north": "forest"},
                    "items": [
                        {"id": "key", "name": "Ancient Key", "description": "A humming key."}
                    ]
                },
                {
                    "id": "forest",
                    "title": "Whispering Forest",
                    "description": "The trees murmur.",
                    "exits": {"south": "village"},
                    "items": []
                }
            ],
            "player_start_inventory": []
        }"#;

        let path = write_temp_adventure(json);
        let mut game = AdventureGame::new(path.to_string_lossy().to_string());

        let intro = game.load_adventure().expect("string-id demo should load");
        assert!(intro.contains("Demo Adventure"));
        assert_eq!(game.rooms.len(), 2);
        assert_eq!(game.player.current_room, 1);
        assert!(game.look().contains("Quiet Village"));
        assert!(game.look().contains("Ancient Key"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn autosave_command_creates_snapshot_after_progress() {
        let json = r#"{
            "title": "Autosave Demo",
            "intro": "",
            "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Village", "description": "Start", "exits": {"north": 2}},
                {"id": 2, "name": "Forest", "description": "Ahead", "exits": {"south": 1}}
            ],
            "items": [],
            "monsters": [],
            "quests": []
        }"#;

        let adventure_path = write_temp_adventure(json);
        let save_path = write_temp_adventure("{}");
        let _ = fs::remove_file(&save_path);

        let mut game = AdventureGame::new(adventure_path.to_string_lossy().to_string());
        game.add_system(Box::new(crate::systems::BasicWorldSystem));
        game.load_adventure().expect("adventure should load");

        let enable_output = game
            .process_command(&format!("autosave on {}", save_path.to_string_lossy()))
            .join("\n");
        assert!(enable_output.to_lowercase().contains("autosave enabled"));

        game.process_command("north");
        assert!(save_path.exists());

        let mut restored = AdventureGame::new("placeholder.json".to_string());
        restored.add_system(Box::new(crate::systems::BasicWorldSystem));
        restored
            .load_game(&save_path)
            .expect("autosave should load");
        assert_eq!(restored.player.current_room, 2);

        let _ = fs::remove_file(adventure_path);
        let _ = fs::remove_file(save_path);
    }

    #[test]
    fn talk_to_friendly_npc_returns_conversation() {
        let mut game = AdventureGame::new("test.json".to_string());
        game.rooms.insert(
            1,
            Room::new(1, "Village Square".to_string(), "A calm plaza.".to_string()),
        );
        game.player.current_room = 1;
        game.monsters.insert(
            1,
            Monster::new(
                1,
                "Old Hermit".to_string(),
                "A weathered guide who knows these lands well.".to_string(),
                1,
                8,
                8,
                MonsterStatus::Friendly,
                50,
            ),
        );
        game.add_system(Box::new(crate::systems::BasicWorldSystem));

        let output = game.process_command("talk hermit").join("\n");
        assert!(output.contains("Old Hermit"));
        assert!(!output.contains("Unknown command"));
    }

    #[test]
    fn ask_npc_about_topic_mentions_that_topic() {
        let mut game = AdventureGame::new("test.json".to_string());
        game.rooms.insert(
            1,
            Room::new(1, "Village Square".to_string(), "A calm plaza.".to_string()),
        );
        game.player.current_room = 1;
        game.monsters.insert(
            1,
            Monster::new(
                1,
                "Town Guard".to_string(),
                "A veteran watchman keeping a close eye on the gate.".to_string(),
                1,
                10,
                10,
                MonsterStatus::Friendly,
                60,
            ),
        );
        game.add_system(Box::new(crate::systems::BasicWorldSystem));

        let output = game.process_command("ask guard about gate").join("\n");
        assert!(output.contains("Town Guard"));
        assert!(output.to_lowercase().contains("gate"));
        assert!(!output.contains("Unknown command"));
    }

    #[test]
    fn save_and_load_round_trip_preserves_progress() {
        let json = r#"{
            "title": "Save Demo",
            "intro": "Welcome back",
            "start_room": 1,
            "rooms": [
                {"id": 1, "name": "Village", "description": "Start", "exits": {"north": 2}},
                {"id": 2, "name": "Forest", "description": "Trees everywhere", "exits": {"south": 1}}
            ],
            "items": [
                {"id": 10, "name": "Apple", "description": "Fresh fruit", "type": "edible", "value": 5, "location": 1}
            ],
            "monsters": [],
            "quests": [
                {"id": 1, "title": "Scout Ahead", "description": "Reach the forest", "objectives": ["Enter Forest"], "rewards_gold": 5, "rewards_xp": 10}
            ]
        }"#;

        let adventure_path = write_temp_adventure(json);
        let save_path = write_temp_adventure("{}");

        let mut game = AdventureGame::new(adventure_path.to_string_lossy().to_string());
        game.add_system(Box::new(crate::systems::BasicWorldSystem));
        game.add_system(Box::new(crate::systems::QuestSystem::new()));
        game.load_adventure().expect("adventure should load");

        let accept_output = game.process_command("accept 1").join("\n");
        assert!(accept_output.contains("Accepted quest"));
        game.take_item("Apple").expect("item should be collectible");
        game.process_command("north");
        game.player.gold = 321;

        game.save_game(&save_path).expect("save should succeed");

        let mut restored = AdventureGame::new("placeholder.json".to_string());
        restored.add_system(Box::new(crate::systems::BasicWorldSystem));
        restored.add_system(Box::new(crate::systems::QuestSystem::new()));
        restored.load_game(&save_path).expect("load should succeed");

        assert_eq!(restored.player.current_room, 2);
        assert_eq!(restored.player.gold, 321);
        assert!(restored.player.inventory.contains(&10));

        let quest_output = restored.process_command("quests").join("\n");
        assert!(quest_output.contains("Scout Ahead"));
        assert!(quest_output.contains("Active Quests"));

        let complete_output = restored.process_command("complete 1").join("\n");
        assert!(complete_output.contains("Completed quest: Scout Ahead (1)"));

        let _ = fs::remove_file(adventure_path);
        let _ = fs::remove_file(save_path);
    }
}
