//! The Items tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_items_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.item_filter)
                    .hint_text("Filter items")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.item_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Item").clicked() {
                self.add_item();
            }
            if ui.button("Delete Item").clicked() && self.selected_item.is_some() {
                self.delete_item();
            }
        });

        ui.separator();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                // Item list
                theme::section_heading(&mut columns[0], "Items", Some("Select an item to edit"));
                let item_filter = self.item_filter.trim().to_lowercase();
                let visible_items = self
                    .adventure
                    .items
                    .iter()
                    .filter(|item| {
                        item_filter.is_empty()
                            || item.name.to_lowercase().contains(&item_filter)
                            || item.description.to_lowercase().contains(&item_filter)
                            || item.id.to_string().contains(&item_filter)
                    })
                    .count();
                columns[0].label(format!(
                    "Showing {} of {}",
                    visible_items,
                    self.adventure.items.len()
                ));
                egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                    for (i, item) in self.adventure.items.iter().enumerate() {
                        if !item_filter.is_empty()
                            && !item.name.to_lowercase().contains(&item_filter)
                            && !item.description.to_lowercase().contains(&item_filter)
                            && !item.id.to_string().contains(&item_filter)
                        {
                            continue;
                        }
                        let selected = self.selected_item == Some(i);
                        if ui
                            .add(
                                egui::Button::new(format!("{}: {}", item.id, item.name))
                                    .selected(selected),
                            )
                            .clicked()
                        {
                            self.selected_item = Some(i);
                        }
                    }
                });

                // Item editor
                theme::section_heading(&mut columns[1], "Item details", None);
                if let Some(item_idx) = self.selected_item {
                    if let Some(item) = self.adventure.items.get_mut(item_idx) {
                        let mut changed = false;
                        egui::Grid::new("item_grid")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(&mut columns[1], |ui| {
                                theme::labelled_row(ui, "ID:", |ui| {
                                    ui.add(egui::DragValue::new(&mut item.id))
                                })
                                .changed();
                                ui.end_row();

                                theme::labelled_row(ui, "Name:", |ui| {
                                    ui.text_edit_singleline(&mut item.name)
                                })
                                .changed();
                                ui.end_row();

                                let combo_label = ui.label("Type:").id;
                                egui::ComboBox::from_id_salt("item_type")
                                    .selected_text(format!("{:?}", item.item_type))
                                    .show_ui(ui, |ui: &mut egui::Ui| {
                                        for variant in [
                                            ItemType::Normal,
                                            ItemType::Weapon,
                                            ItemType::Armor,
                                            ItemType::Treasure,
                                            ItemType::Readable,
                                            ItemType::Edible,
                                            ItemType::Drinkable,
                                            ItemType::Container,
                                        ] {
                                            changed |= ui
                                                .selectable_value(
                                                    &mut item.item_type,
                                                    variant.clone(),
                                                    format!("{variant:?}"),
                                                )
                                                .changed();
                                        }
                                    })
                                    .response
                                    .labelled_by(combo_label);
                                ui.end_row();

                                theme::labelled_row(ui, "Value:", |ui| {
                                    ui.add(egui::DragValue::new(&mut item.value))
                                })
                                .changed();
                                ui.end_row();

                                theme::labelled_row(ui, "Weight:", |ui| {
                                    ui.add(egui::DragValue::new(&mut item.weight))
                                })
                                .changed();
                                ui.end_row();

                                theme::labelled_row(ui, "Location:", |ui| {
                                    ui.add(egui::DragValue::new(&mut item.location))
                                })
                                .changed();
                                ui.end_row();

                                ui.label("");
                                ui.label(
                                    "0 = carried, -1 = worn, -2 = shop, otherwise room/monster ID",
                                );
                                ui.end_row();

                                let checkbox_label_0 = ui.label("Takeable:").id;
                                changed |= ui
                                    .checkbox(&mut item.is_takeable, "")
                                    .labelled_by(checkbox_label_0)
                                    .changed();
                                ui.end_row();

                                let checkbox_label_1 = ui.label("Light source:").id;
                                changed |= ui
                                    .checkbox(&mut item.is_light_source, "")
                                    .labelled_by(checkbox_label_1)
                                    .changed();
                                ui.end_row();

                                if item.item_type == ItemType::Container {
                                    theme::labelled_row(ui, "Container capacity:", |ui| {
                                        ui.add(egui::DragValue::new(&mut item.container_capacity))
                                    })
                                    .changed();
                                    ui.end_row();
                                }

                                let checkbox_label_2 = ui.label("Is Weapon:").id;
                                changed |= ui
                                    .checkbox(&mut item.is_weapon, "")
                                    .labelled_by(checkbox_label_2)
                                    .changed();
                                ui.end_row();

                                if item.is_weapon {
                                    theme::labelled_row(ui, "Weapon Type (1-5):", |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut item.weapon_type)
                                                .range(1..=5),
                                        )
                                    })
                                    .changed();
                                    ui.end_row();

                                    theme::labelled_row(ui, "Damage Dice:", |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut item.weapon_dice)
                                                .range(1..=10),
                                        )
                                    })
                                    .changed();
                                    ui.end_row();

                                    theme::labelled_row(ui, "Damage Sides:", |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut item.weapon_sides)
                                                .range(2..=20),
                                        )
                                    })
                                    .changed();
                                    ui.end_row();
                                }

                                let checkbox_label_3 = ui.label("Is Armor:").id;
                                changed |= ui
                                    .checkbox(&mut item.is_armor, "")
                                    .labelled_by(checkbox_label_3)
                                    .changed();
                                ui.end_row();

                                if item.is_armor {
                                    theme::labelled_row(ui, "Armor Value:", |ui| {
                                        ui.add(
                                            egui::DragValue::new(&mut item.armor_value)
                                                .range(0..=20),
                                        )
                                    })
                                    .changed();
                                    ui.end_row();

                                    let checkbox_label_4 = ui.label("Wearable:").id;
                                    changed |= ui
                                        .checkbox(&mut item.is_wearable, "")
                                        .labelled_by(checkbox_label_4)
                                        .changed();
                                    ui.end_row();
                                }
                            });
                        changed |= theme::labelled_field(&mut columns[1], "Description", |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut item.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
                        })
                        .changed();
                        if changed {
                            self.mark_modified();
                        }
                    }
                } else {
                    columns[1].label("Select an item to edit");
                }
            });
        });
    }
}
