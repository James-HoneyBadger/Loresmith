//! The Quests tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_quests_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.quest_filter)
                    .hint_text("Filter quests")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.quest_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Quest").clicked() {
                self.add_quest();
            }
            if ui.button("Delete Quest").clicked() && self.selected_quest.is_some() {
                self.delete_quest();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Quest list
                theme::section_heading(&mut columns[0], "Quests", Some("Select a quest to edit"));
                let quest_filter = self.quest_filter.trim().to_lowercase();
                let visible_quests = self
                    .adventure
                    .quests
                    .iter()
                    .filter(|quest| {
                        quest_filter.is_empty()
                            || quest.title.to_lowercase().contains(&quest_filter)
                            || quest.description.to_lowercase().contains(&quest_filter)
                            || quest.id.to_string().contains(&quest_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_quests,
                    self.adventure.quests.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, quest) in self.adventure.quests.iter().enumerate() {
                        if !quest_filter.is_empty()
                            && !quest.title.to_lowercase().contains(&quest_filter)
                            && !quest.description.to_lowercase().contains(&quest_filter)
                            && !quest.id.to_string().contains(&quest_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_quest == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", quest.id, quest.title))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_quest = Some(i);
                        }
                    }
                });

                // Quest editor
                theme::section_heading(&mut columns[1], "Quest details", None);
                if let Some(quest_idx) = self.selected_quest {
                    if let Some(quest) = self.adventure.quests.get_mut(quest_idx) {
                        let mut changed = false;
                        columns[1].horizontal(|ui| {
                            theme::labelled_row(ui, "ID:", |ui| {
                                ui.add(egui::DragValue::new(&mut quest.id))
                            })
                            .changed();
                            theme::labelled_row(ui, "Title:", |ui| {
                                ui.text_edit_singleline(&mut quest.title)
                            })
                            .changed();
                        });
                        columns[1].horizontal(|ui| {
                            theme::labelled_row(ui, "Giver NPC:", |ui| {
                                ui.text_edit_singleline(&mut quest.giver_npc)
                            })
                            .changed();
                            let combo_label = ui.label("Difficulty:").id;
                            egui::ComboBox::from_id_salt("quest_difficulty")
                                .selected_text(format!("{:?}", quest.difficulty))
                                .show_ui(ui, |ui| {
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Easy,
                                            "Easy",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Moderate,
                                            "Moderate",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Challenging,
                                            "Challenging",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut quest.difficulty,
                                            loresmith_rs::QuestDifficultyDef::Hard,
                                            "Hard",
                                        )
                                        .changed();
                                })
                                .response
                                .labelled_by(combo_label);
                        });
                        let description_label = columns[1].label("Description:").id;
                        changed |= columns[1]
                            .text_edit_multiline(&mut quest.description)
                            .labelled_by(description_label)
                            .changed();

                        if quest.stages.is_empty() {
                            quest.stages.push(StageDef {
                                stage_id: "main".to_string(),
                                stage_number: 1,
                                title: "Main Objectives".to_string(),
                                description: quest.description.clone(),
                                objectives: quest.objectives.clone(),
                                stage_reward_xp: quest.rewards.xp,
                                extra: Default::default(),
                            });
                        }

                        let mut remove_stage: Option<usize> = None;
                        for (stage_idx, stage) in quest.stages.iter_mut().enumerate() {
                            columns[1].group(|ui| {
                                ui.horizontal(|ui| {
                                    let stage_label =
                                        ui.label(format!("Stage {}", stage.stage_number)).id;
                                    changed |= ui
                                        .text_edit_singleline(&mut stage.title)
                                        .labelled_by(stage_label)
                                        .changed();
                                    theme::labelled_row(ui, "Stage XP:", |ui| {
                                        ui.add(egui::DragValue::new(&mut stage.stage_reward_xp))
                                    })
                                    .changed();
                                    if ui.button("Remove stage").clicked() {
                                        remove_stage = Some(stage_idx);
                                    }
                                });
                                let objectives_label = ui.label("Objectives:").id;
                                let mut remove_objective: Option<usize> = None;
                                for (objective_idx, objective) in
                                    stage.objectives.iter_mut().enumerate()
                                {
                                    let objective =
                                        ensure_structured_objective(objective, objective_idx);
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::from_id_salt(format!(
                                            "objective_kind_{}_{}",
                                            stage_idx, objective_idx
                                        ))
                                        .selected_text(objective_kind_label(objective.kind))
                                        .show_ui(ui, |ui| {
                                            for kind in [
                                                ObjectiveKind::KillMonster,
                                                ObjectiveKind::CollectItem,
                                                ObjectiveKind::ReachRoom,
                                                ObjectiveKind::TalkToNpc,
                                                ObjectiveKind::Discover,
                                            ] {
                                                changed |= ui
                                                    .selectable_value(
                                                        &mut objective.kind,
                                                        kind,
                                                        objective_kind_label(kind),
                                                    )
                                                    .changed();
                                            }
                                        })
                                        .response
                                        .labelled_by(objectives_label);
                                        let mut target = objective
                                            .target
                                            .clone()
                                            .unwrap_or_else(|| objective.target_text());
                                        let target_width =
                                            (ui.available_width() - 210.0).max(120.0);
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut target)
                                                    .hint_text("target")
                                                    .desired_width(target_width),
                                            )
                                            .changed()
                                        {
                                            objective.target_id = Some(
                                                target
                                                    .parse::<i64>()
                                                    .map(|number| {
                                                        serde_json::Value::Number(number.into())
                                                    })
                                                    .unwrap_or_else(|_| {
                                                        serde_json::Value::String(target.clone())
                                                    }),
                                            );
                                            objective.target = Some(target);
                                            changed = true;
                                        }
                                        changed |= ui
                                            .add(
                                                egui::DragValue::new(&mut objective.count)
                                                    .range(1..=999),
                                            )
                                            .labelled_by(objectives_label)
                                            .changed();
                                        if ui.button("Remove").clicked() {
                                            remove_objective = Some(objective_idx);
                                        }
                                    });
                                    changed |= ui
                                        .add(
                                            egui::TextEdit::singleline(&mut objective.description)
                                                .hint_text("Objective description")
                                                .desired_width(f32::INFINITY),
                                        )
                                        .changed();
                                    ui.add_space(4.0);
                                }
                                if let Some(index) = remove_objective {
                                    stage.objectives.remove(index);
                                    changed = true;
                                }
                                if ui.button("Add Objective").clicked() {
                                    stage.objectives.push(ObjectiveDef::Structured(
                                        StructuredObjective {
                                            kind: ObjectiveKind::KillMonster,
                                            description: "Defeat an enemy".to_string(),
                                            target: Some("enemy".to_string()),
                                            target_id: Some(serde_json::Value::String(
                                                "enemy".to_string(),
                                            )),
                                            count: 1,
                                            ..StructuredObjective::default()
                                        },
                                    ));
                                    changed = true;
                                }
                            });
                        }
                        if let Some(index) = remove_stage
                            && quest.stages.len() > 1
                        {
                            quest.stages.remove(index);
                            changed = true;
                        }
                        if columns[1].button("Add Stage").clicked() {
                            let number = quest.stages.len() as i32 + 1;
                            quest.stages.push(StageDef {
                                stage_id: format!("stage_{}", number),
                                stage_number: number,
                                title: format!("Stage {}", number),
                                ..StageDef::default()
                            });
                            changed = true;
                        }

                        // Mirror the first stage into the legacy objectives field.
                        if let Some(first) = quest.stages.first() {
                            quest.objectives = first.objectives.clone();
                        }

                        columns[1].separator();
                        columns[1].horizontal(|ui| {
                            theme::labelled_row(ui, "Gold reward:", |ui| {
                                ui.add(egui::DragValue::new(&mut quest.rewards.gold))
                            })
                            .changed();
                            theme::labelled_row(ui, "XP reward:", |ui| {
                                ui.add(egui::DragValue::new(&mut quest.rewards.xp))
                            })
                            .changed();
                        });
                        let reward_items_label = columns[1]
                            .label("Reward items (IDs or names, comma separated):")
                            .id;
                        let mut reward_items = quest
                            .rewards
                            .items
                            .iter()
                            .map(|item| match item {
                                RewardItem::Id(id) => id.to_string(),
                                RewardItem::Name(name) => name.clone(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        if columns[1]
                            .text_edit_singleline(&mut reward_items)
                            .labelled_by(reward_items_label)
                            .changed()
                        {
                            quest.rewards.items = reward_items
                                .split(',')
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(|value| {
                                    value
                                        .parse::<i32>()
                                        .map(RewardItem::Id)
                                        .unwrap_or_else(|_| RewardItem::Name(value.to_string()))
                                })
                                .collect();
                            changed = true;
                        }
                        let reputation_label = columns[1]
                            .label("Reputation rewards (Faction: amount, comma separated):")
                            .id;
                        let mut reputation_text = quest
                            .rewards
                            .reputation
                            .iter()
                            .map(|(faction, amount)| format!("{}: {}", faction, amount))
                            .collect::<Vec<_>>()
                            .join(", ");
                        if columns[1]
                            .text_edit_singleline(&mut reputation_text)
                            .labelled_by(reputation_label)
                            .changed()
                        {
                            quest.rewards.reputation.clear();
                            for entry in reputation_text.split(',') {
                                if let Some((faction, amount)) = entry.split_once(':')
                                    && let Ok(amount) = amount.trim().parse::<i32>()
                                {
                                    quest
                                        .rewards
                                        .reputation
                                        .insert(faction.trim().to_string(), amount);
                                }
                            }
                            changed = true;
                        }
                        quest.rewards_gold = Some(quest.rewards.gold);
                        quest.rewards_xp = Some(quest.rewards.xp);
                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select a quest to edit");
                }
            });
        });
    }
}
