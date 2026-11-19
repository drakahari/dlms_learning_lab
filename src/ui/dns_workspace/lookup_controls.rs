//! Opt-in lookup controls; legacy native query fields keep their original path.
use super::*;
use dlms_learning_lab::{dns_query_tools as tools, dns_workspace::UtilityDraft};

pub(super) fn enabled(
    ui: &mut egui::Ui,
    draft: &mut Draft,
    plan: Option<&tools::Plan>,
    metrics: &mut Metrics,
) -> bool {
    if plan.is_none() {
        return false;
    }
    let mut enabled = draft.utility.as_ref().is_some_and(|u| u.enabled);
    let response = ui.checkbox(&mut enabled, "Use lookup commands");
    metrics
        .dns_choices
        .push(("lookup:commands".into(), response.rect));
    if response.changed() {
        draft
            .utility
            .get_or_insert_with(|| UtilityDraft {
                enabled: false,
                input: String::new(),
            })
            .enabled = enabled;
        draft.output.clear();
    }
    enabled
}
pub(super) fn draw(
    ui: &mut egui::Ui,
    server: &Server,
    draft: &mut Draft,
    plan: &tools::Plan,
    metrics: &mut Metrics,
) -> bool {
    ui.horizontal_wrapped(|ui| {
        for endpoint in &plan.endpoints {
            lesson_presentation::monospace(
                ui,
                &format!("{}: {}", endpoint.served.label(), endpoint.address),
                false,
            );
        }
    });
    let Some(command) = draft.utility.as_mut() else {
        return false;
    };
    let mut input = command.input.clone();
    let mut enter = false;
    let mut input_valid = true;
    controls::form_row(ui, "Lookup command", |ui, width| {
        let response = ui.add(
            controls::singleline(ui, &mut input)
                .desired_width((width - 2. * f32::from(controls::PAD_X)).max(1.))
                .hint_text("A supported dig or host command")
                .font(egui::TextStyle::Monospace)
                .char_limit(tools::MAX_COMMAND),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                ui.is_enabled(),
                "DNS lookup command",
            )
        });
        metrics.inputs.push(response.rect);
        metrics
            .dns_fields
            .push(("lookup-command".into(), response.rect, response.id));
        enter = ui.is_enabled()
            && (response.has_focus() || response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
    });
    if input != command.input {
        draft.output.clear();
        if input.len() <= tools::MAX_COMMAND {
            command.input = input;
        } else {
            input_valid = false;
            draft.notice =
                Some("Lookup command limit is512bytes; shorten the text before running it.".into());
        }
    }
    let run = ui.add(button("Look up"));
    metrics.probe = Some(run.rect);
    let queried = run.clicked() || enter;
    if queried && input_valid {
        draft.notice = None;
        if let Err(error) = draft.lookup_utility(server, plan) {
            draft.output = format!("Lookup not evaluated: {error}");
        }
    } else if queried {
        draft.output = "Lookup not evaluated: command exceeds the512-byte limit.".into();
    }
    controls::collapsing_header(ui, "Supported lookup commands")
        .show(ui, |ui| lesson_presentation::prose(ui, tools::HELP));
    queried
}
