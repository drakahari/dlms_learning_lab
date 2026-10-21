use dlms_learning_lab::{
    catalog::{self, Filter},
    engine, ipv4,
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace as model,
};
use eframe::egui::{self, Color32, RichText};
use std::{path::PathBuf, time::Instant};
mod ipv4_workspace;
mod smoke;
#[cfg(test)]
mod tests;
mod workspace;
const ACCENT: Color32 = Color32::from_rgb(184, 165, 255);
const MUTED: Color32 = Color32::from_rgb(165, 180, 205);
const PANEL: Color32 = Color32::from_rgb(25, 34, 55);
const GREEN: Color32 = Color32::from_rgb(142, 225, 183);
const RED: Color32 = Color32::from_rgb(255, 164, 166);
const CONTROL: f32 = 36.;
#[derive(Default)]
struct Metrics {
    header: Vec<egui::Rect>,
    filters: Vec<egui::Rect>,
    card_actions: Vec<(egui::Rect, egui::Rect)>,
    pagination: bool,
    inputs: Vec<egui::Rect>,
    terminal: Option<egui::Rect>,
    output: Option<egui::Rect>,
    insert: Option<egui::Rect>,
    check: Option<egui::Rect>,
    focus: Option<egui::Id>,
    completion: bool,
    guide: bool,
    editor_save: Option<egui::Rect>,
    hint: Option<egui::Rect>,
    why: Option<egui::Rect>,
    help_visible: bool,
    help: Option<egui::Rect>,
    reset: Option<egui::Rect>,
    back: Option<egui::Rect>,
    extras: Option<egui::Rect>,
    walkthrough: Option<egui::Rect>,
    confirm_reset: Option<egui::Rect>,
}
pub struct LabApp {
    scenarios: Vec<Scenario>,
    progress: Progress,
    path: PathBuf,
    filter: Filter,
    selected: Option<usize>,
    input: String,
    checked: bool,
    reset_pending: bool,
    dirty: bool,
    storage_error: Option<String>,
    read_only: bool,
    synthetic: bool,
    synthetic_catalog: Vec<Scenario>,
    developer: bool,
    page: usize,
    scale: f32,
    focus_input: bool,
    scroll_command: bool,
    notice: Option<String>,
    metrics: Metrics,
    help_open: bool,
    extra_filters: bool,
    host_activity: bool,
    ipv4_view: Option<ipv4_workspace::Assignment>,
    smoke: bool,
    started: Instant,
    smoke_stage: usize,
    pending_events: Vec<egui::Event>,
    capture_pending: bool,
}
impl LabApp {
    pub fn new(cc: &eframe::CreationContext<'_>, smoke: bool, developer: bool) -> Self {
        Self::from_context(&cc.egui_ctx, smoke, developer)
    }
    fn from_context(ctx: &egui::Context, smoke: bool, developer: bool) -> Self {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(14, 21, 38);
        style.visuals.window_fill = PANEL;
        style.visuals.extreme_bg_color = Color32::from_rgb(9, 15, 28);
        style.visuals.faint_bg_color = PANEL;
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 48, 73);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(65, 54, 101);
        style.visuals.selection.bg_fill = Color32::from_rgb(80, 60, 135);
        style.visuals.override_text_color = Some(Color32::from_rgb(235, 239, 250));
        style.spacing.item_spacing = egui::vec2(12., 8.);
        style.spacing.button_padding = egui::vec2(14., 8.);
        style.spacing.interact_size.y = CONTROL;
        style.spacing.combo_width = 160.;
        style.spacing.scroll.floating = false;
        for (kind, size) in [
            (egui::TextStyle::Body, 16.),
            (egui::TextStyle::Button, 16.),
            (egui::TextStyle::Small, 13.),
        ] {
            style
                .text_styles
                .insert(kind, egui::FontId::proportional(size));
        }
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(15.));
        ctx.set_style(style);
        let scenarios = pack::bundled().expect("Bundled pack validates");
        let path = if smoke {
            PathBuf::from("/tmp/dlms-workspaces-smoke/progress-v1.json")
        } else {
            progress::default_path()
                .unwrap_or_else(|_| PathBuf::from("local-data/progress-v1.json"))
        };
        let fresh = || Progress {
            schema_version: 1,
            ..Default::default()
        };
        let (progress, storage_error) = if smoke {
            (fresh(), None)
        } else {
            match progress::load(&path, &scenarios) {
                Ok(p) => (p, None),
                Err(e) => (fresh(), Some(e)),
            }
        };
        let read_only = storage_error.is_some();
        Self {
            scenarios,
            progress,
            path,
            filter: Filter::default(),
            selected: None,
            input: String::new(),
            checked: false,
            reset_pending: false,
            dirty: false,
            storage_error,
            read_only,
            synthetic: false,
            synthetic_catalog: Vec::new(),
            developer,
            page: 0,
            scale: 1.,
            focus_input: false,
            scroll_command: false,
            notice: None,
            metrics: Metrics::default(),
            help_open: false,
            extra_filters: false,
            host_activity: false,
            ipv4_view: None,
            smoke,
            started: Instant::now(),
            smoke_stage: 0,
            pending_events: Vec::new(),
            capture_pending: false,
        }
    }
    fn launch(&mut self, i: usize) {
        self.progress.clock = self.progress.clock.saturating_add(1);
        let clock = self.progress.clock;
        let s = &self.scenarios[i];
        let a = self
            .progress
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, clock));
        model::prepare(a);
        a.visited = clock;
        let data = a.workspace.as_ref().unwrap();
        self.input = data.command.clone();
        self.checked = data.checked;
        self.selected = Some(i);
        self.reset_pending = false;
        self.focus_input = true;
        self.scroll_command = false;
        self.notice = None;
        self.help_open = false;
        self.host_activity = false;
        self.load_ipv4_view(i);
        self.dirty = true;
    }
    fn load_ipv4_view(&mut self, i: usize) {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        self.ipv4_view = if let Some(original) = ipv4::Exercise::from_state(&a.state) {
            let practice = a.ipv4.as_ref().is_some_and(|l| l.use_variant);
            let e = if practice {
                a.ipv4
                    .as_ref()
                    .and_then(|l| l.variant.clone())
                    .unwrap_or(original)
            } else {
                original
            };
            let w = a.workspace.as_ref().unwrap();
            Some(ipv4_workspace::Assignment::from_saved(
                e,
                if practice {
                    w.practice_check.clone()
                } else {
                    w.ipv4_check.clone()
                },
                a.hints,
            ))
        } else {
            None
        };
    }
    fn reset_attempt(&mut self, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        *a = Attempt::new(s, a.visited);
        model::prepare(a);
        self.input.clear();
        self.checked = false;
        self.reset_pending = false;
        self.notice = None;
        self.focus_input = true;
        self.scroll_command = false;
        self.help_open = false;
        self.host_activity = false;
        self.load_ipv4_view(i);
        self.dirty = true;
    }
    fn save(&mut self) {
        if self.dirty && !self.read_only {
            match progress::save(&self.path, &self.progress) {
                Ok(()) => {
                    self.dirty = false;
                    self.storage_error = None
                }
                Err(e) => self.storage_error = Some(format!("Progress not saved: {e}")),
            }
        }
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("brand")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(18, 26, 44))
                    .inner_margin(egui::Margin::symmetric(20, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.metrics.header.push(
                        ui.label(
                            RichText::new("DLMS Learning Lab")
                                .strong()
                                .size(20.)
                                .color(ACCENT),
                        )
                        .rect,
                    );
                    self.metrics.header.push(
                        ui.label(RichText::new("Offline simulation").small().color(MUTED))
                            .rect,
                    );
                    ui.label(RichText::new("Text size").small().color(MUTED));
                    let r = egui::ComboBox::from_id_salt("text-size")
                        .width(84.)
                        .selected_text(format!("{:.0}%", self.scale * 100.))
                        .show_ui(ui, |ui| {
                            for scale in [1., 1.15, 1.25, 1.5] {
                                ui.selectable_value(
                                    &mut self.scale,
                                    scale,
                                    format!("{:.0}%", scale * 100.),
                                );
                            }
                        });
                    self.metrics.header.push(r.response.rect);
                });
            });
    }
    fn library(&mut self, ui: &mut egui::Ui) {
        ui.heading("Practice library");
        ui.add_space(2.);
        ui.horizontal_wrapped(|ui| {
            for (label, value) in [
                ("All tracks", ""),
                ("CCNA", "CCNA"),
                ("LPIC-1", "LPIC-1"),
                ("LPIC-2", "LPIC-2"),
            ] {
                if ui
                    .selectable_label(self.filter.track == value, label)
                    .clicked()
                {
                    self.filter.track = value.into();
                    self.page = 0;
                }
            }
            if ui
                .toggle_value(&mut self.filter.favorites, "Favorites")
                .changed()
            {
                self.page = 0;
            }
            if ui
                .toggle_value(&mut self.filter.recent, "Recent / continue")
                .changed()
            {
                self.page = 0;
            }
        });
        if ui
            .add_sized(
                [ui.available_width(), CONTROL],
                egui::TextEdit::singleline(&mut self.filter.search)
                    .hint_text("Search titles, goals or topics…")
                    .char_limit(200),
            )
            .changed()
        {
            self.page = 0;
        }
        ui.horizontal_wrapped(|ui| {
            let topics: std::collections::BTreeSet<_> =
                self.scenarios.iter().map(|s| s.topic.clone()).collect();
            let before = self.filter.topic.clone();
            let r = combo(
                ui,
                "topic",
                "Any topic",
                &mut self.filter.topic,
                topics.iter().map(String::as_str),
                220_f32.min(ui.available_width()),
            );
            self.metrics.filters.push(r.rect);
            if before != self.filter.topic {
                self.page = 0;
            }
            let count = usize::from(!self.filter.difficulty.is_empty())
                + usize::from(!self.filter.status.is_empty())
                + usize::from(self.filter.max_minutes > 0);
            let r = ui.add(button(&if count > 0 {
                format!("More filters ({count})")
            } else {
                "More filters".into()
            }));
            self.metrics.extras = Some(r.rect);
            if r.clicked() {
                self.extra_filters = !self.extra_filters;
            }
            if !self.extra_filters
                && (!self.filter.search.is_empty()
                    || !self.filter.track.is_empty()
                    || !self.filter.topic.is_empty()
                    || count > 0
                    || self.filter.favorites
                    || self.filter.recent)
                && ui.add(button("Clear filters")).clicked()
            {
                self.filter = Filter::default();
                self.page = 0;
            }
        });
        if self.extra_filters {
            self.filters(ui);
        } else {
            let mut active = Vec::new();
            if !self.filter.difficulty.is_empty() {
                active.push(self.filter.difficulty.clone());
            }
            if !self.filter.status.is_empty() {
                active.push(self.filter.status.clone());
            }
            if self.filter.max_minutes > 0 {
                active.push(format!("Up to {} min", self.filter.max_minutes));
            }
            if !active.is_empty() {
                ui.label(RichText::new(active.join(" · ")).small().color(MUTED));
            }
        }
        ui.add_space(4.);
        ui.separator();
        if self.developer && self.synthetic && self.synthetic_catalog.is_empty() {
            self.synthetic_catalog = catalog::synthetic_catalog(&self.scenarios, 900);
        }
        let source = if self.developer && self.synthetic {
            &self.synthetic_catalog
        } else {
            &self.scenarios
        };
        let mut items: Vec<_> = source
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                let a = self.progress.attempts.get(&s.id);
                catalog::matches(
                    s,
                    &self.filter,
                    a.is_some(),
                    a.is_some_and(|a| engine::solved(&a.state, &s.goal)),
                    self.progress.favorites.contains(&s.id),
                )
            })
            .collect();
        if self.filter.recent {
            items.sort_by_key(|(_, s)| {
                std::cmp::Reverse(self.progress.attempts.get(&s.id).map_or(0, |a| a.visited))
            });
        }
        self.page = self.page.min(items.len().saturating_sub(1) / 12);
        self.metrics.pagination = items.len() > 12;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    items.len(),
                    if self.synthetic {
                        "synthetic previews"
                    } else {
                        if items.len() == 1 { "lab" } else { "labs" }
                    }
                ))
                .color(MUTED),
            );
            if self.metrics.pagination {
                if ui
                    .add_enabled(self.page > 0, button("Previous page"))
                    .clicked()
                {
                    self.page -= 1;
                }
                ui.label(format!(
                    "Page {} / {}",
                    self.page + 1,
                    items.len().div_ceil(12)
                ));
                if ui
                    .add_enabled((self.page + 1) * 12 < items.len(), button("Next page"))
                    .clicked()
                {
                    self.page += 1;
                }
            }
        });
        if items.is_empty() {
            ui.label("No labs match. Clear a filter or try another search.");
        }
        let columns = if ui.available_width() >= 980. { 2 } else { 1 };
        let mut launch = None;
        for row in items
            .iter()
            .skip(self.page * 12)
            .take(12)
            .collect::<Vec<_>>()
            .chunks(columns)
        {
            ui.columns(columns, |uis| {
                for (col, (i, s)) in row.iter().enumerate() {
                    let ui = &mut uis[col];
                    card().show(ui, |ui| {
                        ui.set_min_width((ui.available_width() - 1.).max(0.));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(format!("{} / {}", s.track, s.topic))
                                    .small()
                                    .color(ACCENT),
                            );
                            ui.label(
                                RichText::new(format!("{} · {} min", s.difficulty, s.minutes))
                                    .small()
                                    .color(MUTED),
                            );
                        });
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 28.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_min_height(28.);
                                ui.label(RichText::new(&s.title).strong().size(21.));
                            },
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 42.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_min_height(42.);
                                ui.label(&s.summary);
                            },
                        );
                        let a = self.progress.attempts.get(&s.id);
                        let complete = a.is_some_and(|a| engine::solved(&a.state, &s.goal));
                        let favorite = self.progress.favorites.contains(&s.id);
                        let status = if complete {
                            "Complete"
                        } else if a.is_some() {
                            "In progress"
                        } else {
                            "Not started"
                        };
                        let mut rects = None;
                        ui.horizontal_wrapped(|ui| {
                            let start = ui.add_enabled(
                                !self.synthetic,
                                primary(if self.synthetic {
                                    "Preview only"
                                } else if a.is_none() {
                                    "Start lab"
                                } else {
                                    "Continue"
                                })
                                .min_size(egui::vec2(108., CONTROL)),
                            );
                            if start.clicked() {
                                launch = Some(*i);
                            }
                            let fav = ui.add_enabled(
                                !self.synthetic,
                                button(if favorite { "Favorited" } else { "Favorite" })
                                    .min_size(egui::vec2(108., CONTROL)),
                            );
                            if fav.clicked() {
                                if favorite {
                                    self.progress.favorites.remove(&s.id);
                                } else {
                                    self.progress.favorites.insert(s.id.clone());
                                }
                                self.dirty = true;
                            }
                            rects = Some((start.rect, fav.rect));
                            ui.label(RichText::new(status).small().color(if complete {
                                GREEN
                            } else {
                                MUTED
                            }));
                        });
                        if let Some(rects) = rects {
                            self.metrics.card_actions.push(rects);
                        }
                    });
                    ui.add_space(4.);
                }
            });
        }
        if let Some(i) = launch {
            self.launch(i);
        }
        ui.label(
            RichText::new("Illustrative practice • not full exam coverage")
                .small()
                .color(MUTED),
        );
        if self.developer {
            egui::CollapsingHeader::new("Developer options").show(ui, |ui| {
                if ui
                    .checkbox(
                        &mut self.synthetic,
                        "900 synthetic catalog previews (not playable labs)",
                    )
                    .changed()
                {
                    self.page = 0;
                }
            });
        }
    }
    fn filters(&mut self, ui: &mut egui::Ui) {
        let old = (
            self.filter.topic.clone(),
            self.filter.difficulty.clone(),
            self.filter.status.clone(),
            self.filter.max_minutes,
        );
        let columns = (((ui.available_width() + 10.) / 174.).floor() as usize).clamp(1, 4);
        let width =
            ((ui.available_width() - 10. * (columns - 1) as f32) / columns as f32).min(240.);
        egui::Grid::new("library-filters")
            .num_columns(columns)
            .spacing([10., 10.])
            .show(ui, |ui| {
                for index in 0..4 {
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, CONTROL),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let response = match index {
                                0 => combo(
                                    ui,
                                    "difficulty",
                                    "Any difficulty",
                                    &mut self.filter.difficulty,
                                    ["Beginner", "Intermediate"].into_iter(),
                                    width,
                                ),
                                1 => combo(
                                    ui,
                                    "status",
                                    "Any status",
                                    &mut self.filter.status,
                                    ["Not started", "In progress", "Complete"].into_iter(),
                                    width,
                                ),
                                2 => {
                                    egui::ComboBox::from_id_salt("duration")
                                        .width(width)
                                        .selected_text(if self.filter.max_minutes == 0 {
                                            "Any duration".into()
                                        } else {
                                            format!("Up to {} min", self.filter.max_minutes)
                                        })
                                        .show_ui(ui, |ui| {
                                            for (v, label) in [
                                                (0, "Any duration"),
                                                (6, "Up to 6 min"),
                                                (8, "Up to 8 min"),
                                                (10, "Up to 10 min"),
                                            ] {
                                                ui.selectable_value(
                                                    &mut self.filter.max_minutes,
                                                    v,
                                                    label,
                                                );
                                            }
                                        })
                                        .response
                                }
                                _ => {
                                    let r = ui.add_sized([width, CONTROL], button("Clear filters"));
                                    if r.clicked() {
                                        self.filter = Filter::default();
                                        self.page = 0;
                                    }
                                    r
                                }
                            };
                            self.metrics.filters.push(response.rect);
                        },
                    );
                    if (index + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
        if old
            != (
                self.filter.topic.clone(),
                self.filter.difficulty.clone(),
                self.filter.status.clone(),
                self.filter.max_minutes,
            )
        {
            self.page = 0;
        }
    }
    fn render(&mut self, ctx: &egui::Context) {
        self.metrics = Metrics::default();
        ctx.set_zoom_factor(self.scale);
        self.header(ctx);
        if let Some(error) = &self.storage_error {
            egui::TopBottomPanel::bottom("storage-error").show(ctx, |ui| {
                ui.colored_label(RED, error);
                ui.label(if self.read_only {
                    "Original progress retained; this session cannot overwrite it."
                } else {
                    "Changes remain in memory until saved."
                });
                if !self.read_only && ui.button("Retry save").clicked() {
                    self.dirty = true;
                }
            });
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(14, 21, 38))
                    .inner_margin(18.),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(("main", self.selected))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let width = ui.available_width().min(if self.selected.is_some() {
                            1060.
                        } else {
                            1280.
                        });
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(width, 0.),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(width);
                                    if let Some(i) = self.selected {
                                        self.workspace(ui, i)
                                    } else {
                                        self.library(ui)
                                    }
                                },
                            );
                        });
                    });
            });
        if self.host_activity
            && let Some(i) = self.selected
        {
            self.host_bits(ctx, i);
        }
    }
}
fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1_f32, Color32::from_rgb(52, 63, 86)))
        .inner_margin(16.)
        .corner_radius(10.)
}
fn button(text: &str) -> egui::Button<'_> {
    egui::Button::new(text)
        .min_size(egui::vec2(0., CONTROL))
        .corner_radius(6.)
}
fn primary(text: &str) -> egui::Button<'_> {
    button(text).fill(Color32::from_rgb(83, 62, 140))
}
fn combo<'a>(
    ui: &mut egui::Ui,
    id: &str,
    any: &str,
    value: &mut String,
    options: impl Iterator<Item = &'a str>,
    width: f32,
) -> egui::Response {
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .selected_text(if value.is_empty() {
            any.to_owned()
        } else {
            value.clone()
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(value, String::new(), any);
            for option in options {
                ui.selectable_value(value, option.to_owned(), option);
            }
        })
        .response
}
impl eframe::App for LabApp {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        if self.smoke {
            if let Some(modifiers) = self.pending_events.iter().rev().find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            }) {
                input.modifiers = modifiers;
            }
            input.events.append(&mut self.pending_events);
        }
    }
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.smoke {
            self.capture_events(ctx);
        }
        self.render(ctx);
        self.save();
        if self.smoke {
            self.smoke_tick(ctx);
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}
