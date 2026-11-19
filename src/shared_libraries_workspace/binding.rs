//! Source-bound adapters for metadata-only library learning; no host operations.
use super::*;
pub const MAX_CONTEXT: usize = 1024 * 1024;
pub const PREFIX: &str = "shared-libraries ";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    EditConfiguration {
        #[serde(deserialize_with = "text::<_,4096>")]
        value: String,
    },
    SaveConfiguration,
    EditLaunch {
        #[serde(deserialize_with = "text::<_,256>")]
        program: String,
        #[serde(deserialize_with = "text::<_,1024>")]
        value: String,
    },
    SaveLaunch {
        #[serde(deserialize_with = "text::<_,256>")]
        program: String,
    },
    Run {
        #[serde(deserialize_with = "text::<_,2048>")]
        command: String,
    },
    Check,
}
impl Session {
    /// Shape validation only. Catalog admission/restoration must additionally bind
    /// to the actual authored Definition; this never grants completion.
    pub fn validate_unbound(&self) -> Result<(), String> {
        if self.version != VERSION || !fits(self, MAX_SESSION) || self.selected_example >= 12 {
            return Err("Invalid bounded library session".into());
        }
        model(&self.initial)?;
        model(&self.example_initial)?;
        self.assignment_plan
            .validate(&self.initial, &self.initial_recipes)?;
        self.example_plan
            .validate(&self.example_initial, &self.example_initial_recipes)?;
        self.assignment.validate(&self.initial)?;
        self.example.validate(&self.example_initial)
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.bound(d)?;
        if input.len() > MAX_COMMAND + PREFIX.len() {
            return Err("Library action exceeds its input limit".into());
        }
        let mut next = self.clone();
        let output = if let Some(raw) = input.strip_prefix(PREFIX) {
            let action: Action = serde_json::from_str(raw)
                .map_err(|e| format!("Invalid native library action: {e}"))?;
            match action {
                Action::EditConfiguration { value } => {
                    next.edit_configuration(d, Scope::Assignment, value)?;
                    "Configuration draft updated".into()
                }
                Action::SaveConfiguration => {
                    next.save_configuration(d, Scope::Assignment)?;
                    "Virtual configuration saved; cache unchanged".into()
                }
                Action::EditLaunch { program, value } => {
                    next.edit_launch(d, Scope::Assignment, &program, value)?;
                    "Simulated launch-setting draft updated".into()
                }
                Action::SaveLaunch { program } => {
                    next.save_launch(d, Scope::Assignment, &program)?;
                    "Simulated launch settings applied".into()
                }
                Action::Run { command } => {
                    let row = next.run(d, Scope::Assignment, &command)?;
                    if row.outcome != Outcome::Supported {
                        return Err(row.output);
                    }
                    row.output
                }
                Action::Check => {
                    let r = next.check(d, Scope::Assignment)?;
                    format!(
                        "{} of {} objective results satisfied",
                        r.checks.iter().filter(|c| c.passed).count(),
                        r.checks.len()
                    )
                }
            }
        } else {
            let row = next.run(d, Scope::Assignment, input)?;
            if row.outcome != Outcome::Supported {
                return Err(row.output);
            }
            row.output
        };
        next.bound(d)?;
        *self = next;
        Ok(output)
    }
}
impl Definition {
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        let Ok(report) = s.current_report(self, Scope::Assignment) else {
            return vec![(
                false,
                "Saved library work does not match this assignment; original retained".into(),
            )];
        };
        let mut out = vec![(
            s.assignment.checked,
            "Check the current configuration after applying your edits".into(),
        )];
        out.extend(
            report
                .checks
                .into_iter()
                .map(|c| (c.passed, format!("{}: {}", c.label, c.feedback))),
        );
        out
    }
}
pub fn metadata(a: &crate::progress::Attempt) -> Result<(), String> {
    if !matches!(a.state, crate::engine::State::SharedLibraries { .. }) {
        return Ok(());
    }
    if a.ipv4.is_some() {
        return Err("Addressing metadata on shared-library work".into());
    }
    if let Some(w) = &a.workspace
        && !foreign_empty(w)
    {
        return Err("Foreign workspace metadata on shared-library work; original retained".into());
    }
    Ok(())
}
fn foreign_empty(w: &crate::workspace::WorkspaceData) -> bool {
    let crate::workspace::WorkspaceData {
        one_off_session: _,
        one_off_learning: _,
        physical_link_draft: _,
        physical_link_learning: _,
        file_backup_learning: _,
        etherchannel_draft: _,
        etherchannel_learning: _,
        cron_session: _,
        cron_learning: _,
        nfs_session: _,
        nfs_learning: _,
        nat_learning: _,
        rpm_session: _,
        rpm_learning: _,
        nat_draft: _,
        debian_packages_terminal: _,
        apache_static_draft: _,
        apache_static_learning: _,
        debian_packages_draft: _,
        debian_packages_learning: _,
        spanning_tree_draft: _,
        spanning_tree_learning: _,
        dhcpv4_draft: _,
        dhcpv4_learning: _,
        version: _,
        unit_manager_draft: _,
        unit_manager_learning: _,
        unit_manager_terminal: _,
        ospf_draft: _,
        ospf_learning: _,
        acl_draft: _,
        acl_learning: _,
        script_learning: _,
        script_history: _,
        script_history_more: _,
        terminal_learning: _,
        process_learning: _,
        accounts_learning: _,
        accounts_template_drafts: _,
        discovery_draft: _,
        discovery_learning: _,
        storage_draft: _,
        storage_learning: _,
        ipv6_draft: _,
        ipv6_learning: _,
        routing_v4_draft: _,
        routing_v4_learning: _,
        address_learning: _,
        service_learning: _,
        client_learning: _,
        seed_dns_learning: _,
        seed_mount_learning: _,
        legacy_access_learning: _,
        legacy_logs_learning: _,
        client_draft: _,
        dns_learning: _,
        dns_draft: _,
        switching_draft: _,
        switching_learning: _,
        mac_draft: _,
        mac_learning: _,
        command: _,
        service_port: _,
        dns_address: _,
        dns_serial: _,
        mount_device: _,
        checked: _,
        ipv4_check: _,
        practice_check: _,
        migrated_host: _,
        legacy_host_reviewed: _,
        legacy_practice_reviewed: _,
        migrated_practice_host: _,
    } = w;
    w.one_off_session.is_none()
        && w.one_off_learning.is_none()
        && w.physical_link_draft.is_none()
        && w.physical_link_learning.is_none()
        && w.file_backup_learning.is_none()
        && w.etherchannel_draft.is_none()
        && w.etherchannel_learning.is_none()
        && w.cron_session.is_none()
        && w.cron_learning.is_none()
        && w.nfs_session.is_none()
        && w.nfs_learning.is_none()
        && w.nat_learning.is_none()
        && w.rpm_session.is_none()
        && w.rpm_learning.is_none()
        && w.nat_draft.is_none()
        && w.debian_packages_terminal.is_none()
        && w.apache_static_draft.is_none()
        && w.apache_static_learning.is_none()
        && w.debian_packages_draft.is_none()
        && w.debian_packages_learning.is_none()
        && w.spanning_tree_draft.is_none()
        && w.spanning_tree_learning.is_none()
        && w.dhcpv4_draft.is_none()
        && w.dhcpv4_learning.is_none()
        && w.version == 1
        && w.unit_manager_draft.is_none()
        && w.unit_manager_learning.is_none()
        && w.unit_manager_terminal.is_none()
        && w.ospf_draft.is_none()
        && w.ospf_learning.is_none()
        && w.acl_draft.is_none()
        && w.acl_learning.is_none()
        && w.script_learning.is_none()
        && w.script_history.is_none()
        && w.script_history_more.is_empty()
        && w.terminal_learning.is_none()
        && w.process_learning.is_none()
        && w.accounts_learning.is_none()
        && w.accounts_template_drafts.is_none()
        && w.discovery_draft.is_none()
        && w.discovery_learning.is_none()
        && w.storage_draft.is_none()
        && w.storage_learning.is_none()
        && w.ipv6_draft.is_none()
        && w.ipv6_learning.is_none()
        && w.routing_v4_draft.is_none()
        && w.routing_v4_learning.is_none()
        && w.address_learning.is_none()
        && w.service_learning.is_none()
        && w.client_learning.is_none()
        && w.seed_dns_learning.is_none()
        && w.seed_mount_learning.is_none()
        && w.legacy_access_learning.is_none()
        && w.legacy_logs_learning.is_none()
        && w.client_draft.is_none()
        && w.dns_learning.is_none()
        && w.dns_draft.is_none()
        && w.switching_draft.is_none()
        && w.switching_learning.is_none()
        && w.mac_draft.is_none()
        && w.mac_learning.is_none()
        && w.command.is_empty()
        && w.service_port.is_none()
        && w.dns_address.is_none()
        && w.dns_serial.is_none()
        && w.mount_device.is_none()
        && !w.checked
        && w.ipv4_check.is_none()
        && w.practice_check.is_none()
        && !w.migrated_host
        && !w.legacy_host_reviewed
        && !w.legacy_practice_reviewed
        && !w.migrated_practice_host
}

pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    metadata(a)?;
    source(s)?;
    let (
        crate::engine::State::SharedLibraries { simulation },
        crate::engine::Goal::SharedLibraries { definition },
    ) = (&a.state, &s.goal)
    else {
        return Err("Library work requires its authored task".into());
    };
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.transcript.iter().any(|t| t.len() > 8192)
        || !fits(&(s, a), MAX_CONTEXT)
    {
        return Err("Library source/attempt exceeds its bound; original retained".into());
    }
    simulation.bound(definition)
}
pub fn source(s: &crate::pack::Scenario) -> Result<(), String> {
    let (
        crate::engine::State::SharedLibraries { simulation },
        crate::engine::Goal::SharedLibraries { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Library source/goal kinds differ".into());
    };
    let fresh = Session::new(definition)?;
    if **simulation != fresh {
        return Err(
            "Authored library state must be the exact fresh task and separate example".into(),
        );
    }
    let crate::pack::Scenario {
        one_off_teaching: _,
        physical_link_teaching: _,
        file_backup_teaching: _,
        etherchannel_teaching: _,
        cron_teaching: _,
        nfs_teaching: _,
        nat_teaching: _,
        rpm_packages_teaching: _,
        apache_static_teaching: _,
        spanning_tree_teaching: _,
        dhcpv4_teaching: _,
        id: _,
        version: _,
        track: _,
        topic: _,
        unit: _,
        title: _,
        summary: _,
        objective: _,
        retry_guidance: _,
        difficulty: _,
        minutes: _,
        steps: _,
        guide: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        initial: _,
        goal: _,
        solution: _,
        prerequisites: _,
        teaching: _,
        terminal_teaching: _,
        process_teaching: _,
        accounts_teaching: _,
        unit_manager_teaching: _,
        ospf_teaching: _,
        discovery_teaching: _,
        storage_teaching: _,
        acl_teaching: _,
        ipv6_teaching: _,
        ipv4_routing_teaching: _,
        address_teaching: _,
        service_teaching: _,
        client_teaching: _,
        seed_dns_teaching: _,
        seed_mount_teaching: _,
        legacy_access_teaching: _,
        legacy_logs_teaching: _,
        dns_teaching: _,
        switching_teaching: _,
        mac_teaching: _,
        debian_packages_teaching: _,
    } = s;
    if s.one_off_teaching.is_some()
        || s.physical_link_teaching.is_some()
        || s.file_backup_teaching.is_some()
        || s.etherchannel_teaching.is_some()
        || s.cron_teaching.is_some()
        || s.nfs_teaching.is_some()
        || s.nat_teaching.is_some()
        || s.rpm_packages_teaching.is_some()
        || s.apache_static_teaching.is_some()
        || s.spanning_tree_teaching.is_some()
        || s.dhcpv4_teaching.is_some()
        || s.teaching.is_some()
        || s.terminal_teaching.is_some()
        || s.process_teaching.is_some()
        || s.accounts_teaching.is_some()
        || s.unit_manager_teaching.is_some()
        || s.ospf_teaching.is_some()
        || s.discovery_teaching.is_some()
        || s.storage_teaching.is_some()
        || s.acl_teaching.is_some()
        || s.ipv6_teaching.is_some()
        || s.ipv4_routing_teaching.is_some()
        || s.address_teaching.is_some()
        || s.service_teaching.is_some()
        || s.client_teaching.is_some()
        || s.seed_dns_teaching.is_some()
        || s.seed_mount_teaching.is_some()
        || s.legacy_access_teaching.is_some()
        || s.legacy_logs_teaching.is_some()
        || s.dns_teaching.is_some()
        || s.switching_teaching.is_some()
        || s.mac_teaching.is_some()
        || s.debian_packages_teaching.is_some()
    {
        return Err("Foreign teaching metadata on shared-library source".into());
    }
    Ok(())
}
