//! Learner controls for default access membership and switched IPv4 gateways.
use super::*;
use dlms_learning_lab::{
    automation_workshop::{
        access_defaults::{Choice, access_jack},
        svi_draft::{AddressText, Draft},
    },
    switching::{Fabric, Mode, Visibility},
};

#[derive(Clone, Default)]
struct Trial {
    pair: usize,
    context: Option<TrialContext>,
    observed: Option<(&'static str, String)>,
}

#[derive(Clone, PartialEq, Eq)]
enum TrialContext {
    Access(Choice),
    Routing(Box<Draft>),
}

fn fabric_overview(ui: &mut egui::Ui, fabric: &Fabric, gateway_switch: Option<&str>) {
    let mut switches = fabric.switches.iter();
    let (left_id, left) = switches.next().expect("Validated first switch");
    let (right_id, right) = switches.next().expect("Validated second switch");
    let role = |id: &str| match gateway_switch {
        Some(switch) if switch == id => "Hosts both IPv4 gateways",
        Some(_) => "Carries station traffic",
        None => "Access and trunk ports",
    };
    let native = |switch: &dlms_learning_lab::switching::Switch| {
        if let Some(Mode::Trunk { native, .. }) = switch
            .ports
            .values()
            .find_map(|p| matches!(p.mode, Mode::Trunk { .. }).then_some(&p.mode))
        {
            format!("Native VLAN {native}")
        } else {
            "No active trunk configuration".into()
        }
    };
    controls::topology_pair(
        ui,
        [&left.label, role(left_id), &native(left)],
        [&right.label, role(right_id), &native(right)],
        palette(ui).accent,
        "Supplied inter-switch trunk",
    );
}

fn station(ui: &mut egui::Ui, fabric: &Fabric, id: &str, extra: &str) {
    let (switch, port, vlan) = access_jack(fabric, id).expect("Known supplied attachment");
    let descriptor = format!("{} · {port} · VLAN {vlan}", fabric.switches[switch].label);
    controls::form_row(ui, &fabric.hosts[id].label, |ui, width| {
        controls::label(
            ui,
            if extra.is_empty() {
                descriptor
            } else {
                format!("{descriptor} · {extra}")
            },
            width,
        )
    });
}

fn view_trial(ui: &mut egui::Ui, key: egui::Id, context: TrialContext) -> Trial {
    let mut trial = ui
        .ctx()
        .data(|data| data.get_temp::<Trial>(key))
        .unwrap_or_default();
    if trial.context.as_ref() != Some(&context) {
        trial.context = Some(context);
        trial.observed = None;
    }
    trial
}

fn save_trial(ui: &mut egui::Ui, trial: Trial, key: egui::Id) {
    ui.ctx().data_mut(|data| data.insert_temp(key, trial));
}

fn show_trial(ui: &mut egui::Ui, trial: &Trial) {
    if let Some((title, details)) = &trial.observed {
        ui.strong(*title);
        lesson_presentation::prose(ui, details);
    }
}

fn describe(v: Visibility, a: &str, b: &str) -> String {
    match (v.forward, v.reverse) {
        (true, true) => format!("{a} and {b} each received the other's Ethernet broadcast."),
        (false, false) => "Neither Ethernet broadcast reached the other station.".into(),
        (true, false) => format!(
            "{b} received the broadcast from {a}. Traffic in the opposite direction did not arrive."
        ),
        (false, true) => format!(
            "{a} received the broadcast from {b}. Traffic in the opposite direction did not arrive."
        ),
    }
}

pub(super) fn defaults_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let (Task::AccessDefault { supplied }, Scope::AccessDefault { choice }) = (task, &work.current)
    else {
        return;
    };
    let key = id.with("access-default-observation");
    let mut trial = view_trial(ui, key, TrialContext::Access(*choice));
    let before = *choice;
    let mut candidate = before;
    guide(ui, practice, 0);
    fabric_overview(ui, &supplied.fabric, None);
    let choices = std::iter::once((None, "Use switch default".into()))
        .chain(
            supplied
                .available()
                .unwrap()
                .iter()
                .map(|n| (Some(*n), format!("Explicit VLAN {n}"))),
        )
        .collect::<Vec<_>>();
    select(
        ui,
        id,
        "default-access",
        "New desk access VLAN",
        &mut candidate.access,
        &choices,
        m,
    );
    if candidate != before {
        apply(
            task,
            work,
            Action::SetAccessDefault { choice: candidate },
            notice,
        );
        trial.context = Some(TrialContext::Access(candidate));
        trial.observed = None;
    }
    let Scope::AccessDefault { choice } = &work.current else {
        return;
    };
    controls::form_row(ui, "Effective access VLAN", |ui, width| {
        controls::label(ui, format!("VLAN {}", choice.effective()), width)
    });
    let f = supplied.configured(*choice).unwrap();
    for endpoint in [
        &supplied.desk,
        &supplied.desk_peer,
        &supplied.legacy[0],
        &supplied.legacy[1],
    ] {
        station(ui, &f, endpoint, "");
    }
    ui.separator();
    guide(ui, practice, 1);
    let pairs = [
        (&supplied.desk, &supplied.desk_peer),
        (&supplied.desk, &supplied.legacy[0]),
        (&supplied.desk, &supplied.legacy[1]),
        (&supplied.legacy[0], &supplied.legacy[1]),
    ];
    let choices = pairs
        .iter()
        .enumerate()
        .map(|(i, (a, b))| {
            (
                i,
                format!("{} and {}", f.hosts[*a].label, f.hosts[*b].label),
            )
        })
        .collect::<Vec<_>>();
    let previous = trial.pair;
    select(
        ui,
        id,
        "default-pair",
        "Ethernet endpoints",
        &mut trial.pair,
        &choices,
        m,
    );
    if trial.pair != previous {
        trial.observed = None;
    }
    let r = ui.add(button("Test Ethernet"));
    m.record(ui, "default-ethernet", &r);
    if r.clicked() {
        let (a, b) = pairs[trial.pair];
        trial.observed = Some((
            "Actual Ethernet test",
            f.visibility(a, b)
                .map(|v| describe(v, &f.hosts[a].label, &f.hosts[b].label))
                .unwrap_or_else(|e| e),
        ));
    }
    show_trial(ui, &trial);
    guide(ui, practice, 2);
    save_trial(ui, trial, key);
}

fn address_field(
    ui: &mut egui::Ui,
    id: egui::Id,
    key: &str,
    label: &str,
    field: &mut AddressText,
    prefix: bool,
    m: &mut Metrics,
) {
    controls::form_row(ui, label, |ui, _| {
        let response = shared_libraries_page::fitting_input(
            ui,
            &field.0,
            id.with(key),
            32,
            ui.is_enabled(),
            if prefix {
                "IPv4 address/prefix"
            } else {
                "Dotted-decimal IPv4 address"
            },
        );
        m.record(ui, key, &response.response);
        if let Some(text) = response.value {
            field.0 = text;
        }
    });
    let invalid = if prefix {
        field.cidr().err()
    } else {
        field.gateway().err()
    };
    if let Some(message) = invalid {
        lesson_presentation::prose(ui, message.as_str());
    }
}

fn toggle(ui: &mut egui::Ui, key: &str, label: &str, value: &mut bool, m: &mut Metrics) {
    controls::form_row(ui, label, |ui, width| {
        let row = controls::cell(ui, width, |ui| ui.checkbox(value, "Enabled"));
        m.record(ui, key, &row.inner);
    });
}

pub(super) fn routed_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let (Task::RoutedVlans { supplied }, Scope::RoutedVlans { draft }) = (task, &work.current)
    else {
        return;
    };
    let key = id.with("routed-vlan-observation");
    let mut trial = view_trial(ui, key, TrialContext::Routing(draft.clone()));
    let mut candidate: Draft = (**draft).clone();
    guide(ui, practice, 0);
    fabric_overview(ui, &supplied.fabric, Some(&supplied.router_switch));
    ui.strong(format!(
        "VLAN gateway interfaces · {}",
        supplied.fabric.switches[&supplied.router_switch].label
    ));
    for (i, svi) in supplied.interfaces.iter().enumerate() {
        lesson_presentation::prose(ui, &format!("VLAN {} · {}", svi.vlan, svi.subnet.text()));
        address_field(
            ui,
            id,
            &format!("svi-address-{i}"),
            "Gateway IPv4/prefix",
            &mut candidate.interfaces[i],
            true,
            m,
        );
        toggle(
            ui,
            &format!("svi-enable-{i}"),
            "Interface",
            &mut candidate.enable[i],
            m,
        );
    }
    toggle(ui, "svi-routing", "IPv4 routing", &mut candidate.routing, m);
    ui.separator();
    guide(ui, practice, 1);
    ui.strong("Far uplink and station gateways");
    for (i, svi) in supplied.interfaces.iter().enumerate() {
        toggle(
            ui,
            &format!("svi-allowed-{i}"),
            &format!("Allow VLAN {}", svi.vlan),
            &mut candidate.allow_remote[i],
            m,
        );
    }
    for (i, endpoint) in supplied.endpoints.iter().enumerate() {
        station(
            ui,
            &supplied.fabric,
            &endpoint.id,
            &format!("{}/{}", endpoint.address.address, endpoint.address.prefix),
        );
        address_field(
            ui,
            id,
            &format!("station-gateway-{i}"),
            "Default gateway",
            &mut candidate.gateways[i],
            false,
            m,
        );
    }
    if candidate != **draft {
        apply(
            task,
            work,
            Action::ChangeSviDraft {
                draft: Box::new(candidate),
            },
            notice,
        );
        trial.observed = None;
    }
    let Scope::RoutedVlans { draft } = &work.current else {
        return;
    };
    let edit = draft.compile();
    trial.context = Some(TrialContext::Routing(draft.clone()));
    ui.separator();
    guide(ui, practice, 2);
    if let Ok(states) = supplied.availability(&edit) {
        for (svi, state) in supplied.interfaces.iter().zip(states) {
            controls::form_row(ui, format!("VLAN {} gateway", svi.vlan), |ui, width| {
                controls::label(ui, state.text(), width)
            });
        }
    }
    let pairs = [(0, 1), (0, 2), (1, 2)];
    let label = |index: usize| &supplied.fabric.hosts[&supplied.endpoints[index].id].label;
    let choices = pairs
        .iter()
        .enumerate()
        .map(|(i, (a, b))| (i, format!("{} and {}", label(*a), label(*b))))
        .collect::<Vec<_>>();
    let previous = trial.pair;
    select(
        ui,
        id,
        "svi-trial",
        "Path endpoints",
        &mut trial.pair,
        &choices,
        m,
    );
    if previous != trial.pair {
        trial.observed = None;
    }
    let (a, b) = pairs[trial.pair];
    ui.horizontal_wrapped(|ui| {
        let ethernet = ui.add(button("Test Ethernet"));
        m.record(ui, "svi-ethernet", &ethernet);
        if ethernet.clicked() {
            trial.observed = Some((
                "Actual Ethernet test",
                supplied
                    .ethernet(&edit, a, b)
                    .map(|v| describe(v, label(a), label(b)))
                    .unwrap_or_else(|e| e),
            ));
        }
        let ip = ui.add(button("Test IPv4"));
        m.record(ui, "svi-ipv4", &ip);
        if ip.clicked() {
            let value = supplied
                .probe(&edit, a, b)
                .map(|p| {
                    let route = |trace: dlms_learning_lab::ipv4_routing::Trace| {
                        trace
                            .hops
                            .into_iter()
                            .map(|hop| format!("{} sends via {}", hop.device, hop.next_hop))
                            .collect::<Vec<_>>()
                            .join(" → ")
                    };
                    format!(
                        "IPv4 delivery and reply worked. Outbound: {}. Return: {}.",
                        route(p.forward),
                        route(p.reply)
                    )
                })
                .unwrap_or_else(|e| e);
            trial.observed = Some(("Actual IPv4 test", value));
        }
    });
    show_trial(ui, &trial);
    save_trial(ui, trial, key);
}

pub(in crate::ui) fn last_vlan_observation(
    ctx: &egui::Context,
    lesson: &str,
    practice: bool,
    routed: bool,
) -> Option<String> {
    let id = egui::Id::new(("automation-workshop", lesson, practice));
    let key = id.with(if routed {
        "routed-vlan-observation"
    } else {
        "access-default-observation"
    });
    ctx.data(|data| data.get_temp::<Trial>(key))
        .and_then(|trial| trial.observed.map(|(_, details)| details))
}
