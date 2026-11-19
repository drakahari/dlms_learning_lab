//! Administration tools opt in only for the administration activity kind.
use super::*;
pub(super) fn console(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    m: &mut Metrics,
    notice: &mut Option<String>,
    phase: (bool, &mut bool),
) {
    let (practice, focus) = phase;
    ui.strong("Simulated server administration");
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    if practice {
        let commands = source
            .guide
            .iter()
            .flat_map(|g| &g.commands)
            .filter(|c| !c.simulator_only)
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            let key = egui::Id::new(("mail-admin-example-command", &source.id));
            let mut n = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, width| {
                let r = controls::dropdown(ui, "mail-admin-command-choice", width)
                    .selected_text(&commands[n].command)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        let start = m.controls.len();
                        for (at, c) in commands.iter().enumerate() {
                            let r = ui.selectable_value(&mut n, at, &c.command);
                            m.record(ui, &format!("example-command:{at}"), &r);
                        }
                        let clip = controls::popup_content_clip(ui);
                        for r in &mut m.controls[start..] {
                            r.2 = clip;
                        }
                    });
                controls::retain_dropdown_focus(ui, &r.response);
                m.record(ui, "example-command", &r.response);
            });
            lesson_presentation::prose(ui, &commands[n].explanation);
            let r = ui.add(button("Insert command"));
            m.record(ui, "insert-command", &r);
            if r.clicked() {
                apply(
                    s,
                    t,
                    &work::Action::Input {
                        text: commands[n].command.clone(),
                    },
                    notice,
                );
                *focus = true;
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, n));
        }
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace)
        * s.output.lines().count().clamp(3, 10) as f32;
    let output=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
        let r=ui.strong("Actual command results");m.record(ui,"output-title",&r);
        controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("mail-admin-output").auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{
            if s.output.is_empty(){lesson_presentation::prose(ui,"Run a supported doveconf or doveadm command. Results appear here; no command has run yet.");}
            else {ui.add(egui::Label::new(RichText::new(s.output.replace('\u{c}',"\n")).monospace()).extend());}
        });
    });
    m.record(ui, "output-viewport", &output.response);
    let mut run = false;
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("mail-admin-input", &source.id, practice)),
            2048,
            ui.is_enabled(),
            "Administration command · Enter to run",
        );
        m.record(ui, "command", &r.response);
        m.fields
            .push(("command".into(), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            apply(s, t, &work::Action::Input { text }, notice);
        }
        if *focus && ui.is_enabled() {
            r.response.request_focus();
            *focus = false;
        }
        run |= ui.is_enabled()
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "Use one command within 2 KiB; the previous input was retained.",
            );
        }
    };
    let width = controls::button_width(ui, "Run command");
    let clicked = if ui.available_width()
        > width + ui.text_style_height(&egui::TextStyle::Body) * 8. + ui.spacing().item_spacing.x
    {
        ui
            .horizontal(|ui| {
                let available =
                    (ui.available_width() - width - ui.spacing().item_spacing.x).max(1.);
                ui.allocate_ui_with_layout(
                    egui::vec2(available, controls::height(ui)),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| field(ui, m),
                );
                let r = ui.add(button("Run command"));
                m.record(ui, "run-admin", &r);
                r.clicked()
            })
            .inner
    } else {
        field(ui, m);
        let r = ui.add(button("Run command"));
        m.record(ui, "run-admin", &r);
        r.clicked()
    };
    if run || clicked {
        let line = s.input.clone();
        apply(s, t, &work::Action::Admin { line }, notice);
        *focus = true;
    }
}
