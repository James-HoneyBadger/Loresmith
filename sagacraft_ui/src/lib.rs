//! Shared egui theme and widgets for Saga Foundry applications.
//!
//! The palette is deliberately centralized so the editor, standalone player,
//! and future tools share one visual language while each app keeps its own
//! layout and workflow.

use egui::{self, Color32, CornerRadius, FontId, Stroke, StrokeKind, TextStyle};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    Windows2000,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::Dark, ThemeMode::Light, ThemeMode::Windows2000];

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
            ThemeMode::Windows2000 => "Windows 2000",
        }
    }

    /// Stable key used by `eframe` storage.
    pub fn key(self) -> &'static str {
        match self {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
            ThemeMode::Windows2000 => "windows2000",
        }
    }

    pub fn from_key(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "light" => ThemeMode::Light,
            "windows2000" | "windows 2000" | "windows-2000" | "win2000" | "classic" => {
                ThemeMode::Windows2000
            }
            _ => ThemeMode::Dark,
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

const CLASSIC: Palette = Palette {
    bg: Color32::from_rgb(192, 192, 192),
    panel: Color32::from_rgb(192, 192, 192),
    surface: Color32::from_rgb(192, 192, 192),
    surface_alt: Color32::from_rgb(212, 208, 200),
    border: Color32::from_rgb(128, 128, 128),
    text: Color32::from_rgb(0, 0, 0),
    muted: Color32::from_rgb(64, 64, 64),
    accent: Color32::from_rgb(0, 0, 128),
    accent_dark: Color32::from_rgb(0, 0, 128),
    gold: Color32::from_rgb(128, 128, 0),
    danger: Color32::from_rgb(128, 0, 0),
    success: Color32::from_rgb(0, 128, 0),
    extreme: Color32::from_rgb(255, 255, 255),
    on_accent: Color32::from_rgb(255, 255, 255),
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
    match MODE.load(Ordering::Relaxed) {
        1 => ThemeMode::Light,
        2 => ThemeMode::Windows2000,
        _ => ThemeMode::Dark,
    }
}

pub fn set_mode(mode: ThemeMode) {
    MODE.store(
        match mode {
            ThemeMode::Dark => 0,
            ThemeMode::Light => 1,
            ThemeMode::Windows2000 => 2,
        },
        Ordering::Relaxed,
    );
}

/// True when the classic Windows 2000 desktop treatment is active.
pub fn is_windows_2000() -> bool {
    mode() == ThemeMode::Windows2000
}

fn palette() -> &'static Palette {
    match mode() {
        ThemeMode::Dark => &DARK,
        ThemeMode::Light => &LIGHT,
        ThemeMode::Windows2000 => &CLASSIC,
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
        ThemeMode::Windows2000 => Color32::from_rgb(212, 208, 200),
    }
}

pub fn badge_stroke(color: Color32) -> Color32 {
    match mode() {
        ThemeMode::Dark => color.gamma_multiply(0.55),
        ThemeMode::Light => blend_towards(color, 255, 0.55),
        ThemeMode::Windows2000 => Color32::from_rgb(128, 128, 128),
    }
}

/// Classic 3-D face color used by Windows 2000 chrome.
pub fn classic_face() -> Color32 {
    Color32::from_rgb(192, 192, 192)
}

/// Highlight color of a classic raised bevel.
pub fn classic_highlight() -> Color32 {
    Color32::from_rgb(255, 255, 255)
}

/// Shadow color of a classic raised bevel.
pub fn classic_shadow() -> Color32 {
    Color32::from_rgb(128, 128, 128)
}

/// Darkest outline used around classic windows and buttons.
pub fn classic_outline() -> Color32 {
    Color32::from_rgb(64, 64, 64)
}

/// Title-bar gradient colors, left to right.
pub fn caption_colors() -> (Color32, Color32) {
    (
        Color32::from_rgb(10, 36, 106),
        Color32::from_rgb(120, 168, 220),
    )
}

pub fn apply(ctx: &egui::Context, mode: ThemeMode) {
    set_mode(mode);
    let classic = mode == ThemeMode::Windows2000;
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

    if classic {
        apply_windows_2000(&mut style);
        ctx.set_fonts(classic_font_definitions());
    } else {
        ctx.set_fonts(egui::FontDefinitions::default());
        apply_modern(&mut style, palette);
    }

    ctx.set_theme(if dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });
    ctx.all_styles_mut(|target| *target = style.clone());
}

fn apply_modern(style: &mut egui::Style, palette: &Palette) {
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
}

fn apply_windows_2000(style: &mut egui::Style) {
    let face = classic_face();
    let highlight = classic_highlight();
    let shadow = classic_shadow();
    let outline = classic_outline();
    let navy = Color32::from_rgb(0, 0, 128);
    let zero = CornerRadius::ZERO;

    style.visuals.dark_mode = false;
    style.visuals.panel_fill = face;
    style.visuals.window_fill = face;
    style.visuals.window_stroke = Stroke::new(1.0, outline);
    style.visuals.window_corner_radius = zero;
    style.visuals.window_shadow = egui::Shadow::NONE;
    style.visuals.popup_shadow = egui::Shadow::NONE;
    style.visuals.menu_corner_radius = zero;
    style.visuals.faint_bg_color = Color32::from_rgb(212, 208, 200);
    style.visuals.extreme_bg_color = Color32::WHITE;
    style.visuals.text_edit_bg_color = Some(Color32::WHITE);
    style.visuals.code_bg_color = Color32::WHITE;
    style.visuals.hyperlink_color = Color32::from_rgb(0, 0, 238);
    style.visuals.selection.bg_fill = navy;
    style.visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.button_frame = true;
    style.visuals.striped = false;
    style.visuals.indent_has_left_vline = false;
    style.visuals.interact_cursor = None;

    style.visuals.widgets.noninteractive.bg_fill = face;
    style.visuals.widgets.noninteractive.weak_bg_fill = face;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, shadow);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    style.visuals.widgets.noninteractive.corner_radius = zero;
    style.visuals.widgets.noninteractive.expansion = 0.0;

    style.visuals.widgets.inactive.bg_fill = face;
    style.visuals.widgets.inactive.weak_bg_fill = face;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, shadow);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    style.visuals.widgets.inactive.corner_radius = zero;
    style.visuals.widgets.inactive.expansion = 0.0;

    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(212, 208, 200);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(212, 208, 200);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, highlight);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    style.visuals.widgets.hovered.corner_radius = zero;
    style.visuals.widgets.hovered.expansion = 0.0;

    style.visuals.widgets.active.bg_fill = Color32::from_rgb(212, 208, 200);
    style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(212, 208, 200);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, shadow);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    style.visuals.widgets.active.corner_radius = zero;
    style.visuals.widgets.active.expansion = 0.0;

    // Open controls are the classic navy highlight, including egui window
    // captions and open menu buttons.
    style.visuals.widgets.open.bg_fill = navy;
    style.visuals.widgets.open.weak_bg_fill = navy;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, navy);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.open.corner_radius = zero;
    style.visuals.widgets.open.expansion = 0.0;

    style.spacing.item_spacing = egui::vec2(4.0, 4.0);
    style.spacing.button_padding = egui::vec2(10.0, 3.0);
    style.spacing.interact_size = egui::vec2(24.0, 22.0);
    style.spacing.slider_width = 140.0;
    style.spacing.icon_width = 12.0;
    style.spacing.icon_spacing = 4.0;
    style.spacing.window_margin = egui::Margin::same(4);
    style.spacing.menu_margin = egui::Margin::same(3);
    style.spacing.indent = 14.0;
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(12.0));
}

/// Prefer Tahoma when the host provides it; otherwise keep egui's default
/// family so the classic layout still renders everywhere.
fn classic_font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    for path in [
        "C:\\Windows\\Fonts\\tahoma.ttf",
        "C:\\WINNT\\Fonts\\tahoma.ttf",
        "/usr/share/fonts/truetype/msttcorefonts/tahoma.ttf",
        "/usr/share/fonts/truetype/tahoma/tahoma.ttf",
        "/Library/Fonts/Tahoma.ttf",
        "/System/Library/Fonts/Supplemental/Tahoma.ttf",
    ] {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        fonts.font_data.insert(
            "tahoma".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "tahoma".to_owned());
        }
        break;
    }
    fonts
}

pub fn card_frame() -> egui::Frame {
    if is_windows_2000() {
        egui::Frame::new()
            .fill(classic_face())
            .corner_radius(CornerRadius::ZERO)
            .inner_margin(egui::Margin::same(8))
    } else {
        egui::Frame::new()
            .fill(surface())
            .stroke(Stroke::new(1.0, border()))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(egui::Margin::same(14))
    }
}

pub fn content_frame() -> egui::Frame {
    if is_windows_2000() {
        egui::Frame::new()
            .fill(bg())
            .inner_margin(egui::Margin::same(4))
    } else {
        egui::Frame::new()
            .fill(bg())
            .inner_margin(egui::Margin::symmetric(18, 14))
    }
}

/// Draw a classic two-pixel 3-D bevel around `rect`.
pub fn paint_bevel(painter: &egui::Painter, rect: egui::Rect, raised: bool) {
    let light = classic_highlight();
    let mid = Color32::from_rgb(223, 223, 223);
    let shadow = classic_shadow();
    let dark = classic_outline();
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, dark),
        StrokeKind::Inside,
    );

    let inner = rect.shrink(1.0);
    let (outer_light, outer_dark) = if raised {
        (light, shadow)
    } else {
        (shadow, light)
    };
    let (inner_light, inner_dark) = if raised { (mid, dark) } else { (dark, mid) };
    for (area, light, dark) in [
        (inner, outer_light, outer_dark),
        (inner.shrink(1.0), inner_light, inner_dark),
    ] {
        if area.width() <= 1.0 || area.height() <= 1.0 {
            continue;
        }
        painter.line_segment([area.left_top(), area.right_top()], Stroke::new(1.0, light));
        painter.line_segment(
            [area.left_top(), area.left_bottom()],
            Stroke::new(1.0, light),
        );
        painter.line_segment(
            [area.left_bottom(), area.right_bottom()],
            Stroke::new(1.0, dark),
        );
        painter.line_segment(
            [area.right_top(), area.right_bottom()],
            Stroke::new(1.0, dark),
        );
    }
}

fn paint_caption_gradient(painter: &egui::Painter, rect: egui::Rect) {
    let (left, right) = caption_colors();
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.colored_vertex(rect.left_bottom(), left);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

fn paint_app_icon(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, CornerRadius::ZERO, Color32::WHITE);
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, classic_outline()),
        StrokeKind::Inside,
    );
    let page = rect.shrink(3.0);
    painter.rect_filled(page, CornerRadius::ZERO, Color32::from_rgb(10, 36, 106));
    painter.hline(
        page.x_range(),
        page.center().y,
        Stroke::new(1.0, Color32::from_rgb(246, 193, 119)),
    );
    painter.vline(
        page.center().x,
        page.y_range(),
        Stroke::new(1.0, Color32::WHITE),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptionAction {
    Minimize,
    Maximize,
    Close,
}

/// Paint a Windows 2000-style caption bar across the available width and
/// return any caption button the user clicked.
pub fn caption_bar(ui: &mut egui::Ui, title: &str, maximized: bool) -> Option<CaptionAction> {
    if !is_windows_2000() {
        return None;
    }
    let height = ui.available_height().max(22.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    paint_caption_gradient(painter, rect);
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, classic_outline()),
        StrokeKind::Inside,
    );

    let icon_size = 16.0;
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 3.0, rect.center().y - icon_size / 2.0),
        egui::vec2(icon_size, icon_size),
    );
    paint_app_icon(painter, icon_rect);

    let button_width = 18.0;
    let button_height = (rect.height() - 6.0).max(12.0);
    let button_top = rect.center().y - button_height / 2.0;
    let mut button_x = rect.right() - 5.0 - button_width;

    let title_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 24.0, rect.top()),
        egui::pos2(button_x - 4.0, rect.bottom()),
    );
    painter.with_clip_rect(title_rect).text(
        egui::pos2(title_rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        title,
        FontId::proportional(12.0),
        Color32::WHITE,
    );

    let mut clicked = None;
    let drag = ui.interact(
        title_rect,
        ui.auto_id_with("caption_drag"),
        egui::Sense::click_and_drag(),
    );
    if drag.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        clicked = Some(CaptionAction::Maximize);
    }
    // Drawn right-to-left: Close, Maximize, Minimize.
    for action in [
        CaptionAction::Close,
        CaptionAction::Maximize,
        CaptionAction::Minimize,
    ] {
        let button_rect = egui::Rect::from_min_size(
            egui::pos2(button_x, button_top),
            egui::vec2(button_width, button_height),
        );
        let response = ui
            .interact(
                button_rect,
                ui.auto_id_with(("caption", action)),
                egui::Sense::click(),
            )
            .on_hover_text(match action {
                CaptionAction::Minimize => "Minimize",
                CaptionAction::Maximize if maximized => "Restore",
                CaptionAction::Maximize => "Maximize",
                CaptionAction::Close => "Close",
            });
        let pressed = response.is_pointer_button_down_on();
        let button_painter = ui.painter();
        button_painter.rect_filled(
            button_rect,
            CornerRadius::ZERO,
            if response.hovered() {
                Color32::from_rgb(212, 208, 200)
            } else {
                classic_face()
            },
        );
        paint_bevel(button_painter, button_rect, !pressed);

        let symbol = classic_outline();
        let stroke = Stroke::new(1.0, symbol);
        match action {
            CaptionAction::Minimize => {
                button_painter.line_segment(
                    [
                        egui::pos2(button_rect.left() + 4.0, button_rect.bottom() - 4.0),
                        egui::pos2(button_rect.right() - 4.0, button_rect.bottom() - 4.0),
                    ],
                    stroke,
                );
            }
            CaptionAction::Maximize => {
                let box_rect = egui::Rect::from_min_max(
                    egui::pos2(button_rect.left() + 4.0, button_rect.top() + 4.0),
                    egui::pos2(button_rect.right() - 4.0, button_rect.bottom() - 4.0),
                );
                button_painter.rect_stroke(
                    box_rect,
                    CornerRadius::ZERO,
                    stroke,
                    StrokeKind::Inside,
                );
                if maximized {
                    button_painter.rect_stroke(
                        box_rect.translate(egui::vec2(1.0, 1.0)),
                        CornerRadius::ZERO,
                        stroke,
                        StrokeKind::Inside,
                    );
                }
            }
            CaptionAction::Close => {
                button_painter.line_segment(
                    [
                        egui::pos2(button_rect.left() + 4.0, button_rect.top() + 4.0),
                        egui::pos2(button_rect.right() - 4.0, button_rect.bottom() - 4.0),
                    ],
                    stroke,
                );
                button_painter.line_segment(
                    [
                        egui::pos2(button_rect.right() - 4.0, button_rect.top() + 4.0),
                        egui::pos2(button_rect.left() + 4.0, button_rect.bottom() - 4.0),
                    ],
                    stroke,
                );
            }
        }
        if response.clicked() {
            clicked = Some(action);
        }
        button_x -= button_width + 2.0;
    }
    clicked
}

/// Invisible resize grips for the undecorated classic window frame.
pub fn window_resize_handles(ui: &mut egui::Ui) {
    if !is_windows_2000() {
        return;
    }
    use egui::{CursorIcon, Rect, Vec2};
    let rect = ui.ctx().content_rect();
    let t = 5.0;
    let handles = [
        (
            egui::ResizeDirection::NorthWest,
            Rect::from_min_size(rect.left_top(), Vec2::splat(t)),
            CursorIcon::ResizeNwSe,
        ),
        (
            egui::ResizeDirection::NorthEast,
            Rect::from_min_size(egui::pos2(rect.right() - t, rect.top()), Vec2::splat(t)),
            CursorIcon::ResizeNeSw,
        ),
        (
            egui::ResizeDirection::SouthWest,
            Rect::from_min_size(egui::pos2(rect.left(), rect.bottom() - t), Vec2::splat(t)),
            CursorIcon::ResizeNeSw,
        ),
        (
            egui::ResizeDirection::SouthEast,
            Rect::from_min_size(
                egui::pos2(rect.right() - t, rect.bottom() - t),
                Vec2::splat(t),
            ),
            CursorIcon::ResizeNwSe,
        ),
        (
            egui::ResizeDirection::North,
            Rect::from_min_max(
                egui::pos2(rect.left() + t, rect.top()),
                egui::pos2(rect.right() - t, rect.top() + t),
            ),
            CursorIcon::ResizeVertical,
        ),
        (
            egui::ResizeDirection::South,
            Rect::from_min_max(
                egui::pos2(rect.left() + t, rect.bottom() - t),
                egui::pos2(rect.right() - t, rect.bottom()),
            ),
            CursorIcon::ResizeVertical,
        ),
        (
            egui::ResizeDirection::West,
            Rect::from_min_max(
                egui::pos2(rect.left(), rect.top() + t),
                egui::pos2(rect.left() + t, rect.bottom() - t),
            ),
            CursorIcon::ResizeHorizontal,
        ),
        (
            egui::ResizeDirection::East,
            Rect::from_min_max(
                egui::pos2(rect.right() - t, rect.top() + t),
                egui::pos2(rect.right(), rect.bottom() - t),
            ),
            CursorIcon::ResizeHorizontal,
        ),
    ];
    for (index, (direction, area, cursor)) in handles.into_iter().enumerate() {
        let response = ui.interact(
            area,
            ui.auto_id_with(("window_resize", index)),
            egui::Sense::drag(),
        );
        if response.hovered() {
            ui.ctx().set_cursor_icon(cursor);
        }
        if response.drag_started() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
        }
    }
}

/// Menu styling: frameless menu-bar buttons plus the classic navy highlight
/// for hovered items in Windows 2000.
pub fn menu_style(style: &mut egui::Style) {
    egui::containers::menu::menu_style(style);
    if is_windows_2000() {
        style.spacing.button_padding = egui::vec2(6.0, 2.0);
        style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0, 0, 128);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0, 0, 128);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    }
}

/// Paint the classic menu strip background before menu items are added.
pub fn menu_background(ui: &egui::Ui) {
    if !is_windows_2000() {
        return;
    }
    let rect = ui.max_rect();
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, classic_face());
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, classic_shadow()),
    );
}

/// A recessed strip used for toolbars and the status bar.
pub fn toolbar<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let classic = is_windows_2000();
    let frame = if classic {
        egui::Frame::new()
            .fill(classic_face())
            .inner_margin(egui::Margin::symmetric(5, 3))
    } else {
        egui::Frame::new()
            .fill(surface_alt())
            .stroke(Stroke::new(1.0, border()))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(8, 5))
    };
    let response = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(add_contents).inner
    });
    if classic {
        paint_bevel(ui.painter(), response.response.rect, true);
    }
    response.inner
}

/// A recessed status field, matching the panels along the bottom of a
/// Windows 2000 application window.
pub fn status_panel<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    if !is_windows_2000() {
        return add_contents(ui);
    }
    let response = egui::Frame::new()
        .fill(classic_face())
        .inner_margin(egui::Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui)
        });
    paint_bevel(ui.painter(), response.response.rect, false);
    response.inner
}

/// `card_frame` with the classic raised bevel painted around it.
pub fn card_panel<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let classic = is_windows_2000();
    let response = card_frame().show(ui, add_contents);
    if classic {
        paint_bevel(ui.painter(), response.response.rect, true);
    }
    response.inner
}

/// A raised grouping panel: rounded card chrome in the modern themes, a
/// classic raised grey group box in Windows 2000.
pub fn raised_panel<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let classic = is_windows_2000();
    let response = egui::Frame::new()
        .fill(surface())
        .stroke(if classic {
            Stroke::NONE
        } else {
            Stroke::new(1.0, border())
        })
        .corner_radius(if classic {
            CornerRadius::ZERO
        } else {
            CornerRadius::same(10)
        })
        .inner_margin(if classic {
            egui::Margin::same(8)
        } else {
            egui::Margin::symmetric(14, 10)
        })
        .show(ui, add_contents);
    if classic {
        paint_bevel(ui.painter(), response.response.rect, true);
    }
    response.inner
}

/// A recessed data field: rounded surface in the modern themes, a white
/// sunken field in Windows 2000.
pub fn field_panel<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let classic = is_windows_2000();
    let response = egui::Frame::new()
        .fill(if classic {
            Color32::WHITE
        } else {
            extreme_bg()
        })
        .stroke(if classic {
            Stroke::NONE
        } else {
            Stroke::new(1.0, border())
        })
        .corner_radius(if classic {
            CornerRadius::ZERO
        } else {
            CornerRadius::same(8)
        })
        .inner_margin(if classic {
            egui::Margin::same(6)
        } else {
            egui::Margin::same(10)
        })
        .show(ui, add_contents);
    if classic {
        paint_bevel(ui.painter(), response.response.rect, false);
    }
    response.inner
}

pub fn field_label(ui: &mut egui::Ui, text: &str) {
    let size = if is_windows_2000() { 11.0 } else { 12.0 };
    ui.label(egui::RichText::new(text).size(size).color(muted()));
}

pub fn section_heading(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>) {
    let classic = is_windows_2000();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(title)
                .size(if classic { 12.0 } else { 17.0 })
                .strong()
                .color(text_color()),
        );
        if let Some(subtitle) = subtitle {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(subtitle)
                    .size(if classic { 11.0 } else { 12.0 })
                    .color(muted()),
            );
        }
    });
}

pub fn card<R>(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: Option<&str>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let classic = is_windows_2000();
    let response = card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        section_heading(ui, title, subtitle);
        ui.add_space(if classic { 3.0 } else { 6.0 });
        add_contents(ui)
    });
    if classic {
        paint_bevel(ui.painter(), response.response.rect, true);
    }
    response.inner
}

pub fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    let classic = is_windows_2000();
    let response = egui::Frame::new()
        .fill(if classic {
            Color32::WHITE
        } else {
            surface_alt()
        })
        .stroke(if classic {
            Stroke::NONE
        } else {
            Stroke::new(1.0, border())
        })
        .corner_radius(if classic {
            CornerRadius::ZERO
        } else {
            CornerRadius::same(10)
        })
        .inner_margin(if classic {
            egui::Margin::symmetric(8, 4)
        } else {
            egui::Margin::symmetric(14, 10)
        })
        .show(ui, |ui| {
            ui.set_min_width(if classic { 96.0 } else { 128.0 });
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(if classic { 10.0 } else { 11.0 })
                        .color(muted()),
                );
                ui.label(
                    egui::RichText::new(value)
                        .size(if classic { 14.0 } else { 20.0 })
                        .strong()
                        .color(color),
                );
            });
        });
    if classic {
        paint_bevel(ui.painter(), response.response.rect, false);
    }
}

/// A button that renders as classic 3-D chrome in the Windows 2000 theme and
/// as the existing accent button in the modern themes.
pub struct ActionButton {
    text: &'static str,
    primary: bool,
}

impl egui::Widget for ActionButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if !is_windows_2000() {
            let text = if self.primary {
                egui::RichText::new(self.text).strong().color(on_accent())
            } else {
                egui::RichText::new(self.text).color(text_color())
            };
            return if self.primary {
                ui.add(
                    egui::Button::new(text)
                        .fill(accent())
                        .stroke(Stroke::new(1.0, accent())),
                )
            } else {
                ui.add(egui::Button::new(text).fill(surface_alt()))
            };
        }

        let text_color = if ui.is_enabled() {
            Color32::BLACK
        } else {
            Color32::from_rgb(109, 109, 109)
        };
        let response = ui.add(
            egui::Button::new(egui::RichText::new(self.text).size(12.0).color(text_color))
                .fill(classic_face())
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::ZERO),
        );
        paint_bevel(
            ui.painter(),
            response.rect,
            !response.is_pointer_button_down_on(),
        );
        if self.primary {
            // Win2k marked the default button with an extra outline inside the bevel.
            ui.painter().rect_stroke(
                response.rect.shrink(3.0),
                CornerRadius::ZERO,
                Stroke::new(1.0, classic_outline()),
                StrokeKind::Inside,
            );
        }
        if response.has_focus() {
            let focus = response.rect.shrink(4.0);
            ui.painter().rect_stroke(
                focus,
                CornerRadius::ZERO,
                Stroke::new(1.0, classic_outline()),
                StrokeKind::Inside,
            );
        }
        response
    }
}

pub fn primary_button(text: &'static str) -> ActionButton {
    ActionButton {
        text,
        primary: true,
    }
}

pub fn ghost_button(text: &'static str) -> ActionButton {
    ActionButton {
        text,
        primary: false,
    }
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    let classic = is_windows_2000();
    let response = egui::Frame::new()
        .fill(badge_fill(color))
        .stroke(Stroke::new(
            1.0,
            if classic {
                classic_shadow()
            } else {
                badge_stroke(color)
            },
        ))
        .corner_radius(if classic {
            CornerRadius::ZERO
        } else {
            CornerRadius::same(6)
        })
        .inner_margin(egui::Margin::symmetric(if classic { 5 } else { 8 }, 2))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .size(if classic { 10.0 } else { 10.5 })
                    .strong()
                    .color(color),
            );
        });
    if classic {
        paint_bevel(ui.painter(), response.response.rect, false);
    }
}

pub fn nav_button(ui: &mut egui::Ui, selected: bool, title: &str, subtitle: &str) -> bool {
    if is_windows_2000() {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let fill = if selected {
                palette().accent_dark
            } else if response.hovered() {
                Color32::from_rgb(212, 208, 200)
            } else {
                Color32::TRANSPARENT
            };
            painter.rect_filled(rect, CornerRadius::ZERO, fill);
            painter.text(
                rect.left_top() + egui::vec2(9.0, 3.0),
                egui::Align2::LEFT_TOP,
                title,
                FontId::proportional(12.0),
                if selected {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                },
            );
            painter.text(
                rect.left_top() + egui::vec2(9.0, 18.0),
                egui::Align2::LEFT_TOP,
                subtitle,
                FontId::proportional(11.0),
                if selected {
                    Color32::from_rgb(212, 208, 200)
                } else {
                    palette().muted
                },
            );
        }
        return response.clicked();
    }

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

#[cfg(test)]
mod tests {
    use super::ThemeMode;

    #[test]
    fn theme_keys_round_trip() {
        for mode in ThemeMode::ALL {
            assert_eq!(ThemeMode::from_key(mode.key()), mode);
        }
    }

    #[test]
    fn unknown_theme_keys_fall_back_to_dark() {
        assert_eq!(ThemeMode::from_key(""), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_key("solarized"), ThemeMode::Dark);
    }

    #[test]
    fn classic_aliases_are_accepted() {
        for key in ["windows2000", "Windows 2000", "win2000", "classic"] {
            assert_eq!(ThemeMode::from_key(key), ThemeMode::Windows2000);
        }
    }
}
