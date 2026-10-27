//! Canonical, versioned adventure authoring format.
//!
//! This module is the single source of truth for adventure documents. It can
//! read both historical formats:
//!
//! * the integer-ID format consumed by `AdventureGame` since 4.0.0, and
//! * the string-ID format used by the TUI (`crate::adventure::Adventure`).
//!
//! Both are normalised into [`AdventureDocument`] and can be written back as
//! the canonical integer-ID schema (`schema_version: 2`). Unknown fields are
//! preserved in `extra` maps so editor metadata survives a round-trip.

use crate::adventure::{Adventure, AdventureError};
use crate::game_state::{ItemType, MonsterStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

pub const SCHEMA_VERSION: u32 = 2;

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}
fn default_one() -> i32 {
    1
}
fn default_six() -> i32 {
    6
}
fn default_ten() -> i32 {
    10
}
fn default_hundred() -> i32 {
    100
}
fn default_true() -> bool {
    true
}
fn default_buy_multiplier() -> f32 {
    1.0
}
fn default_sell_multiplier() -> f32 {
    0.5
}
fn default_item_type() -> ItemType {
    ItemType::Normal
}
fn default_monster_status() -> MonsterStatus {
    MonsterStatus::Neutral
}

/// Editor metadata that the engine preserves but does not interpret.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AdventureSettings {
    #[serde(default)]
    pub allow_save: bool,
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub enable_magic: bool,
    #[serde(default)]
    pub enable_puzzles: bool,
    #[serde(default)]
    pub enable_combat_xp: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// An exit destination. The object form adds an optional lock.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ExitDef {
    Plain(i32),
    Locked(LockedExitDef),
}

impl ExitDef {
    pub fn destination(&self) -> i32 {
        match self {
            ExitDef::Plain(id) => *id,
            ExitDef::Locked(lock) => lock.to,
        }
    }

    pub fn lock(&self) -> Option<&LockedExitDef> {
        match self {
            ExitDef::Plain(_) => None,
            ExitDef::Locked(lock) => Some(lock),
        }
    }

    pub fn lock_mut(&mut self) -> Option<&mut LockedExitDef> {
        match self {
            ExitDef::Plain(_) => None,
            ExitDef::Locked(lock) => Some(lock),
        }
    }

    pub fn destination_mut(&mut self) -> &mut i32 {
        match self {
            ExitDef::Plain(id) => id,
            ExitDef::Locked(lock) => &mut lock.to,
        }
    }

    pub fn set_locked(&mut self, locked: bool, key: Option<String>) {
        match self {
            ExitDef::Plain(id) => {
                if locked {
                    let mut new_lock = LockedExitDef::new(*id);
                    new_lock.key = key;
                    *self = ExitDef::Locked(new_lock);
                }
            }
            ExitDef::Locked(lock) => {
                lock.locked = locked;
                lock.key = key;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockedExitDef {
    pub to: i32,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub consume_key: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl LockedExitDef {
    pub fn new(to: i32) -> Self {
        Self {
            to,
            locked: true,
            key: None,
            consume_key: false,
            message: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RoomDef {
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_dark: bool,
    #[serde(default)]
    pub exits: BTreeMap<String, ExitDef>,
    #[serde(default)]
    pub items: Vec<i32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemDef {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type", default = "default_item_type")]
    pub item_type: ItemType,
    #[serde(default = "default_one")]
    pub weight: i32,
    #[serde(default)]
    pub value: i32,
    #[serde(default)]
    pub is_weapon: bool,
    #[serde(default)]
    pub weapon_type: i32,
    #[serde(default = "default_one")]
    pub weapon_dice: i32,
    #[serde(default = "default_six")]
    pub weapon_sides: i32,
    #[serde(default)]
    pub is_armor: bool,
    #[serde(default)]
    pub armor_value: i32,
    #[serde(default = "default_true")]
    pub is_takeable: bool,
    #[serde(default)]
    pub is_wearable: bool,
    /// Whether carrying this item lets the player see in dark rooms.
    #[serde(default)]
    pub is_light_source: bool,
    /// Number of item IDs a container can hold (reserved for future use).
    #[serde(default)]
    pub container_capacity: i32,
    /// `0` = carried when listed in starting inventory / picked up,
    /// `-1` = worn, otherwise a room or monster ID.
    #[serde(default)]
    pub location: i32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for ItemDef {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            item_type: ItemType::Normal,
            weight: 1,
            value: 0,
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
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DialogueDef {
    #[serde(default)]
    pub greeting: String,
    #[serde(default)]
    pub topics: BTreeMap<String, String>,
    #[serde(default)]
    pub unknown: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonsterDef {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_one")]
    pub room_id: i32,
    #[serde(default = "default_ten")]
    pub hardiness: i32,
    #[serde(default = "default_ten")]
    pub agility: i32,
    #[serde(default = "default_monster_status")]
    pub friendliness: MonsterStatus,
    #[serde(default = "default_hundred")]
    pub courage: i32,
    #[serde(default)]
    pub weapon_id: Option<i32>,
    #[serde(default)]
    pub armor_worn: i32,
    #[serde(default)]
    pub gold: i32,
    #[serde(default)]
    pub dialogue: Option<DialogueDef>,
    #[serde(default)]
    pub loot: Vec<i32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for MonsterDef {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: String::new(),
            room_id: 1,
            hardiness: 10,
            agility: 10,
            friendliness: MonsterStatus::Neutral,
            courage: 100,
            weapon_id: None,
            armor_worn: 0,
            gold: 0,
            dialogue: None,
            loot: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveKind {
    KillMonster,
    CollectItem,
    ReachRoom,
    TalkToNpc,
    Defend,
    Deliver,
    Discover,
    Puzzle,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredObjective {
    #[serde(rename = "type", default)]
    pub kind: ObjectiveKind,
    #[serde(default)]
    pub target_id: Option<Value>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_one")]
    pub count: i32,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for StructuredObjective {
    fn default() -> Self {
        Self {
            kind: ObjectiveKind::Other,
            target_id: None,
            target: None,
            description: String::new(),
            count: 1,
            topic: None,
            extra: BTreeMap::new(),
        }
    }
}

impl StructuredObjective {
    pub fn target_text(&self) -> String {
        if let Some(value) = &self.target_id {
            match value {
                Value::String(s) => return s.clone(),
                Value::Number(n) => return n.to_string(),
                Value::Bool(b) => return b.to_string(),
                _ => {}
            }
        }
        self.target.clone().unwrap_or_default()
    }

    pub fn target_room_id(&self) -> Option<i32> {
        match &self.target_id {
            Some(Value::Number(n)) => n.as_i64().and_then(|n| i32::try_from(n).ok()),
            Some(Value::String(s)) => s.parse::<i32>().ok(),
            _ => self.target.as_deref().and_then(|s| s.parse::<i32>().ok()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ObjectiveDef {
    Text(String),
    Structured(StructuredObjective),
}

impl ObjectiveDef {
    pub fn structured(&self, index: usize) -> StructuredObjective {
        match self {
            ObjectiveDef::Text(text) => parse_text_objective(index, text),
            ObjectiveDef::Structured(obj) => obj.clone(),
        }
    }
}

fn normalize_words(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Turn a free-form objective line into a structured objective.
///
/// This mirrors the heuristics used by the quest system so that GUI-authored
/// strings keep working after normalisation to the canonical format.
pub fn parse_text_objective(index: usize, text: &str) -> StructuredObjective {
    let description = text.trim().to_string();
    let normalized = normalize_words(&description);

    let (kind, target) = if let Some(rest) = normalized
        .strip_prefix("pick up ")
        .or_else(|| normalized.strip_prefix("collect "))
        .or_else(|| normalized.strip_prefix("take "))
    {
        (ObjectiveKind::CollectItem, rest.trim().to_string())
    } else if let Some(rest) = normalized
        .strip_prefix("talk to ")
        .or_else(|| normalized.strip_prefix("speak to "))
    {
        (ObjectiveKind::TalkToNpc, rest.trim().to_string())
    } else if let Some(rest) = normalized
        .strip_prefix("enter ")
        .or_else(|| normalized.strip_prefix("reach "))
        .or_else(|| normalized.strip_prefix("visit "))
    {
        (ObjectiveKind::ReachRoom, rest.trim().to_string())
    } else {
        (ObjectiveKind::Discover, normalized.clone())
    };

    StructuredObjective {
        kind,
        target_id: Some(Value::String(if target.is_empty() {
            normalized.clone()
        } else {
            target.clone()
        })),
        target: Some(if target.is_empty() {
            normalized
        } else {
            target
        }),
        description: if description.is_empty() {
            format!("Objective {}", index + 1)
        } else {
            description
        },
        count: 1,
        topic: None,
        extra: BTreeMap::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StageDef {
    #[serde(default)]
    pub stage_id: String,
    #[serde(default = "default_one")]
    pub stage_number: i32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub objectives: Vec<ObjectiveDef>,
    #[serde(default)]
    pub stage_reward_xp: i32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum QuestDifficultyDef {
    Easy,
    #[default]
    Moderate,
    Challenging,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RewardItem {
    Id(i32),
    Name(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QuestRewardsDef {
    #[serde(default)]
    pub gold: i32,
    #[serde(default, alias = "experience_points")]
    pub xp: i32,
    #[serde(default)]
    pub items: Vec<RewardItem>,
    #[serde(default)]
    pub reputation: BTreeMap<String, i32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestDef {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub giver_npc: String,
    #[serde(default = "default_one")]
    pub quest_giver_level: i32,
    #[serde(default)]
    pub difficulty: QuestDifficultyDef,
    #[serde(default)]
    pub objectives: Vec<ObjectiveDef>,
    #[serde(default)]
    pub stages: Vec<StageDef>,
    #[serde(default)]
    pub rewards: QuestRewardsDef,
    #[serde(default)]
    pub rewards_gold: Option<i32>,
    #[serde(default)]
    pub rewards_xp: Option<i32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for QuestDef {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            description: String::new(),
            giver_npc: String::new(),
            quest_giver_level: 1,
            difficulty: QuestDifficultyDef::Moderate,
            objectives: Vec::new(),
            stages: Vec::new(),
            rewards: QuestRewardsDef::default(),
            rewards_gold: None,
            rewards_xp: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ShopStockDef {
    #[serde(default)]
    pub item_id: i32,
    #[serde(default = "default_one")]
    pub quantity: i32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShopDef {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_one")]
    pub room_id: i32,
    #[serde(default)]
    pub gold: i32,
    #[serde(default)]
    pub stock: Vec<ShopStockDef>,
    #[serde(default = "default_buy_multiplier")]
    pub buy_multiplier: f32,
    #[serde(default = "default_sell_multiplier")]
    pub sell_multiplier: f32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for ShopDef {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            room_id: 1,
            gold: 0,
            stock: Vec::new(),
            buy_multiplier: default_buy_multiplier(),
            sell_multiplier: default_sell_multiplier(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EndingConditions {
    #[serde(default)]
    pub all_quests: bool,
    #[serde(default)]
    pub quests: Vec<i32>,
    #[serde(default)]
    pub rooms: Vec<i32>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EndingDef {
    #[serde(default)]
    pub victory_text: String,
    #[serde(default)]
    pub defeat_text: Option<String>,
    #[serde(default)]
    pub conditions: EndingConditions,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum InventoryRef {
    Id(i32),
    Item(ItemDef),
}

/// Canonical adventure document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdventureDocument {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub settings: AdventureSettings,
    #[serde(default)]
    pub start_room: i32,
    #[serde(default)]
    pub rooms: Vec<RoomDef>,
    #[serde(default)]
    pub items: Vec<ItemDef>,
    #[serde(default)]
    pub monsters: Vec<MonsterDef>,
    #[serde(default)]
    pub quests: Vec<QuestDef>,
    #[serde(default)]
    pub shops: Vec<ShopDef>,
    #[serde(default)]
    pub ending: Option<EndingDef>,
    #[serde(default)]
    pub player_start_inventory: Vec<InventoryRef>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for AdventureDocument {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: String::new(),
            title: String::new(),
            description: String::new(),
            intro: String::new(),
            author: None,
            settings: AdventureSettings::default(),
            start_room: 0,
            rooms: Vec::new(),
            items: Vec::new(),
            monsters: Vec::new(),
            quests: Vec::new(),
            shops: Vec::new(),
            ending: None,
            player_start_inventory: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

/// Structured validation output for editors.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl AdventureDocument {
    /// Load and validate a canonical document from disk.
    pub fn load_json_file(path: impl AsRef<Path>) -> Result<Self, AdventureError> {
        let text = fs::read_to_string(path)?;
        let doc = Self::from_json_str(&text)?;
        doc.validate()?;
        Ok(doc)
    }

    pub fn from_json_str(text: &str) -> Result<Self, AdventureError> {
        let value: Value = serde_json::from_str(text)?;
        Self::from_value(value)
    }

    pub fn from_value(value: Value) -> Result<Self, AdventureError> {
        let is_string_schema = value.get("start_room").is_some_and(|v| v.is_string())
            || value
                .get("rooms")
                .and_then(|v| v.as_array())
                .and_then(|rooms| rooms.first())
                .and_then(|room| room.get("id"))
                .is_some_and(|id| id.is_string());

        let mut doc = if is_string_schema {
            let adventure: Adventure = serde_json::from_value(value.clone())?;
            adventure.validate()?;
            Self::from_legacy_string(&adventure, &value)
        } else {
            serde_json::from_value::<Self>(value)?
        };
        doc.normalize();
        Ok(doc)
    }

    pub fn to_json_string(&self) -> Result<String, AdventureError> {
        let normalized = self.clone().normalized();
        Ok(serde_json::to_string_pretty(&normalized)?)
    }

    pub fn to_json_value(&self) -> Result<Value, AdventureError> {
        let normalized = self.clone().normalized();
        Ok(serde_json::to_value(normalized)?)
    }

    /// Save atomically so a crash cannot leave a half-written adventure file.
    pub fn save_json_file(&self, path: impl AsRef<Path>) -> Result<(), AdventureError> {
        self.validate()?;
        let text = self.to_json_string()?;
        crate::util::write_atomic(path, text.as_bytes())?;
        Ok(())
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    fn normalize(&mut self) {
        self.schema_version = SCHEMA_VERSION;
        if self.id.trim().is_empty() {
            self.id = slugify(&self.title);
        }
        if self.title.trim().is_empty() {
            self.title = "Untitled Adventure".to_string();
        }
        if self.description.trim().is_empty() {
            self.description = self.intro.clone();
        }

        // Normalise exit keys so lookups are consistent regardless of author casing.
        for room in &mut self.rooms {
            let mut exits = BTreeMap::new();
            for (direction, exit) in std::mem::take(&mut room.exits) {
                exits.insert(direction.to_lowercase(), exit);
            }
            room.exits = exits;
        }

        // Room item lists define placement, matching historical loader behavior.
        let placements: HashMap<i32, i32> = self
            .rooms
            .iter()
            .flat_map(|room| room.items.iter().map(|id| (*id, room.id)))
            .collect();
        for item in &mut self.items {
            if let Some(room_id) = placements.get(&item.id) {
                item.location = *room_id;
            }
        }

        // Assign IDs to inline starting-inventory items.
        let mut next_item_id = self
            .items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            .max(0)
            + 1;
        for entry in &mut self.player_start_inventory {
            if let InventoryRef::Item(item) = entry {
                if item.id <= 0 || self.items.iter().any(|existing| existing.id == item.id) {
                    item.id = next_item_id;
                    next_item_id += 1;
                }
                item.location = 0;
                self.items.push(item.clone());
                *entry = InventoryRef::Id(item.id);
            }
        }
        for entry in &self.player_start_inventory {
            if let InventoryRef::Id(id) = entry
                && let Some(item) = self.items.iter_mut().find(|item| item.id == *id)
            {
                item.location = 0;
            }
        }

        // `location: 0` means "carried", but monster weapons and shop stock are
        // also stored at 0. Only grant genuinely unplaced items to the player.
        let reserved: HashSet<i32> = self
            .monsters
            .iter()
            .flat_map(|monster| monster.loot.iter().copied().chain(monster.weapon_id))
            .chain(
                self.shops
                    .iter()
                    .flat_map(|shop| shop.stock.iter().map(|stock| stock.item_id)),
            )
            .collect();
        let already_starting: HashSet<i32> = self
            .player_start_inventory
            .iter()
            .filter_map(|entry| match entry {
                InventoryRef::Id(id) => Some(*id),
                InventoryRef::Item(_) => None,
            })
            .collect();
        let granted: Vec<i32> = self
            .items
            .iter()
            .filter(|item| {
                item.location == 0
                    && !reserved.contains(&item.id)
                    && !already_starting.contains(&item.id)
            })
            .map(|item| item.id)
            .collect();
        self.player_start_inventory
            .extend(granted.into_iter().map(InventoryRef::Id));

        for quest in &mut self.quests {
            normalize_quest(quest);
        }
    }

    /// Hard validation. Returns the first error as `AdventureError::Validation`.
    pub fn validate(&self) -> Result<(), AdventureError> {
        let report = self.report();
        if report.is_valid() {
            Ok(())
        } else {
            Err(AdventureError::Validation(report.errors.join("; ")))
        }
    }

    /// Full validation report used by editors and diagnostics.
    pub fn report(&self) -> ValidationReport {
        let mut report = ValidationReport::default();

        if self.rooms.is_empty() {
            report
                .errors
                .push("adventure.rooms must not be empty".to_string());
        }

        let room_ids: HashSet<i32> = self.rooms.iter().map(|room| room.id).collect();
        let item_ids: HashSet<i32> = self.items.iter().map(|item| item.id).collect();
        let monster_ids: HashSet<i32> = self.monsters.iter().map(|m| m.id).collect();
        let quest_ids: HashSet<i32> = self.quests.iter().map(|q| q.id).collect();
        let item_names: HashSet<String> = self
            .items
            .iter()
            .map(|item| normalize_words(&item.name))
            .collect();
        let monster_names: HashSet<String> = self
            .monsters
            .iter()
            .map(|m| normalize_words(&m.name))
            .collect();

        check_unique(&mut report, "room", self.rooms.iter().map(|r| r.id));
        check_unique(&mut report, "item", self.items.iter().map(|i| i.id));
        check_unique(&mut report, "monster", self.monsters.iter().map(|m| m.id));
        check_unique(&mut report, "quest", self.quests.iter().map(|q| q.id));
        check_unique(&mut report, "shop", self.shops.iter().map(|s| s.id));

        for room in &self.rooms {
            if room.id <= 0 {
                report
                    .errors
                    .push(format!("room '{}' has a non-positive id", room.name));
            }
            for (direction, exit) in &room.exits {
                if direction.trim().is_empty() {
                    report
                        .errors
                        .push(format!("room {} has an empty exit direction", room.id));
                }
                let destination = exit.destination();
                if !room_ids.contains(&destination) {
                    report.errors.push(format!(
                        "room {} exit '{}' points to unknown room {}",
                        room.id, direction, destination
                    ));
                }
                if let Some(lock) = exit.lock()
                    && lock.locked
                    && lock.key.as_deref().unwrap_or("").trim().is_empty()
                {
                    report.warnings.push(format!(
                        "room {} exit '{}' is locked but has no key",
                        room.id, direction
                    ));
                }
            }
            for item_id in &room.items {
                if !item_ids.contains(item_id) {
                    report.errors.push(format!(
                        "room {} references unknown item {}",
                        room.id, item_id
                    ));
                }
            }
        }

        if self.start_room <= 0 || !room_ids.contains(&self.start_room) {
            report
                .errors
                .push(format!("start_room {} does not exist", self.start_room));
        }

        for item in &self.items {
            if item.id <= 0 {
                report
                    .errors
                    .push(format!("item '{}' has a non-positive id", item.name));
            }
            let valid_location = matches!(item.location, -1 | 0)
                || room_ids.contains(&item.location)
                || monster_ids.contains(&item.location);
            if !valid_location {
                report.errors.push(format!(
                    "item '{}' references unknown location {}",
                    item.name, item.location
                ));
            }
            if item.is_weapon && item.weapon_dice <= 0 {
                report.errors.push(format!(
                    "weapon '{}' has non-positive weapon_dice",
                    item.name
                ));
            }
        }

        for monster in &self.monsters {
            if monster.id <= 0 {
                report
                    .errors
                    .push(format!("monster '{}' has a non-positive id", monster.name));
            }
            if !room_ids.contains(&monster.room_id) {
                report.errors.push(format!(
                    "monster '{}' references unknown room {}",
                    monster.name, monster.room_id
                ));
            }
            if let Some(weapon_id) = monster.weapon_id
                && !item_ids.contains(&weapon_id)
            {
                report.errors.push(format!(
                    "monster '{}' references unknown weapon item {}",
                    monster.name, weapon_id
                ));
            }
            for loot_id in &monster.loot {
                if !item_ids.contains(loot_id) {
                    report.errors.push(format!(
                        "monster '{}' references unknown loot item {}",
                        monster.name, loot_id
                    ));
                }
            }
        }

        for quest in &self.quests {
            if quest.id <= 0 {
                report
                    .warnings
                    .push(format!("quest '{}' has a non-positive id", quest.title));
            }
            if quest.title.trim().is_empty() {
                report
                    .warnings
                    .push(format!("quest {} has an empty title", quest.id));
            }
            let stages = quest_effective_stages(quest);
            if stages.is_empty() {
                report
                    .warnings
                    .push(format!("quest {} has no objectives", quest.id));
            }
            for stage in &stages {
                for (index, objective) in stage.objectives.iter().enumerate() {
                    let objective = objective.structured(index);
                    let target = objective.target_text();
                    if target.trim().is_empty() {
                        report.warnings.push(format!(
                            "quest {} stage '{}' has an objective without a target",
                            quest.id, stage.title
                        ));
                    }
                    match objective.kind {
                        ObjectiveKind::ReachRoom => {
                            let numeric = objective.target_room_id();
                            let found = numeric.is_some_and(|id| room_ids.contains(&id))
                                || self.rooms.iter().any(|room| {
                                    normalize_words(&room.name).contains(&normalize_words(&target))
                                });
                            if !found {
                                report.warnings.push(format!(
                                    "quest {} reach_room target '{}' does not match a room",
                                    quest.id, target
                                ));
                            }
                        }
                        ObjectiveKind::KillMonster => {
                            if !monster_names
                                .iter()
                                .any(|name| name.contains(&normalize_words(&target)))
                            {
                                report.warnings.push(format!(
                                    "quest {} kill target '{}' does not match a monster",
                                    quest.id, target
                                ));
                            }
                        }
                        ObjectiveKind::CollectItem => {
                            if !item_names
                                .iter()
                                .any(|name| name.contains(&normalize_words(&target)))
                            {
                                report.warnings.push(format!(
                                    "quest {} collect target '{}' does not match an item",
                                    quest.id, target
                                ));
                            }
                        }
                        ObjectiveKind::TalkToNpc
                            if !monster_names
                                .iter()
                                .any(|name| name.contains(&normalize_words(&target))) =>
                        {
                            report.warnings.push(format!(
                                "quest {} talk target '{}' does not match an NPC",
                                quest.id, target
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }

        for shop in &self.shops {
            if shop.id <= 0 {
                report
                    .errors
                    .push(format!("shop '{}' has a non-positive id", shop.name));
            }
            if !room_ids.contains(&shop.room_id) {
                report.errors.push(format!(
                    "shop '{}' references unknown room {}",
                    shop.name, shop.room_id
                ));
            }
            if shop.name.trim().is_empty() {
                report
                    .warnings
                    .push(format!("shop {} has an empty name", shop.id));
            }
            if shop.gold < 0 {
                report
                    .errors
                    .push(format!("shop '{}' has negative gold", shop.name));
            }
            if !shop.buy_multiplier.is_finite()
                || !shop.sell_multiplier.is_finite()
                || shop.buy_multiplier <= 0.0
                || shop.sell_multiplier < 0.0
            {
                report
                    .errors
                    .push(format!("shop '{}' has invalid multipliers", shop.name));
            }
            let mut stocked = HashSet::new();
            for stock in &shop.stock {
                if !stocked.insert(stock.item_id) {
                    report.warnings.push(format!(
                        "shop '{}' has duplicate stock entries for item {}",
                        shop.name, stock.item_id
                    ));
                }
                if !item_ids.contains(&stock.item_id) {
                    report.errors.push(format!(
                        "shop '{}' stocks unknown item {}",
                        shop.name, stock.item_id
                    ));
                }
                if stock.quantity <= 0 {
                    report.warnings.push(format!(
                        "shop '{}' stocks item {} with non-positive quantity",
                        shop.name, stock.item_id
                    ));
                }
            }
        }

        for entry in &self.player_start_inventory {
            if let InventoryRef::Id(id) = entry
                && !item_ids.contains(id)
            {
                report.errors.push(format!(
                    "player_start_inventory references unknown item {}",
                    id
                ));
            }
        }

        if let Some(ending) = &self.ending {
            for quest_id in &ending.conditions.quests {
                if !quest_ids.contains(quest_id) {
                    report
                        .warnings
                        .push(format!("ending references unknown quest {}", quest_id));
                }
            }
            for room_id in &ending.conditions.rooms {
                if !room_ids.contains(room_id) {
                    report
                        .warnings
                        .push(format!("ending references unknown room {}", room_id));
                }
            }
            for item_name in &ending.conditions.items {
                let normalized = normalize_words(item_name);
                if !item_names.iter().any(|name| name.contains(&normalized)) {
                    report
                        .warnings
                        .push(format!("ending references unknown item '{}'", item_name));
                }
            }
            if !ending.conditions.all_quests
                && ending.conditions.quests.is_empty()
                && ending.conditions.rooms.is_empty()
                && ending.conditions.items.is_empty()
            {
                report
                    .warnings
                    .push("ending has no victory conditions".to_string());
            }
        }

        // Warn about rooms that cannot be reached from the start room.
        let mut reachable = HashSet::new();
        let mut stack = vec![self.start_room];
        while let Some(room_id) = stack.pop() {
            if !reachable.insert(room_id) {
                continue;
            }
            if let Some(room) = self.rooms.iter().find(|room| room.id == room_id) {
                // Locked exits still count as reachable; a key can open them.
                for exit in room.exits.values() {
                    stack.push(exit.destination());
                }
            }
        }
        for room in &self.rooms {
            if !reachable.contains(&room.id) {
                report.warnings.push(format!(
                    "room {} ('{}') is not reachable from start_room",
                    room.id, room.name
                ));
            }
        }

        report
    }

    fn from_legacy_string(adventure: &Adventure, raw: &Value) -> Self {
        let room_id_map: HashMap<String, i32> = adventure
            .rooms
            .iter()
            .enumerate()
            .map(|(index, room)| (room.id.clone(), index as i32 + 1))
            .collect();

        let mut doc = Self {
            schema_version: SCHEMA_VERSION,
            id: adventure.id.clone(),
            title: adventure.title.clone(),
            description: raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            intro: raw
                .get("intro")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            author: raw
                .get("author")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            ..Self::default()
        };

        let mut next_item_id = 1;
        for room in &adventure.rooms {
            let numeric_id = room_id_map[&room.id];
            let mut exits = BTreeMap::new();
            for (direction, destination) in &room.exits {
                if let Some(target) = room_id_map.get(destination) {
                    exits.insert(direction.clone(), ExitDef::Plain(*target));
                }
            }
            let mut item_ids = Vec::new();
            for item in &room.items {
                let mut extra = BTreeMap::new();
                extra.insert("legacy_id".to_string(), Value::String(item.id.clone()));
                doc.items.push(ItemDef {
                    id: next_item_id,
                    name: item.name.clone(),
                    description: item.description.clone(),
                    location: numeric_id,
                    extra,
                    ..ItemDef::default()
                });
                item_ids.push(next_item_id);
                next_item_id += 1;
            }
            doc.rooms.push(RoomDef {
                id: numeric_id,
                name: if room.title.trim().is_empty() {
                    room.id.clone()
                } else {
                    room.title.clone()
                },
                description: room.description.clone(),
                is_dark: false,
                exits,
                items: item_ids,
                extra: BTreeMap::new(),
            });
        }

        for item in &adventure.player_start_inventory {
            let mut extra = BTreeMap::new();
            extra.insert("legacy_id".to_string(), Value::String(item.id.clone()));
            doc.items.push(ItemDef {
                id: next_item_id,
                name: item.name.clone(),
                description: item.description.clone(),
                location: 0,
                extra,
                ..ItemDef::default()
            });
            doc.player_start_inventory
                .push(InventoryRef::Id(next_item_id));
            next_item_id += 1;
        }

        doc.start_room = room_id_map.get(&adventure.start_room).copied().unwrap_or(0);
        doc
    }
}

fn slugify(value: &str) -> String {
    let slug = value
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let slug = slug
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if slug.is_empty() {
        "adventure".to_string()
    } else {
        slug
    }
}

fn normalize_quest(quest: &mut QuestDef) {
    for stage in &mut quest.stages {
        normalize_stage(stage, quest.id);
    }

    let structured_objectives: Vec<StructuredObjective> = quest
        .objectives
        .iter()
        .enumerate()
        .map(|(index, objective)| objective.structured(index))
        .collect();

    if quest.stages.is_empty() {
        quest.stages.push(StageDef {
            stage_id: "main".to_string(),
            stage_number: 1,
            title: "Main Objectives".to_string(),
            description: quest.description.clone(),
            objectives: structured_objectives
                .iter()
                .cloned()
                .map(ObjectiveDef::Structured)
                .collect(),
            stage_reward_xp: quest.rewards.xp,
            extra: BTreeMap::new(),
        });
        if let Some(stage) = quest.stages.first_mut() {
            normalize_stage(stage, quest.id);
        }
    } else if structured_objectives.is_empty() {
        quest.objectives = quest.stages[0].objectives.clone();
    }

    // Keep the flat legacy fields in sync.
    if quest.rewards.gold == 0
        && let Some(gold) = quest.rewards_gold
    {
        quest.rewards.gold = gold;
    }
    if quest.rewards.xp == 0
        && let Some(xp) = quest.rewards_xp
    {
        quest.rewards.xp = xp;
    }
    if quest.rewards_gold.is_none() {
        quest.rewards_gold = Some(quest.rewards.gold);
    }
    if quest.rewards_xp.is_none() {
        quest.rewards_xp = Some(quest.rewards.xp);
    }

    // Stage rewards default to the quest reward on single-stage quests.
    if quest.stages.len() == 1 && quest.stages[0].stage_reward_xp == 0 {
        quest.stages[0].stage_reward_xp = quest.rewards.xp;
    }
}

fn normalize_stage(stage: &mut StageDef, quest_id: i32) {
    if stage.stage_id.trim().is_empty() {
        stage.stage_id = format!("stage_{}", stage.stage_number);
    }
    if stage.title.trim().is_empty() {
        stage.title = format!("Stage {}", stage.stage_number);
    }
    let normalized: Vec<ObjectiveDef> = stage
        .objectives
        .iter()
        .enumerate()
        .map(|(index, objective)| {
            let mut obj = objective.structured(index);
            if obj.count <= 0 {
                obj.count = 1;
            }
            if obj.target.is_none() {
                let text = obj.target_text();
                if !text.is_empty() {
                    obj.target = Some(text);
                }
            }
            if obj.target_id.is_none()
                && let Some(target) = &obj.target
            {
                obj.target_id = Some(Value::String(target.clone()));
            }
            if obj.description.trim().is_empty() {
                obj.description = obj
                    .target
                    .clone()
                    .unwrap_or_else(|| format!("Objective {}", index + 1));
            }
            let _ = quest_id;
            ObjectiveDef::Structured(obj)
        })
        .collect();
    stage.objectives = normalized;
}

fn quest_effective_stages(quest: &QuestDef) -> Vec<StageDef> {
    if !quest.stages.is_empty() {
        quest
            .stages
            .iter()
            .map(|stage| StageDef {
                objectives: stage
                    .objectives
                    .iter()
                    .enumerate()
                    .map(|(index, objective)| ObjectiveDef::Structured(objective.structured(index)))
                    .collect(),
                ..stage.clone()
            })
            .collect()
    } else {
        vec![StageDef {
            stage_id: "main".to_string(),
            stage_number: 1,
            title: "Main Objectives".to_string(),
            description: quest.description.clone(),
            objectives: quest
                .objectives
                .iter()
                .enumerate()
                .map(|(index, objective)| ObjectiveDef::Structured(objective.structured(index)))
                .collect(),
            stage_reward_xp: quest.rewards.xp,
            extra: BTreeMap::new(),
        }]
    }
}

fn check_unique(report: &mut ValidationReport, label: &str, ids: impl Iterator<Item = i32>) {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            report
                .errors
                .push(format!("duplicate {} id: {}", label, id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_schema_converts_to_canonical() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "id": "demo",
                "title": "Demo",
                "start_room": "village",
                "rooms": [
                    {"id": "village", "title": "Village", "description": "Home",
                     "exits": {"north": "forest"},
                     "items": [{"id": "key", "name": "Key", "description": "A key."}]},
                    {"id": "forest", "title": "Forest", "description": "Trees", "exits": {}}
                ],
                "player_start_inventory": []
            }"#,
        )
        .expect("document should parse");

        assert_eq!(doc.schema_version, SCHEMA_VERSION);
        assert_eq!(doc.start_room, 1);
        assert_eq!(doc.rooms.len(), 2);
        assert_eq!(doc.items.len(), 1);
        assert_eq!(doc.items[0].location, 1);
        doc.validate().expect("document should validate");
    }

    #[test]
    fn locked_exits_round_trip() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Locks",
                "start_room": 1,
                "rooms": [
                    {"id": 1, "name": "Hall", "description": "A hall",
                     "exits": {"north": {"to": 2, "locked": true, "key": "Brass Key"}}},
                    {"id": 2, "name": "Vault", "description": "A vault", "exits": {}}
                ],
                "items": [{"id": 1, "name": "Brass Key", "description": "A key."}]
            }"#,
        )
        .expect("document should parse");

        let json = doc.to_json_string().expect("should serialise");
        assert!(json.contains("\"locked\": true"));
        let reparsed = AdventureDocument::from_json_str(&json).expect("should reparse");
        assert_eq!(reparsed.rooms[0].exits.len(), 1);
        assert!(reparsed.rooms[0].exits["north"].lock().is_some());
    }

    #[test]
    fn structured_objective_text_is_normalised() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Quests",
                "start_room": 1,
                "rooms": [{"id": 1, "name": "Camp", "description": "A camp"}],
                "quests": [{"id": 1, "title": "Provisions",
                    "objectives": ["Pick up the lantern"],
                    "rewards_gold": 5, "rewards_xp": 10}]
            }"#,
        )
        .expect("document should parse");

        let quest = &doc.quests[0];
        assert_eq!(quest.stages.len(), 1);
        assert_eq!(quest.stages[0].objectives.len(), 1);
        assert_eq!(quest.rewards.gold, 5);
        assert_eq!(quest.rewards.xp, 10);
    }

    #[test]
    fn validation_reports_unknown_exit() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Bad",
                "start_room": 1,
                "rooms": [{"id": 1, "name": "A", "description": "A", "exits": {"north": 2}}]
            }"#,
        )
        .expect("document should parse");

        let report = doc.report();
        assert!(report.errors.iter().any(|e| e.contains("unknown room")));
    }

    #[test]
    fn unknown_fields_are_preserved() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Meta",
                "start_room": 1,
                "custom_top_level": {"keep": true},
                "rooms": [{"id": 1, "name": "A", "description": "A", "ambient_sound": "wind"}],
                "items": [{"id": 1, "name": "Rock", "description": "A rock", "custom_item": 7}]
            }"#,
        )
        .expect("document should parse");

        assert!(doc.extra.contains_key("custom_top_level"));
        assert!(doc.rooms[0].extra.contains_key("ambient_sound"));
        assert!(doc.items[0].extra.contains_key("custom_item"));
    }

    #[test]
    fn validation_reports_duplicate_ids() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Dupes",
                "start_room": 1,
                "rooms": [
                    {"id": 1, "name": "A", "description": "A"},
                    {"id": 1, "name": "B", "description": "B"}
                ],
                "items": [
                    {"id": 1, "name": "Rock", "description": "A rock"},
                    {"id": 1, "name": "Stone", "description": "A stone"}
                ],
                "shops": [
                    {"id": 1, "name": "One", "room_id": 1, "gold": 10},
                    {"id": 1, "name": "Two", "room_id": 1, "gold": 10}
                ]
            }"#,
        )
        .expect("document should parse");

        let report = doc.report();
        for expected in [
            "duplicate room id: 1",
            "duplicate item id: 1",
            "duplicate shop id: 1",
        ] {
            assert!(
                report.errors.iter().any(|error| error.contains(expected)),
                "missing '{expected}' in {:?}",
                report.errors
            );
        }
    }

    #[test]
    fn validation_reports_dangling_references() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Refs",
                "start_room": 1,
                "rooms": [{"id": 1, "name": "A", "description": "A"}],
                "shops": [{"id": 1, "name": "Stall", "room_id": 1, "gold": 10,
                           "stock": [{"item_id": 42, "quantity": 1}]}],
                "ending": {"victory_text": "Done", "conditions": {
                    "quests": [7], "rooms": [9], "items": ["Ghost Key"]
                }},
                "player_start_inventory": [99]
            }"#,
        )
        .expect("document should parse");

        let report = doc.report();
        for expected in [
            "stocks unknown item 42",
            "player_start_inventory references unknown item 99",
        ] {
            assert!(
                report.errors.iter().any(|error| error.contains(expected)),
                "missing '{expected}' in {:?}",
                report.errors
            );
        }
        for expected in [
            "ending references unknown quest 7",
            "ending references unknown room 9",
            "ending references unknown item 'Ghost Key'",
        ] {
            assert!(
                report
                    .warnings
                    .iter()
                    .any(|warning| warning.contains(expected)),
                "missing '{expected}' in {:?}",
                report.warnings
            );
        }
    }

    #[test]
    fn validation_warns_about_unreachable_rooms() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Islands",
                "start_room": 1,
                "rooms": [
                    {"id": 1, "name": "Shore", "description": "Sand"},
                    {"id": 2, "name": "Islet", "description": "Rock"}
                ]
            }"#,
        )
        .expect("document should parse");

        let report = doc.report();
        assert!(report.is_valid(), "unreachable rooms are warnings only");
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.contains("is not reachable from start_room")),
            "missing reachability warning in {:?}",
            report.warnings
        );
    }

    #[test]
    fn validation_reports_unknown_start_room() {
        let doc = AdventureDocument::from_json_str(
            r#"{
                "title": "Lost",
                "start_room": 99,
                "rooms": [{"id": 1, "name": "A", "description": "A"}]
            }"#,
        )
        .expect("document should parse");

        let report = doc.report();
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("start_room 99 does not exist")),
            "missing start_room error in {:?}",
            report.errors
        );
    }
}
