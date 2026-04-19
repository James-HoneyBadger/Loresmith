use crate::adventure::Adventure as AuthoringAdventure;
use crate::systems::System;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    pub location: i32, // 0=inventory, -1=worn, room_id or monster_id
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
            location: 0,
        }
    }

    pub fn get_damage(&self) -> i32 {
        if !self.is_weapon {
            return 0;
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..self.weapon_dice)
            .map(|_| rng.gen_range(1..=self.weapon_sides))
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, i32>, // direction -> room_id
    pub is_dark: bool,
}

impl Room {
    pub fn new(id: i32, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            exits: HashMap::new(),
            is_dark: false,
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
    MonsterKilled { monster_name: String, room_id: i32 },
    ItemCollected { item_name: String, item_id: i32 },
    RoomEntered { room_id: i32 },
    ItemUsed { item_name: String },
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
    pub quests: Vec<serde_json::Value>, // Quest definitions
    pub events: Vec<GameEvent>,         // Inter-system event bus
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
        }
    }

    fn reset_runtime_state(&mut self) {
        self.rooms.clear();
        self.items.clear();
        self.monsters.clear();
        self.quests.clear();
        self.events.clear();
        self.turn_count = 0;
        self.game_over = false;
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

    fn load_string_id_adventure(
        &mut self,
        data: &serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let adventure: AuthoringAdventure = serde_json::from_value(data.clone())?;
        adventure.validate()?;

        self.reset_runtime_state();
        self.adventure_title = adventure.title.clone();
        self.adventure_intro = data
            .get("intro")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let room_id_map: HashMap<String, i32> = adventure
            .rooms
            .iter()
            .enumerate()
            .map(|(index, room)| (room.id.clone(), index as i32 + 1))
            .collect();

        let mut next_item_id = 1;
        for room in &adventure.rooms {
            let numeric_room_id = *room_id_map.get(&room.id).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("room id mapping missing for '{}'", room.id),
                )
            })?;

            let exits = room
                .exits
                .iter()
                .map(|(dir, dest)| {
                    room_id_map
                        .get(dest)
                        .copied()
                        .map(|dest_id| (dir.clone(), dest_id))
                })
                .collect::<Option<HashMap<_, _>>>()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("room '{}' has an exit to an unknown room", room.id),
                    )
                })?;

            self.rooms.insert(
                numeric_room_id,
                Room {
                    id: numeric_room_id,
                    name: if room.title.trim().is_empty() {
                        room.id.clone()
                    } else {
                        room.title.clone()
                    },
                    description: room.description.clone(),
                    exits,
                    is_dark: false,
                },
            );

            for item in &room.items {
                let mut world_item = Item::new(
                    next_item_id,
                    item.name.clone(),
                    item.description.clone(),
                    ItemType::Normal,
                    1,
                    0,
                );
                world_item.location = numeric_room_id;
                self.items.insert(next_item_id, world_item);
                next_item_id += 1;
            }
        }

        for item in &adventure.player_start_inventory {
            let mut world_item = Item::new(
                next_item_id,
                item.name.clone(),
                item.description.clone(),
                ItemType::Normal,
                1,
                0,
            );
            world_item.location = 0;
            self.player.inventory.push(next_item_id);
            self.items.insert(next_item_id, world_item);
            next_item_id += 1;
        }

        self.player.current_room = *room_id_map.get(&adventure.start_room).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("start_room does not exist: {}", adventure.start_room),
            )
        })?;
        self.validate_loaded_state()
            .map_err(|msg| std::io::Error::new(std::io::ErrorKind::InvalidData, msg))?;

        Ok(self.build_header())
    }

    pub fn load_adventure(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let data: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&self.adventure_file)?)?;

        let uses_string_ids = data.get("start_room").is_some_and(|v| v.is_string())
            || data
                .get("rooms")
                .and_then(|v| v.as_array())
                .and_then(|rooms| rooms.first())
                .and_then(|room| room.get("id"))
                .is_some_and(|id| id.is_string());

        if uses_string_ids {
            return self.load_string_id_adventure(&data);
        }

        self.reset_runtime_state();

        self.adventure_title = data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Adventure")
            .to_string();
        self.adventure_intro = data
            .get("intro")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut room_item_locations: Vec<(i32, Vec<i32>)> = Vec::new();

        // Load rooms
        if let Some(rooms) = data.get("rooms").and_then(|v| v.as_array()) {
            for room_data in rooms {
                let room_id = room_data.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let listed_items: Vec<i32> = room_data
                    .get("items")
                    .and_then(|v| v.as_array())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_i64().map(|id| id as i32))
                            .collect()
                    })
                    .unwrap_or_default();

                let room = Room {
                    id: room_id,
                    name: room_data
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: room_data
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    exits: room_data
                        .get("exits")
                        .and_then(|v| v.as_object())
                        .map(|obj| {
                            obj.iter()
                                .map(|(k, v)| (k.clone(), v.as_i64().unwrap_or(0) as i32))
                                .collect()
                        })
                        .unwrap_or_default(),
                    is_dark: room_data
                        .get("is_dark")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                };
                if self.rooms.insert(room.id, room).is_some() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "duplicate room id found while loading: {}",
                            room_data.get("id").and_then(|v| v.as_i64()).unwrap_or(0)
                        ),
                    )
                    .into());
                }

                if !listed_items.is_empty() {
                    room_item_locations.push((room_id, listed_items));
                }
            }
        }

        // Load items
        if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
            for item_data in items {
                let item = Item {
                    id: item_data.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    name: item_data
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: item_data
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    item_type: match item_data.get("type").and_then(|v| v.as_str()) {
                        Some("weapon") => ItemType::Weapon,
                        Some("armor") => ItemType::Armor,
                        Some("treasure") => ItemType::Treasure,
                        Some("readable") => ItemType::Readable,
                        Some("edible") => ItemType::Edible,
                        Some("drinkable") => ItemType::Drinkable,
                        Some("container") => ItemType::Container,
                        _ => ItemType::Normal,
                    },
                    weight: item_data
                        .get("weight")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1) as i32,
                    value: item_data.get("value").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    is_weapon: item_data
                        .get("is_weapon")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    weapon_type: item_data
                        .get("weapon_type")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    weapon_dice: item_data
                        .get("weapon_dice")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1) as i32,
                    weapon_sides: item_data
                        .get("weapon_sides")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(6) as i32,
                    is_armor: item_data
                        .get("is_armor")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    armor_value: item_data
                        .get("armor_value")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    is_takeable: item_data
                        .get("is_takeable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    is_wearable: item_data
                        .get("is_wearable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    location: item_data
                        .get("location")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                };
                if self.items.insert(item.id, item).is_some() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "duplicate item id found while loading: {}",
                            item_data.get("id").and_then(|v| v.as_i64()).unwrap_or(0)
                        ),
                    )
                    .into());
                }
            }
        }

        for (room_id, item_ids) in &room_item_locations {
            for item_id in item_ids {
                if let Some(item) = self.items.get_mut(item_id) {
                    item.location = *room_id;
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("room {} references unknown item {}", room_id, item_id),
                    )
                    .into());
                }
            }
        }

        // Load monsters
        if let Some(monsters) = data.get("monsters").and_then(|v| v.as_array()) {
            for mon_data in monsters {
                let friendliness = match mon_data.get("friendliness").and_then(|v| v.as_str()) {
                    Some("friendly") => MonsterStatus::Friendly,
                    Some("hostile") => MonsterStatus::Hostile,
                    _ => MonsterStatus::Neutral,
                };
                let monster = Monster::new(
                    mon_data.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    mon_data
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    mon_data
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    mon_data
                        .get("room_id")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1) as i32,
                    mon_data
                        .get("hardiness")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(10) as i32,
                    mon_data
                        .get("agility")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(10) as i32,
                    friendliness,
                    mon_data
                        .get("courage")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(100) as i32,
                );
                // Set additional fields
                let mut monster = monster;
                monster.weapon_id = mon_data
                    .get("weapon_id")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
                monster.armor_worn = mon_data
                    .get("armor_worn")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                monster.gold = mon_data.get("gold").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                if self.monsters.insert(monster.id, monster).is_some() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "duplicate monster id found while loading: {}",
                            mon_data.get("id").and_then(|v| v.as_i64()).unwrap_or(0)
                        ),
                    )
                    .into());
                }
            }
        }

        // Load quests
        if let Some(quests) = data.get("quests").and_then(|v| v.as_array()) {
            self.quests = quests.clone();
        }

        // Set player starting position
        self.player.current_room =
            data.get("start_room").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        self.validate_loaded_state()
            .map_err(|msg| std::io::Error::new(std::io::ErrorKind::InvalidData, msg))?;

        Ok(self.build_header())
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
            let valid_location = matches!(item.location, -1 | 0)
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
        let snapshot = GameSnapshot {
            version: 1,
            adventure_file: self.adventure_file.clone(),
            player: self.player.clone(),
            items: self.items.clone(),
            monsters: self.monsters.clone(),
            turn_count: self.turn_count,
            game_over: self.game_over,
            autosave_enabled: self.autosave_enabled,
            autosave_path: self.autosave_path.clone(),
            system_states: self.collect_system_states(),
        };

        let json = serde_json::to_string_pretty(&snapshot)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_game(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let snapshot: GameSnapshot = serde_json::from_str(&fs::read_to_string(path)?)?;
        self.adventure_file = snapshot.adventure_file.clone();
        let intro = self.load_adventure()?;

        self.player = snapshot.player;
        self.items = snapshot.items;
        self.monsters = snapshot.monsters;
        self.turn_count = snapshot.turn_count;
        self.game_over = snapshot.game_over;
        self.autosave_enabled = snapshot.autosave_enabled;
        self.autosave_path = snapshot.autosave_path;
        self.events.clear();

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

    pub fn look(&self) -> String {
        let mut out = String::new();

        if let Some(room) = self.get_current_room() {
            if room.is_dark {
                return "It is pitch black. You can't see a thing.".to_string();
            }

            out.push('\n');
            out.push_str(&room.name);
            out.push('\n');
            out.push_str(&"-".repeat(room.name.len()));
            out.push('\n');
            out.push_str(&room.description);

            // Show exits
            if !room.exits.is_empty() {
                let mut exits: Vec<String> = room.exits.keys().cloned().collect();
                exits.sort();
                out.push_str(&format!("\n\nObvious exits: {}", exits.join(", ")));
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
        if let Some(room) = self.get_current_room()
            && let Some(new_room_id) = room.get_exit(direction)
            && self.rooms.contains_key(&new_room_id)
        {
            self.player.current_room = new_room_id;
            self.turn_count += 1;
            self.events.push(GameEvent::RoomEntered {
                room_id: new_room_id,
            });
            return Some(self.look());
        }
        None
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
                Ok(format!("Taken: {}.", name))
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
            "sagacraft_test_{}.json",
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
        game.move_player("north").expect("north exit should exist");
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
        assert!(complete_output.contains("Completed quest: 1"));

        let _ = fs::remove_file(adventure_path);
        let _ = fs::remove_file(save_path);
    }
}
