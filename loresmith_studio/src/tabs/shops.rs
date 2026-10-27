//! The Shops tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_shops_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Search")
                    .size(11.0)
                    .color(theme::muted()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.shop_filter)
                    .hint_text("Filter shops")
                    .desired_width(280.0),
            );
            if ui.button("Clear").clicked() {
                self.shop_filter.clear();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Shop").clicked() {
                self.add_shop();
            }
            if ui.button("Delete Shop").clicked() && self.selected_shop.is_some() {
                self.delete_shop();
            }
        });

        ui.separator();
        let item_names: Vec<(i32, String)> = self
            .adventure
            .items
            .iter()
            .map(|item| (item.id, item.name.clone()))
            .collect();

        ui.add_space(10.0);
        theme::card_panel(ui, |ui| {
            let content_height = 340.0;
            ui.columns(2, |columns| {
                    columns[0].set_min_height(content_height);
                    columns[1].set_min_height(content_height);
                    theme::section_heading(&mut columns[0], "Shops", Some("Select a shop to edit"));
                    let filter = self.shop_filter.trim().to_lowercase();
                    egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
                        for (index, shop) in self.adventure.shops.iter().enumerate() {
                            if !filter.is_empty()
                                && !shop.name.to_lowercase().contains(&filter)
                                && !shop.id.to_string().contains(&filter)
                            {
                                continue;
                            }
                            if ui
                                .add(
                                    egui::Button::new(format!("{}: {}", shop.id, shop.name))
                                        .selected(self.selected_shop == Some(index)),
                                )
                                .clicked()
                            {
                                self.selected_shop = Some(index);
                            }
                        }
                    });

                    theme::section_heading(&mut columns[1], "Shop details", None);
                    let Some(index) = self.selected_shop else {
                        columns[1].label("Select a shop to edit");
                        return;
                    };
                    let Some(shop) = self.adventure.shops.get_mut(index) else {
                        columns[1].label("Select a shop to edit");
                        return;
                    };
                    let mut changed = false;
                    egui::Grid::new("shop_grid")
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(&mut columns[1], |ui| {
                            ui.label("ID:");
                            changed |= ui.add(egui::DragValue::new(&mut shop.id)).changed();
                            ui.end_row();
                            ui.label("Name:");
                            changed |= ui.text_edit_singleline(&mut shop.name).changed();
                            ui.end_row();
                            ui.label("Room ID:");
                            changed |= ui.add(egui::DragValue::new(&mut shop.room_id)).changed();
                            ui.end_row();
                            ui.label("Gold:");
                            changed |= ui.add(egui::DragValue::new(&mut shop.gold)).changed();
                            ui.end_row();
                            ui.label("Buy multiplier:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut shop.buy_multiplier)
                                        .speed(0.05)
                                        .range(0.01..=10.0),
                                )
                                .changed();
                            ui.end_row();
                            ui.label("Sell multiplier:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut shop.sell_multiplier)
                                        .speed(0.05)
                                        .range(0.0..=10.0),
                                )
                                .changed();
                            ui.end_row();
                        });
                    columns[1].label(
                        egui::RichText::new(
                            "The player pays value × buy multiplier (reduced by charisma) and receives value × sell multiplier (increased by charisma). A shop never pays more than its own asking price for stocked goods.",
                        )
                        .size(11.0)
                        .color(theme::muted()),
                    );

                    columns[1].separator();
                    columns[1].label("Stock:");
                    let mut remove: Option<usize> = None;
                    for (stock_index, stock) in shop.stock.iter_mut().enumerate() {
                        columns[1].horizontal(|ui| {
                            let label = item_names
                                .iter()
                                .find(|(id, _)| *id == stock.item_id)
                                .map(|(_, name)| name.clone())
                                .unwrap_or_else(|| format!("item {}", stock.item_id));
                            egui::ComboBox::from_id_salt(("shop_stock_item", stock_index))
                                .selected_text(label)
                                .show_ui(ui, |ui| {
                                    for (id, name) in &item_names {
                                        changed |= ui
                                            .selectable_value(
                                                &mut stock.item_id,
                                                *id,
                                                name.as_str(),
                                            )
                                            .changed();
                                    }
                                });
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut stock.quantity)
                                        .prefix("qty ")
                                        .range(0..=99),
                                )
                                .changed();
                            if ui.button("Remove").clicked() {
                                remove = Some(stock_index);
                            }
                        });
                    }
                    if let Some(index) = remove {
                        shop.stock.remove(index);
                        changed = true;
                    }
                    if columns[1].button("Add Stock").clicked() {
                        let item_id = item_names.first().map(|(id, _)| *id).unwrap_or(1);
                        shop.stock.push(ShopStockDef {
                            item_id,
                            quantity: 1,
                            ..ShopStockDef::default()
                        });
                        changed = true;
                    }
                    if changed {
                        self.mark_modified();
                    }
                });
        });
    }
}
