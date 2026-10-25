//! Shared egui theme and widgets for Saga Foundry applications.
//!
//! The palette is deliberately centralized so the editor, standalone player,
//! and future tools share one visual language while each app keeps its own
//! layout and workflow.

use egui::{self, Color32, CornerRadius, FontId, Stroke, TextStyle};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }
}

struct Palette {
    bg: Color32,
    panel: Color32,
    surface: Color32,
    surface_alt: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    accent_dark: Color32,
    gold: Color32,
    danger: Color32,
    success: Color32,
    extreme: Color32,
    on_accent: Color32,
}

const DARK: Palette = Palette {
    bg: Color32::from_rgb(11, 16, 24),
    panel: Color32::from_rgb(16, 23, 33),
    surface: Color32::from_rgb(22, 31, 44),
    surface_alt: Color32::from_rgb(28, 40, 56),
    border: Color32::from_rgb(42, 58, 80),
    text: Color32::from_rgb(230, 238, 247),
    muted: Color32::from_rgb(145, 163, 184),
    accent: Color32::from_rgb(79, 209, 197),
    accent_dark: Color32::from_rgb(27, 75, 76),
    gold: Color32::from_rgb(246, 193, 119),
    danger: Color32::from_rgb(240, 113, 120),
    success: Color32::from_rgb(126, 231, 135),
    extreme: Color32::from_rgb(8, 12, 18),
    on_accent: Color32::from_rgb(6, 21, 22),
};

const LIGHT: Palette = Palette {
    bg: Color32::from_rgb(243, 246, 250),
    panel: Color32::from_rgb(255, 255, 255),
    surface: Color32::from_rgb(255, 255, 255),
    surface_alt: Color32::from_rgb(238, 242, 247),
    border: Color32::from_rgb(208, 218, 229),
    text: Color32::from_rgb(23, 33, 45),
    muted: Color32::from_rgb(88, 104, 122),
    accent: Color32::from_rgb(13, 124, 116),
    accent_dark: Color32::from_rgb(204, 251, 241),
    gold: Color32::from_rgb(180, 83, 9),
    danger: Color32::from_rgb(185, 28, 28),
    success: Color32::from_rgb(21, 128, 61),
    extreme: Color32::from_rgb(248, 250, 252),
    on_accent: Color32::from_rgb(255, 255, 255),
};

static MODE: AtomicU8 = AtomicU8::new(0);

pub fn mode() -> ThemeMode {
    if MODE.load(Ordering::Relaxed) == 1 {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    }
}

pub fn set_mode(mode: ThemeMode) {
    MODE.store(
        match mode {
            ThemeMode::Dark => 0,
            ThemeMode::Light => 1,
        },
        Ordering::Relaxed,
    );
}

fn palette() -> &'static Palette {
    match mode() {
        ThemeMode::Dark => &DARK,
        ThemeMode::Light => &LIGHT,
    }
}

pub fn bg() -> Color32 {
    palette().bg
}
pub fn surface() -> Color32 {
    palette().surface
}
pub fn surface_alt() -> Color32 {
    palette().surface_alt
}
pub fn border() -> Color32 {
    palette().border
}
pub fn text_color() -> Color32 {
    palette().text
}
pub fn muted() -> Color32 {
    palette().muted
}
pub fn accent() -> Color32 {
    palette().accent
}
pub fn accent_dark() -> Color32 {
    palette().accent_dark
}
pub fn gold() -> Color32 {
    palette().gold
}
pub fn danger() -> Color32 {
    palette().danger
}
pub fn success() -> Color32 {
    palette().success
}
pub fn extreme_bg() -> Color32 {
    palette().extreme
}
pub fn on_accent() -> Color32 {
    palette().on_accent
}

fn blend_towards(color: Color32, target: u8, amount: f32) -> Color32 {
    let mix = |channel: u8| {
        (channel as f32 + (target as f32 - channel as f32) * amount.clamp(0.0, 1.0)) as u8
    };
    Color32::from_rgb(mix(color.r()), mix(color.g()), mix(color.b()))
}

pub fn badge_fill(color: Color32) -> Color32 {
    match mode() {
        ThemeMode::Dark => color.gamma_multiply(0.16),
        ThemeMode::Light => blend_towards(color, 255, 0.88),
    }
}

pub fn badge_stroke(color: Color32) -> Color32 {
    match mode() {
        ThemeMode::Dark => color.gamma_multiply(0.55),
        ThemeMode::Light => blend_towards(color, 255, 0.55),
    }
}

pub fn apply(ctx: &egui::Context, mode: ThemeMode) {
    set_mode(mode);
    let dark = mode == ThemeMode::Dark;
    let mut style = (*ctx.style_of(if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    }))
    .clone();
    style.visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let palette = palette();
    style.visuals.override_text_color = Some(palette.text);
    style.visuals.panel_fill = palette.bg;
    style.visuals.window_fill = palette.panel;
    style.visuals.faint_bg_color = palette.surface_alt;
    style.visuals.extreme_bg_color = palette.extreme;
    style.visuals.code_bg_color = palette.extreme;
    style.visuals.hyperlink_color = palette.accent;
    style.visuals.window_stroke = Stroke::new(1.0, palette.border);
    style.visuals.window_corner_radius = CornerRadius::same(12);
    style.visuals.menu_corner_radius = CornerRadius::same(10);
    style.visuals.selection.bg_fill = palette.accent_dark;
    style.visuals.selection.stroke = Stroke::new(1.0, palette.accent);

    let radius = CornerRadius::same(8);
    style.visuals.widgets.noninteractive.bg_fill = palette.panel;
    style.visuals.widgets.noninteractive.weak_bg_fill = palette.panel;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.border);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.noninteractive.corner_radius = radius;

    style.visuals.widgets.inactive.bg_fill = palette.surface;
    style.visuals.widgets.inactive.weak_bg_fill = palette.surface;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.border);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.inactive.corner_radius = radius;

    style.visuals.widgets.hovered.bg_fill = palette.surface_alt;
    style.visuals.widgets.hovered.weak_bg_fill = palette.surface_alt;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.hovered.corner_radius = radius;

    style.visuals.widgets.active.bg_fill = palette.accent_dark;
    style.visuals.widgets.active.weak_bg_fill = palette.accent_dark;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.active.corner_radius = radius;

    style.visuals.widgets.open.bg_fill = palette.surface_alt;
    style.visuals.widgets.open.weak_bg_fill = palette.surface_alt;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, palette.text);
    style.visuals.widgets.open.corner_radius = radius;

    style.spacing.item_spacing = egui::vec2(9.0, 8.0);
    style.spacing.button_padding = egui::vec2(11.0, 6.0);
    style.spacing.interact_size = egui::vec2(32.0, 28.0);
    style.spacing.slider_width = 160.0;
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(22.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.5));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(13.0));
    ctx.set_theme(if dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });
    ctx.all_styles_mut(|target| *target = style.clone());
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(surface())
        .stroke(Stroke::new(1.0, border()))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::same(14))
}

pub fn content_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(bg())
        .inner_margin(egui::Margin::symmetric(18, 14))
}

pub fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(12.0).color(muted()));
}

pub fn section_heading(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(title)
                .size(17.0)
                .strong()
                .color(text_color()),
        );
        if let Some(subtitle) = subtitle {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(subtitle).size(12.0).color(muted()));
        }
    });
}

pub fn card<R>(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: Option<&str>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    card_frame()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            section_heading(ui, title, subtitle);
            ui.add_space(6.0);
            add_contents(ui)
        })
        .inner
}

pub fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    egui::Frame::new()
        .fill(surface_alt())
        .stroke(Stroke::new(1.0, border()))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(128.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(label).size(11.0).color(muted()));
                ui.label(egui::RichText::new(value).size(20.0).strong().color(color));
            });
        });
}

pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).strong().color(on_accent()))
        .fill(accent())
        .stroke(Stroke::new(1.0, accent()))
}

pub fn ghost_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(text_color())).fill(surface_alt())
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(badge_fill(color))
        .stroke(Stroke::new(1.0, badge_stroke(color)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(10.5).strong().color(color));
        });
}

pub fn nav_button(ui: &mut egui::Ui, selected: bool, title: &str, subtitle: &str) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 46.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let fill = if selected {
            accent_dark()
        } else if response.hovered() {
            surface_alt()
        } else {
            Color32::TRANSPARENT
        };
        painter.rect_filled(rect, CornerRadius::same(9), fill);
        if selected {
            painter.rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())),
                CornerRadius::same(2),
                accent(),
            );
        }
        painter.text(
            rect.left_top() + egui::vec2(15.0, 7.0),
            egui::Align2::LEFT_TOP,
            title,
            FontId::proportional(14.0),
            if selected { accent() } else { text_color() },
        );
        painter.text(
            rect.left_top() + egui::vec2(15.0, 25.0),
            egui::Align2::LEFT_TOP,
            subtitle,
            FontId::proportional(10.5),
            muted(),
        );
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}
