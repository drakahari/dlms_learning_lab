use dlms_learning_lab::{
    catalog::{self, Filter},
    engine,
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
};
use eframe::egui::{self, Color32, RichText};
use std::{path::PathBuf, time::Instant};
const ACCENT: Color32 = Color32::from_rgb(171, 151, 255);
const MUTED: Color32 = Color32::from_rgb(165, 179, 205);
pub struct LabApp {
    scenarios: Vec<Scenario>,
    progress: Progress,
    path: PathBuf,
    filter: Filter,
    selected: Option<usize>,
    input: String,
    feedback: Option<String>,
    reset_pending: bool,
    dirty: bool,
    storage_error: Option<String>,
    read_only: bool,
    synthetic: bool,
    synthetic_catalog: Vec<Scenario>,
    page: usize,
    scale: f32,
    focus_command: bool,
    smoke: bool,
    started: Instant,
    smoke_stage: usize,
    pending_events: Vec<egui::Event>,
    smoke_focus: Option<egui::Id>,
    smoke_width: f32,
}
impl LabApp {
    pub fn new(cc: &eframe::CreationContext<'_>, smoke: bool) -> Self {
        Self::from_context(&cc.egui_ctx, smoke)
    }
    fn from_context(ctx: &egui::Context, smoke: bool) -> Self {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(14, 21, 38);
        style.visuals.window_fill = Color32::from_rgb(22, 30, 51);
        style.visuals.extreme_bg_color = Color32::from_rgb(9, 15, 28);
        style.visuals.faint_bg_color = Color32::from_rgb(24, 34, 55);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 45, 70);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(64, 53, 99);
        style.visuals.selection.bg_fill = Color32::from_rgb(80, 60, 135);
        style.visuals.override_text_color = Some(Color32::from_rgb(231, 237, 249));
        style.spacing.item_spacing = egui::vec2(12., 12.);
        style.spacing.button_padding = egui::vec2(14., 9.);
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(17.));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(16.));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(16.));
        ctx.set_style(style);
        let scenarios = pack::bundled().expect("Bundled scenarios must pass validation");
        let path = if smoke {
            PathBuf::from("/tmp/dlms-ui-smoke/progress-v1.json")
        } else {
            progress::default_path()
                .unwrap_or_else(|_| PathBuf::from("local-data/progress-v1.json"))
        };
        let (progress, storage_error) = match progress::load(&path, &scenarios) {
            Ok(p) => (p, None),
            Err(e) => (
                Progress {
                    schema_version: 1,
                    ..Default::default()
                },
                Some(e),
            ),
        };
        let read_only = storage_error.is_some();
        Self {
            scenarios,
            progress,
            path,
            filter: Filter::default(),
            selected: None,
            input: String::new(),
            feedback: None,
            reset_pending: false,
            dirty: false,
            storage_error,
            read_only,
            synthetic: false,
            synthetic_catalog: Vec::new(),
            page: 0,
            scale: 1.,
            focus_command: false,
            smoke,
            started: Instant::now(),
            smoke_stage: 0,
            pending_events: Vec::new(),
            smoke_focus: None,
            smoke_width: 0.,
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
        a.visited = clock;
        self.selected = Some(i);
        self.input.clear();
        self.feedback = None;
        self.reset_pending = false;
        self.focus_command = true;
        self.dirty = true;
    }
    fn save(&mut self) {
        if self.dirty && !self.read_only {
            match progress::save(&self.path, &self.progress) {
                Ok(()) => {
                    self.dirty = false;
                    self.storage_error = None
                }
                Err(e) => self.storage_error = Some(format!("Progress is not saved: {e}")),
            }
        }
    }
    fn library(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.);
        ui.label(
            RichText::new("YOUR PRACTICE LIBRARY")
                .color(ACCENT)
                .size(13.),
        );
        ui.heading(RichText::new("Small labs. Real understanding.").size(30.));
        ui.label(
            RichText::new("Nine guided scenarios. Explore a track and build one skill at a time.")
                .color(MUTED),
        );
        ui.add_space(8.);
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
        let search = ui.add(
            egui::TextEdit::singleline(&mut self.filter.search)
                .hint_text("Search titles, goals or topics…")
                .desired_width(f32::INFINITY),
        );
        if search.changed() {
            self.page = 0;
        }
        let old_filters = (
            self.filter.topic.clone(),
            self.filter.difficulty.clone(),
            self.filter.status.clone(),
            self.filter.max_minutes,
        );
        ui.horizontal_wrapped(|ui| {
            let topics: std::collections::BTreeSet<_> =
                self.scenarios.iter().map(|s| s.topic.clone()).collect();
            combo(
                ui,
                "Topic",
                &mut self.filter.topic,
                topics.iter().map(String::as_str),
            );
            combo(
                ui,
                "Difficulty",
                &mut self.filter.difficulty,
                ["Beginner", "Intermediate"].into_iter(),
            );
            combo(
                ui,
                "Status",
                &mut self.filter.status,
                ["Not started", "In progress", "Complete"].into_iter(),
            );
            egui::ComboBox::from_id_salt("duration")
                .selected_text(if self.filter.max_minutes == 0 {
                    "Any duration".into()
                } else {
                    format!("≤ {} min", self.filter.max_minutes)
                })
                .show_ui(ui, |ui| {
                    for (v, label) in [
                        (0, "Any duration"),
                        (6, "≤ 6 min"),
                        (8, "≤ 8 min"),
                        (10, "≤ 10 min"),
                    ] {
                        ui.selectable_value(&mut self.filter.max_minutes, v, label);
                    }
                });
            if ui.button("Clear filters").clicked() {
                self.filter = Filter::default();
                self.page = 0;
            }
        });
        if old_filters
            != (
                self.filter.topic.clone(),
                self.filter.difficulty.clone(),
                self.filter.status.clone(),
                self.filter.max_minutes,
            )
        {
            self.page = 0;
        }
        ui.separator();
        if self.synthetic && self.synthetic_catalog.is_empty() {
            self.synthetic_catalog = catalog::synthetic_catalog(&self.scenarios, 900);
        }
        let source = if self.synthetic {
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
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{} results", items.len()));
            if ui
                .add_enabled(self.page > 0, egui::Button::new("Previous page"))
                .clicked()
            {
                self.page -= 1;
            }
            ui.label(format!(
                "Page {} / {}",
                self.page + 1,
                items.len().div_ceil(12).max(1)
            ));
            if ui
                .add_enabled(
                    (self.page + 1) * 12 < items.len(),
                    egui::Button::new("Next page"),
                )
                .clicked()
            {
                self.page += 1;
            }
        });
        if items.is_empty() {
            ui.label("No labs match. Clear a filter or try another search.");
        }
        let mut launch = None;
        for (i, s) in items.iter().skip(self.page * 12).take(12) {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(24, 33, 54))
                .inner_margin(18.)
                .corner_radius(10.)
                .show(ui, |ui| {
                    ui.set_min_width((ui.available_width() - 8.).max(0.));
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(format!("{}  /  {}", s.track, s.topic))
                                .color(ACCENT)
                                .size(13.),
                        );
                        ui.label(
                            RichText::new(format!("{}  ·  {} min", s.difficulty, s.minutes))
                                .color(MUTED),
                        );
                    });
                    ui.label(RichText::new(&s.title).strong().size(21.));
                    ui.label(&s.summary);
                    ui.horizontal_wrapped(|ui| {
                        let a = self.progress.attempts.get(&s.id);
                        let complete = a.is_some_and(|a| engine::solved(&a.state, &s.goal));
                        let label = if self.synthetic {
                            "Synthetic • preview only"
                        } else if a.is_none() {
                            "Start lab >"
                        } else {
                            "Continue >"
                        };
                        if ui
                            .add_enabled(!self.synthetic, egui::Button::new(label))
                            .clicked()
                        {
                            launch = Some(*i);
                        }
                        ui.label(
                            RichText::new(if complete {
                                "Complete"
                            } else if a.is_some() {
                                "In progress"
                            } else {
                                "Ready to explore"
                            })
                            .color(if complete {
                                Color32::LIGHT_GREEN
                            } else {
                                MUTED
                            }),
                        );
                        let favorite = self.progress.favorites.contains(&s.id);
                        if ui
                            .add_enabled(
                                !self.synthetic,
                                egui::Button::new(if favorite { "Unfavorite" } else { "Favorite" }),
                            )
                            .clicked()
                        {
                            if favorite {
                                self.progress.favorites.remove(&s.id);
                            } else {
                                self.progress.favorites.insert(s.id.clone());
                            }
                            self.dirty = true;
                        }
                    });
                });
            ui.add_space(3.);
        }
        if let Some(i) = launch {
            self.launch(i);
        }
        ui.add_space(12.);
        ui.label(
            RichText::new("Initial illustrative practice • not full exam coverage").color(MUTED),
        );
        if ui
            .checkbox(
                &mut self.synthetic,
                "Developer test: 900 clearly marked synthetic catalog entries (not finished labs)",
            )
            .changed()
        {
            self.page = 0;
        }
    }
    fn workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let mut back = false;
        ui.horizontal_wrapped(|ui| {
            back = ui.button("< Back to library").clicked();
            ui.label(RichText::new(format!("{} / {}", s.track, s.topic)).color(ACCENT));
        });
        ui.heading(RichText::new(&s.title).size(28.));
        ui.label(&s.objective);
        ui.label(RichText::new("Simulation only • no host commands. State, mode, hints and step save locally after each action; leaving keeps this attempt.").color(MUTED).size(14.));
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(!a.challenge, "Learn mode").clicked() {
                a.challenge = false;
                self.dirty = true;
            }
            if ui.selectable_label(a.challenge, "Challenge mode").clicked() {
                a.challenge = true;
                self.dirty = true;
            }
            if ui.button("Reset this attempt").clicked() {
                self.reset_pending = true;
            }
        });
        if self.reset_pending {
            ui.label("Reset discards only this lab’s state, transcript, hints and step. Favorites and all other labs stay saved.");
            ui.horizontal(|ui| {
                if ui.button("Confirm reset").clicked() {
                    *a = Attempt::new(s, a.visited);
                    self.feedback = None;
                    self.dirty = true;
                    self.reset_pending = false;
                }
                if ui.button("Cancel").clicked() {
                    self.reset_pending = false;
                }
            });
        }
        if !a.challenge {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(35, 30, 62))
                .inner_margin(16.)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("CURRENT STEP  {} / {}", a.step + 1, s.steps.len()))
                            .color(ACCENT)
                            .strong(),
                    );
                    ui.label(&s.steps[a.step]);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(a.step > 0, egui::Button::new("Previous step"))
                            .clicked()
                        {
                            a.step -= 1;
                            self.dirty = true;
                        }
                        if ui
                            .add_enabled(a.step + 1 < s.steps.len(), egui::Button::new("Next step"))
                            .clicked()
                        {
                            a.step += 1;
                            self.dirty = true;
                        }
                    });
                });
        } else {
            ui.label(
                RichText::new(
                    "Walkthrough hidden. Solve the objective above; hints remain optional.",
                )
                .color(MUTED),
            );
        }
        ui.add_space(6.);
        ui.label(RichText::new("SIMULATED WORKSPACE").color(ACCENT).strong());
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(9, 15, 28))
            .inner_margin(16.)
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(RichText::new(engine::describe(&a.state)).monospace()).wrap(),
                );
            });
        egui::CollapsingHeader::new("Supported commands and simulation limits").show(ui,|ui|{ui.label(engine::help(&a.state));ui.label(&s.simplifications);ui.label("No general shell: no sudo, scripts, command chaining, redirection or host paths. Commands change only this attempt. Use one command per submission.");});
        if let Some(text) = &self.feedback {
            ui.label(RichText::new(text).strong().color(ACCENT));
            for (pass, text) in engine::checks(&a.state, &s.goal) {
                ui.label(format!("{} {text}", if pass { "PASS" } else { "TODO" }));
            }
        }
        for hint in s.hints.iter().take(a.hints) {
            ui.label(format!("Hint: {hint}"));
        }
        egui::CollapsingHeader::new("Content references").show(ui,|ui|{ui.label("Official objective alignment and technical references; see docs/CONTENT.md for scope.");for source in &s.sources{ui.label(source);}});
        if back {
            self.selected = None;
            self.feedback = None;
            self.reset_pending = false;
        }
    }
    fn workspace_actions(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        egui::ScrollArea::vertical()
            .id_salt("transcript")
            .max_height(110.)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &a.transcript {
                    ui.add(egui::Label::new(RichText::new(line).monospace().size(15.)).wrap());
                }
            });
        let mut run = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new("sim >").color(ACCENT).monospace());
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .id_salt("command")
                    .hint_text("Type help, or a supported command")
                    .desired_width((ui.available_width() - 95.).max(100.))
                    .char_limit(4096),
            );
            if self.focus_command {
                response.request_focus();
                self.focus_command = false;
            }
            run = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            run |= ui.button("Run").clicked();
        });
        if run && !self.input.trim().is_empty() {
            let cmd = std::mem::take(&mut self.input);
            let output = engine::execute(&mut a.state, &cmd)
                .unwrap_or_else(|e| format!("Limitation / error: {e}"));
            a.transcript.push(format!("sim > {cmd}\n{output}"));
            if a.transcript.len() > 100 {
                a.transcript.drain(..a.transcript.len() - 100);
            }
            self.feedback = None;
            self.dirty = true;
            self.focus_command = true;
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(RichText::new("Check work").strong().color(ACCENT))
                .clicked()
            {
                let list = engine::checks(&a.state, &s.goal);
                let n = list.iter().filter(|c| c.0).count();
                self.feedback = Some(if n == list.len() {
                    "All checks pass. You can return to the library or keep experimenting.".into()
                } else {
                    format!(
                        "{n}/{} checks pass. Review the remaining criteria in the lesson area.",
                        list.len()
                    )
                });
            }
            if ui
                .add_enabled(
                    a.hints < s.hints.len(),
                    egui::Button::new(format!("Reveal hint ({}/{})", a.hints, s.hints.len())),
                )
                .clicked()
            {
                a.hints += 1;
                self.dirty = true;
            }
        });

        if let Some(text) = &self.feedback {
            ui.label(RichText::new(text).color(ACCENT));
        }
    }
}
fn combo<'a>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    options: impl Iterator<Item = &'a str>,
) {
    egui::ComboBox::from_id_salt(label)
        .selected_text(if value.is_empty() {
            format!("Any {}", label.to_lowercase())
        } else {
            value.clone()
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(value, String::new(), "Any");
            for option in options {
                ui.selectable_value(value, option.into(), option);
            }
        });
}
impl eframe::App for LabApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        if self.smoke {
            input.events.append(&mut self.pending_events);
        }
    }
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.smoke {
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Screenshot {
                    image, user_data, ..
                } = event
                {
                    let name = user_data
                        .data
                        .as_ref()
                        .and_then(|d| d.downcast_ref::<String>())
                        .cloned()
                        .unwrap_or("capture".into());
                    let dir = std::path::Path::new("screenshots");
                    std::fs::create_dir_all(dir).unwrap();
                    let file = std::fs::File::create(dir.join(format!("{name}.png"))).unwrap();
                    let mut encoder =
                        png::Encoder::new(file, image.width() as u32, image.height() as u32);
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().unwrap();
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    writer.write_image_data(&bytes).unwrap();
                    println!("Captured {name}: {}x{}", image.width(), image.height());
                }
            }
        }
        egui::TopBottomPanel::top("brand").show(ctx, |ui| {
            ui.add_space(8.);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("DLMS").strong().color(ACCENT).size(24.));
                ui.label(RichText::new("LEARNING LAB").strong().size(17.));
                ui.label(RichText::new("OFFLINE / NATIVE").size(12.).color(MUTED));
                ui.add(
                    egui::Slider::new(&mut self.scale, 0.85..=1.5)
                        .text("Text scale")
                        .step_by(0.05)
                        .fixed_decimals(2),
                );
            });
            ui.add_space(6.);
        });
        ctx.set_zoom_factor(self.scale);
        if let Some(error) = &self.storage_error {
            egui::TopBottomPanel::bottom("storage-error").show(ctx,|ui|{ui.colored_label(Color32::LIGHT_RED,error);ui.label(if self.read_only{"Original progress preserved. This session is read-only; repair or move the progress file before restarting."}else{"Changes remain in memory. Use Retry save after storage is available."});if !self.read_only&&ui.button("Retry save").clicked(){self.dirty=true;}});
        }
        if let Some(i) = self.selected {
            egui::TopBottomPanel::bottom("workspace-actions").show(ctx, |ui| {
                ui.add_space(5.);
                self.workspace_actions(ui, i);
                ui.add_space(5.);
            });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().id_salt("main").show(ui, |ui| {
                ui.set_max_width(1150.);
                if let Some(i) = self.selected {
                    self.workspace(ui, i)
                } else {
                    self.library(ui)
                }
            });
        });
        self.save();
        if self.smoke {
            self.smoke_tick(ctx);
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
        }
    }
}

impl LabApp {
    fn smoke_tick(&mut self, ctx: &egui::Context) {
        if self.started.elapsed().as_millis() < ((self.smoke_stage + 1) * 700) as u128 {
            return;
        }
        let key = |key, shift| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                shift,
                ..Default::default()
            },
        };
        let shot = |name: &str| {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                name.to_string(),
            )))
        };
        match self.smoke_stage {
            0 => shot("01-library"),
            1 => {
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "lpic2-service")
                    .unwrap();
                self.launch(i);
            }
            2 => {
                self.smoke_focus = ctx.memory(|m| m.focused());
                assert!(
                    self.smoke_focus.is_some(),
                    "command receives keyboard focus"
                );
                self.pending_events
                    .push(egui::Event::Text("config set port 8080".into()));
            }
            3 => {
                assert_eq!(self.input, "config set port 8080", "native egui text input");
                self.pending_events.push(key(egui::Key::Enter, false));
            }
            4 => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                assert!(
                    matches!(&a.state,engine::State::Service{port,..} if port=="8080"),
                    "Enter submitted simulation command"
                );
                self.pending_events.push(key(egui::Key::Tab, false));
            }
            5 => {
                assert_ne!(
                    ctx.memory(|m| m.focused()),
                    self.smoke_focus,
                    "Tab moves focus to next control"
                );
                self.pending_events.push(key(egui::Key::Tab, true));
            }
            6 => {
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    self.smoke_focus,
                    "Shift+Tab returns focus"
                );
                shot("02-workspace");
            }
            7 => {
                self.smoke_width = ctx.input(|i| i.content_rect().width()) * ctx.zoom_factor();
                self.scale = 1.25;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(800., 640.)));
            }
            8 => {
                let size = ctx.input(|i| i.content_rect().size());
                println!("Resized content area: {size:?}; zoom {}", ctx.zoom_factor());
                assert!(ctx.zoom_factor() > 1.2, "125% text scaling applied");
                assert!(
                    (size.x * ctx.zoom_factor() - self.smoke_width).abs() > 10.,
                    "native viewport actually resized"
                );
                shot("03-resized-125-percent");
            }
            9 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                self.selected = None;
                self.synthetic = true;
                self.scale = 1.;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            10 => shot("04-synthetic-catalog"),
            11 => {
                println!(
                    "PASS native Fedora smoke: render, text input, Enter, Tab, Shift+Tab, resize, 125% scale, synthetic catalog. Screenshots exclude desktop."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        self.smoke_stage += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_at_narrow_and_wide_sizes_and_keyboard_controls() {
        for width in [720., 980., 1400.] {
            let ctx = egui::Context::default();
            let mut app = LabApp::from_context(&ctx, true);
            app.progress = Progress {
                schema_version: 1,
                ..Default::default()
            };
            app.smoke = false;
            app.scale = 1.25;
            for _ in 0..2 {
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 640.),
                    )),
                    ..Default::default()
                };
                let _ = ctx.run(input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| app.library(ui));
                    });
                });
            }
            app.launch(0);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 640.),
                )),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
                    app.workspace_actions(ui, 0);
                    assert!(
                        ui.min_rect().right() <= width + 1.,
                        "command bar fits {width}"
                    );
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| app.workspace(ui, 0));
                });
            });
            assert!(ctx.memory(|m| m.focused()).is_some());
        }
    }
    #[test]
    fn synthetic_catalog_paginates_and_searches_in_the_real_ui() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true);
        app.smoke = false;
        app.synthetic = true;
        app.page = 74;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.library(ui));
        });
        assert_eq!(app.page, 74);
        app.filter.search = "impossible query zyx".into();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.library(ui));
        });
        assert_eq!(app.page, 0);
    }
}
