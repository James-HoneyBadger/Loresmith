//! The Info tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_info_tab(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        ui.columns(2, |columns| {
                theme::card(
                    &mut columns[0],
                    "Identity",
                    Some("What players and tooling see first"),
                    |ui| {
                        egui::Grid::new("info_identity")
                            .num_columns(2)
                            .spacing([12.0, 9.0])
                            .show(ui, |ui| {
                                theme::labelled_field(ui, "Title", |ui| ui.add(
                                        egui::TextEdit::singleline(&mut self.adventure.title)
                                            .hint_text("The name of your adventure")
                                            .desired_width(320.0),
                                    ))
                                    .changed();
                                ui.end_row();

                                theme::labelled_field(ui, "Identifier", |ui| ui.add(
                                        egui::TextEdit::singleline(&mut self.adventure.id)
                                            .hint_text("stable-id")
                                            .desired_width(320.0),
                                    ))
                                    .changed();
                                ui.end_row();

                                theme::field_label(ui, "Author");
                                let mut author = self.adventure.author.clone().unwrap_or_default();
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut author)
                                            .hint_text("Your name")
                                            .desired_width(320.0),
                                    )
                                    .changed()
                                {
                                    self.adventure.author = if author.trim().is_empty() {
                                        None
                                    } else {
                                        Some(author)
                                    };
                                    changed = true;
                                }
                                ui.end_row();
                            });
                        theme::labelled_field(ui, "Description", |ui| ui.add(
                                egui::TextEdit::multiline(&mut self.adventure.description)
                                    .hint_text("A short pitch for the adventure")
                                    .desired_rows(4)
                                    .desired_width(f32::INFINITY),
                            ))
                            .changed();
                    },
                );
                columns[0].add_space(10.0);
                theme::card(
                    &mut columns[0],
                    "Presentation",
                    Some("The opening banner and introduction"),
                    |ui| {
                        theme::labelled_field(ui, "Introduction", |ui| ui.add(
                                egui::TextEdit::multiline(&mut self.adventure.intro)
                                    .hint_text("Set the scene before the first room description…")
                                    .desired_rows(7)
                                    .desired_width(f32::INFINITY),
                            ))
                            .changed();
                    },
                );

                theme::card(
                    &mut columns[1],
                    "Starting state",
                    Some("Where the player begins and what they carry"),
                    |ui| {
                        ui.horizontal(|ui| {
                            theme::labelled_field(ui, "Start room", |ui| ui.add(
                                    egui::DragValue::new(&mut self.adventure.start_room)
                                        .range(0..=10_000),
                                ))
                                .changed();
                            if ui.add(theme::ghost_button("Use first room")).clicked()
                                && let Some(room) = self.adventure.rooms.first()
                            {
                                self.adventure.start_room = room.id;
                                changed = true;
                            }
                        });
                        ui.add_space(6.0);
                        theme::field_label(ui, "Starting inventory");
                        if self.adventure.player_start_inventory.is_empty() {
                            ui.label(
                                egui::RichText::new(
                                    "Empty. Set an item's location to 0 in the Items tab to grant it at the start.",
                                )
                                .size(11.5)
                                .color(theme::muted()),
                            );
                        } else {
                            for entry in &self.adventure.player_start_inventory {
                                let label = match entry {
                                    InventoryRef::Id(id) => self
                                        .adventure
                                        .items
                                        .iter()
                                        .find(|item| item.id == *id)
                                        .map(|item| format!("[{}] {}", item.id, item.name))
                                        .unwrap_or_else(|| format!("item {}", id)),
                                    InventoryRef::Item(item) => {
                                        format!("[{}] {}", item.id, item.name)
                                    }
                                };
                                ui.label(egui::RichText::new(format!("• {}", label)).size(12.0));
                            }
                        }
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} rooms · {} items · {} monsters · {} quests · {} shops",
                                self.adventure.rooms.len(),
                                self.adventure.items.len(),
                                self.adventure.monsters.len(),
                                self.adventure.quests.len(),
                                self.adventure.shops.len()
                            ))
                            .size(11.0)
                            .color(theme::muted()),
                        );
                    },
                );
                columns[1].add_space(10.0);
                theme::card(
                    &mut columns[1],
                    "Ending",
                    Some("Optional victory conditions and closing text"),
                    |ui| {
                        let mut has_ending = self.adventure.ending.is_some();
                        if ui.checkbox(&mut has_ending, "Enable victory conditions").changed() {
                            self.adventure.ending = if has_ending {
                                Some(EndingDef::default())
                            } else {
                                None
                            };
                            changed = true;
                        }
                        if let Some(ending) = &mut self.adventure.ending {
                            theme::labelled_field(ui, "Victory text", |ui| ui.add(
                                    egui::TextEdit::multiline(&mut ending.victory_text)
                                        .desired_rows(2)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                            theme::field_label(ui, "Defeat text");
                            let mut defeat = ending.defeat_text.clone().unwrap_or_default();
                            if ui
                                .add(
                                    egui::TextEdit::multiline(&mut defeat)
                                        .desired_rows(2)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                ending.defeat_text = if defeat.trim().is_empty() {
                                    None
                                } else {
                                    Some(defeat)
                                };
                                changed = true;
                            }
                            changed |= ui
                                .checkbox(
                                    &mut ending.conditions.all_quests,
                                    "All quests must be completed",
                                )
                                .changed();

                            theme::field_label(ui, "Required room IDs (comma separated)");
                            let mut rooms_text = ending
                                .conditions
                                .rooms
                                .iter()
                                .map(|id| id.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut rooms_text)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                ending.conditions.rooms = rooms_text
                                    .split(',')
                                    .filter_map(|value| value.trim().parse::<i32>().ok())
                                    .collect();
                                changed = true;
                            }

                            theme::field_label(ui, "Required quest IDs (comma separated)");
                            let mut quests_text = ending
                                .conditions
                                .quests
                                .iter()
                                .map(|id| id.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut quests_text)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                ending.conditions.quests = quests_text
                                    .split(',')
                                    .filter_map(|value| value.trim().parse::<i32>().ok())
                                    .collect();
                                changed = true;
                            }

                            theme::field_label(ui, "Required items (names, comma separated)");
                            let mut items_text = ending.conditions.items.join(", ");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut items_text)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                ending.conditions.items = items_text
                                    .split(',')
                                    .map(str::trim)
                                    .filter(|value| !value.is_empty())
                                    .map(str::to_string)
                                    .collect();
                                changed = true;
                            }
                        }
                    },
                );
            });
        if changed {
            self.mark_modified();
        }
    }
}
