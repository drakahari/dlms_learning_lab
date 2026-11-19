//! One broadcast segment, editable interface priorities and three observations.
use super::*;
use dlms_learning_lab::automation_workshop::ospf_broadcast::{
    election::{Relationship, Role},
    workspace::Stage,
};

pub(super) fn body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let (Task::OspfBroadcast { supplied }, Scope::OspfBroadcast { draft }) = (task, &work.current)
    else {
        return;
    };
    guide(ui, practice, 0);
    controls::form_row(ui, "Broadcast segment", |ui, width| {
        controls::label(ui, &supplied.segment, width);
    });
    ui.strong("Interface priorities");
    let mut edited = draft.clone();
    for (i, member) in supplied.members[..3].iter().enumerate() {
        let input_id = format!("broadcast-priority-{i}");
        controls::form_row(ui, &member.label, |ui, _| {
            let input = shared_libraries_page::fitting_input(
                ui,
                &draft.priorities[i],
                id.with(&input_id),
                5,
                ui.is_enabled(),
                "0–255",
            );
            m.record(ui, &input_id, &input.response);
            if let Some(value) = input.value {
                edited.priorities[i] = value;
            }
        });
    }
    let late = &supplied.members[3];
    controls::form_row(ui, &late.label, |ui, width| {
        controls::label(
            ui,
            format!("Scheduled arrival · priority {}", late.priority),
            width,
        );
    });
    if &edited != draft {
        apply(
            task,
            work,
            Action::ChangeOspfBroadcast { draft: edited },
            notice,
        );
    }
    let Scope::OspfBroadcast { draft } = &work.current else {
        return;
    };
    let results = match supplied.observe(draft) {
        Ok(results) => results,
        Err(error) => {
            lesson_presentation::prose(ui, &error);
            return;
        }
    };
    ui.separator();
    guide(ui, practice, 1);
    let view_id = id.with("broadcast-view");
    let mut selected = ui.data(|data| data.get_temp::<usize>(view_id)).unwrap_or(0);
    let outage = format!("{} outage", supplied.members[0].label);
    let stages = [
        ("Fresh election", &results.fresh),
        ("New router joins", &results.arrival),
        (outage.as_str(), &results.outage),
    ];
    ui.horizontal_wrapped(|ui| {
        for (i, (label, _)) in stages.iter().enumerate() {
            let button = ui.selectable_value(&mut selected, i, *label);
            m.record(ui, &format!("broadcast-view-{i}"), &button);
        }
    });
    ui.data_mut(|data| data.insert_temp(view_id, selected));
    let (heading, stage) = stages[selected];
    let heading = ui.strong(heading);
    m.record(ui, "broadcast-roles", &heading);
    render_roles(ui, stage);
    if selected > 0 {
        guide(ui, practice, 2);
        for (left, right) in [(2, 3), (2, 1)] {
            let a = &stage.members[left].label;
            let b = &stage.members[right].label;
            let relation = match stage.roles.relationship(&stage.members, left, right) {
                Relationship::WouldAttemptAdjacency => "Would attempt adjacency",
                Relationship::RemainTwoWay => "Remain two-way neighbors",
                Relationship::NoCurrentNeighbor => "No current neighbor",
            };
            controls::form_row(ui, format!("{a} and {b}"), |ui, width| {
                controls::label(ui, relation, width);
            });
        }
    }
    if let Some(error) = notice {
        lesson_presentation::prose(ui, error);
    }
}

fn render_roles(ui: &mut egui::Ui, stage: &Stage) {
    for (index, member) in stage.members.iter().enumerate() {
        let role = match stage.roles.member_role(&stage.members, index) {
            Role::Designated => "Designated router (DR)",
            Role::Backup => "Backup designated router (BDR)",
            Role::Other => "Other router (DROther)",
            Role::Absent => "Off this segment",
        };
        controls::form_row(ui, &member.label, |ui, width| {
            controls::label(
                ui,
                format!(
                    "{} · Priority {} · {role}",
                    member.router_id, member.priority
                ),
                width,
            );
        });
    }
}
