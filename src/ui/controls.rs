//! Shared native control metrics. TextEdit owns text/cursor/selection geometry.
use super::*;
pub const PAD_X: i8 = 12;
pub const PAD_Y: i8 = 8;
pub const GAP: f32 = 12.;

pub fn configure(ctx: &egui::Context) {
    let style = ctx.style();
    let row = [
        egui::TextStyle::Body,
        egui::TextStyle::Button,
        egui::TextStyle::Monospace,
    ]
    .iter()
    .map(|s| ctx.fonts_mut(|f| f.row_height(&s.resolve(&style))))
    .fold(0_f32, f32::max);
    ctx.style_mut(|s| {
        s.spacing.interact_size.y = row.ceil() + 2. * PAD_Y as f32;
        s.spacing.button_padding = egui::vec2(PAD_X as f32, PAD_Y as f32);
        s.spacing.item_spacing = egui::vec2(GAP, 8.);
    });
}

pub fn height(ui: &egui::Ui) -> f32 {
    [
        egui::TextStyle::Body,
        egui::TextStyle::Button,
        egui::TextStyle::Monospace,
    ]
    .iter()
    .map(|style| ui.text_style_height(style))
    .fold(0_f32, f32::max)
    .ceil()
        + 2. * PAD_Y as f32
}
pub fn singleline<'a>(ui: &egui::Ui, value: &'a mut dyn egui::TextBuffer) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(value)
        .font(egui::TextStyle::Body)
        .margin(egui::Margin::symmetric(PAD_X, PAD_Y))
        .vertical_align(egui::Align::Center)
        .min_size(egui::vec2(0., height(ui)))
}
pub fn cell<R>(
    ui: &mut egui::Ui,
    width: f32,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.allocate_ui_with_layout(
        egui::vec2(width, height(ui)),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(width);
            body(ui)
        },
    )
}
pub fn label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>, width: f32) {
    cell(ui, width, |ui| {
        ui.add(egui::Label::new(text).wrap());
    });
}
pub fn topology_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(PAD_X)
        .corner_radius(6.)
}

/// Reserve equal title/address/status lines; connectors attach to card centers.
pub fn topology_pair(
    ui: &mut egui::Ui,
    left: [&str; 3],
    right: [&str; 3],
    status: egui::Color32,
    caption: &str,
) {
    let line = ui.text_style_height(&egui::TextStyle::Body).ceil();
    let h = 3. * line + 2. * PAD_X as f32;
    let stacked = ui.available_width() < 360.;
    let total_h = if stacked { 2. * h + 24. } else { h };
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), total_h + line + GAP),
        egui::Sense::hover(),
    );
    let width = if stacked {
        rect.width()
    } else {
        (rect.width() * 0.42).min(270.)
    };
    let a = egui::Rect::from_min_size(rect.min, egui::vec2(width, h));
    let b = egui::Rect::from_min_size(
        if stacked {
            a.left_bottom() + egui::vec2(0., 24.)
        } else {
            egui::pos2(rect.right() - width, rect.top())
        },
        egui::vec2(width, h),
    );
    ui.painter().line_segment(
        if stacked {
            [a.center_bottom(), b.center_top()]
        } else {
            [a.right_center(), b.left_center()]
        },
        egui::Stroke::new(2_f32, palette(ui).accent),
    );
    for (r, text, border) in [(a, left, palette(ui).muted), (b, right, status)] {
        ui.painter().rect(
            r,
            6.,
            palette(ui).background,
            egui::Stroke::new(1_f32, border),
            egui::StrokeKind::Inside,
        );
        for (i, text) in text.into_iter().enumerate() {
            ui.painter().with_clip_rect(r.shrink(4.)).text(
                egui::pos2(
                    r.center().x,
                    r.top() + PAD_X as f32 + line * (i as f32 + 0.5),
                ),
                egui::Align2::CENTER_CENTER,
                text,
                egui::TextStyle::Body.resolve(ui.style()),
                palette(ui).text,
            );
        }
    }
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() - line / 2.),
        egui::Align2::CENTER_CENTER,
        caption,
        egui::TextStyle::Small.resolve(ui.style()),
        palette(ui).muted,
    );
}
