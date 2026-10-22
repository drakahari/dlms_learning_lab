use dlms_learning_lab::{
    catalog::{self, Filter},
    engine, ipv4,
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace as model,
};
use dlms_learning_lab::{course, script_teaching as teaching, shell, theme};
use eframe::egui::{self, RichText};
mod data;
mod script_learning;
mod scripting;
mod terminal_learning;
use std::{path::PathBuf, time::Instant};
mod ipv4_workspace;
mod ipv6_workspace;
mod smoke;
mod switching_workspace;
#[cfg(test)]
mod tests;
mod workspace;
const CONTROL: f32 = 36.;
#[derive(Default)]
struct Metrics {
    header: Vec<egui::Rect>,
    lesson_tabs: Vec<egui::Rect>,
    example_action: Option<egui::Rect>,
    assignment_action: Option<egui::Rect>,
    settings: Option<egui::Rect>,
    data_export: Option<egui::Rect>,
    data_restore: Option<egui::Rect>,
    data_file_action: Option<egui::Rect>,
    data_filename: Option<egui::Rect>,
    data_confirm: Option<egui::Rect>,
    data_cancel: Option<egui::Rect>,
    course_start: Option<egui::Rect>,
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
    editor_viewport: Option<egui::Rect>,
    probe: Option<egui::Rect>,
    script_history: Option<egui::Rect>,
    stop: Option<egui::Rect>,
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
    units: Vec<course::Unit>,
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
    theme_id: String,
    theme_preview: bool,
    settings_open: bool,
    settings_data: bool,
    data_tools: data::Tools,
    profile_lock: Option<progress::ProfileLock>,
    preferences_error: Option<String>,
    preferences_read_only: bool,
    script_job: Option<scripting::Job>,
    script_case: usize,
    focus_input: bool,
    scroll_command: bool,
    lesson_scroll_reset: bool,
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
        let units = course::bundled(&scenarios).expect("Bundled units validate");
        let path = if smoke {
            std::env::temp_dir().join("dlms-workspaces-smoke/progress-v1.json")
        } else {
            progress::default_path()
                .unwrap_or_else(|_| PathBuf::from("local-data/progress-v1.json"))
        };
        let fresh = || Progress {
            schema_version: 1,
            ..Default::default()
        };
        let lock = if smoke {
            Ok(None)
        } else {
            progress::lock_profile(&path).map(Some)
        };
        let lock_error = lock.as_ref().err().cloned();
        let profile_lock = lock.ok().flatten();
        let (progress, storage_error) = if smoke {
            (fresh(), None)
        } else {
            match progress::load(&path, &scenarios) {
                Ok(p) => (p, None),
                Err(e) => (fresh(), Some(e)),
            }
        };
        let storage_error = storage_error.or(lock_error.clone());
        let read_only = storage_error.is_some();
        let preferences = if smoke {
            Ok(theme::Preferences::default())
        } else {
            theme::Preferences::load(&path.with_file_name("preferences-v1.json"))
        };
        let preferences_error = preferences.as_ref().err().cloned();
        let preferences_read_only = preferences_error.is_some() || lock_error.is_some();
        let preferences = preferences.unwrap_or_default();
        theme::apply(ctx, &preferences.theme);
        Self {
            scenarios,
            units,
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
            scale: preferences.text_scale,
            theme_id: preferences.theme,
            theme_preview: false,
            settings_open: false,
            settings_data: false,
            data_tools: data::Tools::default(),
            profile_lock,
            preferences_error,
            preferences_read_only,
            script_job: None,
            script_case: 0,
            focus_input: false,
            scroll_command: false,
            lesson_scroll_reset: false,
            notice: None,
            metrics: Metrics::default(),
            help_open: false,
            extra_filters: false,
            host_activity: false,
            ipv4_view: None,
            smoke,
            started: Instant::now(),
            smoke_stage: if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("file-batches")
            {
                2500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("expansion") {
                2400
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("vlan-resume") {
                2320
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("vlans") {
                2200
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("switching") {
                2100
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("files-layout") {
                1209
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("scope-recovery")
            {
                1500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("access-library")
            {
                2090
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("access") {
                2000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("counts-stop") {
                1980
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("counts") {
                1900
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("arithmetic") {
                1800
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("text") {
                1700
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("scope") {
                1600
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ipv6-layout") {
                1408
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ipv6") {
                1300
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("files") {
                1100
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("reusable") {
                900
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("composition-layout")
            {
                772
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("composition") {
                700
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("courses") {
                600
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("recovery") {
                500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("teaching-layout")
            {
                400
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("teaching") {
                250
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("scripts") {
                176
            } else {
                0
            },
            pending_events: Vec::new(),
            capture_pending: false,
        }
    }
    fn launch(&mut self, i: usize) {
        self.script_job = None;
        self.script_case = 0;
        self.progress.clock = self.progress.clock.saturating_add(1);
        let clock = self.progress.clock;
        let s = &self.scenarios[i];
        let new_attempt = !self.progress.attempts.contains_key(&s.id);
        let a = self
            .progress
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, clock));
        model::prepare(a);
        if let Some(lesson) = &s.teaching {
            teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.terminal_teaching {
            dlms_learning_lab::terminal_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.switching_teaching {
            dlms_learning_lab::switching_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.ipv6_teaching {
            dlms_learning_lab::ipv6_teaching::prepare(a, lesson, new_attempt);
        }
        if new_attempt
            && let engine::Goal::Ipv6 { requirements } = &s.goal
            && let Some(device) = dlms_learning_lab::ipv6_teaching::preferred_device(requirements)
        {
            a.workspace
                .as_mut()
                .unwrap()
                .ipv6_draft
                .as_mut()
                .unwrap()
                .selected_device = device;
        }
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
        if let Some(lesson) = &s.teaching {
            teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.terminal_teaching {
            dlms_learning_lab::terminal_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.switching_teaching {
            dlms_learning_lab::switching_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.ipv6_teaching {
            dlms_learning_lab::ipv6_teaching::prepare(a, lesson, true);
        }
        if let engine::Goal::Ipv6 { requirements } = &s.goal
            && let Some(device) = dlms_learning_lab::ipv6_teaching::preferred_device(requirements)
        {
            a.workspace
                .as_mut()
                .unwrap()
                .ipv6_draft
                .as_mut()
                .unwrap()
                .selected_device = device;
        }
        self.script_job = None;
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
                Ok(saved) => {
                    self.dirty = false;
                    self.storage_error = saved.durability_warning
                }
                Err(e) => self.storage_error = Some(format!("Progress not saved: {e}")),
            }
        }
    }
    fn text_size_picker(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Text size").small().color(palette(ui).muted));
        let r = egui::ComboBox::from_id_salt("text-size")
            .width(84.)
            .selected_text(format!("{:.0}%", self.scale * 100.))
            .show_ui(ui, |ui| {
                for scale in [1., 1.15, 1.25, 1.5] {
                    ui.selectable_value(&mut self.scale, scale, format!("{:.0}%", scale * 100.));
                }
            });
        self.metrics.header.push(r.response.rect);
    }
    fn theme_picker(&mut self, ui: &mut egui::Ui) {
        self.theme_dropdown(ui);
        let r = ui.button("Settings");
        self.metrics.settings = Some(r.rect);
        if r.clicked() {
            self.settings_open = !self.settings_open;
        }
    }
    fn theme_dropdown(&mut self, ui: &mut egui::Ui) {
        let r = egui::ComboBox::from_id_salt("native-theme-picker")
            .width(195.)
            .truncate()
            .selected_text(format!(
                "{} · {}",
                theme::find(&self.theme_id).group,
                theme::find(&self.theme_id).name
            ))
            .show_ui(ui, |ui| {
                for t in theme::inventory() {
                    ui.selectable_value(
                        &mut self.theme_id,
                        t.id.clone(),
                        format!("{} · {}", t.group, t.name),
                    );
                }
            });
        self.metrics.header.push(r.response.rect);
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("brand")
            .frame(
                egui::Frame::new()
                    .fill(theme::roles(ctx).panel)
                    .inner_margin(egui::Margin::symmetric(20, 10)),
            )
            .show(ctx, |ui| {
                let narrow = ui.available_width() < 1050.;
                ui.horizontal_wrapped(|ui| {
                    self.metrics.header.push(
                        ui.label(
                            RichText::new("DLMS Learning Lab")
                                .strong()
                                .size(20.)
                                .color(palette(ui).accent),
                        )
                        .rect,
                    );
                    self.metrics.header.push(
                        ui.label(
                            RichText::new("Offline simulation")
                                .small()
                                .color(palette(ui).muted),
                        )
                        .rect,
                    );
                    if !narrow {
                        self.text_size_picker(ui);
                        self.theme_picker(ui);
                    }
                });
                if narrow {
                    ui.horizontal_wrapped(|ui| {
                        self.text_size_picker(ui);
                        self.theme_picker(ui);
                    });
                }
            });
    }
    fn sync_appearance(&mut self, ctx: &egui::Context, previous: (String, f32)) {
        if previous != (self.theme_id.clone(), self.scale) {
            theme::apply(ctx, &self.theme_id);
            if !self.preferences_read_only {
                self.preferences_error = theme::Preferences {
                    version: 1,
                    theme: self.theme_id.clone(),
                    text_scale: self.scale,
                }
                .save(&self.path.with_file_name("preferences-v1.json"))
                .err();
            }
        }
    }
    fn course_entry(&mut self, ui: &mut egui::Ui) {
        if let Some(unit) = self
            .units
            .iter()
            .find(|u| u.id == self.filter.unit)
            .cloned()
        {
            let done = unit
                .labs
                .iter()
                .filter(|id| {
                    self.progress.attempts.get(*id).is_some_and(|a| {
                        self.scenarios
                            .iter()
                            .find(|s| &s.id == *id)
                            .is_some_and(|s| progress::current_solved(a, s))
                    })
                })
                .count();
            let earlier = unit
                .labs
                .iter()
                .filter(|id| {
                    self.progress.attempts.get(*id).is_some_and(|a| {
                        self.scenarios
                            .iter()
                            .find(|s| &s.id == *id)
                            .is_some_and(|s| {
                                !progress::current_solved(a, s)
                                    && progress::legacy_script_credit(a, s)
                            })
                    })
                })
                .count();
            ui.label(if earlier > 0 {
                format!(
                    "{} · {done}/{} currently verified · {earlier} earlier completions retained",
                    unit.title,
                    unit.labs.len()
                )
            } else {
                format!("{} · {done}/{} labs complete", unit.title, unit.labs.len())
            });
            ui.label(&unit.summary);
            if !unit.prerequisites.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "Suggested preparation: {}. You can still open any lab.",
                        unit.prerequisites
                            .iter()
                            .filter_map(|id| self.units.iter().find(|u| &u.id == id))
                            .map(|u| u.title.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .small()
                    .color(palette(ui).muted),
                );
            }
        } else if !self.filter.track.is_empty()
            && self.filter.search.is_empty()
            && !self.filter.favorites
            && !self.filter.recent
        {
            let next = self
                .units
                .iter()
                .filter(|u| u.track == self.filter.track)
                .flat_map(|u| u.labs.iter())
                .find_map(|id| {
                    self.scenarios.iter().position(|s| {
                        &s.id == id
                            && self
                                .progress
                                .attempts
                                .get(id)
                                .is_none_or(|a| !progress::current_solved(a, s))
                    })
                });
            ui.horizontal_wrapped(|ui| {
                ui.label("Follow the course order, or choose any lab below.");
                if let Some(i) = next {
                    let r = ui.add(button(
                        if self
                            .progress
                            .attempts
                            .get(&self.scenarios[i].id)
                            .is_some_and(|a| progress::legacy_script_credit(a, &self.scenarios[i]))
                        {
                            "Verify earlier run"
                        } else if self.progress.attempts.contains_key(&self.scenarios[i].id) {
                            "Continue course"
                        } else {
                            "Start course"
                        },
                    ));
                    self.metrics.course_start = Some(r.rect);
                    if r.clicked() {
                        self.launch(i);
                    }
                }
            });
        }
    }
    fn next_unit_lab(&mut self, ui: &mut egui::Ui, i: usize) {
        let Some(unit) = self.units.iter().find(|u| u.id == self.scenarios[i].unit) else {
            return;
        };
        if let Some(next) =
            course::next(unit, &self.scenarios[i].id, &self.scenarios, &self.progress)
        {
            if ui
                .link(format!("Next in unit: {}", self.scenarios[next].title))
                .clicked()
            {
                self.launch(next);
            }
        } else {
            ui.label("Unit complete · All its current assignments pass.");
            if ui.link("Back to this unit").clicked() {
                self.filter.track = unit.track.clone();
                self.filter.unit = unit.id.clone();
                self.filter.topic.clear();
                self.filter.search.clear();
                self.selected = None;
            }
        }
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
                ("Bash/Shell", "Bash/Shell"),
            ] {
                if ui
                    .selectable_label(self.filter.track == value, label)
                    .clicked()
                {
                    if self.filter.track != value {
                        self.filter.unit.clear();
                        self.filter.topic.clear();
                    }
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
        self.course_entry(ui);
        ui.horizontal_wrapped(|ui| {
            let previous = self.filter.unit.clone();
            let chosen = self
                .units
                .iter()
                .find(|u| u.id == self.filter.unit)
                .map_or("Any unit", |u| u.title.as_str());
            let unit_width = 220_f32.min(ui.available_width());
            let r = ui
                .allocate_ui_with_layout(
                    egui::vec2(unit_width, CONTROL),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_width(unit_width);
                        egui::ComboBox::from_id_salt("unit-filter")
                            .width(220_f32.min(ui.available_width()))
                            .truncate()
                            .selected_text(chosen)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.filter.unit,
                                    String::new(),
                                    "Any unit",
                                );
                                for u in self.units.iter().filter(|u| {
                                    self.filter.track.is_empty() || u.track == self.filter.track
                                }) {
                                    ui.selectable_value(
                                        &mut self.filter.unit,
                                        u.id.clone(),
                                        &u.title,
                                    );
                                }
                            })
                    },
                )
                .inner;
            self.metrics.filters.push(r.response.rect);
            if previous != self.filter.unit {
                self.filter.topic.clear();
                self.page = 0;
            }
            let topics: std::collections::BTreeSet<_> = self
                .scenarios
                .iter()
                .filter(|s| {
                    (self.filter.track.is_empty() || s.track == self.filter.track)
                        && (self.filter.unit.is_empty() || s.unit == self.filter.unit)
                })
                .map(|s| s.topic.clone())
                .collect();
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
                    || !self.filter.unit.is_empty()
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
                ui.label(
                    RichText::new(active.join(" · "))
                        .small()
                        .color(palette(ui).muted),
                );
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
                    a.is_some_and(|a| progress::current_solved(a, s)),
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
                .color(palette(ui).muted),
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
                    card(ui).show(ui, |ui| {
                        ui.set_min_width((ui.available_width() - 1.).max(0.));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(format!("{} / {}", s.track, s.topic))
                                    .small()
                                    .color(palette(ui).accent),
                            );
                            ui.label(
                                RichText::new(format!("{} · {} min", s.difficulty, s.minutes))
                                    .small()
                                    .color(palette(ui).muted),
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
                        let complete = a.is_some_and(|a| progress::current_solved(a, s));
                        let favorite = self.progress.favorites.contains(&s.id);
                        let status = if a.is_some_and(|a| progress::legacy_script_credit(a, s))
                            && !complete
                        {
                            "Earlier run complete · Rerun to verify"
                        } else if a.is_some_and(progress::legacy_credit) && !complete {
                            "Earlier analysis complete · New requirement remaining"
                        } else if complete {
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
                                primary(
                                    ui,
                                    if self.synthetic {
                                        "Preview only"
                                    } else if a.is_none() {
                                        "Start lab"
                                    } else {
                                        "Continue"
                                    },
                                )
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
                                palette(ui).success
                            } else {
                                palette(ui).muted
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
                .color(palette(ui).muted),
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
        self.advance_scripts(ctx);
        let previous = (self.theme_id.clone(), self.scale);
        self.header(ctx);
        self.appearance_preview(ctx);
        self.data_dialog(ctx);
        self.sync_appearance(ctx, previous);
        if let Some(error) = &self.storage_error {
            egui::TopBottomPanel::bottom("storage-error").show(ctx, |ui| {
                ui.colored_label(palette(ui).error, error);
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
                    .fill(theme::roles(ctx).background)
                    .inner_margin(18.),
            )
            .show(ctx, |ui| {
                let mut area = egui::ScrollArea::vertical().id_salt(("main", self.selected));
                if std::mem::take(&mut self.lesson_scroll_reset) {
                    area = area.vertical_scroll_offset(0.);
                }
                area.auto_shrink([false, false]).show(ui, |ui| {
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
                                ui.add_enabled_ui(!self.data_tools.modal_open(), |ui| {
                                    if let Some(i) = self.selected {
                                        self.workspace(ui, i)
                                    } else {
                                        self.library(ui)
                                    }
                                });
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
        // egui shares selection foreground with TextEdit's built-in focus border.
        // Paint a separate contrasting native focus ring so both remain readable.
        if let Some(id) = ctx.memory(|m| m.focused())
            && let Some(response) = ctx.read_response(id)
        {
            ctx.layer_painter(response.layer_id).rect_stroke(
                response.interact_rect,
                4_u8,
                egui::Stroke::new(2_f32, theme::roles(ctx).accent),
                egui::StrokeKind::Inside,
            );
        }
    }
}
fn card(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(palette(ui).panel)
        .stroke(egui::Stroke::new(1_f32, palette(ui).border))
        .inner_margin(16.)
        .corner_radius(10.)
}
fn button(text: &str) -> egui::Button<'_> {
    egui::Button::new(text)
        .min_size(egui::vec2(0., CONTROL))
        .corner_radius(6.)
}
fn primary<'a>(ui: &egui::Ui, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(RichText::new(text).color(palette(ui).on_accent))
        .min_size(egui::vec2(0., CONTROL))
        .corner_radius(6.)
        .fill(palette(ui).accent)
        .stroke(egui::Stroke::new(1_f32, palette(ui).border))
}
fn combo<'a>(
    ui: &mut egui::Ui,
    id: &str,
    any: &str,
    value: &mut String,
    options: impl Iterator<Item = &'a str>,
    width: f32,
) -> egui::Response {
    ui.allocate_ui_with_layout(
        egui::vec2(width, CONTROL),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(width);
            egui::ComboBox::from_id_salt(id)
                .width(width)
                .truncate()
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
        },
    )
    .inner
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

fn palette(ui: &egui::Ui) -> theme::Roles {
    theme::roles(ui.ctx())
}
