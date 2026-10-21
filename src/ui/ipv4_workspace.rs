//! Approved compact IPv4 assignment used by the installed native workspace.
use super::*;
use dlms_learning_lab::ipv4::{self, Diagnosis, Exercise, SecondResult};
use dlms_learning_lab::workspace::{WorkspaceData, snapshot};

pub(super) struct Assignment {
    pub exercise: Exercise,
    checked: bool,
    pub checked_answers: Option<[String; 5]>,
    pub hints: usize,
    hints_shown: bool,
    binary: bool,
    guidance_requested: bool,
    pub focus: bool,
}
impl Assignment {
    pub fn new(state: &engine::State) -> Self {
        Self {
            exercise: Exercise::from_state(state)
                .expect("IPv4 workspace uses the validated subnet scenario"),

            checked: false,
            checked_answers: None,
            hints: 0,
            hints_shown: false,
            binary: false,
            guidance_requested: false,
            focus: true,
        }
    }
    pub fn hide_guidance(&mut self) {
        self.hints_shown = false;
        self.binary = false;
        self.guidance_requested = false;
    }
    pub fn from_saved(exercise: Exercise, checked: Option<[String; 5]>, hints: usize) -> Self {
        let state = engine::State::Subnet {
            ip: exercise.ip,
            prefix: exercise.prefix,
            answers: exercise.answers.clone(),
            second: exercise.second.clone(),
        };
        let mut view = Self::new(&state);
        view.exercise = exercise;
        view.checked = checked.is_some();
        view.checked_answers = checked;
        view.hints = hints;
        view
    }
    fn second_result(&self) -> SecondResult {
        ipv4::second_result(
            self.exercise.ip,
            self.exercise.prefix,
            self.exercise.second.as_deref().unwrap_or(""),
        )
    }
    pub fn verified(&self) -> bool {
        self.exercise.solved() && self.checked_answers.as_ref() == Some(&snapshot(&self.exercise))
    }
    pub fn save_ui(&self, data: &mut WorkspaceData, practice: bool) {
        if practice {
            data.practice_check = self.checked_answers.clone();
        } else {
            data.ipv4_check = self.checked_answers.clone();
        }
    }
    fn message(&self, i: usize) -> Option<&'static str> {
        if i == 4 {
            return match self.second_result() {
                SecondResult::Blank => Some("Enter an unused address for workstation 2."),
                SecondResult::Malformed => Some(
                    "Use four numbers from 0 to 255, separated by dots. Leave off the /prefix.",
                ),
                SecondResult::Network => Some(
                    "This identifies the whole network. It cannot be assigned to a workstation.",
                ),
                SecondResult::Broadcast => {
                    Some("This is the broadcast address. Choose an ordinary host address instead.")
                }
                SecondResult::Occupied => Some(
                    "Workstation 1 already uses this address. Choose a different usable address.",
                ),
                SecondResult::Outside => Some(
                    "This address is outside the supplied subnet. Both workstations must share that subnet.",
                ),
                SecondResult::Valid => None,
            };
        }
        match dlms_learning_lab::ipv4::diagnose(&self.exercise, i) {
            Diagnosis::Correct => None,
            Diagnosis::Blank => Some("Enter a full IPv4 address."),
            Diagnosis::Malformed => {
                Some("Use four numbers from 0 to 255, separated by dots. Leave off the /prefix.")
            }
            Diagnosis::HostInsteadOfNetwork => Some(
                "This is a host inside the subnet. The network address must be the first address of its block.",
            ),
            Diagnosis::WrongBoundary if i == 0 => Some(
                "This is not the start of the subnet containing workstation 1. Check the block boundary.",
            ),
            Diagnosis::WrongBoundary => Some(
                "This is not the last address of that subnet. Broadcast is the end of the same block.",
            ),
            Diagnosis::ReservedHost => Some(
                "This is reserved for network or broadcast. Usable hosts lie between those two addresses.",
            ),
            Diagnosis::OutsideSubnet => Some(
                "This address is outside workstation 1’s subnet. Check the network and broadcast boundaries.",
            ),
            Diagnosis::WrongHostEnd if i == 2 => {
                Some("The first usable address is one after the network address.")
            }
            Diagnosis::WrongHostEnd => {
                Some("The last usable address is one before the broadcast address.")
            }
        }
    }
    pub fn render(&mut self, ui: &mut egui::Ui, challenge: bool, metrics: &mut Metrics) {
        ui.heading("Configure a second workstation");
        ui.label(format!("A workstation is configured as {}/{}. Determine its network address, broadcast address, and usable host range, then assign a valid, different address to a second workstation.",self.exercise.ip,self.exercise.prefix));
        let checked_second = self
            .checked_answers
            .as_ref()
            .is_some_and(|v| v[4] == self.exercise.second.as_deref().unwrap_or(""));
        topology(ui, &self.exercise, checked_second);
        ui.add_space(2.);
        let wide = ui.available_width() >= 640.;
        let labels = [
            "Network address",
            "Broadcast address",
            "Usable range — first",
            "Usable range — last",
            "Workstation 2",
        ];
        let label_width = (ui.available_width() * 0.45).clamp(100., 154.);
        let field_width = (ui.available_width() - label_width - 12.).clamp(70., 220.);
        let feedback_width = (ui.available_width() - label_width - field_width - 24.).max(80.);
        let mut submitted = false;
        if wide {
            egui::Grid::new("ipv4-assignment-fields")
                .num_columns(3)
                .spacing([12., 6.])
                .show(ui, |ui| {
                    for (i, label) in labels.iter().enumerate() {
                        field_label(ui, label, label_width);
                        submitted |= self.field(ui, i, label, field_width, metrics);
                        ui.allocate_ui_with_layout(
                            egui::vec2(feedback_width, CONTROL),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                if self.checked
                                    && let Some(message) = self.message(i)
                                {
                                    ui.add(
                                        egui::Label::new(RichText::new(message).small().color(RED))
                                            .wrap(),
                                    );
                                }
                            },
                        );
                        ui.end_row();
                    }
                });
        } else {
            for (i, label) in labels.iter().enumerate() {
                ui.horizontal(|ui| {
                    field_label(ui, label, label_width);
                    submitted |= self.field(ui, i, label, field_width, metrics);
                });
                if self.checked
                    && let Some(message) = self.message(i)
                {
                    ui.add(egui::Label::new(RichText::new(message).small().color(RED)).wrap());
                }
            }
        }
        ui.add_space(6.);
        let r = if field_width >= 190. {
            ui.horizontal(|ui| {
                let offset = metrics.inputs[0].left() - ui.cursor().left();
                ui.add_space(offset.max(0.));
                ui.add_sized([field_width, CONTROL], primary("Check configuration"))
            })
            .inner
        } else {
            ui.add_sized(
                [220_f32.min(ui.available_width()), CONTROL],
                primary("Check configuration"),
            )
        };
        metrics.check = Some(r.rect);
        if r.clicked() || submitted {
            self.checked = true;
            self.checked_answers = Some(snapshot(&self.exercise));
        }
        if self.verified() {
            ui.label(RichText::new("Configuration valid. The boundaries and usable range are correct, and workstation 2 has a different usable address.").color(GREEN));
        }
        self.guidance(ui, challenge, metrics);
    }
    fn field(
        &mut self,
        ui: &mut egui::Ui,
        i: usize,
        label: &str,
        width: f32,
        metrics: &mut Metrics,
    ) -> bool {
        let value = if i < 4 {
            &mut self.exercise.answers[i]
        } else {
            self.exercise.second.get_or_insert_with(String::new)
        };
        let r = ui.add_sized(
            [width, CONTROL],
            egui::TextEdit::singleline(value)
                .id_salt(("assignment-address", i))
                .font(egui::TextStyle::Monospace)
                .hint_text("IPv4 address")
                .char_limit(64),
        );
        if r.changed() {
            super::workspace::bound(value, 64);
        }
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label));
        metrics.inputs.push(r.rect);
        if i == 0 {
            metrics.focus = Some(r.id);
            if self.focus {
                r.request_focus();
                self.focus = false;
            }
        }
        r.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter))
    }
    fn guidance(&mut self, ui: &mut egui::Ui, challenge: bool, metrics: &mut Metrics) {
        if challenge && !self.guidance_requested {
            if ui.link("Request guidance (optional)").clicked() {
                self.guidance_requested = true;
            }
            return;
        }
        ui.horizontal_wrapped(|ui| {
            let hint_label = match if self.hints_shown { self.hints } else { 0 } {
                0 => "Show a hint",
                1 | 2 => "Next hint",
                _ => "Hide hints",
            };
            let r = ui.link(hint_label);
            metrics.hint = Some(r.rect);
            if r.clicked() {
                if self.hints_shown {
                    self.hints = (self.hints + 1) % 4;
                } else {
                    self.hints = self.hints.max(1);
                }
                self.hints_shown = self.hints > 0;
            }
            let r = ui.link(if self.binary {
                "Hide binary explanation"
            } else {
                "How does the prefix work?"
            });
            metrics.why = Some(r.rect);
            if r.clicked() {
                self.binary = !self.binary;
            }
        });
        metrics.help_visible = (self.hints_shown && self.hints > 0) || self.binary;
        if self.hints_shown && self.hints > 0 {
            ui.label(
                RichText::new(format!("Hint {} of 3", self.hints))
                    .small()
                    .color(ACCENT),
            );
            let host_bits = 32 - self.exercise.prefix;
            let hint=match self.hints {
                1=>format!("/{} leaves {host_bits} host bits, so each block contains {} addresses. Find the block containing the supplied address.",self.exercise.prefix,self.exercise.capacity().unwrap_or(0)),
                2=>"Keep the network bits fixed. At the octet containing the boundary, block starts are multiples of that octet’s block size; later host octets also vary.".into(),
                _=>"The first and last addresses are reserved for network and broadcast. Usable hosts lie strictly between them; the second workstation must also avoid the address already in use.".into(),
            };
            ui.label(hint);
        }
        if self.binary {
            ui.separator();
            let host_bits = 32 - self.exercise.prefix;
            ui.label(format!("An IPv4 address has 32 bits. /{} fixes the first {} and leaves {host_bits} that can change: 2^{host_bits} = {} addresses per block.", self.exercise.prefix,self.exercise.prefix,self.exercise.capacity().unwrap_or(0)));
            let octet = (self.exercise.prefix as usize / 8).min(3);
            let split = self.exercise.prefix as usize % 8;
            let address = format!("{:08b}", self.exercise.ip.octets()[octet]);
            let mask = format!(
                "{:08b}",
                std::net::Ipv4Addr::from(ipv4::mask(self.exercise.prefix).unwrap_or(0)).octets()
                    [octet]
            );
            ui.label(
                RichText::new("network bits | host bits")
                    .strong()
                    .color(ACCENT),
            );
            ui.label(format!(
                "Octet {} (earlier octets are fixed network bits):",
                octet + 1
            ));
            ui.add(
                egui::Label::new(
                    RichText::new(format!(
                        "Address bits: {} | {}",
                        &address[..split],
                        &address[split..]
                    ))
                    .monospace(),
                )
                .wrap(),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(format!(
                        "Mask bits:    {} | {}",
                        &mask[..split],
                        &mask[split..]
                    ))
                    .monospace(),
                )
                .wrap(),
            );
            ui.label("Address bits encode the computer’s number. Mask bits select which positions stay fixed: 1 keeps that address bit; 0 lets it vary.");
            ui.label("Keep the network bits unchanged. Setting every host bit to 0 gives the network address; setting every host bit to 1 gives the broadcast address in this exercise. Neither boundary can be assigned to a workstation.");
            ui.label("Other prefixes can move the boundary into earlier octets. This conventional broadcast-subnet exercise excludes /31 and /32.");
        }
    }
}
fn field_label(ui: &mut egui::Ui, label: &str, width: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, CONTROL),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.add(egui::Label::new(label).wrap());
        },
    );
}
fn topology(ui: &mut egui::Ui, e: &Exercise, checked: bool) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 86.), egui::Sense::hover());
    let width = (rect.width() * 0.42).clamp(120., 270.);
    let left = egui::Rect::from_min_size(rect.min + egui::vec2(0., 4.), egui::vec2(width, 60.));
    let right = egui::Rect::from_min_size(
        egui::pos2(rect.right() - width, rect.top() + 4.),
        egui::vec2(width, 60.),
    );
    let entered = e.second.as_deref().unwrap_or("");
    let display = if entered.trim().is_empty() {
        "Address to assign".into()
    } else {
        entered.chars().take(28).collect::<String>()
    };
    let valid = ipv4::second_result(e.ip, e.prefix, entered) == SecondResult::Valid;
    let status = if !checked {
        "Not checked"
    } else if valid {
        "Valid host"
    } else {
        "Needs correction"
    };
    let painter = ui.painter();
    painter.line_segment(
        [left.right_center(), right.left_center()],
        egui::Stroke::new(2_f32, ACCENT),
    );
    for (r, text, color) in [
        (left, format!("Workstation 1\n{}", e.ip), MUTED),
        (
            right,
            format!("Workstation 2\n{display}\n{status}"),
            if checked {
                if valid { GREEN } else { RED }
            } else {
                MUTED
            },
        ),
    ] {
        painter.rect(
            r,
            6.,
            Color32::from_rgb(14, 21, 38),
            egui::Stroke::new(1_f32, color),
            egui::StrokeKind::Inside,
        );
        painter.with_clip_rect(r.shrink(3.)).text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(14.),
            Color32::from_rgb(235, 239, 250),
        );
    }
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 7.),
        egui::Align2::CENTER_CENTER,
        format!("Same local network · /{}", e.prefix),
        egui::FontId::proportional(13.),
        MUTED,
    );
}
