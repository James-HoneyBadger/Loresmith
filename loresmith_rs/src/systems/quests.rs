use crate::document::{ObjectiveKind, QuestDef, QuestDifficultyDef, RewardItem};
use crate::game_state::{AdventureGame, GameEvent, award_experience, name_matches};
use crate::systems::System;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestStatus {
    Available,
    Active,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObjectiveType {
    Kill,
    Collect,
    Explore,
    Talk,
    Defend,
    Deliver,
    Discover,
    Puzzle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestDifficulty {
    Easy,
    Moderate,
    Challenging,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub objective_id: String,
    pub obj_type: ObjectiveType,
    pub description: String,
    pub target: String,
    #[serde(default)]
    pub target_room_id: Option<i32>,
    #[serde(default)]
    pub topic: Option<String>,
    pub required_count: i32,
    pub current_count: i32,
}

impl QuestObjective {
    pub fn new(
        objective_id: String,
        obj_type: ObjectiveType,
        description: String,
        target: String,
        required_count: i32,
    ) -> Self {
        Self {
            objective_id,
            obj_type,
            description,
            target,
            target_room_id: None,
            topic: None,
            required_count,
            current_count: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_count >= self.required_count
    }

    pub fn progress(&mut self, amount: i32) -> i32 {
        let old_count = self.current_count;
        self.current_count = (self.current_count + amount).min(self.required_count);
        self.current_count - old_count
    }

    pub fn get_progress_percentage(&self) -> i32 {
        if self.required_count == 0 {
            100
        } else {
            ((self.current_count as f32 / self.required_count as f32) * 100.0) as i32
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestStage {
    pub stage_id: String,
    pub stage_number: i32,
    pub title: String,
    pub description: String,
    pub objectives: Vec<QuestObjective>,
    pub stage_reward_xp: i32,
}

impl QuestStage {
    pub fn new(stage_id: String, stage_number: i32, title: String, description: String) -> Self {
        Self {
            stage_id,
            stage_number,
            title,
            description,
            objectives: Vec::new(),
            stage_reward_xp: 0,
        }
    }

    pub fn add_objective(&mut self, objective: QuestObjective) {
        self.objectives.push(objective);
    }

    pub fn is_complete(&self) -> bool {
        self.objectives.iter().all(|o| o.is_complete())
    }

    pub fn get_progress_percentage(&self) -> i32 {
        if self.objectives.is_empty() {
            100
        } else {
            let total_progress: i32 = self
                .objectives
                .iter()
                .map(|o| o.get_progress_percentage())
                .sum();
            total_progress / self.objectives.len() as i32
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestReward {
    pub experience_points: i32,
    pub gold: i32,
    pub items: Vec<String>,
    pub reputation_changes: HashMap<String, i32>,
    pub special_rewards: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub quest_id: String,
    pub title: String,
    pub description: String,
    pub giver_npc: String,
    pub quest_giver_level: i32,
    pub difficulty: QuestDifficulty,
    pub stages: Vec<QuestStage>,
    pub rewards: QuestReward,
    pub status: QuestStatus,
    pub acceptance_time: Option<String>,
    pub completion_time: Option<String>,
    pub current_stage_index: usize,
    #[serde(default)]
    pub ready_notified: bool,
}

impl Quest {
    pub fn new(quest_id: String, title: String, description: String, giver_npc: String) -> Self {
        Self {
            quest_id,
            title,
            description,
            giver_npc,
            quest_giver_level: 1,
            difficulty: QuestDifficulty::Moderate,
            stages: Vec::new(),
            rewards: QuestReward::default(),
            status: QuestStatus::Available,
            acceptance_time: None,
            completion_time: None,
            current_stage_index: 0,
            ready_notified: false,
        }
    }

    pub fn get_current_stage(&self) -> Option<&QuestStage> {
        self.stages.get(self.current_stage_index)
    }

    pub fn is_complete(&self) -> bool {
        !self.stages.is_empty()
            && self.current_stage_index >= self.stages.len() - 1
            && self
                .get_current_stage()
                .is_some_and(|s: &QuestStage| s.is_complete())
    }

    /// True when the active stage's objectives are all done.
    pub fn current_stage_complete(&self) -> bool {
        self.get_current_stage().is_none_or(QuestStage::is_complete)
    }

    /// True when the quest is on its final stage and every objective is done.
    pub fn is_ready_to_turn_in(&self) -> bool {
        self.current_stage_complete()
            && (self.stages.is_empty() || self.current_stage_index + 1 >= self.stages.len())
    }

    /// Missing objective descriptions for the active stage, if any.
    pub fn remaining_objectives(&self) -> Vec<String> {
        self.get_current_stage()
            .map(|stage| {
                stage
                    .objectives
                    .iter()
                    .filter(|objective| !objective.is_complete())
                    .map(|objective| {
                        format!(
                            "{} ({}/{})",
                            objective.description,
                            objective.current_count,
                            objective.required_count
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Advance to the next stage. Returns false when already on the last stage.
    pub fn advance_stage(&mut self) -> bool {
        if self.current_stage_index + 1 < self.stages.len() {
            self.current_stage_index += 1;
            self.ready_notified = false;
            true
        } else {
            false
        }
    }

    pub fn mark_complete(&mut self) {
        self.status = QuestStatus::Completed;
        self.completion_time = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestTracker {
    pub active_quests: HashMap<String, Quest>,
    pub completed_quests: HashSet<String>,
    pub failed_quests: HashSet<String>,
    pub quest_history: Vec<(String, QuestStatus, String)>,
}

impl QuestTracker {
    pub fn new() -> Self {
        Self {
            active_quests: HashMap::new(),
            completed_quests: HashSet::new(),
            failed_quests: HashSet::new(),
            quest_history: Vec::new(),
        }
    }
}

impl Default for QuestTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestTracker {
    pub fn accept_quest(&mut self, mut quest: Quest) -> bool {
        if self.active_quests.contains_key(&quest.quest_id)
            || self.completed_quests.contains(&quest.quest_id)
        {
            return false;
        }

        quest.status = QuestStatus::Active;
        quest.acceptance_time = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        let quest_id = quest.quest_id.clone();
        self.active_quests.insert(quest_id.clone(), quest);
        self.record_history(quest_id, QuestStatus::Active);
        true
    }

    pub fn complete_quest(&mut self, quest_id: &str) -> Option<QuestReward> {
        if let Some(quest) = self.active_quests.get_mut(quest_id) {
            quest.mark_complete();
            let reward = quest.rewards.clone();
            self.completed_quests.insert(quest_id.to_string());
            self.active_quests.remove(quest_id);
            self.record_history(quest_id.to_string(), QuestStatus::Completed);
            Some(reward)
        } else {
            None
        }
    }

    pub fn get_active_count(&self) -> usize {
        self.active_quests.len()
    }

    pub fn get_completed_count(&self) -> usize {
        self.completed_quests.len()
    }

    fn record_history(&mut self, quest_id: String, status: QuestStatus) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.quest_history.push((quest_id, status, timestamp));
    }
} // end impl QuestTracker (methods)

const JOURNAL_VERBS: &[&str] = &["quests", "journal"];
const ACCEPT_VERBS: &[&str] = &["accept"];
const COMPLETE_VERBS: &[&str] = &["complete", "finish"];
const ABANDON_VERBS: &[&str] = &["abandon"];

pub struct QuestSystem {
    pub tracker: QuestTracker,
    pub available_quests: HashMap<String, Quest>,
    loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuestSystemState {
    tracker: QuestTracker,
    available_quests: HashMap<String, Quest>,
    loaded: bool,
}

impl QuestSystem {
    /// Verbs this system answers to, used by the command-reference tests.
    pub const VERB_GROUPS: &'static [&'static [&'static str]] =
        &[JOURNAL_VERBS, ACCEPT_VERBS, COMPLETE_VERBS, ABANDON_VERBS];

    pub fn new() -> Self {
        Self {
            tracker: QuestTracker::new(),
            available_quests: HashMap::new(),
            loaded: false,
        }
    }
}

impl Default for QuestSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestSystem {
    fn normalize_text(value: &str) -> String {
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

    pub fn load_quests_from_game(&mut self, game: &AdventureGame) {
        if self.loaded {
            return;
        }
        self.loaded = true;

        for quest_data in &game.quests {
            if let Ok(quest) = self.parse_quest_from_json(quest_data) {
                self.available_quests.insert(quest.quest_id.clone(), quest);
            }
        }
    }

    fn parse_quest_from_json(
        &self,
        data: &serde_json::Value,
    ) -> Result<Quest, Box<dyn std::error::Error>> {
        let def: crate::document::QuestDef = serde_json::from_value(data.clone())?;
        Ok(quest_from_def(&def))
    }

    pub fn add_available_quest(&mut self, quest: Quest) {
        self.available_quests.insert(quest.quest_id.clone(), quest);
    }

    pub fn get_available_quests(&self) -> Vec<&Quest> {
        self.available_quests.values().collect()
    }

    pub fn accept_quest(&mut self, quest_id: &str) -> Result<String, String> {
        if let Some(quest) = self.available_quests.remove(quest_id) {
            let title = quest.title.clone();
            if self.tracker.accept_quest(quest) {
                Ok(format!("Accepted quest: {}", title))
            } else {
                Err("Quest already active or completed".to_string())
            }
        } else {
            Err("Quest not found".to_string())
        }
    }

    fn progress_matching(
        &mut self,
        kind: ObjectiveType,
        name: &str,
        entity_id: Option<i32>,
        topic: Option<&str>,
        notifications: &mut Vec<String>,
    ) {
        let normalized_name = Self::normalize_text(name);

        for quest in self.tracker.active_quests.values_mut() {
            let Some(stage) = quest.stages.get_mut(quest.current_stage_index) else {
                continue;
            };
            for objective in &mut stage.objectives {
                if objective.obj_type != kind || objective.is_complete() {
                    continue;
                }
                let target = Self::normalize_text(&objective.target);
                let id_matches =
                    entity_id.is_some() && objective.target.trim().parse::<i32>().ok() == entity_id;
                if !id_matches && (target.is_empty() || !normalized_name.contains(&target)) {
                    continue;
                }
                if kind == ObjectiveType::Talk
                    && let Some(required_topic) = &objective.topic
                {
                    let required = Self::normalize_text(required_topic);
                    let provided = topic.map(Self::normalize_text).unwrap_or_default();
                    if !required.is_empty() && !provided.contains(&required) {
                        continue;
                    }
                }
                if objective.progress(1) > 0 {
                    notifications.push(format!(
                        "[Quest: {}] {} ({}/{})",
                        quest.title,
                        objective.description,
                        objective.current_count,
                        objective.required_count
                    ));
                }
            }
        }
    }

    fn progress_rooms(&mut self, room_id: i32, room_name: &str, notifications: &mut Vec<String>) {
        let normalized_room = Self::normalize_text(room_name);
        let room_id_text = room_id.to_string();

        for quest in self.tracker.active_quests.values_mut() {
            let Some(stage) = quest.stages.get_mut(quest.current_stage_index) else {
                continue;
            };
            for objective in &mut stage.objectives {
                if objective.obj_type != ObjectiveType::Explore || objective.is_complete() {
                    continue;
                }
                let target = Self::normalize_text(&objective.target);
                let matches_id = objective.target_room_id == Some(room_id)
                    || (!target.is_empty() && target == room_id_text);
                let matches_name = !target.is_empty()
                    && !normalized_room.is_empty()
                    && (normalized_room.contains(&target) || target.contains(&normalized_room));
                if !matches_id && !matches_name {
                    continue;
                }
                if objective.progress(1) > 0 {
                    notifications.push(format!(
                        "[Quest: {}] {} ({}/{})",
                        quest.title,
                        objective.description,
                        objective.current_count,
                        objective.required_count
                    ));
                }
            }
        }
    }

    pub fn show_quests(&self) -> String {
        let mut result = String::from("Active Quests:\n");
        let mut active: Vec<&Quest> = self.tracker.active_quests.values().collect();
        active.sort_by_key(|quest| quest.quest_id.parse::<i32>().unwrap_or(i32::MAX));
        if active.is_empty() {
            result.push_str("  (none)\n");
        }
        for quest in active {
            result.push_str(&format!(
                "- [{}] {} ({:?})\n  {}\n",
                quest.quest_id, quest.title, quest.difficulty, quest.description
            ));
            if let Some(stage) = quest.get_current_stage() {
                result.push_str(&format!(
                    "  Stage {}/{}: {}\n",
                    quest.current_stage_index + 1,
                    quest.stages.len().max(1),
                    stage.title
                ));
                for objective in &stage.objectives {
                    let mark = if objective.is_complete() { "x" } else { " " };
                    result.push_str(&format!(
                        "    [{}] {} ({}/{})\n",
                        mark,
                        objective.description,
                        objective.current_count,
                        objective.required_count
                    ));
                }
            }
            if quest.is_ready_to_turn_in() {
                result.push_str(&format!(
                    "  Ready to turn in: complete {}\n",
                    quest.quest_id
                ));
            }
        }

        result.push_str("\nAvailable Quests:\n");
        let mut available: Vec<&Quest> = self.available_quests.values().collect();
        available.sort_by_key(|quest| quest.quest_id.parse::<i32>().unwrap_or(i32::MAX));
        if available.is_empty() {
            result.push_str("  (none)\n");
        }
        for quest in available {
            result.push_str(&format!(
                "- [{}] {}: {}\n",
                quest.quest_id, quest.title, quest.description
            ));
        }
        result.trim_end().to_string()
    }
}

impl System for QuestSystem {
    fn state_key(&self) -> Option<&'static str> {
        Some("quests")
    }

    fn save_state(&self) -> Option<serde_json::Value> {
        serde_json::to_value(QuestSystemState {
            tracker: self.tracker.clone(),
            available_quests: self.available_quests.clone(),
            loaded: self.loaded,
        })
        .ok()
    }

    fn load_state(&mut self, state: Option<&serde_json::Value>, game: &mut AdventureGame) {
        if let Some(raw) = state
            && let Ok(snapshot) = serde_json::from_value::<QuestSystemState>(raw.clone())
        {
            self.tracker = snapshot.tracker;
            self.available_quests = snapshot.available_quests;
            self.loaded = snapshot.loaded;
            return;
        }

        self.tracker = QuestTracker::new();
        self.available_quests.clear();
        self.loaded = false;
        self.load_quests_from_game(game);
    }

    fn on_adventure_loaded(&mut self, _game: &mut AdventureGame) {
        self.tracker = QuestTracker::new();
        self.available_quests.clear();
        self.loaded = false;
    }

    fn on_command(
        &mut self,
        command: &str,
        args: &[&str],
        game: &mut AdventureGame,
    ) -> Option<String> {
        self.load_quests_from_game(game);

        match command {
            c if JOURNAL_VERBS.contains(&c) => Some(self.show_quests()),
            c if ACCEPT_VERBS.contains(&c) => {
                if args.is_empty() {
                    Some(
                        "Usage: accept <quest_id>. Use 'quests' to see available quests."
                            .to_string(),
                    )
                } else {
                    match self.accept_quest(args[0]) {
                        Ok(msg) => Some(msg),
                        Err(err) => Some(format!("Error: {}", err)),
                    }
                }
            }
            c if COMPLETE_VERBS.contains(&c) => {
                if args.is_empty() {
                    Some(
                        "Usage: complete <quest_id>. Use 'quests' to see active quests."
                            .to_string(),
                    )
                } else {
                    let quest_id = args[0];
                    let Some(quest) = self.tracker.active_quests.get(quest_id) else {
                        return Some(format!(
                            "Quest '{}' not found or not active. Use 'quests' to see active quests.",
                            quest_id
                        ));
                    };
                    if !quest.is_ready_to_turn_in() {
                        let remaining = quest.remaining_objectives();
                        let detail = if remaining.is_empty() {
                            "the current stage is not finished".to_string()
                        } else {
                            remaining.join("; ")
                        };
                        return Some(format!(
                            "Quest '{}' cannot be completed yet. Remaining: {}",
                            quest_id, detail
                        ));
                    }

                    let title = quest.title.clone();
                    let Some(reward) = self.tracker.complete_quest(quest_id) else {
                        return Some(format!("Quest '{}' could not be completed.", quest_id));
                    };

                    let mut lines = vec![format!("Completed quest: {} ({})", title, quest_id)];
                    game.player.gold += reward.gold;
                    if reward.gold > 0 {
                        lines.push(format!("+{} gold", reward.gold));
                    }
                    if reward.experience_points > 0 {
                        lines.push(format!("+{} XP", reward.experience_points));
                        lines.extend(award_experience(game, reward.experience_points));
                    }
                    for item_ref in &reward.items {
                        if let Some(name) = grant_reward_item(game, item_ref) {
                            lines.push(format!("Received: {}", name));
                        } else {
                            lines.push(format!("Reward item '{}' was not found.", item_ref));
                        }
                    }
                    for (faction, delta) in &reward.reputation_changes {
                        *game.player.reputation.entry(faction.clone()).or_insert(0) += *delta;
                        lines.push(format!("Reputation with {}: {:+}", faction, delta));
                    }

                    game.completed_quests.insert(quest_id.to_string());
                    game.events.push(GameEvent::QuestCompleted {
                        quest_id: quest_id.to_string(),
                        title,
                    });
                    if let Some(ending) = game.check_ending() {
                        lines.push(ending);
                    }
                    Some(lines.join("\n"))
                }
            }
            c if ABANDON_VERBS.contains(&c) => {
                if args.is_empty() {
                    Some("Usage: abandon <quest_id>".to_string())
                } else if self.tracker.active_quests.remove(args[0]).is_some() {
                    Some(format!("Abandoned quest: {}", args[0]))
                } else {
                    Some(format!("Quest '{}' is not active.", args[0]))
                }
            }
            _ => None,
        }
    }

    fn on_events(&mut self, events: &[GameEvent], game: &mut AdventureGame) -> Option<String> {
        let mut notifications: Vec<String> = Vec::new();

        for event in events {
            match event {
                GameEvent::MonsterKilled {
                    monster_name,
                    monster_id,
                    ..
                } => {
                    self.progress_matching(
                        ObjectiveType::Kill,
                        monster_name,
                        Some(*monster_id),
                        None,
                        &mut notifications,
                    );
                }
                GameEvent::ItemCollected {
                    item_name, item_id, ..
                } => {
                    self.progress_matching(
                        ObjectiveType::Collect,
                        item_name,
                        Some(*item_id),
                        None,
                        &mut notifications,
                    );
                }
                GameEvent::RoomEntered { room_id } => {
                    let room_name = game
                        .rooms
                        .get(room_id)
                        .map(|room| room.name.clone())
                        .unwrap_or_default();
                    self.progress_rooms(*room_id, &room_name, &mut notifications);
                }
                GameEvent::TalkedTo {
                    npc_name,
                    npc_id,
                    topic,
                } => {
                    self.progress_matching(
                        ObjectiveType::Talk,
                        npc_name,
                        Some(*npc_id),
                        topic.as_deref(),
                        &mut notifications,
                    );
                }
                _ => {}
            }
        }

        // Stage advancement and turn-in notices are evaluated once per batch.
        for quest in self.tracker.active_quests.values_mut() {
            loop {
                if !quest.current_stage_complete() {
                    break;
                }
                let stage_number = quest.current_stage_index + 1;
                let stage_xp = quest
                    .get_current_stage()
                    .map(|stage| stage.stage_reward_xp)
                    .unwrap_or(0);
                if quest.advance_stage() {
                    if stage_xp > 0 {
                        notifications.push(format!(
                            "[Quest: {}] Stage {} complete (+{} XP).",
                            quest.title, stage_number, stage_xp
                        ));
                        notifications.extend(award_experience(game, stage_xp));
                    } else {
                        notifications.push(format!(
                            "[Quest: {}] Stage {} complete.",
                            quest.title, stage_number
                        ));
                    }
                    if let Some(next) = quest.get_current_stage() {
                        notifications.push(format!(
                            "[Quest: {}] Next stage: {}",
                            quest.title, next.title
                        ));
                    }
                    continue;
                }
                if !quest.ready_notified {
                    quest.ready_notified = true;
                    notifications.push(format!(
                        "[Quest: {}] All objectives complete — turn it in with 'complete {}'.",
                        quest.title, quest.quest_id
                    ));
                }
                break;
            }
        }

        if notifications.is_empty() {
            None
        } else {
            Some(format!(
                "Quest update:
{}",
                notifications.join(
                    "
"
                )
            ))
        }
    }
}

fn grant_reward_item(game: &mut AdventureGame, item_ref: &str) -> Option<String> {
    let item_id = if let Ok(id) = item_ref.parse::<i32>() {
        Some(id)
    } else {
        game.items
            .values()
            .find(|item| name_matches(&item.name, item_ref))
            .map(|item| item.id)
    }?;

    let name = game.items.get(&item_id)?.name.clone();
    if let Some(item) = game.items.get_mut(&item_id) {
        item.location = 0;
    }
    if !game.player.inventory.contains(&item_id) {
        game.player.inventory.push(item_id);
    }
    Some(name)
}

fn objective_type(kind: ObjectiveKind) -> ObjectiveType {
    match kind {
        ObjectiveKind::KillMonster => ObjectiveType::Kill,
        ObjectiveKind::CollectItem => ObjectiveType::Collect,
        ObjectiveKind::ReachRoom => ObjectiveType::Explore,
        ObjectiveKind::TalkToNpc => ObjectiveType::Talk,
        ObjectiveKind::Defend => ObjectiveType::Defend,
        ObjectiveKind::Deliver => ObjectiveType::Deliver,
        ObjectiveKind::Discover => ObjectiveType::Discover,
        ObjectiveKind::Puzzle => ObjectiveType::Puzzle,
        ObjectiveKind::Other => ObjectiveType::Discover,
    }
}

fn quest_difficulty(difficulty: &QuestDifficultyDef) -> QuestDifficulty {
    match difficulty {
        QuestDifficultyDef::Easy => QuestDifficulty::Easy,
        QuestDifficultyDef::Moderate => QuestDifficulty::Moderate,
        QuestDifficultyDef::Challenging => QuestDifficulty::Challenging,
        QuestDifficultyDef::Hard => QuestDifficulty::Hard,
    }
}

fn quest_from_def(def: &QuestDef) -> Quest {
    let mut stages: Vec<QuestStage> = Vec::new();

    if !def.stages.is_empty() {
        for stage in &def.stages {
            let mut objectives = Vec::new();
            for (index, objective) in stage.objectives.iter().enumerate() {
                let structured = objective.structured(index);
                let mut runtime = QuestObjective::new(
                    format!("{}_{}", stage.stage_id, index),
                    objective_type(structured.kind),
                    if structured.description.trim().is_empty() {
                        structured.target_text()
                    } else {
                        structured.description.clone()
                    },
                    structured.target_text(),
                    structured.count.max(1),
                );
                runtime.target_room_id = if structured.kind == ObjectiveKind::ReachRoom {
                    structured.target_room_id()
                } else {
                    None
                };
                runtime.topic = structured.topic.clone();
                objectives.push(runtime);
            }
            stages.push(QuestStage {
                stage_id: if stage.stage_id.trim().is_empty() {
                    format!("stage_{}", stage.stage_number)
                } else {
                    stage.stage_id.clone()
                },
                stage_number: stage.stage_number,
                title: if stage.title.trim().is_empty() {
                    format!("Stage {}", stage.stage_number)
                } else {
                    stage.title.clone()
                },
                description: stage.description.clone(),
                objectives,
                stage_reward_xp: stage.stage_reward_xp,
            });
        }
    } else {
        let mut objectives = Vec::new();
        for (index, objective) in def.objectives.iter().enumerate() {
            let structured = objective.structured(index);
            let mut runtime = QuestObjective::new(
                format!("obj_{}", index),
                objective_type(structured.kind),
                if structured.description.trim().is_empty() {
                    structured.target_text()
                } else {
                    structured.description.clone()
                },
                structured.target_text(),
                structured.count.max(1),
            );
            runtime.target_room_id = if structured.kind == ObjectiveKind::ReachRoom {
                structured.target_room_id()
            } else {
                None
            };
            runtime.topic = structured.topic.clone();
            objectives.push(runtime);
        }
        stages.push(QuestStage {
            stage_id: "main".to_string(),
            stage_number: 1,
            title: "Main Objectives".to_string(),
            description: def.description.clone(),
            objectives,
            stage_reward_xp: def.rewards.xp,
        });
    }

    let gold = if def.rewards.gold != 0 {
        def.rewards.gold
    } else {
        def.rewards_gold.unwrap_or(0)
    };
    let xp = if def.rewards.xp != 0 {
        def.rewards.xp
    } else {
        def.rewards_xp.unwrap_or(0)
    };

    Quest {
        quest_id: def.id.to_string(),
        title: def.title.clone(),
        description: def.description.clone(),
        giver_npc: def.giver_npc.clone(),
        quest_giver_level: def.quest_giver_level,
        difficulty: quest_difficulty(&def.difficulty),
        stages,
        rewards: QuestReward {
            experience_points: xp,
            gold,
            items: def
                .rewards
                .items
                .iter()
                .map(|item| match item {
                    RewardItem::Id(id) => id.to_string(),
                    RewardItem::Name(name) => name.clone(),
                })
                .collect(),
            reputation_changes: def
                .rewards
                .reputation
                .iter()
                .map(|(key, value)| (key.clone(), *value))
                .collect(),
            special_rewards: HashMap::new(),
        },
        status: QuestStatus::Available,
        acceptance_time: None,
        completion_time: None,
        current_stage_index: 0,
        ready_notified: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gui_string_objectives_into_meaningful_entries() {
        let system = QuestSystem::new();
        let data = serde_json::json!({
            "id": 7,
            "title": "Light the Path",
            "description": "Reach the gallery with a lantern.",
            "giver_npc": "Guide",
            "objectives": [
                "Pick up the brass lantern",
                "Enter the Shadow Gallery with light"
            ],
            "rewards_gold": 40,
            "rewards_xp": 60
        });

        let quest = system
            .parse_quest_from_json(&data)
            .expect("quest should parse");
        let stage = quest.get_current_stage().expect("stage should exist");

        assert_eq!(stage.objectives.len(), 2);
        assert_eq!(stage.objectives[0].description, "Pick up the brass lantern");
        assert_eq!(stage.objectives[0].obj_type, ObjectiveType::Collect);
        assert!(stage.objectives[0].target.contains("brass lantern"));
        assert_eq!(
            stage.objectives[1].description,
            "Enter the Shadow Gallery with light"
        );
        assert_eq!(stage.objectives[1].obj_type, ObjectiveType::Explore);
        assert!(stage.objectives[1].target.contains("shadow gallery"));
        assert_eq!(quest.rewards.gold, 40);
        assert_eq!(quest.rewards.experience_points, 60);
    }

    #[test]
    fn parses_structured_objectives_with_targets_and_counts() {
        let system = QuestSystem::new();
        let data = serde_json::json!({
            "id": 9,
            "title": "Wolf Hunt",
            "description": "Thin the pack.",
            "objectives": [
                {
                    "type": "kill_monster",
                    "target_id": "wolf",
                    "description": "Defeat wolves",
                    "count": 3
                }
            ],
            "rewards": { "gold": 12, "xp": 25 }
        });

        let quest = system
            .parse_quest_from_json(&data)
            .expect("quest should parse");
        let objective = &quest.get_current_stage().expect("stage").objectives[0];

        assert_eq!(objective.obj_type, ObjectiveType::Kill);
        assert_eq!(objective.target, "wolf");
        assert_eq!(objective.description, "Defeat wolves");
        assert_eq!(objective.required_count, 3);
    }
}
