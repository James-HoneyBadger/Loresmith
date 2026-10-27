//! The Preview tab.

use super::*;

impl LoresmithStudio {
    pub(crate) fn show_preview_tab(&mut self, ui: &mut egui::Ui) {
        let report = self.adventure.report();
        ui.horizontal(|ui| {
            if ui.add(theme::primary_button("Copy JSON")).clicked() {
                self.copy_json_to_clipboard(ui.ctx());
            }
            if ui.add(theme::ghost_button("Show diff")).clicked() {
                self.show_json_diff();
            }
            if ui.add(theme::ghost_button("Save as JSON")).clicked() {
                self.save_adventure_as();
            }
            if ui.add(theme::ghost_button("Validate & Play")).clicked() && self.validate_adventure()
            {
                self.start_game();
                self.active_tab = Tab::Play;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (label, color) = if report.is_valid() {
                    ("VALID", theme::success())
                } else {
                    ("INVALID", theme::danger())
                };
                theme::badge(ui, label, color);
            });
        });
        ui.add_space(10.0);

        let content_height = ui.available_height().max(360.0);
        ui.columns(2, |columns| {
                columns[0].set_min_height(content_height);
                columns[1].set_min_height(content_height);
                theme::card(
                    &mut columns[0],
                    "Canonical JSON",
                    Some("schema_version 2 · normalized on save"),
                    |ui| {
                        theme::field_panel(ui, |ui| {
                                egui::ScrollArea::both()
                                    .auto_shrink([false, false])
                                    .max_height(content_height - 86.0)
                                    .show(ui, |ui| {
                                        let json = self.generate_json_preview();
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(json)
                                                    .monospace()
                                                    .size(12.0)
                                                    .color(theme::text_color()),
                                            )
                                            .wrap_mode(egui::TextWrapMode::Extend)
                                            .selectable(true),
                                        );
                                    });
                            });
                    },
                );
                theme::card(
                    &mut columns[1],
                    "Document health",
                    Some("Validation and export"),
                    |ui| {
                        let stats = [
                            ("ROOMS", self.adventure.rooms.len(), theme::accent()),
                            ("ITEMS", self.adventure.items.len(), theme::text_color()),
                            ("MONSTERS", self.adventure.monsters.len(), theme::gold()),
                            ("QUESTS", self.adventure.quests.len(), theme::success()),
                            ("SHOPS", self.adventure.shops.len(), theme::muted()),
                        ];
                        egui::Grid::new("preview_stats")
                            .num_columns(3)
                            .spacing([8.0, 8.0])
                            .show(ui, |ui| {
                                for (index, (label, value, color)) in stats.iter().enumerate() {
                                    theme::stat_card(ui, label, &value.to_string(), *color);
                                    if (index + 1).is_multiple_of(3) {
                                        ui.end_row();
                                    }
                                }
                                if !stats.len().is_multiple_of(3) {
                                    ui.end_row();
                                }
                            });
                        ui.add_space(10.0);
                        theme::field_label(ui, "VALIDATION");
                        if report.is_valid() && report.warnings.is_empty() {
                            ui.label(
                                egui::RichText::new("No errors or warnings.")
                                    .size(12.0)
                                    .color(theme::success()),
                            );
                        } else {
                            for error in &report.errors {
                                ui.label(
                                    egui::RichText::new(format!("Error: {}", error))
                                        .size(12.0)
                                        .color(theme::danger()),
                                );
                            }
                            for warning in &report.warnings {
                                ui.label(
                                    egui::RichText::new(format!("Warning: {}", warning))
                                        .size(12.0)
                                        .color(theme::gold()),
                                );
                            }
                        }
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);
                        theme::field_label(ui, "ACTIONS");
                        ui.horizontal(|ui| {
                            if ui.add(theme::ghost_button("Validate now")).clicked() {
                                self.validate_adventure();
                            }
                            if ui.add(theme::ghost_button("Copy JSON")).clicked() {
                                self.copy_json_to_clipboard(ui.ctx());
                            }
                        });
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(
                                "Saving normalizes the document, preserves unknown editor fields, and writes atomically.",
                            )
                            .size(11.0)
                            .color(theme::muted()),
                        );
                    },
                );
            });
    }
}
