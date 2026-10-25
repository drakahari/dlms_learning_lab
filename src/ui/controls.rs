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
pub fn button_width(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let color = palette(ui).text;
    ui.fonts_mut(|f| f.layout_no_wrap(text.into(), font, color).size().x)
        .ceil()
        + 2. * ui.spacing().button_padding.x
}
pub fn singleline<'a>(ui: &egui::Ui, value: &'a mut dyn egui::TextBuffer) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(value)
        .font(egui::TextStyle::Body)
        .margin(egui::Margin::symmetric(PAD_X, PAD_Y))
        .vertical_align(egui::Align::Center)
        .min_size(egui::vec2(0., height(ui)))
}
/// Fit a dropdown in the remaining part of a wrapped row, including long values.
/// available_width() permits a fresh whole row and can overrun adjacent labels.
pub fn dropdown(ui: &egui::Ui, id: impl std::hash::Hash, maximum: f32) -> egui::ComboBox {
    egui::ComboBox::from_id_salt(id)
        .width(ui.available_size_before_wrap().x.min(maximum).max(0.))
        .wrap_mode(egui::TextWrapMode::Truncate)
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
/// Call after popup options render: scrolling clips Y but can inherit window X.
/// The used content rectangle supplies the actual popup's horizontal bounds.
pub fn popup_content_clip(ui: &egui::Ui) -> egui::Rect {
    ui.clip_rect().intersect(ui.min_rect())
}
/// Existing native form column; all related rows reserve the same label width.
pub fn form_label_width(ui: &egui::Ui) -> f32 {
    (ui.available_width() * 0.4).clamp(90., 165.)
}
pub fn label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>, width: f32) {
    cell(ui, width, |ui| {
        ui.add(egui::Label::new(text).wrap());
    });
}
/// Reserve the label column, then give the value the actual remaining space.
/// Unlike a stateful Grid for each single row, this layout needs no invisible
/// measurement pass when columns reflow, so it does not surrender field focus.
pub fn form_row<R>(
    ui: &mut egui::Ui,
    text: impl Into<egui::WidgetText>,
    value: impl FnOnce(&mut egui::Ui, f32) -> R,
) -> egui::InnerResponse<R> {
    let right = ui.next_widget_position().x + ui.available_width();
    let labels = form_label_width(ui);
    let minimum_value = ui.text_style_height(&egui::TextStyle::Body) * 6. + 2. * PAD_X as f32;
    if ui.available_width() < labels + ui.spacing().item_spacing.x + minimum_value {
        ui.vertical(|ui| {
            ui.add(egui::Label::new(text).wrap());
            value(ui, ui.available_width())
        })
    } else {
        ui.horizontal(|ui| {
            label(ui, text, labels);
            let width = ui
                .available_width()
                .min(right - ui.next_widget_position().x)
                .max(1.);
            value(ui, width)
        })
    }
}
/// Read-only table cells keep body text readable without reserving an input's click target.
pub fn table_height(ui: &egui::Ui) -> f32 {
    ui.text_style_height(&egui::TextStyle::Body).ceil() + 8.
}
pub fn table_text(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>, width: f32) {
    let row_height = table_height(ui);
    ui.allocate_ui_with_layout(
        egui::vec2(width, row_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(width);
            ui.add(egui::Label::new(text).wrap());
        },
    );
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
    let description = format!(
        "Network diagram. First device: {}. Second device: {}. Connection: {caption}.",
        left.join("; "),
        right.join("; ")
    );
    let line = ui.text_style_height(&egui::TextStyle::Body).ceil();
    let stacked = ui.available_width() < 360.;
    let width = if stacked {
        ui.available_width()
    } else {
        (ui.available_width() * 0.42).min(270.)
    };
    let font = egui::TextStyle::Body.resolve(ui.style());
    let rows: Vec<Vec<_>> = [left, right]
        .into_iter()
        .map(|texts| {
            texts
                .into_iter()
                .map(|text| {
                    ui.painter().layout(
                        text.into(),
                        font.clone(),
                        palette(ui).text,
                        (width - 2. * PAD_X as f32).max(1.),
                    )
                })
                .collect()
        })
        .collect();
    let row_heights: Vec<_> = (0..3)
        .map(|i| rows[0][i].size().y.max(rows[1][i].size().y).max(line))
        .collect();
    let content_height: f32 = row_heights.iter().sum();
    let h = content_height + 2. * PAD_X as f32;
    let total_h = if stacked { 2. * h + 24. } else { h };
    let caption_galley = ui.painter().layout(
        caption.into(),
        egui::TextStyle::Small.resolve(ui.style()),
        palette(ui).muted,
        ui.available_width(),
    );
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(
            ui.available_width(),
            total_h + caption_galley.size().y.max(line) + GAP,
        ),
        egui::Sense::hover(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), &description)
    });
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
    for ((r, galleys), border) in [(a, &rows[0]), (b, &rows[1])]
        .into_iter()
        .zip([palette(ui).muted, status])
    {
        ui.painter().rect(
            r,
            6.,
            palette(ui).background,
            egui::Stroke::new(1_f32, border),
            egui::StrokeKind::Inside,
        );
        let mut y = r.top() + PAD_X as f32;
        for (galley, row_height) in galleys.iter().zip(&row_heights) {
            ui.painter().with_clip_rect(r.shrink(4.)).galley(
                egui::pos2(
                    r.center().x - galley.size().x / 2.,
                    y + (row_height - galley.size().y) / 2.,
                ),
                galley.clone(),
                palette(ui).text,
            );
            y += row_height;
        }
    }
    ui.painter().galley(
        egui::pos2(
            rect.center().x - caption_galley.size().x / 2.,
            rect.top() + total_h + GAP,
        ),
        caption_galley,
        palette(ui).muted,
    );
}

/// A small hub-and-spoke diagram keeps each device in one equal card.
/// Narrow or non-star topologies use the caller's truthful pair/list fallback.
pub fn topology_star(
    ui: &mut egui::Ui,
    hub: [&str; 3],
    peers: &[[&str; 3]],
    caption: &str,
) -> bool {
    if !(2..=3).contains(&peers.len()) || ui.available_width() < 360. {
        return false;
    }
    let description = format!(
        "Network diagram. Central device: {}. Connected devices: {}. Physical connections: {caption}.",
        hub.join("; "),
        peers
            .iter()
            .map(|p| p.join("; "))
            .collect::<Vec<_>>()
            .join(". ")
    );
    let line = ui.text_style_height(&egui::TextStyle::Body).ceil();
    let width =
        ((ui.available_width() - GAP * (peers.len() - 1) as f32) / peers.len() as f32).min(270.);
    let font = egui::TextStyle::Body.resolve(ui.style());
    let text: Vec<Vec<_>> = std::iter::once(hub)
        .chain(peers.iter().copied())
        .map(|side| {
            side.into_iter()
                .map(|s| {
                    ui.painter().layout(
                        s.into(),
                        font.clone(),
                        palette(ui).text,
                        (width - 2. * PAD_X as f32).max(1.),
                    )
                })
                .collect()
        })
        .collect();
    let heights: Vec<_> = (0..3)
        .map(|i| text.iter().map(|s| s[i].size().y).fold(line, f32::max))
        .collect();
    let h = heights.iter().sum::<f32>() + 2. * PAD_X as f32;
    let cap = ui.painter().layout(
        caption.into(),
        egui::TextStyle::Small.resolve(ui.style()),
        palette(ui).muted,
        ui.available_width(),
    );
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(
            ui.available_width(),
            2. * h + 24. + GAP + cap.size().y.max(line),
        ),
        egui::Sense::hover(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), &description)
    });
    let hub_rect = egui::Rect::from_min_size(
        egui::pos2(rect.center().x - width / 2., rect.top()),
        egui::vec2(width, h),
    );
    let row_width = width * peers.len() as f32 + GAP * (peers.len() - 1) as f32;
    let mut cards = vec![hub_rect];
    for i in 0..peers.len() {
        let peer = egui::Rect::from_min_size(
            egui::pos2(
                rect.center().x - row_width / 2. + i as f32 * (width + GAP),
                rect.top() + h + 24.,
            ),
            egui::vec2(width, h),
        );
        ui.painter().line_segment(
            [hub_rect.center_bottom(), peer.center_top()],
            egui::Stroke::new(2_f32, palette(ui).accent),
        );
        cards.push(peer);
    }
    for (r, rows) in cards.iter().zip(text) {
        ui.painter().rect(
            *r,
            6.,
            palette(ui).background,
            egui::Stroke::new(1_f32, palette(ui).muted),
            egui::StrokeKind::Inside,
        );
        let mut y = r.top() + PAD_X as f32;
        for (g, row_height) in rows.iter().zip(&heights) {
            ui.painter().with_clip_rect(r.shrink(4.)).galley(
                egui::pos2(
                    r.center().x - g.size().x / 2.,
                    y + (row_height - g.size().y) / 2.,
                ),
                g.clone(),
                palette(ui).text,
            );
            y += row_height;
        }
    }
    ui.painter().galley(
        egui::pos2(
            rect.center().x - cap.size().x / 2.,
            rect.top() + 2. * h + 24. + GAP,
        ),
        cap,
        palette(ui).muted,
    );
    true
}

/// Reveal the result of an explicit check in the nearest existing scroll area.
/// It adds no permanent panel and never scrolls merely because state rerenders.
pub fn feedback<R>(
    ui: &mut egui::Ui,
    reveal: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let result = ui.vertical(body);
    if reveal && result.response.rect.height() > 0. {
        let align = if result.response.rect.height() > ui.clip_rect().height() {
            egui::Align::TOP
        } else {
            egui::Align::BOTTOM
        };
        result.response.scroll_to_me(Some(align));
    }
    result
}

/// Equal measured cards for a simple three-device path. Narrow views stack cleanly.
pub fn topology_chain(ui: &mut egui::Ui, cards: &[[&str; 3]], caption: &str) -> egui::Response {
    let roles = palette(ui);
    let line = ui.text_style_height(&egui::TextStyle::Body).ceil();
    let count = cards.len().max(1);
    let stacked = ui.available_width() < line * 9. * count as f32;
    let gap = 2. * GAP;
    let width = if stacked {
        ui.available_width()
    } else {
        ((ui.available_width() - gap * (count - 1) as f32) / count as f32).min(270.)
    };
    let font = egui::TextStyle::Body.resolve(ui.style());
    let rows: Vec<Vec<_>> = cards
        .iter()
        .map(|texts| {
            texts
                .iter()
                .map(|text| {
                    ui.painter().layout(
                        (*text).into(),
                        font.clone(),
                        roles.text,
                        (width - 2. * PAD_X as f32).max(1.),
                    )
                })
                .collect()
        })
        .collect();
    let heights: Vec<_> = (0..3)
        .map(|row| rows.iter().map(|g| g[row].size().y).fold(line, f32::max))
        .collect();
    let height = heights.iter().sum::<f32>() + 2. * PAD_X as f32;
    let graph_height = if stacked {
        height * count as f32 + gap * (count - 1) as f32
    } else {
        height
    };
    let description = format!(
        "Network diagram. {}. Connections: {caption}.",
        cards
            .iter()
            .map(|c| c.join("; "))
            .collect::<Vec<_>>()
            .join(". ")
    );
    let caption_galley = ui.painter().layout(
        caption.into(),
        egui::TextStyle::Small.resolve(ui.style()),
        roles.muted,
        ui.available_width(),
    );
    let (area, response) = ui.allocate_exact_size(
        egui::vec2(
            ui.available_width(),
            graph_height + GAP + caption_galley.size().y.max(line),
        ),
        egui::Sense::hover(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), &description)
    });
    let mut rects = Vec::new();
    let horizontal_start = area.center().x - (count as f32 * width + (count - 1) as f32 * gap) / 2.;
    for index in 0..cards.len() {
        let origin = if stacked {
            area.min + egui::vec2(0., index as f32 * (height + gap))
        } else {
            egui::pos2(horizontal_start + index as f32 * (width + gap), area.top())
        };
        rects.push(egui::Rect::from_min_size(origin, egui::vec2(width, height)));
    }
    for pair in rects.windows(2) {
        ui.painter().line_segment(
            if stacked {
                [pair[0].center_bottom(), pair[1].center_top()]
            } else {
                [pair[0].right_center(), pair[1].left_center()]
            },
            egui::Stroke::new(2_f32, roles.accent),
        );
    }
    for (rect, galleys) in rects.iter().zip(&rows) {
        ui.painter().rect(
            *rect,
            6.,
            roles.background,
            egui::Stroke::new(1_f32, roles.muted),
            egui::StrokeKind::Inside,
        );
        let mut y = rect.top() + PAD_X as f32;
        for (galley, h) in galleys.iter().zip(&heights) {
            ui.painter().with_clip_rect(rect.shrink(2.)).galley(
                egui::pos2(
                    rect.center().x - galley.size().x / 2.,
                    y + (h - galley.size().y) / 2.,
                ),
                galley.clone(),
                roles.text,
            );
            y += h;
        }
    }
    ui.painter().galley(
        egui::pos2(
            area.center().x - caption_galley.size().x / 2.,
            area.top() + graph_height + GAP,
        ),
        caption_galley,
        roles.muted,
    );
    response
}
