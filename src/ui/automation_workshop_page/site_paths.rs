//! Native circuit choices and independently traceable outage trials.
use super::*;
use dlms_learning_lab::automation_workshop::site_connectivity::{Location, Outage, Role};

#[derive(Clone, Copy, Default)]
struct Inspection {
    selected: usize,
    observed: Option<(usize, u32)>,
}

fn role_label(role: Role) -> &'static str {
    match role {
        Role::Workstation => "Workstation",
        Role::LocalService => "Local service",
        Role::Gateway => "Gateway",
        Role::ProviderTransit => "Carrier transit",
        Role::HostedService => "Hosted service",
    }
}

fn location_label(location: Location) -> &'static str {
    match location {
        Location::Worksite => "Worksite",
        Location::Branch => "Branch",
        Location::ServiceProvider => "Carrier site",
        Location::HostedSite => "Hosted site",
    }
}

fn fact_row(ui: &mut egui::Ui, label: &str, value: String) {
    controls::form_row(ui, label, |ui, width| controls::label(ui, value, width));
}

pub(super) fn body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::SitePaths { supplied } = task else {
        return;
    };
    let Scope::SitePaths { edit } = &work.current else {
        return;
    };
    let active = edit.active;
    let key = id.with("site-path-inspection");
    let mut look = ui
        .ctx()
        .data(|data| data.get_temp::<Inspection>(key))
        .unwrap_or_default();
    if look.selected >= supplied.trials.len() {
        look = Inspection::default();
    }

    guide(ui, practice, 0);
    let trials = supplied
        .trials
        .iter()
        .enumerate()
        .map(|(i, trial)| (i, trial.label.clone()))
        .collect::<Vec<_>>();
    select(
        ui,
        id,
        "site-trial",
        "Requirement to trace",
        &mut look.selected,
        &trials,
        m,
    );
    let trial = &supplied.trials[look.selected];
    let graph = &supplied.topology;
    fact_row(
        ui,
        "Trial endpoints",
        format!(
            "{} to {}",
            graph.nodes[trial.from].label, graph.nodes[trial.to].label,
        ),
    );
    let outage = match trial.outage {
        Outage::None => "All supplied sites and circuits available".into(),
        Outage::Node { index } => format!("{} is unavailable", graph.nodes[index].label),
        Outage::Link { index } => format!("{} is unavailable", graph.links[index].label),
        Outage::CircuitGroup { id } => format!(
            "Unavailable circuits: {}",
            graph
                .links
                .iter()
                .filter(|link| link.circuit_group == Some(id))
                .map(|link| link.label.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    fact_row(ui, "Condition", outage);
    let r = ui.add(button("Trace path"));
    m.record(ui, "site-trace", &r);
    if r.clicked() {
        look.observed = Some((look.selected, active));
    }
    if look.observed == Some((look.selected, active)) {
        match supplied.observe(edit, look.selected) {
            Ok(Some(path)) => {
                ui.strong("Actual trace · Path available");
                let sites = path
                    .nodes
                    .iter()
                    .map(|i| graph.nodes[*i].label.as_str())
                    .collect::<Vec<_>>();
                lesson_presentation::prose(ui, &sites.join(" to "));
                let wires = path
                    .links
                    .iter()
                    .map(|i| graph.links[*i].label.as_str())
                    .collect::<Vec<_>>();
                lesson_presentation::prose(ui, &format!("Connections used: {}", wires.join(", ")));
            }
            Ok(None) => {
                ui.label("Actual trace · No available path between these endpoints.");
            }
            Err(error) => {
                lesson_presentation::prose(ui, &error);
            }
        }
    } else {
        ui.label("Trace this trial using the current circuit choices.");
    }
    ui.separator();
    guide(ui, practice, 1);
    let used = (active & supplied.choices).count_ones();
    ui.strong(format!(
        "Optional connections · {used} selected of {} allowed",
        supplied.max_choices
    ));
    for (i, link) in graph.links.iter().enumerate() {
        if supplied.choices & (1 << i) == 0 {
            continue;
        }
        let mut enabled = active & (1 << i) != 0;
        let [a, b] = link.ends;
        let label = format!("{} to {}", graph.nodes[a].label, graph.nodes[b].label);
        let control = controls::form_row(ui, &link.label, |ui, width| {
            controls::cell(ui, width, |ui| ui.checkbox(&mut enabled, label)).inner
        });
        m.record(ui, &format!("site-link-{i}"), &control.inner);
        if control.inner.changed() {
            apply(
                task,
                work,
                Action::SiteLink {
                    index: i,
                    connected: enabled,
                },
                notice,
            );
            look.observed = None;
        }
    }
    ui.separator();
    guide(ui, practice, 2);
    controls::collapsing_header(ui, "Supplied topology")
        .id_salt(id.with("supplied-connectivity"))
        .show(ui, |ui| {
            for node in &graph.nodes {
                fact_row(
                    ui,
                    &node.label,
                    format!(
                        "{} · {}",
                        location_label(node.location),
                        role_label(node.role)
                    ),
                );
            }
            ui.strong("Fixed connections");
            for (i, link) in graph.links.iter().enumerate() {
                if supplied.fixed & (1 << i) == 0 {
                    continue;
                }
                let [a, b] = link.ends;
                fact_row(
                    ui,
                    &link.label,
                    format!("{} to {}", graph.nodes[a].label, graph.nodes[b].label),
                );
            }
        });
    ui.ctx().data_mut(|data| data.insert_temp(key, look));
}
