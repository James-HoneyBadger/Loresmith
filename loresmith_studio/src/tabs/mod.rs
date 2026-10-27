//! Per-tab editors for the Studio workspace.
//!
//! Each tab lives in its own module and implements an inherent
//! [`LoresmithStudio`] block, so switching tabs keeps filters, selections, and
//! an in-progress Play session intact.

use crate::app::{
    GameSummary, LoresmithStudio, Tab, ensure_structured_objective, objective_kind_label,
};
use eframe::egui;
use loresmith_rs::{
    DialogueDef, EndingDef, ExitDef, InventoryRef, ItemType, MonsterStatus, ObjectiveDef,
    ObjectiveKind, RewardItem, ShopStockDef, StageDef, StructuredObjective,
};
use loresmith_ui as theme;

mod info;
mod items;
mod monsters;
mod play;
mod preview;
mod quests;
mod rooms;
mod shops;
