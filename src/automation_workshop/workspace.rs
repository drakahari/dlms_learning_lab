//! Typed learner drafts and current-state checks for two automation workspaces.
use super::nfs_tools::workspace as nfs_operator;
use super::ospf_broadcast::workspace as ospf_broadcast;
use super::route_reading::workspace as route_reading;
use super::{access_defaults, svi_draft, svi_projection};
use super::{
    authenticated_transfer as transfer, retirement_console, socket_retirement, transfer_editor,
};
use super::{
    change_rollout as rollout, claim_review, desired_config, intent_path as intent, rollout_task,
    site_connectivity_task,
};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentTask {
    pub spec: intent::Spec,
    pub starter: intent::Current,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,IntentTrial,6>")]
    pub trials: Vec<IntentTrial>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentTrial {
    pub from: usize,
    pub to: usize,
    pub wanted: bool,
}
impl IntentTrial {
    pub const fn exchange(self) -> intent::Exchange {
        intent::Exchange {
            from: self.from,
            to: self.to,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "activity", rename_all = "snake_case", deny_unknown_fields)]
pub enum Task {
    Rollout {
        supplied: Box<rollout_task::Task>,
    },
    Intent {
        supplied: Box<IntentTask>,
    },
    ClaimReview {
        supplied: Box<claim_review::Task>,
    },
    DesiredConfig {
        supplied: Box<desired_config::Task>,
    },
    SitePaths {
        supplied: Box<site_connectivity_task::Task>,
    },
    SocketRetirement {
        supplied: Box<socket_retirement::Task>,
    },
    KeyedTransfer {
        supplied: Box<transfer::Task>,
    },
    AccessDefault {
        supplied: Box<access_defaults::Task>,
    },
    RoutedVlans {
        supplied: Box<svi_projection::Task>,
    },
    NfsExportRetirement {
        supplied: Box<nfs_operator::Task>,
    },
    RouteReading {
        supplied: Box<route_reading::Task>,
    },
    OspfBroadcast {
        supplied: Box<ospf_broadcast::Task>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "activity", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    Rollout {
        draft: rollout::Change,
        #[serde(
            deserialize_with = "crate::cron_workspace::sequence::<_,rollout_task::Recorded,24>"
        )]
        records: Vec<rollout_task::Recorded>,
        preview: bool,
    },
    Intent {
        current: intent::Current,
        request: intent::Exchange,
        allow: bool,
    },
    ClaimReview {
        review: claim_review::Review,
        response: claim_review::Response,
    },
    DesiredConfig {
        edit: Box<desired_config::Edit>,
    },
    SitePaths {
        edit: site_connectivity_task::Edit,
    },
    SocketRetirement {
        console: Box<retirement_console::Console>,
    },
    KeyedTransfer {
        edit: Box<transfer_editor::Edit>,
    },
    AccessDefault {
        choice: access_defaults::Choice,
    },
    RoutedVlans {
        draft: Box<svi_draft::Draft>,
    },
    NfsExportRetirement {
        terminal: Box<nfs_operator::Work>,
    },
    RouteReading {
        draft: route_reading::Draft,
    },
    OspfBroadcast {
        draft: ospf_broadcast::Draft,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub current: Scope,
    pub checked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    RolloutDraft {
        draft: rollout::Change,
    },
    Preview,
    Apply,
    VerifyPilot {
        index: usize,
    },
    IntentDraft {
        request: intent::Exchange,
        allow: bool,
    },
    Submit,
    Publish,
    Uplink {
        up: bool,
    },
    InspectClaim,
    ReadForecastSample,
    AssessClaim {
        assessment: Option<claim_review::Assessment>,
    },
    AssessForecast {
        assessment: Option<claim_review::Assessment>,
    },
    ClaimResponse {
        response: claim_review::Response,
    },
    ApplyClaimResponse,
    ChangePlanDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        text: String,
    },
    PreviewConfig,
    ApplyConfig,
    SiteLink {
        index: usize,
        connected: bool,
    },
    RetirementDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        text: String,
    },
    RetirementCommand {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        text: String,
    },
    RetirementProbe {
        required: bool,
        next_start: bool,
    },
    ChangeTransferRule {
        protected: bool,
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    SelectTransferKey {
        reference: Option<transfer::KeyReference>,
    },
    ApplyTransferPolicy,
    TransferLookup {
        secondary: bool,
    },
    RunTransfer {
        client: transfer::Client,
        protected: bool,
    },
    SetAccessDefault {
        choice: access_defaults::Choice,
    },
    ChangeSviDraft {
        draft: Box<svi_draft::Draft>,
    },
    NfsExportDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        text: String,
    },
    NfsSaveExports,
    NfsTerminalDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,160>")]
        text: String,
    },
    NfsTerminalRun {
        #[serde(deserialize_with = "crate::cron::text::<_,160>")]
        text: String,
    },
    NfsUnmountRetired,
    NfsMount {
        index: usize,
    },
    NfsRead {
        index: usize,
    },
    ChangeRouteReading {
        draft: route_reading::Draft,
    },
    ChangeOspfBroadcast {
        draft: ospf_broadcast::Draft,
    },
    Check,
    Restore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}
impl Outcome {
    fn new(name: impl Into<String>, passed: bool, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed,
            detail: detail.into(),
        }
    }
}

impl IntentTask {
    pub fn outcomes(&self, state: intent::Current) -> Result<Vec<Outcome>> {
        let mut checks = vec![Outcome::new(
            "Transport uplink",
            state.uplink_up,
            if state.uplink_up {
                "The supplied uplink is up."
            } else {
                "Restore the uplink that carries service traffic before verifying its path."
            },
        )];
        for trial in &self.trials {
            let trace = self.spec.trace(state, trial.exchange())?;
            let name = format!(
                "{} to {}",
                self.spec.edges[trial.from].name, self.spec.edges[trial.to].name
            );
            let pass = trace.desired == trial.wanted
                && trace.installed == trial.wanted
                && trace.forwarded() == trial.wanted;
            let detail = if trace.desired != trial.wanted {
                "The requested controller policy differs from the work order.".into()
            } else if trace.installed != trial.wanted {
                "The desired policy is correct. Publish it to update the installed rule.".into()
            } else if trial.wanted
                && let Err(error) = &trace.carrier
            {
                format!("The installed rule allows traffic. Transport failed: {error}")
            } else if trial.wanted {
                "The installed rule allows the exchange and the transport trial returns.".into()
            } else {
                "The installed rule excludes this exchange.".into()
            };
            checks.push(Outcome::new(name, pass, detail));
        }
        Ok(checks)
    }

    pub fn validate(&self) -> Result<()> {
        self.spec.validate()?;
        if !(3..=6).contains(&self.trials.len()) {
            return Err("Provide three to six supplied transport trials.".into());
        }
        let mut tested = std::collections::BTreeSet::new();
        let mut repaired = self.starter;
        for trial in &self.trials {
            if !tested.insert((trial.from, trial.to)) {
                return Err("Use distinct directions for the supplied trial pairs.".into());
            }
            repaired.submit(trial.exchange(), trial.wanted)?;
        }
        repaired.publish();
        repaired.uplink_up = true;
        if !self.trials.iter().any(|case| case.wanted)
            || !self.trials.iter().any(|case| !case.wanted)
            || self.outcomes(self.starter)?.iter().all(|r| r.passed)
            || self.outcomes(repaired)?.iter().any(|r| !r.passed)
        {
            return Err(
                "Provide a repairable policy with both functioning and excluded exchanges.".into(),
            );
        }
        Ok(())
    }
}

impl Task {
    pub(crate) fn track(&self) -> &'static str {
        match self {
            Self::SocketRetirement { .. } => "LPIC-1",
            Self::KeyedTransfer { .. } | Self::NfsExportRetirement { .. } => "LPIC-2",
            _ => "CCNA",
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Rollout { supplied } => supplied.validate(),
            Self::Intent { supplied } => supplied.validate(),
            Self::ClaimReview { supplied } => supplied.validate(),
            Self::DesiredConfig { supplied } => supplied.validate(),
            Self::SitePaths { supplied } => supplied.validate(),
            Self::SocketRetirement { supplied } => supplied.validate(),
            Self::KeyedTransfer { supplied } => supplied.validate(),
            Self::AccessDefault { supplied } => supplied.validate(),
            Self::RoutedVlans { supplied } => supplied.validate(),
            Self::NfsExportRetirement { supplied } => supplied.validate(),
            Self::RouteReading { supplied } => supplied.validate(),
            Self::OspfBroadcast { supplied } => supplied.validate(),
        }
    }
}

impl Work {
    pub fn new(task: &Task) -> Self {
        let current = match task {
            Task::Rollout { .. } => Scope::Rollout {
                draft: rollout::Change::Reconcile,
                records: vec![],
                preview: false,
            },
            Task::Intent { supplied } => Scope::Intent {
                current: supplied.starter,
                request: intent::Exchange { from: 0, to: 1 },
                allow: true,
            },
            Task::ClaimReview { supplied } => Scope::ClaimReview {
                review: supplied.new_review(),
                response: claim_review::Response::NoChange,
            },
            Task::DesiredConfig { supplied } => Scope::DesiredConfig {
                edit: Box::new(supplied.new_edit()),
            },
            Task::SitePaths { supplied } => Scope::SitePaths {
                edit: supplied.initial_edit(),
            },
            Task::SocketRetirement { supplied } => Scope::SocketRetirement {
                console: Box::new(retirement_console::Console::new(supplied)),
            },
            Task::KeyedTransfer { supplied } => Scope::KeyedTransfer {
                edit: Box::new(transfer_editor::Edit::new(supplied)),
            },
            Task::AccessDefault { supplied } => Scope::AccessDefault {
                choice: supplied.initial,
            },
            Task::RoutedVlans { supplied } => Scope::RoutedVlans {
                draft: Box::new(svi_draft::Draft::from(&supplied.initial)),
            },
            Task::NfsExportRetirement { supplied } => Scope::NfsExportRetirement {
                terminal: Box::new(supplied.start()),
            },
            Task::RouteReading { supplied } => Scope::RouteReading {
                draft: supplied.start(),
            },
            Task::OspfBroadcast { supplied } => Scope::OspfBroadcast {
                draft: supplied.start(),
            },
        };
        Self {
            current,
            checked: false,
        }
    }

    pub fn validate(&self, task: &Task) -> Result<()> {
        match (&self.current, task) {
            (
                Scope::Rollout {
                    draft,
                    records,
                    preview,
                },
                Task::Rollout { supplied },
            ) => {
                let current = supplied.replay(records)?;
                if *preview {
                    current.inventory.preview(&supplied.spec, draft)?;
                }
                Ok(())
            }
            (Scope::Intent { request, .. }, Task::Intent { .. }) => {
                if request.from >= 3 || request.to >= 3 {
                    return Err("Select supplied fabric edges.".into());
                }
                Ok(())
            }
            (Scope::ClaimReview { review, .. }, Task::ClaimReview { supplied }) => {
                review.validate(supplied)
            }
            (Scope::DesiredConfig { edit }, Task::DesiredConfig { supplied }) => {
                edit.validate(supplied)
            }
            (Scope::SitePaths { edit }, Task::SitePaths { supplied }) => edit.validate(supplied),
            (Scope::SocketRetirement { console }, Task::SocketRetirement { supplied }) => {
                console.validate(supplied)
            }
            (Scope::KeyedTransfer { edit }, Task::KeyedTransfer { supplied }) => {
                edit.validate(supplied)
            }
            (Scope::AccessDefault { choice }, Task::AccessDefault { supplied }) => {
                supplied.configured(*choice).map(|_| ())
            }
            (Scope::RoutedVlans { draft }, Task::RoutedVlans { .. }) => {
                if !draft.valid() {
                    return Err("Keep the supplied IPv4 fields to one short line each.".into());
                }
                Ok(())
            }
            (Scope::NfsExportRetirement { terminal }, Task::NfsExportRetirement { supplied }) => {
                terminal.validate(supplied)
            }
            (Scope::RouteReading { draft }, Task::RouteReading { .. }) => draft.validate(),
            (Scope::OspfBroadcast { draft }, Task::OspfBroadcast { .. }) => draft.validate(),
            _ => Err("This work belongs to a different activity.".into()),
        }
    }

    pub fn outcomes(&self, task: &Task) -> Vec<Outcome> {
        let checked = match (&self.current, task) {
            (Scope::Rollout { draft, records, .. }, Task::Rollout { supplied }) => {
                let pending = records.iter().rev().find_map(|record| match record {
                    rollout_task::Recorded::Apply { change } => Some(change),
                    _ => None,
                }) != Some(draft);
                supplied
                    .replay(records)
                    .and_then(|work| supplied.checks(&work))
                    .map(|checks| {
                        let mut results = vec![Outcome::new(
                            "Current rollout draft",
                            !pending,
                            if pending {
                                "Preview and apply the final draft before checking."
                            } else {
                                "The current draft is applied."
                            },
                        )];
                        results.extend(checks.into_iter().map(|(passed, text)| {
                            Outcome::new(
                                text,
                                passed,
                                if passed {
                                    "The actual virtual result meets the work order."
                                } else {
                                    "Review the trial and the recorded rollout sequence."
                                },
                            )
                        }));
                        results
                    })
            }
            (
                Scope::Intent {
                    current,
                    request,
                    allow,
                },
                Task::Intent { supplied },
            ) => supplied.outcomes(*current).map(|mut checks| {
                let accepted = request.from != request.to
                    && current
                        .controller
                        .get(request.from)
                        .and_then(|row| row.get(request.to))
                        == Some(allow);
                checks.insert(
                    0,
                    Outcome::new(
                        "Current intent request",
                        accepted,
                        if accepted {
                            "The selected request is in the controller's desired policy."
                        } else {
                            "Submit the selected request before checking the network."
                        },
                    ),
                );
                checks
            }),
            (Scope::ClaimReview { review, .. }, Task::ClaimReview { supplied }) => {
                supplied.findings(*review).map(|checks| {
                    checks
                        .into_iter()
                        .map(|o| Outcome::new(o.title, o.passed, o.detail))
                        .collect()
                })
            }
            (Scope::DesiredConfig { edit }, Task::DesiredConfig { supplied }) => {
                supplied.outcomes(edit).map(|checks| {
                    checks
                        .into_iter()
                        .map(|o| Outcome::new(o.name, o.passed, o.detail))
                        .collect()
                })
            }
            (Scope::SitePaths { edit }, Task::SitePaths { supplied }) => {
                supplied.outcomes(edit).map(|checks| {
                    checks
                        .into_iter()
                        .map(|result| Outcome::new(result.name, result.passed, result.detail))
                        .collect()
                })
            }
            (Scope::SocketRetirement { console }, Task::SocketRetirement { supplied }) => supplied.checks(&console.state).map(|checks| {
                let descriptions = [
                    ("Keep the socket disabled", "A listening socket can return after startup if it remains enabled."),
                    ("Stop the current socket", "Disable changes startup. Stop closes the socket in this session."),
                    ("Disable the obsolete service", "Remove its start instruction while preserving the required status service."),
                    ("Close the obsolete endpoint now", "A running service or listening socket can still handle the supplied request."),
                    ("Keep the obsolete endpoint closed after startup", "Verify the service and socket both remain unavailable in a new session."),
                    ("Preserve the required service", "The status service should answer now and after startup."),
                ];
                checks.into_iter().zip(descriptions).map(|(passed, (name,retry))|
                    Outcome::new(name,passed,if passed {"The current simulated results meet this requirement."} else {retry})
                ).collect()
            }),
            (Scope::KeyedTransfer { edit }, Task::KeyedTransfer { supplied }) => edit.outcomes(supplied).map(|checks| {
                let names = [
                    "Apply the complete configuration",
                    "Require a verified key for the served zone",
                    "Refresh the secondary record and serial",
                    "Exclude unsigned and outdated requests",
                    "Protect the unrelated zone",
                    "Preserve ordinary lookups",
                ];
                checks.into_iter().zip(names).map(|((passed, detail),name)| Outcome::new(name,passed,detail)).collect()
            }),
            (Scope::AccessDefault { choice }, Task::AccessDefault { supplied }) =>
                supplied.outcomes(*choice),
            (Scope::RoutedVlans { draft }, Task::RoutedVlans { supplied }) =>
                supplied.outcomes(&draft.compile()),
            (Scope::NfsExportRetirement { terminal }, Task::NfsExportRetirement { supplied }) => terminal.outcomes(supplied),
            (Scope::RouteReading { draft }, Task::RouteReading { supplied }) => supplied.outcomes(draft),
            (Scope::OspfBroadcast { draft }, Task::OspfBroadcast { supplied }) => supplied.outcomes(draft),
            _ => Err("This work belongs to a different activity.".into()),
        };
        checked.unwrap_or_else(|error| vec![Outcome::new("Virtual workspace", false, error)])
    }

    pub fn apply(&mut self, task: &Task, action: Action) -> Result<()> {
        let mut next = self.clone();
        match (&mut next.current, task, action) {
            (
                Scope::Rollout { draft, preview, .. },
                Task::Rollout { .. },
                Action::RolloutDraft { draft: proposed },
            ) => {
                if *draft != proposed {
                    *draft = proposed;
                    *preview = false;
                }
            }
            (
                Scope::Rollout {
                    draft,
                    records,
                    preview,
                },
                Task::Rollout { supplied },
                Action::Preview,
            ) => {
                supplied
                    .replay(records)?
                    .inventory
                    .preview(&supplied.spec, draft)?;
                *preview = true;
            }
            (
                Scope::Rollout {
                    draft,
                    records,
                    preview,
                },
                Task::Rollout { .. },
                Action::Apply,
            ) => {
                if !*preview {
                    return Err("Preview the exact edit before applying it.".into());
                }
                records.push(rollout_task::Recorded::Apply {
                    change: draft.clone(),
                });
                *preview = false;
            }
            (
                Scope::Rollout { records, .. },
                Task::Rollout { .. },
                Action::VerifyPilot { index },
            ) => {
                records.push(rollout_task::Recorded::VerifyPilot { index });
            }
            (
                Scope::Intent { request, allow, .. },
                Task::Intent { .. },
                Action::IntentDraft {
                    request: proposed,
                    allow: value,
                },
            ) => {
                *request = proposed;
                *allow = value;
            }
            (
                Scope::Intent {
                    current,
                    request,
                    allow,
                },
                Task::Intent { .. },
                Action::Submit,
            ) => current.submit(*request, *allow)?,
            (Scope::Intent { current, .. }, Task::Intent { .. }, Action::Publish) => {
                current.publish()
            }
            (Scope::Intent { current, .. }, Task::Intent { .. }, Action::Uplink { up }) => {
                current.uplink_up = up
            }
            (
                Scope::ClaimReview { review, .. },
                Task::ClaimReview { supplied },
                Action::InspectClaim,
            ) => {
                supplied.baseline()?;
                review.inspected_incident = true;
            }
            (
                Scope::ClaimReview { review, .. },
                Task::ClaimReview { .. },
                Action::ReadForecastSample,
            ) => {
                review.collected_next_sample = true;
            }
            (
                Scope::ClaimReview { review, .. },
                Task::ClaimReview { .. },
                Action::AssessClaim { assessment },
            ) => {
                review.diagnosis = assessment;
            }
            (
                Scope::ClaimReview { review, .. },
                Task::ClaimReview { .. },
                Action::AssessForecast { assessment },
            ) => {
                review.prediction = assessment;
            }
            (
                Scope::ClaimReview { response, .. },
                Task::ClaimReview { .. },
                Action::ClaimResponse { response: next },
            ) => {
                *response = next;
            }
            (
                Scope::ClaimReview { review, response },
                Task::ClaimReview { .. },
                Action::ApplyClaimResponse,
            ) => {
                response.apply(&mut review.active);
            }
            (
                Scope::DesiredConfig { edit },
                Task::DesiredConfig { .. },
                Action::ChangePlanDraft { text },
            ) => {
                edit.change(text)?;
            }
            (
                Scope::DesiredConfig { edit },
                Task::DesiredConfig { supplied },
                Action::PreviewConfig,
            ) => {
                edit.preview(supplied)?;
            }
            (
                Scope::DesiredConfig { edit },
                Task::DesiredConfig { supplied },
                Action::ApplyConfig,
            ) => {
                edit.apply(supplied)?;
            }
            (
                Scope::SitePaths { edit },
                Task::SitePaths { supplied },
                Action::SiteLink { index, connected },
            ) => {
                edit.set_link(supplied, index, connected)?;
            }
            (
                Scope::SocketRetirement { console },
                Task::SocketRetirement { .. },
                Action::RetirementDraft { text },
            ) => console.set_draft(text)?,
            (
                Scope::SocketRetirement { console },
                Task::SocketRetirement { supplied },
                Action::RetirementCommand { text },
            ) => console.execute(supplied, text)?,
            (
                Scope::SocketRetirement { console },
                Task::SocketRetirement { supplied },
                Action::RetirementProbe {
                    required,
                    next_start,
                },
            ) => console.observe_request(supplied, required, next_start)?,
            (
                Scope::KeyedTransfer { edit },
                Task::KeyedTransfer { .. },
                Action::ChangeTransferRule { protected, text },
            ) => edit.change(protected, text)?,
            (
                Scope::KeyedTransfer { edit },
                Task::KeyedTransfer { .. },
                Action::SelectTransferKey { reference },
            ) => {
                edit.selected_key = reference;
                edit.observation = None;
            }
            (
                Scope::KeyedTransfer { edit },
                Task::KeyedTransfer { supplied },
                Action::ApplyTransferPolicy,
            ) => edit.apply(supplied)?,
            (
                Scope::KeyedTransfer { edit },
                Task::KeyedTransfer { supplied },
                Action::RunTransfer { client, protected },
            ) => {
                edit.trial(supplied, client, protected)?;
            }
            (
                Scope::KeyedTransfer { edit },
                Task::KeyedTransfer { supplied },
                Action::TransferLookup { secondary },
            ) => edit.lookup(supplied, secondary)?,
            (
                Scope::AccessDefault { choice },
                Task::AccessDefault { .. },
                Action::SetAccessDefault { choice: next },
            ) => *choice = next,
            (
                Scope::RoutedVlans { draft },
                Task::RoutedVlans { .. },
                Action::ChangeSviDraft { draft: next },
            ) => *draft = next,
            (
                Scope::NfsExportRetirement { terminal },
                Task::NfsExportRetirement { supplied },
                Action::NfsExportDraft { text },
            ) => terminal.edit_exports(supplied, text)?,
            (
                Scope::NfsExportRetirement { terminal },
                Task::NfsExportRetirement { supplied },
                Action::NfsSaveExports,
            ) => terminal.save_exports(supplied)?,
            (
                Scope::NfsExportRetirement { terminal },
                Task::NfsExportRetirement { .. },
                Action::NfsTerminalDraft { text },
            ) => terminal.draft = text,
            (
                Scope::NfsExportRetirement { terminal },
                Task::NfsExportRetirement { supplied },
                Action::NfsTerminalRun { text },
            ) => terminal.command(supplied, text)?,
            (
                Scope::NfsExportRetirement { terminal },
                Task::NfsExportRetirement { supplied },
                Action::NfsUnmountRetired,
            ) => terminal.unmount_retired(supplied)?,
            (
                Scope::NfsExportRetirement { terminal },
                Task::NfsExportRetirement { supplied },
                Action::NfsMount { index },
            ) => terminal.mount(supplied, index)?,
            (
                Scope::NfsExportRetirement { terminal },
                Task::NfsExportRetirement { supplied },
                Action::NfsRead { index },
            ) => terminal.read(supplied, index)?,
            (
                Scope::RouteReading { draft },
                Task::RouteReading { .. },
                Action::ChangeRouteReading { draft: edited },
            ) => *draft = edited,
            (
                Scope::OspfBroadcast { draft },
                Task::OspfBroadcast { .. },
                Action::ChangeOspfBroadcast { draft: update },
            ) => *draft = update,
            (_, _, Action::Check) => next.checked = true,
            (_, _, Action::Restore) => next = Self::new(task),
            _ => return Err("Use the controls for this automation activity.".into()),
        }
        let grading_changed = match (&self.current, &next.current) {
            (
                Scope::SocketRetirement { console: before },
                Scope::SocketRetirement { console: after },
            ) => before.state != after.state,
            (Scope::KeyedTransfer { edit: before }, Scope::KeyedTransfer { edit: after }) => {
                before.served_fragment != after.served_fragment
                    || before.protected_fragment != after.protected_fragment
                    || before.selected_key != after.selected_key
                    || before.actual != after.actual
            }
            (
                Scope::NfsExportRetirement { terminal: before },
                Scope::NfsExportRetirement { terminal: after },
            ) => before.state != after.state,
            (before, after) => before != after,
        };
        if grading_changed {
            next.checked = false;
        }
        next.validate(task)?;
        *self = next;
        Ok(())
    }
}
