//! The Rooms tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_rooms_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.room_filter)
                    .hint_text("Filter by name, description or ID")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.room_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Room").clicked() {
                self.add_room();
            }
            if ui.button("Delete Room").clicked() && self.selected_room.is_some() {
                self.delete_room();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Room list
                theme::section_heading(&mut columns[0], "Rooms", Some("Select a room to edit"));
                let room_filter = self.room_filter.trim().to_lowercase();
                let visible_rooms = self
                    .adventure
                    .rooms
                    .iter()
                    .filter(|room| {
                        room_filter.is_empty()
                            || room.name.to_lowercase().contains(&room_filter)
                            || room.description.to_lowercase().contains(&room_filter)
                            || room.id.to_string().contains(&room_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_rooms,
                    self.adventure.rooms.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, room) in self.adventure.rooms.iter().enumerate() {
                        if !room_filter.is_empty()
                            && !room.name.to_lowercase().contains(&room_filter)
                            && !room.description.to_lowercase().contains(&room_filter)
                            && !room.id.to_string().contains(&room_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_room == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", room.id, room.name))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_room = Some(i);
                        }
                    }
                });

                // Room editor
                theme::section_heading(&mut columns[1], "Room details", None);
                if let Some(room_idx) = self.selected_room {
                    if let Some(room) = self.adventure.rooms.get_mut(room_idx) {
                        let mut changed = false;
                        egui::Grid::new("room_grid")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(&mut columns[1], |ui| {
                                changed |= theme::labelled_row(ui, "ID:", |ui| {
                                    ui.add(egui::DragValue::new(&mut room.id))
                                })
                                .changed();
                                ui.end_row();

                                changed |= theme::labelled_row(ui, "Name:", |ui| {
                                    ui.text_edit_singleline(&mut room.name)
                                })
                                .changed();
                                ui.end_row();

                                let dark_label = ui.label("Dark:").id;
                                changed |= ui
                                    .checkbox(&mut room.is_dark, "")
                                    .labelled_by(dark_label)
                                    .changed();
                                ui.end_row();
                            });
                        changed |= theme::labelled_field(&mut columns[1], "Description", |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut room.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
                        })
                        .changed();

                        columns[1].separator();
                        columns[1].label("Exits:");
                        let exit_dirs: Vec<String> = room.exits.keys().cloned().collect();
                        let mut remove_dir: Option<String> = None;
                        for direction in &exit_dirs {
                            if let Some(exit) = room.exits.get_mut(direction) {
                                let mut is_locked = exit.lock().is_some_and(|lock| lock.locked);
                                let mut key_text = exit
                                    .lock()
                                    .and_then(|lock| lock.key.clone())
                                    .unwrap_or_default();
                                columns[1].horizontal(|ui| {
                                    ui.label(direction.as_str());
                                    ui.label("\u{2192}");
                                    if ui
                                        .add(egui::DragValue::new(exit.destination_mut()))
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                    if ui.checkbox(&mut is_locked, "locked").changed() {
                                        exit.set_locked(
                                            is_locked,
                                            if is_locked {
                                                Some(key_text.clone())
                                            } else {
                                                None
                                            },
                                        );
                                        changed = true;
                                    }
                                    if is_locked
                                        && ui
                                            .add(
                                                egui::TextEdit::singleline(&mut key_text)
                                                    .hint_text("key item"),
                                            )
                                            .changed()
                                    {
                                        exit.set_locked(true, Some(key_text.clone()));
                                        changed = true;
                                    }
                                    if ui.button("Remove").clicked() {
                                        remove_dir = Some(direction.clone());
                                    }
                                });
                            }
                        }
                        if let Some(dir) = remove_dir {
                            room.exits.remove(&dir);
                            changed = true;
                        }
                        columns[1].horizontal(|ui| {
                            let direction_label = ui.label("New exit:").id;
                            egui::ComboBox::from_id_salt("add_exit_dir")
                                .selected_text(if self.new_exit_direction.is_empty() {
                                    "direction"
                                } else {
                                    &self.new_exit_direction
                                })
                                .show_ui(ui, |ui| {
                                    for d in &["north", "south", "east", "west", "up", "down"] {
                                        ui.selectable_value(
                                            &mut self.new_exit_direction,
                                            d.to_string(),
                                            *d,
                                        );
                                    }
                                })
                                .response
                                .labelled_by(direction_label);
                            ui.add(egui::DragValue::new(&mut self.new_exit_target).prefix("room "));
                            if ui.button("Add Exit").clicked()
                                && !self.new_exit_direction.is_empty()
                            {
                                room.exits.insert(
                                    self.new_exit_direction.clone(),
                                    ExitDef::Plain(self.new_exit_target),
                                );
                                changed = true;
                            }
                        });

                        columns[1].separator();
                        columns[1].label("Items in this room (IDs, comma separated):");
                        let mut items_text = room
                            .items
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if columns[1]
                            .add(
                                egui::TextEdit::singleline(&mut items_text)
                                    .hint_text("item IDs, comma separated"),
                            )
                            .changed()
                        {
                            room.items = items_text
                                .split(',')
                                .filter_map(|value| value.trim().parse::<i32>().ok())
                                .collect();
                            changed = true;
                        }

                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select a room to edit");
                }
            });
        });
    }
}
