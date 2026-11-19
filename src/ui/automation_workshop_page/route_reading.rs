//! Native route controls and a derived table. No device CLI or traffic.
use super::*;
use dlms_learning_lab::automation_workshop::route_reading::table::Entry;

pub(super) fn body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let (Task::RouteReading { supplied }, Scope::RouteReading { draft }) = (task, &work.current)
    else {
        return;
    };
    guide(ui, practice, 0);
    controls::form_row(ui, "Ingress router", |ui, width| {
        controls::label(ui, RichText::new(&supplied.ingress).monospace(), width);
    });
    controls::form_row(ui, "Service prefix", |ui, width| {
        controls::label(
            ui,
            RichText::new(supplied.static_backup.destination.text()).monospace(),
            width,
        );
    });
    let mut edited = draft.clone();
    controls::form_row(ui, "Static backup distance", |ui, _| {
        let input = shared_libraries_page::fitting_input(
            ui,
            &draft.distance,
            id.with("route-distance"),
            8,
            ui.is_enabled(),
            "Whole number",
        );
        m.record(ui, "route-distance", &input.response);
        if let Some(value) = input.value {
            edited.distance = value;
        }
    });
    guide(ui, practice, 1);
    controls::form_row(ui, "Outgoing OSPF cost", |ui, _| {
        let input = shared_libraries_page::fitting_input(
            ui,
            &draft.cost,
            id.with("route-cost"),
            8,
            ui.is_enabled(),
            "Whole number",
        );
        m.record(ui, "route-cost", &input.response);
        if let Some(value) = input.value {
            edited.cost = value;
        }
    });
    lesson_presentation::prose(
        ui,
        &format!(
            "Edited interface: {} toward {}.",
            supplied.primary_interface, supplied.primary_peer,
        ),
    );
    controls::form_row(ui, "Inspect with network mask", |ui, _| {
        let input = shared_libraries_page::fitting_input(
            ui,
            &draft.mask_query,
            id.with("route-mask"),
            15,
            ui.is_enabled(),
            "Dotted-decimal mask",
        );
        m.record(ui, "route-mask", &input.response);
        if let Some(value) = input.value {
            edited.mask_query = value;
        }
    });
    if &edited != draft {
        apply(
            task,
            work,
            Action::ChangeRouteReading { draft: edited },
            notice,
        );
    }
    let Scope::RouteReading { draft } = &work.current else {
        return;
    };
    let observed = match supplied.observe(draft) {
        Ok(value) => value,
        Err(error) => {
            lesson_presentation::prose(ui, &error);
            return;
        }
    };
    if !draft.mask_query.trim().is_empty() {
        match supplied.inspect(&observed.active, &draft.mask_query) {
            Ok(entry) => {
                controls::form_row(ui, "Exact service query", |ui, width| {
                    let result = format!("{} · mask {}", display(entry), entry.dotted_mask());
                    controls::label(ui, RichText::new(result).monospace(), width);
                });
            }
            Err(error) => {
                lesson_presentation::prose(ui, &error);
            }
        }
    }
    ui.separator();
    if let Some(candidate) = &observed.ospf_candidate {
        let options = candidate
            .next_hops
            .iter()
            .map(|hop| format!("{} · {}", hop.interface, hop.via))
            .collect::<Vec<_>>()
            .join(", ");
        controls::form_row(ui, "Current OSPF candidate", |ui, width| {
            controls::label(
                ui,
                RichText::new(format!("Cost {} via {options}", candidate.cost)).monospace(),
                width,
            );
        });
    }
    let comparison = controls::collapsing_header(ui, "Compare OSPF branch costs")
        .id_salt(id.with("compare-ospf-costs"))
        .show(ui, |ui| {
            lesson_presentation::prose(ui, "Each trial leaves one branch available and measures the path to the same service network.");
            match supplied.branch_costs(draft) {
                Ok(costs) => {
                    for path in costs {
                        let total = path.total.map_or_else(|| "No route".into(), |cost| cost.to_string());
                        controls::form_row(ui, path.peer, |ui, width| {
                            controls::label(ui, format!("Local cost {} · Total {total}", path.local), width);
                        });
                    }
                }
                Err(error) => { lesson_presentation::prose(ui, &error); }
            }
        });
    m.record(ui, "route-branch-costs", &comparison.header_response);
    ui.separator();
    guide(ui, practice, 2);
    let key = id.with("observed-destination");
    let mut lookup = ui.data(|data| data.get_temp::<usize>(key)).unwrap_or(0);
    let choices = supplied
        .probes
        .iter()
        .enumerate()
        .map(|(index, address)| (index, address.to_string()))
        .collect::<Vec<_>>();
    select(
        ui,
        id,
        "route-lookup",
        "Destination lookup",
        &mut lookup,
        &choices,
        m,
    );
    ui.data_mut(|data| data.insert_temp(key, lookup));
    let address = supplied.probes[lookup];
    let active_heading = ui.strong("Current installed decision");
    m.record(ui, "route-active-decision", &active_heading);
    render_entry(ui, observed.active.lookup(address));
    let trial = controls::collapsing_header(ui, "Trial: withdraw the service announcement")
        .id_salt(id.with("route-withdrawal"))
        .show(ui, |ui| {
            lesson_presentation::prose(
                ui,
                "This comparison leaves the current table and your settings unchanged.",
            );
            render_entry(ui, observed.withdrawal.lookup(address));
        });
    m.record(ui, "route-withdrawal", &trial.header_response);
    controls::collapsing_header(ui, "Full installed table")
        .id_salt(id.with("route-table"))
        .show(ui, |ui| {
            for entry in observed.active.entries() {
                lesson_presentation::monospace(ui, &display(entry), true);
            }
            let alternatives = observed
                .active
                .uninstalled(supplied.static_backup.destination);
            if !alternatives.is_empty() {
                ui.add_space(6.);
                ui.label("Other eligible candidates for the service prefix");
                for entry in alternatives {
                    lesson_presentation::monospace(ui, &display(entry), true);
                }
            }
            if let Some(status) = observed.active.static_eligibility.first() {
                lesson_presentation::prose(ui, status.explanation());
            }
        });
    if let Some(error) = notice {
        lesson_presentation::prose(ui, error);
    }
}

fn hop_text(entry: &Entry) -> String {
    entry
        .exits
        .iter()
        .map(|hop| {
            hop.next_hop.map_or_else(
                || hop.interface.clone(),
                |address| format!("{address} on {}", hop.interface),
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn display(entry: &Entry) -> String {
    format!(
        "{} {} [{}/{}] {}",
        entry.origin.code(entry.destination),
        entry.destination.text(),
        entry.distance,
        entry.metric,
        hop_text(entry)
    )
}

fn render_entry(ui: &mut egui::Ui, entry: Option<&Entry>) {
    let Some(entry) = entry else {
        ui.label("No installed route matches this destination.");
        return;
    };
    for (label, value) in [
        (
            "Source",
            format!(
                "{} · {}",
                entry.origin.code(entry.destination),
                entry.origin.name()
            ),
        ),
        ("Prefix", entry.destination.text()),
        (
            "Distance / metric",
            format!("{} / {}", entry.distance, entry.metric),
        ),
        ("Next hop / interface", hop_text(entry)),
    ] {
        controls::form_row(ui, label, |ui, width| {
            controls::label(ui, RichText::new(value).monospace(), width);
        });
    }
}
