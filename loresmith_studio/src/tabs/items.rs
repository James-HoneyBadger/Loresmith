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
                                ui.label("ID:");
                                changed |= ui.add(egui::DragValue::new(&mut item.id)).changed();
                                ui.end_row();

                                ui.label("Name:");
                                changed |= ui.text_edit_singleline(&mut item.name).changed();
                                ui.end_row();

                                ui.label("Type:");
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
                                    });
                                ui.end_row();

                                ui.label("Value:");
                                changed |= ui.add(egui::DragValue::new(&mut item.value)).changed();
                                ui.end_row();

                                ui.label("Weight:");
                                changed |= ui.add(egui::DragValue::new(&mut item.weight)).changed();
                                ui.end_row();

                                ui.label("Location:");
                                changed |=
                                    ui.add(egui::DragValue::new(&mut item.location)).changed();
                                ui.end_row();

                                ui.label("");
                                ui.label(
                                    "0 = carried, -1 = worn, -2 = shop, otherwise room/monster ID",
                                );
                                ui.end_row();

                                ui.label("Takeable:");
                                changed |= ui.checkbox(&mut item.is_takeable, "").changed();
                                ui.end_row();

                                ui.label("Light source:");
                                changed |= ui.checkbox(&mut item.is_light_source, "").changed();
                                ui.end_row();

                                if item.item_type == ItemType::Container {
                                    ui.label("Container capacity:");
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut item.container_capacity))
                                        .changed();
                                    ui.end_row();
                                }

                                ui.label("Is Weapon:");
                                changed |= ui.checkbox(&mut item.is_weapon, "").changed();
                                ui.end_row();

                                if item.is_weapon {
                                    ui.label("Weapon Type (1-5):");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.weapon_type)
                                                .range(1..=5),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label("Damage Dice:");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.weapon_dice)
                                                .range(1..=10),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label("Damage Sides:");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.weapon_sides)
                                                .range(2..=20),
                                        )
                                        .changed();
                                    ui.end_row();
                                }

                                ui.label("Is Armor:");
                                changed |= ui.checkbox(&mut item.is_armor, "").changed();
                                ui.end_row();

                                if item.is_armor {
                                    ui.label("Armor Value:");
                                    changed |= ui
                                        .add(
                                            egui::DragValue::new(&mut item.armor_value)
                                                .range(0..=20),
                                        )
                                        .changed();
                                    ui.end_row();

                                    ui.label("Wearable:");
                                    changed |= ui.checkbox(&mut item.is_wearable, "").changed();
                                    ui.end_row();
                                }
                            });
                        theme::field_label(&mut columns[1], "Description");
                        changed |= columns[1]
                            .add(
                                egui::TextEdit::multiline(&mut item.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
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
