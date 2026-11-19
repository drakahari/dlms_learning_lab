use super::*;
use dlms_learning_lab::magic_subnet::{self as math, Problem, Stage};

impl LabApp {
    fn magic_practice_controls(
        &mut self,
        ui: &mut egui::Ui,
        session: &mut math::Session,
        active: bool,
    ) {
        ui.add_enabled_ui(active, |ui| {
            let mut changed = false;
            let wide = ui.available_width() >= 720.;
            let mut fields = |ui: &mut egui::Ui| {
                for (label, value, width) in [
                    ("Example IP address", &mut session.example.address, 220_f32),
                    ("Prefix (no slash)", &mut session.example.prefix, 100_f32),
                ] {
                    let mut edit = |ui: &mut egui::Ui, space: f32| {
                        ui.add_sized(
                            [width.min(space), controls::height(ui)],
                            controls::singleline(ui, value)
                                .font(egui::TextStyle::Monospace)
                                .char_limit(64),
                        )
                    };
                    let r = if wide {
                        ui.horizontal(|ui| {
                            ui.label(label);
                            let space = ui.available_size_before_wrap().x;
                            edit(ui, space)
                        })
                        .inner
                    } else {
                        controls::form_row(ui, label, |ui, space| edit(ui, space)).inner
                    };
                    name_control(&r, label);
                    self.metrics.magic_problem_inputs.push(r.rect);
                    changed |= r.changed();
                }
            };
            if wide {
                ui.horizontal_wrapped(|ui| fields(ui));
            } else {
                fields(ui);
            }
            if changed {
                session.example_checked = None;
                // These two values are supplied scaffolding in lesson 1. Keep them
                // synchronized without filling any of the learner's calculations.
                if session.stage == Stage::Blocks
                    && let Ok(p) = session.example_problem()
                {
                    let c = p.calculate().unwrap();
                    session.example.answers[0] = c.mask.to_string();
                    session.example.answers[1] = c.mask_octet.to_string();
                }
            }
            if let Ok(p) = session.example_problem() {
                let checked = session.example_checked.as_ref() == Some(&session.example);
                guided_worksheet(
                    ui,
                    session.stage,
                    p,
                    &mut session.example.answers,
                    checked,
                    &mut self.metrics,
                );
            } else if let Err(e) = session.example_problem() {
                ui.colored_label(palette(ui).error, e);
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(primary(ui, "Check answers"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    session.example_checked = Some(session.example.clone());
                }
                let r = ui.add_enabled(
                    session.example_problem().is_ok(),
                    button("Show worked answer"),
                );
                self.metrics.insert = Some(r.rect);
                if r.clicked()
                    && let Ok(p) = session.example_problem()
                {
                    for (i, f) in session.stage.fields(p).unwrap().iter().enumerate() {
                        session.example.answers[i] = f.expected.clone();
                    }
                    session.example_checked = Some(session.example.clone());
                }
                if ui.add(button("Reset example")).clicked() {
                    session.example = math::Session::new(session.problem, session.stage).example;
                    session.example_checked = None;
                    self.magic_scopes[1] = self.magic_scopes[1].wrapping_add(1);
                }
            });
            if session.example_checked.as_ref() == Some(&session.example) {
                match session.example_problem() {
                    Err(e) => {
                        ui.colored_label(palette(ui).error, e);
                    }
                    Ok(p) => {
                        let c = math::checks(session.stage, p, &session.example.answers);
                        let pass = c.iter().all(|v| v.0);
                        ui.colored_label(
                            if pass {
                                palette(ui).success
                            } else {
                                palette(ui).error
                            },
                            if pass {
                                "Calculations agree"
                            } else {
                                "Review the marked calculations, then try again."
                            },
                        );
                        controls::collapsing_header(ui, "Worked result · compare after checking")
                            .show(ui, |ui| {
                                worked_table(ui, p);
                            });
                    }
                }
            } else {
                ui.label(
                    RichText::new(
                        "Not checked for this address and prefix. Recheck any values you kept.",
                    )
                    .color(palette(ui).muted),
                );
            }
        });
        ui.separator();
        if ui.add(button("Open assignment")).clicked() {
            session.view = teaching::View::Assignment;
            self.lesson_scroll_reset = true;
        }
    }
    pub(super) fn magic_subnet_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let challenge = self.workspace_header(ui, i);
        let engine::State::MagicSubnet { session } = &self.progress.attempts[&s.id].state else {
            return;
        };
        let mut session = (**session).clone();
        let before = session.clone();
        let active = !(self.reset_pending || self.settings_open || self.data_tools.modal_open());
        card(ui).show(ui,|ui| {
            ui.label(RichText::new("IPv4 foundations: the magic-number method").strong().color(palette(ui).accent));
            let number=match session.stage {Stage::Blocks=>1,Stage::Boundaries=>2,Stage::CrossOctet=>3,Stage::Aligned=>4};
            ui.label(RichText::new(format!("CCNA · Lesson {number} of 4")).small().color(palette(ui).muted));
            ui.heading(&s.title);ui.add_space(8.);
            if !challenge {
                ui.horizontal_wrapped(|ui| {
                    for (v,label) in [(teaching::View::Introduction,"Introduction"),(teaching::View::Example,"Try example · ungraded"),(teaching::View::Assignment,"Assignment")] {
                        let r=ui.selectable_label(session.view==v,label); self.metrics.lesson_tabs.push(r.rect);
                        if r.clicked() {session.view=v;self.lesson_scroll_reset=true;self.focus_input=true;}
                    }
                });
            }
            ui.separator();
            let view=if challenge {teaching::View::Assignment}else{session.view};
            let view_key=match view {teaching::View::Introduction=>0,teaching::View::Example=>1,teaching::View::Assignment=>2};
            let practice_generation=if view==teaching::View::Example {self.magic_scopes[1]}else{0};
            ui.push_id((&s.id,view_key,challenge,self.magic_scopes[0],practice_generation),|ui| {match view {
                teaching::View::Introduction=>{
                    introduction(ui,session.stage);
                    ui.horizontal_wrapped(|ui| {
                        let r=ui.add(primary(ui,"Try this example"));self.metrics.example_action=Some(r.rect);
                        if r.clicked(){session.view=teaching::View::Example;self.lesson_scroll_reset=true;}
                        let r=ui.add(button("Go to assignment"));self.metrics.assignment_action=Some(r.rect);
                        if r.clicked(){session.view=teaching::View::Assignment;self.lesson_scroll_reset=true;}
                    });
                }
                teaching::View::Example=>{
                    ui.strong("GUIDED PRACTICE · Find this address’s block.");
                    self.magic_practice_controls(ui,&mut session,active);
                }
                teaching::View::Assignment=>{
                    ccna_presentation::lead(ui,"YOUR TASK",session.stage.objective());
                    ui.monospace(format!("Given address: {}/{}",session.problem.ip,session.problem.prefix));
                    ui.label("Complete every requested value. Your example answers do not carry over.");
                    ui.add_space(8.);ui.separator();ui.add_space(8.);
                    ui.add_enabled_ui(active,|ui| {
                        let checked=session.checked.as_ref()==Some(&session.answers);
                        worksheet(ui,session.stage,session.problem,&mut session.answers,checked,&mut self.metrics);
                        let r=ui.add(primary(ui,"Check answers"));self.metrics.check=Some(r.rect);
                        if r.clicked(){session.checked=Some(session.answers.clone());}
                        if session.checked.as_ref()==Some(&session.answers) {
                            let results=math::checks(session.stage,session.problem,&session.answers);
                            if results.iter().all(|x|x.0) {
                                ui.add_space(10.);ui.separator();
                                ui.colored_label(palette(ui).success,RichText::new("Assignment complete").strong());
                                ui.label("Your boundaries agree with the mask. Network and broadcast are reserved in this exercise; usable hosts lie between them.");
                                worked_table(ui,session.problem);
                                ui.horizontal_wrapped(|ui| {
                                    if ui.add(button("Back to library")).clicked(){self.selected=None;}
                                    let next=self.units.iter().find(|u|u.id==s.unit).and_then(|u|u.labs.iter().position(|id|id==&s.id).and_then(|n|u.labs.get(n+1))).and_then(|id|self.scenarios.iter().position(|v|&v.id==id));
                                    if let Some(next)=next && ui.add(primary(ui,"Next lab")).clicked(){self.launch(next);}
                                });
                            } else {ui.colored_label(palette(ui).error,"Some values need another look. Read the feedback beside each field and retry.");}
                        }
                        if !challenge {mask_reference(ui,session.problem);}
                        controls::collapsing_header(ui,if challenge {"Request a hint"}else{"Show a hint"}).show(ui,|ui|{
                            let n=session.hints as usize;
                            for h in s.hints.iter().take(n+1){ui.label(h);}
                            if n+1<s.hints.len() && ui.add(button("Another hint")).clicked(){session.hints+=1;}
                        });
                        let why=controls::collapsing_header(ui,"Why does this work?").show(ui,|ui|{binary(ui,session.problem);});
                        self.metrics.help=Some(why.header_response.rect);self.metrics.help_visible=why.body_returned.is_some();
                    });
                }
            }});
        });
        if self.focus_input
            && active
            && (challenge || session.view != teaching::View::Introduction)
            && let Some(id) = self.metrics.focus
        {
            ui.memory_mut(|m| m.request_focus(id));
            self.focus_input = false;
        }
        ui.label(
            RichText::new(
                "Your work is saved locally when you leave. All addresses here are simulated.",
            )
            .small()
            .color(palette(ui).muted),
        );
        if before != session {
            if let Some(a) = self.progress.attempts.get_mut(&s.id)
                && let engine::State::MagicSubnet { session: target } = &mut a.state
            {
                **target = session;
            }
            self.dirty = true;
        }
    }
}

// Presentation labels describe the expected value; stable model fields keep
// their grading identity and persisted answer positions.
fn field_label(stage: Stage, p: Problem, i: usize) -> String {
    let c = p.calculate().unwrap();
    if stage == Stage::Blocks {
        match i {
            1 => format!("Mask value in octet {}", c.octet + 1),
            3 => format!("IP octet {} · block start", c.octet + 1),
            4 => format!("IP octet {} · block end", c.octet + 1),
            _ => stage.fields(p).unwrap()[i].label.into(),
        }
    } else if i == 0 && matches!(stage, Stage::CrossOctet | Stage::Aligned) {
        "Mask octet position (1–4)".into()
    } else {
        stage.fields(p).unwrap()[i].label.into()
    }
}
fn field_hint(stage: Stage, i: usize, numeric: bool) -> &'static str {
    if stage == Stage::Blocks && i == 0 {
        "Dotted-decimal mask"
    } else if numeric {
        "Whole number"
    } else {
        "Dotted-decimal address"
    }
}
fn worksheet_row(
    ui: &mut egui::Ui,
    stage: Stage,
    p: Problem,
    answers: &mut [String; 6],
    checked: bool,
    metrics: &mut Metrics,
    field: (usize, Option<String>),
) {
    let (i, equation) = field;
    let fields = stage.fields(p).unwrap();
    let f = &fields[i];
    let label_text = field_label(stage, p, i);
    let feedback = math::checks(stage, p, answers)[i].clone();
    let focus_index = fields
        .iter()
        .enumerate()
        .find(|(n, _)| answers[*n].is_empty())
        .map_or(0, |(n, _)| n);
    let inline = ui.available_width() >= 480.;
    let label_width = if inline {
        240.
    } else {
        ui.available_width().min(240.)
    };
    ui.push_id(("magic-field", i), |ui| {
        let mut row = |ui: &mut egui::Ui| {
            let shown = equation.as_deref().unwrap_or(&label_text);
            if inline {
                controls::label(ui, shown, label_width);
            } else {
                ui.strong(shown);
            }
            let width = 240_f32.min(ui.available_width());
            let r = ui.add_sized(
                [width, controls::height(ui)],
                controls::singleline(ui, &mut answers[i])
                    .font(egui::TextStyle::Monospace)
                    .hint_text(field_hint(stage, i, f.numeric))
                    .char_limit(64),
            );
            metrics.inputs.push(r.rect);
            if i == focus_index {
                metrics.focus = Some(r.id);
            }
            name_control(&r, &label_text);
        };
        if inline {
            ui.horizontal_top(|ui| row(ui));
        } else {
            row(ui);
        }
        if checked && !feedback.0 {
            // The model's first clause is its historical field label. Replace
            // just that label; preserve the specific misconception diagnosis.
            let message = feedback
                .1
                .strip_prefix(&format!("{}: ", f.label))
                .unwrap_or(&feedback.1);
            ui.add(egui::Label::new(RichText::new(message).color(palette(ui).error)).wrap());
        }
    });
    ui.add_space(4.);
}
fn worksheet(
    ui: &mut egui::Ui,
    stage: Stage,
    p: Problem,
    answers: &mut [String; 6],
    checked: bool,
    metrics: &mut Metrics,
) {
    for i in 0..stage.fields(p).unwrap().len() {
        worksheet_row(ui, stage, p, answers, checked, metrics, (i, None));
    }
}
fn guided_worksheet(
    ui: &mut egui::Ui,
    stage: Stage,
    p: Problem,
    answers: &mut [String; 6],
    checked: bool,
    metrics: &mut Metrics,
) {
    let c = p.calculate().unwrap();
    let value = p.ip.octets()[c.octet];
    ui.add_space(6.);
    ui.strong(format!(
        "1. Identify the mask value · /{} ends in octet {}",
        p.prefix,
        c.octet + 1
    ));
    if stage == Stage::Blocks {
        worksheet_row(ui, stage, p, answers, checked, metrics, (0, None));
        worksheet_row(ui, stage, p, answers, checked, metrics, (1, None));
    } else {
        ui.monospace(format!(
            "Mask: {} · value in octet {}: {}",
            c.mask,
            c.octet + 1,
            c.mask_octet
        ));
        if matches!(stage, Stage::CrossOctet | Stage::Aligned) {
            worksheet_row(ui, stage, p, answers, checked, metrics, (0, None));
        }
    }
    if p.prefix.is_multiple_of(8) {
        ccna_presentation::prose(
            ui,
            "The prefix ends on an octet boundary. Use the last network octet, whose mask value is 255; later octets are entirely host bits.",
        );
    }
    ui.add_space(4.);
    ui.strong("2. Calculate the block size");
    if stage == Stage::Blocks {
        worksheet_row(
            ui,
            stage,
            p,
            answers,
            checked,
            metrics,
            (2, Some(format!("Block size: 256 − {} =", c.mask_octet))),
        );
    } else {
        ui.monospace(format!(
            "256 − {} = {} · step in octet {}",
            c.mask_octet,
            c.block,
            c.octet + 1
        ));
    }
    ui.add_space(4.);
    ui.strong("3. Count boundaries from 0 using that block size");
    let reveal = checked && math::checks(stage, p, answers).iter().all(|v| v.0);
    practice_range(ui, p, reveal);
    ui.add_space(4.);
    ui.strong(format!(
        "4. Find the interval containing {value} in IP octet {}",
        c.octet + 1
    ));
    if stage == Stage::Blocks {
        worksheet_row(ui, stage, p, answers, checked, metrics, (3, None));
        worksheet_row(ui, stage, p, answers, checked, metrics, (4, None));
    } else {
        ccna_presentation::prose(
            ui,
            "Network starts the block. Broadcast is one address before the next subnet; usable hosts are between them.",
        );
        if c.octet < 3 {
            ccna_presentation::prose(
                ui,
                "For full addresses, set later octets to 0 for network and 255 for broadcast. The next subnet starts the next block.",
            );
        }
        let first = usize::from(matches!(stage, Stage::CrossOctet | Stage::Aligned));
        for i in first..stage.fields(p).unwrap().len() {
            worksheet_row(ui, stage, p, answers, checked, metrics, (i, None));
        }
    }
}
/// All boundaries are visible, but the address's containing interval stays
/// unmarked until a correct check or an explicit worked-answer request.
fn practice_range(ui: &mut egui::Ui, p: Problem, reveal: bool) {
    let c = p.calculate().unwrap();
    ui.label(format!("IP octet {} · full range 0–255", c.octet + 1));
    let (r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.), egui::Sense::hover());
    let line = r.shrink2(egui::vec2(5., 6.));
    if reveal {
        let start = line.left() + line.width() * f32::from(c.start) / 256.;
        let end = line.left() + line.width() * (f32::from(c.end) + 1.) / 256.;
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(start, line.top()),
                egui::pos2(end.max(start + 2.), line.bottom()),
            ),
            0.,
            palette(ui).accent,
        );
    }
    ui.painter().line_segment(
        [line.left_center(), line.right_center()],
        egui::Stroke::new(1_f32, palette(ui).text),
    );
    let step = usize::from(c.block).max(16);
    for n in (0..=256).step_by(step) {
        let x = line.left() + line.width() * n as f32 / 256.;
        ui.painter().line_segment(
            [egui::pos2(x, line.top()), egui::pos2(x, line.bottom())],
            egui::Stroke::new(1_f32, palette(ui).text),
        );
    }
    let count = 256 / c.block;
    if count <= 16 {
        let boundaries = (0..count)
            .map(|n| (n * c.block).to_string())
            .collect::<Vec<_>>()
            .join(" · ");
        ui.add(
            egui::Label::new(
                RichText::new(format!("Boundaries: {boundaries} · 256 (end of octet)")).monospace(),
            )
            .wrap(),
        );
    } else {
        ui.add(egui::Label::new(format!("Boundaries every {}: 0, {}, {}, …, {}. Diagram ticks every 16; 256 is the end of the octet.",c.block,c.block,2*c.block,256-c.block)).wrap());
    }
    if reveal {
        ui.strong(format!(
            "{} belongs in {}–{}",
            p.ip.octets()[c.octet],
            c.start,
            c.end
        ));
    }
}
fn section(ui: &mut egui::Ui, n: usize, title: &str, text: &str) {
    ui.add_space(10.);
    ui.label(RichText::new(format!("{n}. {title}")).strong().size(19.));
    ccna_presentation::prose(ui, text);
    ui.add_space(6.);
}
fn introduction(ui: &mut egui::Ui, stage: Stage) {
    ccna_presentation::lead(ui, "WHAT YOU’LL LEARN", stage.objective());
    ui.label(
        RichText::new(if stage == Stage::Blocks {
            "Start here. No subnetting or binary knowledge is needed."
        } else {
            "Builds on the earlier magic-number worksheets. You can revisit them at any time."
        })
        .color(palette(ui).muted),
    );
    ui.separator();
    if ui.available_width() >= 920. {
        ui.columns(2, |columns| {
            intro_concepts(&mut columns[0], stage);
            intro_worked(&mut columns[1], stage);
        });
    } else {
        intro_concepts(ui, stage);
        ui.separator();
        intro_worked(ui, stage);
    }
    controls::collapsing_header(ui, "Why 256 minus the mask? · a focused bit explanation")
        .show(ui, |ui| binary(ui, stage.worked()));
    ui.add_space(8.);
    ui.label("These lessons reserve network and broadcast in conventional /1–/30 subnets. /31 point-to-point links and /32 host routes are special cases: subtracting two hosts is not a universal rule.");
    ui.add_space(10.);
}
fn intro_concepts(ui: &mut egui::Ui, stage: Stage) {
    if stage == Stage::Blocks {
        section(
            ui,
            1,
            "Read the address and its mask",
            "An IPv4 address has four numbers (octets), each from 0 to 255. /26 describes its group size using a subnet mask; the mask is a grouping rule, not a second computer’s address.",
        );
        section(
            ui,
            2,
            "Identify the relevant mask octet",
            "For /26, the mask is 255.255.255.192. Use the fourth octet, 192. This is the mask value, not the octet’s position (4).",
        );
        section(
            ui,
            3,
            "Calculate and mark the boundaries",
            "Subtract from 256: 256 − 192 = 64. Start at 0 and add 64: 0, 64, 128, 192. These are the starts of the last-octet blocks for this /26 example.",
        );
        section(
            ui,
            4,
            "Locate 77 and reserve the endpoints",
            "77 lies between 64 and the next boundary, 128: its block is 64–127. Keep 192.168.10 fixed. Network is the first address, .64; broadcast is one before the next boundary, .127. Usable hosts lie inside: .65–.126. The table shows the full addresses.",
        );
    } else if stage == Stage::Boundaries {
        section(
            ui,
            1,
            "The first and last addresses have special jobs",
            "The first address names the network. The last address is broadcast: it addresses the group in this conventional exercise. We reserve both, so hosts use the addresses between them.",
        );
        section(
            ui,
            2,
            "The next boundary closes this block",
            "Find the next subnet first. Broadcast is one address before it; the first usable host is one after network, and the last usable host is one before broadcast. Count whole addresses, not only the final printed number.",
        );
    } else if stage == Stage::CrossOctet {
        section(
            ui,
            1,
            "Find where the prefix ends",
            "/20 fixes two full octets (16 bits) and four more bits in the third octet. The mask is 255.255.240.0. Use the third mask octet: 256 − 240 = 16. The fourth octet is all host bits.",
        );
        section(
            ui,
            2,
            "Carry the whole range forward",
            "A third-octet block of 64–79 covers fourth-octet values 0–255 for every value of the third octet. Its network ends in 64.0 and broadcast ends in 79.255. The next subnet starts at 80.0. That is 16 × 256 = 4096 addresses, not 16 addresses.",
        );
    } else {
        section(
            ui,
            1,
            "No partially filled octet",
            "/24 fixes exactly three full octets. Choose the last network octet, the third: its mask is 255, giving a step of 1. The fourth octet is all host bits and runs from 0 to 255. Do not use the zero mask octet to advance the third octet by 256.",
        );
        section(
            ui,
            2,
            "The same rule works for /16 and /8",
            "For /16, use the second octet and reset both later octets. For /8, use the first and reset the later three. Adding one whole subnet handles carries correctly, including a boundary that crosses into an earlier octet.",
        );
    }
}
fn intro_worked(ui: &mut egui::Ui, stage: Stage) {
    ui.strong("Worked example");
    let p = stage.worked();
    ui.monospace(format!("IP address: {}/{}", p.ip, p.prefix));
    annotated_range(ui, p, true);
    worked_table(ui, p);
}
pub(super) fn worked_table(ui: &mut egui::Ui, p: Problem) {
    let c = p.calculate().unwrap();
    let rows = [
        ("Subnet mask", c.mask.to_string()),
        (
            "Relevant mask octet",
            format!("octet {}: {}", c.octet + 1, c.mask_octet),
        ),
        (
            "Block calculation",
            format!("256 − {} = {}", c.mask_octet, c.block),
        ),
        ("Network · block start", c.network.to_string()),
        (
            "Next subnet",
            c.next.map_or("none".into(), |n| n.to_string()),
        ),
        ("Broadcast (next − 1)", c.broadcast.to_string()),
        ("Usable hosts", format!("{} – {}", c.first, c.last)),
    ];
    if ui.available_width() >= 470. {
        egui::Grid::new(ui.id().with("worked-table"))
            .num_columns(2)
            .min_row_height(0.)
            .spacing([20., 9.])
            .show(ui, |ui| {
                for (label, value) in rows {
                    ui.label(label);
                    ui.add(egui::Label::new(RichText::new(value).monospace()).wrap());
                    ui.end_row();
                }
            });
    } else {
        for (label, value) in rows {
            ui.strong(label);
            ui.add(egui::Label::new(RichText::new(value).monospace()).wrap());
            ui.add_space(5.);
        }
    }
}
pub(super) fn range_diagram(ui: &mut egui::Ui, p: Problem) {
    // Existing IPv4 lesson retains its accepted presentation.
    annotated_range(ui, p, false);
}
fn annotated_range(ui: &mut egui::Ui, p: Problem, explicit_label: bool) {
    let c = p.calculate().unwrap();
    let value = p.ip.octets()[c.octet];
    ui.label(format!(
        "Octet {}: locate {value} between its boundaries",
        c.octet + 1
    ));
    let blocks = 256 / u32::from(c.block);
    let selected = u32::from(c.start) / u32::from(c.block);
    let first = if blocks <= 8 {
        0
    } else {
        selected.saturating_sub(2)
    };
    let last = if blocks <= 8 {
        blocks
    } else {
        (first + 5).min(blocks)
    };
    let labels = (first..last)
        .map(|i| {
            let start = i * u32::from(c.block);
            let end = start + u32::from(c.block) - 1;
            (format!("{start}–{end}"), i == selected)
        })
        .collect::<Vec<_>>();
    lesson_presentation::range_blocks(ui, &labels);
    ui.strong(if explicit_label {
        format!("{value} belongs in {}–{}", c.start, c.end)
    } else {
        format!("{value} is here")
    });
    ui.label(format!(
        "{} ≤ {value} ≤ {}. {}",
        c.start,
        c.end,
        if c.octet == 3 {
            "Keep the first three IP octets fixed."
        } else {
            "Set later octets to 0 for network and 255 for broadcast."
        }
    ));
}
fn binary(ui: &mut egui::Ui, p: Problem) {
    let c = p.calculate().unwrap();
    ui.label("An IPv4 address has 32 bits, grouped into four octets of eight. For /26, three whole octets use 24 network bits; the other two network bits are in octet 4. Their bit places add to 128 + 64 = 192. A /27 fixes one more place: 128 + 64 + 32 = 224.");
    let network_bits = usize::from(p.prefix) % 8;
    let count = if network_bits == 0 { 8 } else { network_bits };
    let mask = format!("{:08b}", c.mask_octet);
    let ip = format!("{:08b}", p.ip.octets()[c.octet]);
    ui.strong("network bits | host bits");
    ui.monospace(format!(
        "Mask octet {}: {} | {}",
        c.octet + 1,
        &mask[..count],
        &mask[count..]
    ));
    ui.monospace(format!(
        "IP octet   {}: {} | {}",
        c.octet + 1,
        &ip[..count],
        &ip[count..]
    ));
    let network = format!("{:08b}", c.network.octets()[c.octet]);
    let broadcast = format!("{:08b}", c.broadcast.octets()[c.octet]);
    ui.monospace(format!(
        "Network:   {} | {}",
        &network[..count],
        &network[count..]
    ));
    ui.monospace(format!(
        "Broadcast: {} | {}",
        &broadcast[..count],
        &broadcast[count..]
    ));
    ui.label("A mask 1 fixes a network bit; a mask 0 leaves a host bit free to change. The mask is a grouping rule, not another computer’s address. Bit places in one octet are 128, 64, 32, 16, 8, 4, 2 and 1.");
    ui.label(format!("IPv4 has 32 bits. /{} leaves {} host bits, so a complete subnet has 2^{} = {} addresses. Setting all host bits to zero gives network; setting them all to one gives broadcast in this exercise.",p.prefix,32-p.prefix,32-p.prefix,1u64<<(32-p.prefix)));
    if c.octet < 3 {
        ui.label("The later octets also contain host bits. A block in this octet is not the same as a total address count.");
    }
}

fn mask_reference(ui: &mut egui::Ui, p: Problem) {
    controls::collapsing_header(ui,"Mask and prefix reference").show(ui,|ui| {
        let c=p.calculate().expect("checked problem");
        ui.monospace(format!("/{} → {}",p.prefix,c.mask));
        ui.label("Each complete set of eight network bits contributes 255 to the mask. In the next octet, add the fixed bit places from left to right: 128, 64, 32, 16, 8, 4, 2, 1. Any remaining octets are zero.");
        ui.label(format!("The last network bit is in octet {}. Its mask value is {}. Subtract that value from 256 to find the step in that octet.",c.octet+1,c.mask_octet));
    });
}
