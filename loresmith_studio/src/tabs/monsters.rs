//! The Monsters tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_monsters_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.monster_filter)
                    .hint_text("Filter monsters")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.monster_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Monster").clicked() {
                self.add_monster();
            }
            if ui.button("Delete Monster").clicked() && self.selected_monster.is_some() {
                self.delete_monster();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Monster list
                theme::section_heading(
                    &mut columns[0],
                    "Monsters",
                    Some("Select a monster to edit"),
                );
                let monster_filter = self.monster_filter.trim().to_lowercase();
                let visible_monsters = self
                    .adventure
                    .monsters
                    .iter()
                    .filter(|monster| {
                        monster_filter.is_empty()
                            || monster.name.to_lowercase().contains(&monster_filter)
                            || monster.description.to_lowercase().contains(&monster_filter)
                            || monster.id.to_string().contains(&monster_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_monsters,
                    self.adventure.monsters.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, monster) in self.adventure.monsters.iter().enumerate() {
                        if !monster_filter.is_empty()
                            && !monster.name.to_lowercase().contains(&monster_filter)
                            && !monster.description.to_lowercase().contains(&monster_filter)
                            && !monster.id.to_string().contains(&monster_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_monster == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", monster.id, monster.name))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_monster = Some(i);
                        }
                    }
                });

                // Monster editor
                theme::section_heading(&mut columns[1], "Monster details", None);
                if let Some(monster_idx) = self.selected_monster {
                    if let Some(monster) = self.adventure.monsters.get_mut(monster_idx) {
                        let mut changed = false;
                        egui::Grid::new("monster_grid")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(&mut columns[1], |ui| {
                                ui.label("ID:");
                                changed |= ui.add(egui::DragValue::new(&mut monster.id)).changed();
                                ui.end_row();

                                ui.label("Name:");
                                changed |= ui.text_edit_singleline(&mut monster.name).changed();
                                ui.end_row();

                                ui.label("Hardiness:");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut monster.hardiness))
                                    .changed();
                                ui.end_row();

                                ui.label("Agility:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.agility)).changed();
                                ui.end_row();

                                ui.label("Gold:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.gold)).changed();
                                ui.end_row();

                                ui.label("Room ID:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.room_id)).changed();
                                ui.end_row();

                                ui.label("Friendliness:");
                                egui::ComboBox::from_id_salt("monster_status")
                                    .selected_text(format!("{:?}", monster.friendliness))
                                    .show_ui(ui, |ui: &mut egui::Ui| {
                                        changed |= ui
                                            .selectable_value(
                                                &mut monster.friendliness,
                                                MonsterStatus::Neutral,
                                                "Neutral",
                                            )
                                            .changed();
                                        changed |= ui
                                            .selectable_value(
                                                &mut monster.friendliness,
                                                MonsterStatus::Friendly,
                                                "Friendly",
                                            )
                                            .changed();
                                        changed |= ui
                                            .selectable_value(
                                                &mut monster.friendliness,
                                                MonsterStatus::Hostile,
                                                "Hostile",
                                            )
                                            .changed();
                                    });
                                ui.end_row();

                                ui.label("Courage:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut monster.courage)).changed();
                                ui.end_row();

                                ui.label("Armor worn:");
                                changed |= ui
                                    .add(egui::DragValue::new(&mut monster.armor_worn))
                                    .changed();
                                ui.end_row();

                                ui.label("Weapon ID:");
                                let mut has_weapon = monster.weapon_id.is_some();
                                ui.horizontal(|ui| {
                                    if ui.checkbox(&mut has_weapon, "Has weapon").changed() {
                                        monster.weapon_id = if has_weapon { Some(1) } else { None };
                                        changed = true;
                                    }
                                    if let Some(weapon_id) = &mut monster.weapon_id {
                                        changed |=
                                            ui.add(egui::DragValue::new(weapon_id)).changed();
                                    }
                                });
                                ui.end_row();

                                ui.label("Loot (item IDs):");
                                let mut loot_text = monster
                                    .loot
                                    .iter()
                                    .map(|id| id.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                if ui.text_edit_singleline(&mut loot_text).changed() {
                                    monster.loot = loot_text
                                        .split(',')
                                        .filter_map(|value| value.trim().parse::<i32>().ok())
                                        .collect();
                                    changed = true;
                                }
                                ui.end_row();

                                ui.label("Dialogue:");
                                let mut has_dialogue = monster.dialogue.is_some();
                                if ui.checkbox(&mut has_dialogue, "Has dialogue").changed() {
                                    monster.dialogue = if has_dialogue {
                                        Some(DialogueDef::default())
                                    } else {
                                        None
                                    };
                                    changed = true;
                                }
                                ui.end_row();

                                if let Some(dialogue) = &mut monster.dialogue {
                                    ui.label("Greeting:");
                                    changed |=
                                        ui.text_edit_multiline(&mut dialogue.greeting).changed();
                                    ui.end_row();

                                    ui.label("Unknown topic reply:");
                                    let mut unknown = dialogue.unknown.clone().unwrap_or_default();
                                    if ui.text_edit_multiline(&mut unknown).changed() {
                                        dialogue.unknown = if unknown.trim().is_empty() {
                                            None
                                        } else {
                                            Some(unknown)
                                        };
                                        changed = true;
                                    }
                                    ui.end_row();

                                    ui.label("Topics (one per line, 'topic: reply'):");
                                    let mut topics_text = dialogue
                                        .topics
                                        .iter()
                                        .map(|(topic, reply)| format!("{}: {}", topic, reply))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    if ui.text_edit_multiline(&mut topics_text).changed() {
                                        dialogue.topics.clear();
                                        for line in topics_text.lines() {
                                            if let Some((topic, reply)) = line.split_once(':') {
                                                let topic = topic.trim();
                                                if !topic.is_empty() {
                                                    dialogue.topics.insert(
                                                        topic.to_string(),
                                                        reply.trim().to_string(),
                                                    );
                                                }
                                            }
                                        }
                                        changed = true;
                                    }
                                    ui.end_row();
                                }
                            });
                        theme::field_label(&mut columns[1], "Description");
                        changed |= columns[1]
                            .add(
                                egui::TextEdit::multiline(&mut monster.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed();
                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select a monster to edit");
                }
            });
        });
    }
}
