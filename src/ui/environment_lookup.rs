//! Opt-in inherited-worker lookup evidence. A selection never runs the artifact.
use super::*;
use dlms_learning_lab::{environment_workspace as env, filesystem::path_lookup as core};
pub(super) fn run_label(practice: bool, lookup: bool) -> &'static str {
    if practice {
        "Run example"
    } else if lookup {
        "Run setup + lookup"
    } else {
        "Run setup + child"
    }
}
pub(super) fn supplied(ui: &mut egui::Ui, task: &env::Task) {
    let Some(plan) = &task.path_lookup else {
        return;
    };
    ui.horizontal_wrapped(|ui| {
        ui.strong("Worker looks up:");
        for name in &plan.profile.names {
            ui.monospace(name);
        }
    });
    controls::collapsing_header(ui, "Supplied lookup inventory · read-only")
        .id_salt(("env-lookup-source", &task.id)).show(ui, |ui| {
            lesson_presentation::prose(ui, "This metadata belongs to the simulated worker. Selected files are not loaded or run.");
            ui.strong("Allowed readonly directories for this caller");
            for dir in &plan.profile.trusted_directories { lesson_presentation::monospace(ui, dir, true); }
            ui.strong("File candidates · supplied mode, owner and group");
            for (path, node) in &plan.profile.tree.nodes {
                if !node.directory {
                    lesson_presentation::monospace(ui, &format!("{path} · {:03o} · {}:{}", node.mode, node.owner, node.group), true);
                }
            }
        });
}
pub(super) fn expected(ui: &mut egui::Ui, task: &env::Task, case: usize) {
    let Some(plan) = &task.path_lookup else {
        return;
    };
    let Some(c) = task.cases.get(case) else {
        return;
    };
    let Some(expected) = plan.cases.iter().find(|e| e.case_name == c.input.name) else {
        return;
    };
    ui.strong("Expected worker lookup · original example");
    for (name, paths) in &expected.selected {
        for path in paths {
            lesson_presentation::monospace(ui, &format!("{name} → {path}"), true);
        }
    }
    lesson_presentation::prose(
        ui,
        "This is a lookup expectation. The token report below is a separate supplied script.",
    );
}
fn state(s: core::CandidateState) -> &'static str {
    match s {
        core::CandidateState::Missing => "No candidate at this path",
        core::CandidateState::NotDirectory => "A parent is not a directory",
        core::CandidateState::NotRegularFile => "Candidate is a directory, not a file",
        core::CandidateState::Denied => "A parent directory lacks search permission",
        core::CandidateState::NotExecutable => "File lacks execute permission",
        core::CandidateState::Selected => "First eligible file selected",
    }
}
fn decision(d: &core::Decision) -> String {
    match d {
        core::Decision::Selected { path } => path.clone(),
        core::Decision::NotFound => "Not found in the inherited PATH".into(),
        core::Decision::Denied => "No executable candidate accessible".into(),
    }
}
pub(super) fn actual(ui: &mut egui::Ui, task: &env::Task, row: &env::CaseRun, case: usize) {
    let Some(plan) = &task.path_lookup else {
        return;
    };
    ui.strong("Worker lookup · actual · no utility run");
    let Some(run) = &row.path_lookup else {
        lesson_presentation::prose(ui, "Lookup did not run. Repair the setup error and retry.");
        return;
    };
    let env::path_lookup::Run::Observed { observation } = run else {
        if let env::path_lookup::Run::Error { category, message } = run {
            lesson_presentation::prose(ui, &format!("{category:?} lookup: {message}"));
        }
        return;
    };
    let expected = task
        .cases
        .get(case)
        .and_then(|c| plan.cases.iter().find(|e| e.case_name == c.input.name));
    let width = (ui.available_width() - ui.spacing().item_spacing.x).max(180.);
    egui::Grid::new(("env-lookup-actual", &task.id)).striped(true).show(ui, |ui| {
        controls::table_text(ui, RichText::new("Name").strong(), width*0.25);
        controls::table_text(ui, RichText::new("Actual selection and requirement").strong(), width*0.75);
        ui.end_row();
        for report in &observation.reports {
            let correct=matches!(&report.decision, core::Decision::Selected {path} if expected.is_some_and(|e|e.selected.get(&report.name).is_some_and(|paths| paths.contains(path))));
            controls::table_text(ui, RichText::new(&report.name).monospace(), width*0.25);
            controls::table_text(ui, format!("{}\n{}", decision(&report.decision), if correct {"Matches provider requirement"} else {"Needs an edit"}), width*0.75);
            ui.end_row();
        }
    });
    match &observation.policy {
        core::Policy::Trusted => lesson_presentation::prose(
            ui,
            "Directory policy: all PATH entries are on the supplied readonly list.",
        ),
        core::Policy::Untrusted { directories } => lesson_presentation::prose(
            ui,
            &format!(
                "Directory policy needs an edit: {} is outside the supplied readonly list. Even unused trailing entries are checked.",
                directories.join(", ")
            ),
        ),
    };
    controls::collapsing_header(ui, "How this lookup searched")
        .id_salt(("env-lookup-trace", &task.id, case))
        .show(ui, |ui| {
            lesson_presentation::monospace(
                ui,
                &format!("Inherited PATH={}", observation.path),
                true,
            );
            for report in &observation.reports {
                ui.strong(&report.name);
                for candidate in &report.candidates {
                    lesson_presentation::monospace(ui, &candidate.path, true);
                    lesson_presentation::prose(ui, state(candidate.state));
                }
            }
        });
}
