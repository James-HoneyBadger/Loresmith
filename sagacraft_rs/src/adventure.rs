//! Legacy string-ID adventure format.
//!
//! This module remains for reading historical TUI/demo files. New content
//! should use [`crate::document::AdventureDocument`], which normalises this
//! format to the canonical integer-ID schema.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum AdventureError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for AdventureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdventureError::Io(e) => write!(f, "io error: {e}"),
            AdventureError::Json(e) => write!(f, "json error: {e}"),
            AdventureError::Validation(msg) => write!(f, "validation error: {msg}"),
        }
    }
}

impl std::error::Error for AdventureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AdventureError::Io(e) => Some(e),
            AdventureError::Json(e) => Some(e),
            AdventureError::Validation(_) => None,
        }
    }
}

impl From<std::io::Error> for AdventureError {
    fn from(value: std::io::Error) -> Self {
        AdventureError::Io(value)
    }
}

impl From<serde_json::Error> for AdventureError {
    fn from(value: serde_json::Error) -> Self {
        AdventureError::Json(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdventureItem {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdventureRoom {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub exits: HashMap<String, String>,
    #[serde(default)]
    pub items: Vec<AdventureItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Adventure {
    pub id: String,
    pub title: String,
    pub start_room: String,
    pub rooms: Vec<AdventureRoom>,
    #[serde(default)]
    pub player_start_inventory: Vec<AdventureItem>,
}

impl Adventure {
    pub fn validate(&self) -> Result<(), AdventureError> {
        if self.id.trim().is_empty() {
            return Err(AdventureError::Validation(
                "adventure.id is required".to_string(),
            ));
        }
        if self.title.trim().is_empty() {
            return Err(AdventureError::Validation(
                "adventure.title is required".to_string(),
            ));
        }
        if self.start_room.trim().is_empty() {
            return Err(AdventureError::Validation(
                "adventure.start_room is required".to_string(),
            ));
        }
        if self.rooms.is_empty() {
            return Err(AdventureError::Validation(
                "adventure.rooms must not be empty".to_string(),
            ));
        }

        let mut ids = HashSet::new();
        for room in &self.rooms {
            if room.id.trim().is_empty() {
                return Err(AdventureError::Validation(
                    "room.id is required".to_string(),
                ));
            }
            if !ids.insert(room.id.clone()) {
                return Err(AdventureError::Validation(format!(
                    "duplicate room id: {}",
                    room.id
                )));
            }
        }

        if !ids.contains(&self.start_room) {
            return Err(AdventureError::Validation(format!(
                "start_room does not exist: {}",
                self.start_room
            )));
        }

        for room in &self.rooms {
            for (dir, dest) in &room.exits {
                if dir.trim().is_empty() {
                    return Err(AdventureError::Validation(format!(
                        "room '{}' has an empty exit direction",
                        room.id
                    )));
                }
                if dest.trim().is_empty() {
                    return Err(AdventureError::Validation(format!(
                        "room '{}' exit '{}' has empty destination",
                        room.id, dir
                    )));
                }
                if !ids.contains(dest) {
                    return Err(AdventureError::Validation(format!(
                        "room '{}' exit '{}' points to unknown room '{}'",
                        room.id, dir, dest
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn load_json_file(path: impl AsRef<Path>) -> Result<Self, AdventureError> {
        let s = fs::read_to_string(path)?;
        let adv: Adventure = serde_json::from_str(&s)?;
        adv.validate()?;
        Ok(adv)
    }

    pub fn save_json_file(&self, path: impl AsRef<Path>) -> Result<(), AdventureError> {
        self.validate()?;
        let s = serde_json::to_string_pretty(self)?;
        fs::write(path, s)?;
        Ok(())
    }

    pub fn demo() -> Self {
        let mut village_exits = HashMap::new();
        village_exits.insert("north".to_string(), "forest".to_string());

        let mut forest_exits = HashMap::new();
        forest_exits.insert("south".to_string(), "village".to_string());

        Self {
            id: "demo".to_string(),
            title: "Demo Adventure".to_string(),
            start_room: "village".to_string(),
            rooms: vec![
                AdventureRoom {
                    id: "village".to_string(),
                    title: "Quiet Village".to_string(),
                    description:
                        "A small village with a single cobblestone path and a warm lantern glow."
                            .to_string(),
                    exits: village_exits,
                    items: vec![AdventureItem {
                        id: "key".to_string(),
                        name: "Ancient Key".to_string(),
                        description: "A tarnished key that seems to hum faintly.".to_string(),
                    }],
                },
                AdventureRoom {
                    id: "forest".to_string(),
                    title: "Whispering Forest".to_string(),
                    description: "Tall pines sway as if sharing secrets. The village lies south."
                        .to_string(),
                    exits: forest_exits,
                    items: vec![],
                },
            ],
            player_start_inventory: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_validates() {
        Adventure::demo().validate().unwrap();
    }

    #[test]
    fn validate_requires_start_room() {
        let mut adv = Adventure::demo();
        adv.start_room = "missing".to_string();
        let err = adv.validate().unwrap_err();
        match err {
            AdventureError::Validation(msg) => assert!(msg.contains("start_room")),
            _ => panic!("expected validation error"),
        }
    }
}
