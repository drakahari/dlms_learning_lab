//! Opt-in native capture evidence. No model, grade, progress or viewport mutation.
//! Geometry comes from each actual pre-laid Label galley; credit requires graphics.
use super::*;
use std::sync::Arc;

pub(super) struct TextLayout {
    pub response: egui::Response,
    pub galley: Arc<egui::Galley>,
    pub clip: egui::Rect,
}
#[derive(Clone)]
struct Part {
    galley: Arc<egui::Galley>,
    rect: egui::Rect,
    clip: egui::Rect,
}
#[derive(Clone)]
struct Layout {
    source: String,
    texts: Vec<String>,
    frame: u64,
    pass: u64,
    viewport: egui::ViewportId,
    screen: egui::Rect,
    zoom: f32,
    pixels: f32,
    parts: Vec<Part>,
}
fn request_key() -> egui::Id {
    egui::Id::new("assignment-layout-request")
}
fn layout_key() -> egui::Id {
    egui::Id::new("assignment-layout-evidence")
}
pub(in crate::ui) fn request(ctx: &egui::Context, s: &Scenario) {
    let source = s.id.clone();
    ctx.data_mut(|d| {
        if d.get_temp::<String>(request_key()).as_deref() != Some(source.as_str()) {
            d.remove::<Layout>(layout_key());
            d.insert_temp(request_key(), source);
        }
    });
}
pub(super) fn requested(ctx: &egui::Context, s: &Scenario) -> bool {
    ctx.data(|d| d.get_temp::<String>(request_key()).as_deref() == Some(s.id.as_str()))
}
pub(super) fn record(ui: &egui::Ui, s: &Scenario, parts: Vec<TextLayout>) {
    let texts = assignment_texts(s)
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(
        parts.len(),
        texts.len(),
        "Every actual assignment label is recorded"
    );
    let value = Layout {
        source: s.id.clone(),
        texts,
        frame: ui.ctx().cumulative_frame_nr(),
        pass: ui.ctx().cumulative_pass_nr(),
        viewport: ui.ctx().viewport_id(),
        screen: ui.ctx().content_rect(),
        zoom: ui.ctx().zoom_factor(),
        pixels: ui.ctx().pixels_per_point(),
        parts: parts
            .into_iter()
            .map(|p| Part {
                galley: p.galley,
                rect: p.response.rect,
                clip: p.clip,
            })
            .collect(),
    };
    // Never enter another Context API inside its data lock.
    ui.ctx().data_mut(|d| d.insert_temp(layout_key(), value));
}
#[derive(Clone)]
pub(in crate::ui) struct Rows {
    pub rows: Vec<egui::Rect>,
    pub clips: Vec<egui::Rect>,
    pub keys: Vec<(usize, usize)>,
    pub witnessed: Vec<bool>,
    pub clip: egui::Rect,
}
impl Rows {
    pub fn all_witnessed(&self) -> bool {
        self.witnessed.iter().all(|v| *v)
    }
}
/// Select a real row for each capture, using vertical distance rather than
/// assuming headings, prose and inter-requirement gaps all have one line height.
/// The first capture aligns its first row to the viewport top; the last keeps
/// the existing centered endpoint and full-row clearance.
pub(in crate::ui) fn slice_target(rows: &Rows, slice: usize, count: usize) -> usize {
    assert!(count >= 2 && slice < count && !rows.rows.is_empty());
    if slice == 0 {
        return 0;
    }
    if slice + 1 == count {
        return rows.rows.len() - 1;
    }
    let first_center = rows.rows[0].top() + rows.clip.height() / 2.;
    let final_center = rows.rows.last().unwrap().center().y;
    let wanted =
        first_center + (final_center - first_center).max(0.) * slice as f32 / (count - 1) as f32;
    rows.rows
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (a.center().y - wanted)
                .abs()
                .total_cmp(&(b.center().y - wanted).abs())
        })
        .unwrap()
        .0
}
#[derive(Clone)]
pub(in crate::ui) struct Proof {
    keys: Vec<(usize, usize)>,
    seen: Vec<bool>,
}
impl Proof {
    pub fn first_missing(&self) -> Option<usize> {
        self.seen.iter().position(|v| !*v)
    }
    pub fn complete(&self) -> bool {
        self.seen.iter().all(|v| *v)
    }
}
pub(in crate::ui) fn observe(ctx: &egui::Context, key: egui::Id, rows: &Rows) -> Proof {
    let mut proof = ctx
        .data(|d| d.get_temp::<Proof>(key))
        .unwrap_or_else(|| Proof {
            keys: rows.keys.clone(),
            seen: vec![false; rows.keys.len()],
        });
    assert_eq!(
        proof.keys, rows.keys,
        "Complete assignment layout stays stable in a settled capture setting"
    );
    for (seen, actual) in proof.seen.iter_mut().zip(&rows.witnessed) {
        *seen |= *actual;
    }
    ctx.data_mut(|d| d.insert_temp(key, proof.clone()));
    proof
}
fn flatten<'a>(
    shape: &'a egui::Shape,
    clip: egui::Rect,
    out: &mut Vec<(egui::Rect, &'a egui::Shape)>,
) {
    if let egui::Shape::Vec(shapes) = shape {
        for s in shapes {
            flatten(s, clip, out);
        }
    } else {
        out.push((clip, shape));
    }
}
/// Call after rendering, within the same egui pass. Initial request waits for a
/// fresh render; previous-frame/pass/width/zoom/source evidence is never accepted.
pub(in crate::ui) fn collect(
    ctx: &egui::Context,
    s: &Scenario,
    visible: egui::Rect,
) -> Option<Rows> {
    request(ctx, s);
    // Transient sizing/discard passes are not observable capture evidence.
    // Return no rows or credit; the next ordinary rendered pass must prove them.
    if !visible.is_finite() || !visible.is_positive() || ctx.will_discard() {
        return None;
    }
    let layout = ctx.data(|d| d.get_temp::<Layout>(layout_key()))?;
    let current_texts = assignment_texts(s);
    if layout.source != s.id
        || layout.texts.iter().map(String::as_str).collect::<Vec<_>>() != current_texts
        || layout.frame != ctx.cumulative_frame_nr()
        || layout.pass != ctx.cumulative_pass_nr()
        || layout.viewport != ctx.viewport_id()
        || layout.screen != ctx.content_rect()
        || layout.zoom != ctx.zoom_factor()
        || layout.pixels != ctx.pixels_per_point()
    {
        return None;
    }
    let tolerance = 1. / ctx.pixels_per_point();
    let mut result = Rows {
        rows: vec![],
        clips: vec![],
        keys: vec![],
        witnessed: vec![],
        clip: visible,
    };
    for (part_index, part) in layout.parts.iter().enumerate() {
        assert!(
            !part.galley.elided,
            "Essential assignment text must never be elided"
        );
        assert_eq!(part.galley.text(), current_texts[part_index]);
        let mut count = 0;
        for (row_index, row) in part
            .galley
            .rows
            .iter()
            .filter(|r| !r.text().trim().is_empty())
            .enumerate()
        {
            result
                .rows
                .push(row.rect().translate(part.rect.min.to_vec2()));
            result.clips.push(part.clip.intersect(visible));
            result.keys.push((part_index, row_index));
            result.witnessed.push(false);
            count += 1;
        }
        assert!(
            count > 0,
            "Each essential assignment label contains actual rows"
        );
    }
    assert!(
        !result.rows.is_empty() && result.rows.len() <= 128,
        "Bounded complete assignment rows"
    );
    // Inspect actual current paint only. No Context methods inside graphics lock.
    ctx.graphics(|graphics| {
        let Some(list) = graphics.get(egui::LayerId::background()) else { return; };
        let mut shapes = vec![];
        for entry in list.all_entries() { flatten(&entry.shape, entry.clip_rect, &mut shapes); }
        for (row_index, &(part_index, local_row)) in result.keys.iter().enumerate() {
            let part = &layout.parts[part_index];
            for (shape_index, (clip, shape)) in shapes.iter().enumerate() {
                let egui::Shape::Text(text) = shape else { continue; };
                if !Arc::ptr_eq(&text.galley, &part.galley) || text.galley.text() != current_texts[part_index]
                    || text.pos.distance(part.rect.min) > tolerance || text.opacity_factor <= 0. || text.angle != 0.
                { continue; }
                let row = text.galley.rows.iter().filter(|r| !r.text().trim().is_empty()).nth(local_row).unwrap();
                let actual = row.rect().translate(text.pos.to_vec2());
                let own_clip = result.clips[row_index].intersect(*clip);
                let vertices = &row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()];
                if vertices.is_empty() || !own_clip.expand(tolerance).contains_rect(actual) { continue; }
                let mut glyph_bounds = egui::Rect::NOTHING;
                let mut opaque_glyphs = true;
                for vertex in vertices {
                    let color = text.override_text_color.unwrap_or_else(|| {
                        if vertex.color == egui::Color32::PLACEHOLDER { text.fallback_color } else { vertex.color }
                    });
                    opaque_glyphs &= color.a() > 0;
                    glyph_bounds.extend_with(vertex.pos + row.pos.to_vec2() + text.pos.to_vec2());
                }
                if !opaque_glyphs || !glyph_bounds.is_positive() || !own_clip.expand(tolerance).contains_rect(glyph_bounds) { continue; }
                let covered = shapes[shape_index+1..].iter().any(|(clip, shape)| {
                    matches!(shape, egui::Shape::Rect(rect) if rect.fill.a() == 255 && clip.intersect(rect.rect).intersects(glyph_bounds))
                });
                if !covered { result.witnessed[row_index] = true; }
            }
        }
    });
    Some(result)
}

/// Independent final FullOutput readback for capture tests. The required set
/// remains the complete real-layout inventory; absent/cropped labels earn no credit.
#[cfg(test)]
pub(in crate::ui) fn output_evidence(
    ctx: &egui::Context,
    s: &Scenario,
    layout: &Rows,
    output: &egui::FullOutput,
) -> Rows {
    let texts = assignment_texts(s);
    assert!(
        layout.clip.is_finite() && layout.clip.is_positive(),
        "Actual capture requires an observed positive finite main viewport"
    );
    assert!(output.pixels_per_point.is_finite() && output.pixels_per_point > 0.);
    let tolerance = 1. / output.pixels_per_point;
    let body = egui::TextStyle::Body.resolve(&ctx.style());
    let mono = egui::TextStyle::Monospace.resolve(&ctx.style());
    assert_eq!(
        layout
            .keys
            .iter()
            .map(|(piece, _)| *piece)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        texts.len(),
        "Complete goal and requirement labels, including entirely culled ones, are required"
    );
    let mut actual = layout.clone();
    actual.witnessed.fill(false);
    let mut shapes = vec![];
    for shape in &output.shapes {
        flatten(&shape.shape, shape.clip_rect, &mut shapes);
    }
    for (index, &(piece, line)) in layout.keys.iter().enumerate() {
        let required = layout.rows[index];
        let own = layout.clips[index].intersect(layout.clip);
        assert!(
            required.left() >= own.left() - tolerance
                && required.right() <= own.right() + tolerance,
            "Every real goal/requirement row fits its own horizontal clip"
        );
        for (at, (clip, shape)) in shapes.iter().enumerate() {
            let egui::Shape::Text(text) = shape else {
                continue;
            };
            if text.galley.text() != texts[piece] {
                continue;
            }
            let Some(row) = text
                .galley
                .rows
                .iter()
                .filter(|r| !r.text().trim().is_empty())
                .nth(line)
            else {
                continue;
            };
            let rect = row.rect().translate(text.pos.to_vec2());
            if (rect.min - required.min).length() > tolerance
                || (rect.max - required.max).length() > tolerance
            {
                continue;
            }
            assert!(
                !text.galley.elided && text.angle == 0. && text.opacity_factor > 0.,
                "Actual captured task text remains unelided, upright and visible"
            );
            assert!(!text.galley.job.sections.is_empty());
            for section in &text.galley.job.sections {
                if piece == 0 {
                    assert_eq!(section.format.font_id.family, body.family);
                    assert_eq!(section.format.font_id.size, 21.);
                } else {
                    assert!(
                        section.format.font_id == body || section.format.font_id == mono,
                        "Requirements retain normal Body/intentional Monospace; no font shrink"
                    );
                }
            }
            let effective = own.intersect(*clip);
            if !effective.expand(tolerance).contains_rect(rect) {
                continue;
            }
            let vertices = &row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()];
            if vertices.is_empty() {
                continue;
            }
            let mut glyphs = egui::Rect::NOTHING;
            let mut visible = true;
            for vertex in vertices {
                let color = text.override_text_color.unwrap_or_else(|| {
                    if vertex.color == egui::Color32::PLACEHOLDER {
                        text.fallback_color
                    } else {
                        vertex.color
                    }
                });
                visible &= color.a() > 0;
                glyphs.extend_with(vertex.pos + row.pos.to_vec2() + text.pos.to_vec2());
            }
            if !visible
                || !glyphs.is_positive()
                || !effective.expand(tolerance).contains_rect(glyphs)
            {
                continue;
            }
            let covered=shapes[at+1..].iter().any(|(clip,shape)|matches!(shape,egui::Shape::Rect(rect)if rect.fill.a()==255 && clip.intersect(rect.rect).intersects(glyphs)));
            if !covered {
                actual.witnessed[index] = true;
            }
        }
    }
    actual
}
#[cfg(test)]
pub(in crate::ui) fn output_proof_key(ctx: &egui::Context, s: &Scenario) -> egui::Id {
    egui::Id::new((
        "final-output-assignment-proof",
        s.id.as_str(),
        ctx.zoom_factor().to_bits(),
        ctx.content_rect().width().to_bits(),
        ctx.style().visuals.dark_mode,
    ))
}
#[cfg(test)]
pub(in crate::ui) fn assert_output_edge(
    ctx: &egui::Context,
    s: &Scenario,
    rows: &Rows,
    output: &egui::FullOutput,
    last: bool,
    clearance: bool,
) -> Rows {
    let actual = output_evidence(ctx, s, rows, output);
    let index = if last { actual.rows.len() - 1 } else { 0 };
    assert!(
        actual.witnessed[index],
        "Actual final capture output paints the complete first/final required glyph row inside its own clip without opaque overpaint"
    );
    let rect = actual.rows[index];
    let clip = actual.clips[index].intersect(actual.clip);
    let tolerance = 1. / output.pixels_per_point;
    if last && clearance {
        assert!(
            rect.bottom() + rect.height() <= clip.bottom() + tolerance,
            "Final task row retains a full actual text-row clearance above edge scrollbars"
        );
        if !actual.all_witnessed() {
            assert!(
                (rect.center().y - clip.center().y).abs() <= tolerance,
                "Long-task final row remains centered in the actual clip"
            );
            assert!(
                rect.height() * 2. < clip.height(),
                "Final task row has observed clearance from scrollbar overpaint"
            );
        }
    }
    actual
}

#[cfg(test)]
mod unavailable_capture_tests {
    use super::*;
    fn source() -> Scenario {
        crate::ui::tests::validated_ui_catalog().0[0].clone()
    }
    fn input() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1100., 800.),
            )),
            ..Default::default()
        }
    }
    #[test]
    fn unavailable_assignment_viewports_never_produce_rows_or_capture_credit() {
        let ctx = egui::Context::default();
        let s = source();
        request(&ctx, &s);
        let mut accepted = false;
        for _ in 0..3 {
            let _ = ctx.run(input(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    super::super::assignment(ui, &s);
                });
                for invalid in [
                    egui::Rect::NOTHING,
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO),
                    egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(5., 5.)),
                ] {
                    assert!(
                        collect(ctx, &s, invalid).is_none(),
                        "No invented rows from zero, negative or nonfinite viewport"
                    );
                }
                if !ctx.will_discard() {
                    let rows = collect(ctx, &s, ctx.content_rect())
                        .expect("Real positive viewport still yields actual label layout");
                    assert!(rows.witnessed.iter().any(|v| *v));
                    accepted = true;
                }
            });
        }
        assert!(accepted);
    }
    #[test]
    fn discarded_assignment_pass_never_counts_until_actual_repaint() {
        let ctx = egui::Context::default();
        ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(2).unwrap());
        let s = source();
        request(&ctx, &s);
        let mut passes = 0;
        let mut accepted = false;
        let _ = ctx.run(input(), |ctx| {
            passes += 1;
            egui::CentralPanel::default().show(ctx, |ui| {
                super::super::assignment(ui, &s);
            });
            if passes == 1 {
                ctx.request_discard("Actual unavailable capture regression");
                assert!(ctx.will_discard());
                assert!(
                    collect(ctx, &s, ctx.content_rect()).is_none(),
                    "Discarded paint cannot count as seen"
                );
            } else {
                let rows = collect(ctx, &s, ctx.content_rect())
                    .expect("Actual second pass must independently render current layout");
                assert!(rows.witnessed.iter().any(|v| *v));
                accepted = true;
            }
        });
        assert_eq!(passes, 2);
        assert!(accepted);
    }
}
