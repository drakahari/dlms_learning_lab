//! Compact native backup terminal. No host paths, processes or network actions.
use super::*;
use dlms_learning_lab::{
    engine::{Goal, State},
    file_backup_workspace as work,
    file_backup_workspace::presentation::{Edit, present},
};
/// Reject every foreign teaching field before a native snapshot or warm lookup.
fn source_shape(s: &Scenario) -> bool {
    let Scenario {
        file_backup_teaching,
        etherchannel_teaching,
        cron_teaching,
        nfs_teaching,
        nat_teaching,
        rpm_packages_teaching,
        apache_static_teaching,
        spanning_tree_teaching,
        dhcpv4_teaching,
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
        teaching,
        terminal_teaching,
        process_teaching,
        accounts_teaching,
        unit_manager_teaching,
        ospf_teaching,
        discovery_teaching,
        storage_teaching,
        acl_teaching,
        ipv6_teaching,
        ipv4_routing_teaching,
        address_teaching,
        service_teaching,
        client_teaching,
        seed_dns_teaching,
        seed_mount_teaching,
        legacy_access_teaching,
        legacy_logs_teaching,
        dns_teaching,
        switching_teaching,
        mac_teaching,
        debian_packages_teaching,
    } = s;
    file_backup_teaching.is_some()
        && cron_teaching.is_none()
        && etherchannel_teaching.is_none()
        && nfs_teaching.is_none()
        && nat_teaching.is_none()
        && rpm_packages_teaching.is_none()
        && apache_static_teaching.is_none()
        && spanning_tree_teaching.is_none()
        && dhcpv4_teaching.is_none()
        && teaching.is_none()
        && terminal_teaching.is_none()
        && process_teaching.is_none()
        && accounts_teaching.is_none()
        && unit_manager_teaching.is_none()
        && ospf_teaching.is_none()
        && discovery_teaching.is_none()
        && storage_teaching.is_none()
        && acl_teaching.is_none()
        && ipv6_teaching.is_none()
        && ipv4_routing_teaching.is_none()
        && address_teaching.is_none()
        && service_teaching.is_none()
        && client_teaching.is_none()
        && seed_dns_teaching.is_none()
        && seed_mount_teaching.is_none()
        && legacy_access_teaching.is_none()
        && legacy_logs_teaching.is_none()
        && dns_teaching.is_none()
        && switching_teaching.is_none()
        && mac_teaching.is_none()
        && debian_packages_teaching.is_none()
        && matches!(
            (&s.initial, &s.goal),
            (State::FileBackup { .. }, Goal::FileBackup { .. })
        )
}
fn same_source(a: &Scenario, b: &Scenario) -> bool {
    if !source_shape(a) || !source_shape(b) {
        return false;
    }
    let (Goal::FileBackup { definition: ad }, Goal::FileBackup { definition: bd }) =
        (&a.goal, &b.goal)
    else {
        return false;
    };
    a.id == b.id
        && a.version == b.version
        && a.track == b.track
        && a.topic == b.topic
        && a.unit == b.unit
        && a.title == b.title
        && a.summary == b.summary
        && a.objective == b.objective
        && a.retry_guidance == b.retry_guidance
        && a.difficulty == b.difficulty
        && a.minutes == b.minutes
        && a.steps == b.steps
        && a.explanation == b.explanation
        && a.hints == b.hints
        && a.sources == b.sources
        && a.simplifications == b.simplifications
        && a.solution == b.solution
        && a.prerequisites == b.prerequisites
        && a.initial == b.initial
        && ad == bd
        && a.file_backup_teaching == b.file_backup_teaching
        && a.guide.len() == b.guide.len()
        && a.guide.iter().zip(&b.guide).all(|(a, b)| {
            a.title == b.title
                && a.why == b.why
                && a.action == b.action
                && a.commands.len() == b.commands.len()
                && a.commands.iter().zip(&b.commands).all(|(a, b)| {
                    a.command == b.command
                        && a.explanation == b.explanation
                        && a.simulator_only == b.simulator_only
                })
        })
}
fn same_attempt(a: &Attempt, b: &Attempt) -> bool {
    // No ..: an added persistent field cannot silently disappear from the key.
    let Attempt {
        scenario_version: _,
        state: _,
        step: _,
        challenge: _,
        hints: _,
        transcript: _,
        visited: _,
        ipv4: _,
        workspace: _,
    } = a;
    a.scenario_version == b.scenario_version
        && a.state == b.state
        && a.step == b.step
        && a.challenge == b.challenge
        && a.hints == b.hints
        && a.transcript == b.transcript
        && a.visited == b.visited
        && a.ipv4.is_none()
        && b.ipv4.is_none()
        && match (&a.workspace, &b.workspace) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                !foreign(a)
                    && !foreign(b)
                    && a.version == b.version
                    && a.command == b.command
                    && a.checked == b.checked
                    && a.file_backup_learning == b.file_backup_learning
            }
            _ => false,
        }
}

fn foreign(w: &model::WorkspaceData) -> bool {
    let model::WorkspaceData {
        etherchannel_draft,
        etherchannel_learning,
        file_backup_learning: _,
        cron_session,
        cron_learning,
        nfs_session,
        nfs_learning,
        nat_learning,
        rpm_session,
        rpm_learning,
        nat_draft,
        debian_packages_terminal,
        apache_static_draft,
        apache_static_learning,
        debian_packages_draft,
        debian_packages_learning,
        spanning_tree_draft,
        spanning_tree_learning,
        dhcpv4_draft,
        dhcpv4_learning,
        version: _,
        unit_manager_draft,
        unit_manager_learning,
        unit_manager_terminal,
        ospf_draft,
        ospf_learning,
        acl_draft,
        acl_learning,
        script_learning,
        script_history,
        script_history_more,
        terminal_learning,
        process_learning,
        accounts_learning,
        discovery_draft,
        discovery_learning,
        storage_draft,
        storage_learning,
        ipv6_draft,
        ipv6_learning,
        routing_v4_draft,
        routing_v4_learning,
        address_learning,
        service_learning,
        client_learning,
        seed_dns_learning,
        seed_mount_learning,
        legacy_access_learning,
        legacy_logs_learning,
        client_draft,
        dns_learning,
        dns_draft,
        switching_draft,
        switching_learning,
        mac_draft,
        mac_learning,
        command: _,
        service_port,
        dns_address,
        dns_serial,
        mount_device,
        checked: _,
        ipv4_check,
        practice_check,
        migrated_host,
        legacy_host_reviewed,
        legacy_practice_reviewed,
        migrated_practice_host,
    } = w;
    cron_session.is_some()
        || cron_learning.is_some()
        || etherchannel_draft.is_some()
        || etherchannel_learning.is_some()
        || nfs_session.is_some()
        || nfs_learning.is_some()
        || nat_learning.is_some()
        || rpm_session.is_some()
        || rpm_learning.is_some()
        || nat_draft.is_some()
        || debian_packages_terminal.is_some()
        || apache_static_draft.is_some()
        || apache_static_learning.is_some()
        || debian_packages_draft.is_some()
        || debian_packages_learning.is_some()
        || spanning_tree_draft.is_some()
        || spanning_tree_learning.is_some()
        || dhcpv4_draft.is_some()
        || dhcpv4_learning.is_some()
        || unit_manager_draft.is_some()
        || unit_manager_learning.is_some()
        || unit_manager_terminal.is_some()
        || ospf_draft.is_some()
        || ospf_learning.is_some()
        || acl_draft.is_some()
        || acl_learning.is_some()
        || script_learning.is_some()
        || script_history.is_some()
        || !script_history_more.is_empty()
        || terminal_learning.is_some()
        || process_learning.is_some()
        || accounts_learning.is_some()
        || discovery_draft.is_some()
        || discovery_learning.is_some()
        || storage_draft.is_some()
        || storage_learning.is_some()
        || ipv6_draft.is_some()
        || ipv6_learning.is_some()
        || routing_v4_draft.is_some()
        || routing_v4_learning.is_some()
        || address_learning.is_some()
        || service_learning.is_some()
        || client_learning.is_some()
        || seed_dns_learning.is_some()
        || seed_mount_learning.is_some()
        || legacy_access_learning.is_some()
        || legacy_logs_learning.is_some()
        || client_draft.is_some()
        || dns_learning.is_some()
        || dns_draft.is_some()
        || switching_draft.is_some()
        || switching_learning.is_some()
        || mac_draft.is_some()
        || mac_learning.is_some()
        || service_port.is_some()
        || dns_address.is_some()
        || dns_serial.is_some()
        || mount_device.is_some()
        || ipv4_check.is_some()
        || practice_check.is_some()
        || *migrated_host
        || *legacy_host_reviewed
        || *legacy_practice_reviewed
        || *migrated_practice_host
}

// Private native presentation proposal; uncompiled, no catalog/acceptance claim.
// Requires exact_keys.rs and the exhaustive WorkspaceData foreign classifier.
const ENTRY_BYTES: usize = 8 * 1024 * 1024;
const TOTAL_BYTES: usize = 32 * 1024 * 1024;
const RUN_BYTES: usize = 16 * 1024 * 1024;
const ALL_BYTES: usize = TOTAL_BYTES + RUN_BYTES;
// Reuse the existing allocation-free native structural meter: every scalar,
// key and container is charged128bytes, in addition to string payload bytes.
use crate::ui::etherchannel_workspace::memo::charged;
const REPORT_RESERVE: usize = 256 * 1024;
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
struct Counters {
    admissions: usize,
    copies: usize,
    ticks: usize,
    hits: usize,
}
struct Entry {
    source: Scenario,
    attempt: Attempt,
    assignment: Vec<(bool, String)>,
    practice: Vec<(bool, String)>,
    charge: usize,
    touched: u64,
    cancelled: bool,
}
struct Job {
    source: Scenario,
    attempt: Attempt,
    run: Option<work::CheckRun>,
    assignment: Option<Vec<(bool, String)>>,
    charge: usize,
    run_charge: usize,
}
#[derive(Default)]
struct CheckMemo {
    entries: Vec<Entry>,
    job: Option<Job>,
    tick: u64,
    counters: Counters,
    notice: Option<String>,
    cancelled: std::collections::BTreeSet<String>,
    background_paused: bool,
}
impl CheckMemo {
    fn clear(&mut self) {
        self.entries.clear();
        self.job = None;
        self.notice = None;
        self.cancelled.clear();
        self.background_paused = false;
    }
    fn cancel(&mut self) {
        if let Some(mut job) = self.job.take() {
            if self.cancelled.len() < 64 {
                self.cancelled.insert(job.source.id.clone());
            } else {
                self.background_paused = true;
            }
            // Keep admission, not a partial grade, so Stop retains an editable
            // workspace without restarting a checker on the next idle frame.
            job.run = None;
            self.entries.retain(|e| e.source.id != job.source.id);
            while self.entries.len() >= 4
                || self
                    .entries
                    .iter()
                    .map(|e| e.charge)
                    .sum::<usize>()
                    .saturating_add(job.charge)
                    > TOTAL_BYTES
            {
                let Some(index) = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.touched)
                    .map(|(i, _)| i)
                else {
                    break;
                };
                self.entries.remove(index);
            }
            self.tick = self.tick.saturating_add(1);
            self.entries.push(Entry {
                source: job.source,
                attempt: job.attempt,
                assignment: Vec::new(),
                practice: Vec::new(),
                charge: job.charge,
                touched: self.tick,
                cancelled: true,
            });
        }
        self.notice = Some(
            "Check cancelled. Files and drafts are retained. Choose Check work to try again."
                .into(),
        );
    }
    fn ready(&self, a: &Attempt, s: &Scenario) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|e| same_source(&e.source, s) && same_attempt(&e.attempt, a))
    }
    fn rows(&self, a: &Attempt, s: &Scenario, practice: bool) -> Option<&[(bool, String)]> {
        self.ready(a, s).filter(|e| !e.cancelled).map(|e| {
            if practice {
                e.practice.as_slice()
            } else {
                e.assignment.as_slice()
            }
        })
    }
    fn passed(&self, a: &Attempt, s: &Scenario) -> Option<bool> {
        self.rows(a, s, false)
            .map(|rows| !rows.is_empty() && rows.iter().all(|r| r.0))
    }
    /// Actual UI caller supplies borrowed current data. No stale result survives a
    /// changed complete key; all owned admission occurs only on a genuine miss.
    fn prepare(&mut self, a: &Attempt, s: &Scenario, force: bool) -> Result<(), String> {
        if !source_shape(s) || a.ipv4.is_some() || a.workspace.as_ref().is_some_and(foreign) {
            self.entries.retain(|e| e.source.id != s.id);
            if self.job.as_ref().is_some_and(|j| j.source.id == s.id) {
                self.job = None;
            }
            return Err("Backup source or workspace contains another activity's fields".into());
        }
        let paused = !force && (self.background_paused || self.cancelled.contains(&s.id));
        if paused && self.ready(a, s).is_some() {
            return Ok(());
        }
        if force {
            self.cancelled.remove(&s.id);
            self.background_paused = false;
        }
        if !force && self.ready(a, s).is_some() {
            self.counters.hits += 1;
            return Ok(());
        }
        if !force
            && self
                .job
                .as_ref()
                .is_some_and(|j| same_source(&j.source, s) && same_attempt(&j.attempt, a))
        {
            return Ok(());
        }
        // A library card must not cancel another card's in-flight checker.
        // The owner advances that job against its own authoritative current ID.
        if !force && !paused && self.job.as_ref().is_some_and(|j| j.source.id != s.id) {
            return Ok(());
        }
        let definition = work::context(a, s)?;
        if a.workspace
            .as_ref()
            .and_then(|w| w.file_backup_learning.as_ref())
            .is_none()
        {
            return Err(
                "Open the lesson's separate practice before native editing; saved work is retained"
                    .into(),
            );
        }
        // Structural admission occurs before copies and rejects pathological
        // tiny-value/container density as well as large string payloads.
        let charge = charged(s, ENTRY_BYTES)
            .and_then(|x| charged(a, ENTRY_BYTES).and_then(|y| x.checked_add(y)))
            .and_then(|n| n.checked_add(REPORT_RESERVE))
            .filter(|n| *n <= ENTRY_BYTES)
            .ok_or("Backup result exceeds the native snapshot bound")?;
        if paused {
            // Navigation changes recency and therefore the complete key. Re-admit
            // only the bounded editable snapshot; preserve Stop and publish no
            // rows or checker. Context/whole-attempt and structural bounds above
            // precede every owned copy, including a paused workspace's new key.
            if self.job.as_ref().is_some_and(|j| j.source.id == s.id) {
                self.job = None;
            }
            self.entries.retain(|e| e.source.id != s.id);
            let live_charge = self.job.as_ref().map_or(0, |j| j.run_charge);
            while self.entries.len() >= 4
                || self
                    .entries
                    .iter()
                    .map(|e| e.charge)
                    .sum::<usize>()
                    .saturating_add(charge)
                    > TOTAL_BYTES
                || self
                    .entries
                    .iter()
                    .map(|e| e.charge)
                    .sum::<usize>()
                    .saturating_add(charge)
                    .saturating_add(live_charge)
                    > ALL_BYTES
            {
                let index = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.touched)
                    .map(|(i, _)| i)
                    .ok_or("Backup cache cannot reserve the paused workspace")?;
                self.entries.remove(index);
            }
            self.tick = self.tick.saturating_add(1);
            self.entries.push(Entry {
                source: s.clone(),
                attempt: a.clone(),
                assignment: Vec::new(),
                practice: Vec::new(),
                charge,
                touched: self.tick,
                cancelled: true,
            });
            self.counters.admissions += 1;
            self.counters.copies += 1;
            return Ok(());
        }
        let State::FileBackup { simulation } = &a.state else {
            return Err("Expected a backup workspace".into());
        };
        // Account for BOTH possible checker stages before any owned copy. A
        // CheckRun owns one Definition, one Store and at most one recovery Tree.
        // The entire definition already contains every fresh recovery probe.
        let assignment_run = charged(definition, RUN_BYTES)
            .and_then(|n| {
                charged(simulation, RUN_BYTES).and_then(|v| n.checked_mul(2)?.checked_add(v))
            })
            .ok_or("Backup assignment checker exceeds its snapshot bound")?;
        let practice_run = if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.file_backup_learning.as_ref())
        {
            let lesson = s
                .file_backup_teaching
                .as_ref()
                .ok_or("Missing backup example")?;
            charged(&lesson.definition, RUN_BYTES)
                .and_then(|n| {
                    charged(&l.state, RUN_BYTES).and_then(|v| n.checked_mul(2)?.checked_add(v))
                })
                .ok_or("Backup example checker exceeds its snapshot bound")?
        } else {
            0
        };
        let run_charge = assignment_run.max(practice_run).checked_add(charge)
            .and_then(|n| n.checked_add(REPORT_RESERVE))
            .filter(|n| *n <= RUN_BYTES)
            .ok_or("Backup checker cannot fit the native working-memory allowance; saved work is retained")?;
        // Drop the earlier owned checker before constructing its replacement.
        self.job = None;
        self.entries.retain(|e| e.source.id != s.id);
        while self
            .entries
            .iter()
            .map(|e| e.charge)
            .sum::<usize>()
            .saturating_add(run_charge)
            > ALL_BYTES
        {
            let index = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.touched)
                .map(|(i, _)| i)
                .ok_or("Backup cache cannot reserve the next checker")?;
            self.entries.remove(index);
        }
        let run = work::CheckRun::new(definition, simulation)?;
        self.job = Some(Job {
            source: s.clone(),
            attempt: a.clone(),
            run: Some(run),
            assignment: None,
            charge,
            run_charge,
        });
        self.counters.admissions += 1;
        self.counters.copies += 1;
        self.notice = None;
        self.entries.retain(|e| e.source.id != s.id);
        Ok(())
    }
    fn advance_progress(
        &mut self,
        p: &dlms_learning_lab::progress::Progress,
        scenarios: &[Scenario],
    ) -> bool {
        let Some(id) = self.job.as_ref().map(|j| j.source.id.clone()) else {
            return false;
        };
        let (Some(a), Some(s)) = (p.attempts.get(&id), scenarios.iter().find(|s| s.id == id))
        else {
            self.job = None;
            return false;
        };
        self.advance(a, s)
    }
    /// At most one archive application per frame. A pending result is never
    /// returned as passed. Stop drops only this Job; live learner data is borrowed.
    fn advance(&mut self, a: &Attempt, s: &Scenario) -> bool {
        let Some(job) = &mut self.job else {
            return false;
        };
        if !same_source(&job.source, s) || !same_attempt(&job.attempt, a) {
            self.job = None;
            return false;
        }
        debug_assert!(job.run_charge <= RUN_BYTES);
        debug_assert!(
            self.entries.iter().map(|e| e.charge).sum::<usize>() + job.run_charge <= ALL_BYTES
        );
        self.counters.ticks += 1;
        if !job.run.as_mut().expect("admitted checker").tick() {
            return true;
        }
        if job.assignment.is_none() {
            let complete = job.run.take().expect("completed assignment checker");
            job.assignment = Some(complete.rows().to_vec());
            drop(complete);
            if let Some(learning) = job
                .attempt
                .workspace
                .as_ref()
                .and_then(|w| w.file_backup_learning.as_ref())
            {
                let lesson = job.source.file_backup_teaching.as_ref().unwrap();
                match work::CheckRun::new(&lesson.definition, &learning.state) {
                    Ok(run) => {
                        job.run = Some(run);
                        return true;
                    }
                    Err(e) => {
                        self.job = None;
                        self.notice = Some(e);
                        return false;
                    }
                }
            }
        }
        let Some(job) = self.job.take() else {
            return false;
        };
        let assignment = job.assignment.unwrap();
        let practice = if job
            .attempt
            .workspace
            .as_ref()
            .is_some_and(|w| w.file_backup_learning.is_some())
        {
            job.run
                .as_ref()
                .expect("completed practice checker")
                .rows()
                .to_vec()
        } else {
            Vec::new()
        };
        self.tick = self.tick.saturating_add(1);
        while self.entries.len() >= 4
            || self
                .entries
                .iter()
                .map(|e| e.charge)
                .sum::<usize>()
                .saturating_add(job.charge)
                > TOTAL_BYTES
        {
            let Some((index, _)) = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.touched)
            else {
                self.notice =
                    Some("Backup result cannot fit its native cache; files are retained".into());
                return false;
            };
            self.entries.remove(index);
        }
        self.entries.push(Entry {
            source: job.source,
            attempt: job.attempt,
            assignment,
            practice,
            charge: job.charge,
            touched: self.tick,
            cancelled: false,
        });
        false
    }
}

/// Byte-bounded edits preserve their exact old value on excessive paste, without
/// cloning any Store, Attempt or Definition on a clean frame.
struct CommandBuffer<'a> {
    text: &'a mut String,
    pub rejected: bool,
    pub received: bool,
    before: Option<String>,
}
impl CommandBuffer<'_> {
    fn remember(&mut self) {
        if self.before.is_none() {
            self.before = Some(self.text.clone());
        }
    }
    fn rollback(&mut self) {
        if let Some(old) = self.before.take() {
            *self.text = old;
        }
    }
}
impl egui::TextBuffer for CommandBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<CommandBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.text
    }
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        if text.is_empty() || self.rejected {
            return 0;
        }
        self.received = true;
        if self
            .text
            .len()
            .checked_add(text.len())
            .is_none_or(|n| n > 4096)
            || text.contains(['\0', '\n', '\r'])
        {
            self.rejected = true;
            return 0;
        }
        self.remember();
        let byte = self
            .text
            .char_indices()
            .nth(char_index)
            .map_or(self.text.len(), |(n, _)| n);
        self.text.insert_str(byte, text);
        text.chars().count()
    }
    fn delete_char_range(&mut self, range: std::ops::Range<usize>) {
        let start = self
            .text
            .char_indices()
            .nth(range.start)
            .map_or(self.text.len(), |(n, _)| n);
        let end = self
            .text
            .char_indices()
            .nth(range.end)
            .map_or(self.text.len(), |(n, _)| n);
        if !self.rejected && start < end {
            self.remember();
            self.text.replace_range(start..end, "");
        }
    }
}
struct NativeCheckpoint {
    state: egui::text_edit::TextEditState,
    undo: egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>,
    id: egui::Id,
}
impl NativeCheckpoint {
    fn restore(mut self, ctx: &egui::Context) {
        self.state.set_undoer(self.undo);
        self.state.store(ctx, self.id);
    }
}
#[derive(Default)]
pub(super) struct TerminalMetrics {
    pub input: Option<(egui::Rect, egui::Id)>,
    pub input_clip: Option<egui::Rect>,
    pub run: Option<egui::Rect>,
    pub output: Option<egui::Rect>,
    pub rejected: bool,
    pub received: bool,
    pub dropped: bool,
    checkpoint: Option<NativeCheckpoint>,
}
// Virtualize output lines, including adversarial restored long lines. Every
// UTF-8 byte remains available by scrolling; only visible bounded chunks become
// font galleys. No owned line-index or complete transcript clone is built.
#[derive(Clone)]
struct LineChunks<'a> {
    remaining: &'a str,
    first: bool,
}
impl<'a> Iterator for LineChunks<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return std::mem::replace(&mut self.first, false).then_some("");
        }
        self.first = false;
        let mut end = self.remaining.len().min(4096);
        while !self.remaining.is_char_boundary(end) {
            end -= 1;
        }
        let (line, remaining) = self.remaining.split_at(end);
        self.remaining = remaining;
        Some(line)
    }
}
fn output_lines(transcript: &[String]) -> impl Iterator<Item = &str> + Clone {
    transcript.iter().flat_map(|entry| {
        entry.split('\n').flat_map(|line| LineChunks {
            remaining: line,
            first: true,
        })
    })
}
/// Rendering borrows transcript text; only an actual Run/Enter creates a bounded
/// command intent. Outer page scrolling and this output viewport stay independent.
struct TerminalView<'a> {
    transcript: &'a [String],
    cwd: &'a str,
    active: bool,
    focus: bool,
    entry_focus: Option<egui::Id>,
    new_generation: bool,
}
fn terminal(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    command: &mut String,
    view: TerminalView<'_>,
    metrics: &mut TerminalMetrics,
) -> Option<String> {
    let TerminalView {
        transcript,
        cwd,
        active,
        focus,
        entry_focus,
        new_generation,
    } = view;
    let active = active && ui.is_enabled();
    let mut intent = None;
    ui.push_id(id,|ui|{
        ui.label(egui::RichText::new(format!("Simulated root · {cwd}")).small());
        let height=(ui.ctx().content_rect().height()*0.28).clamp(ui.text_style_height(&egui::TextStyle::Monospace)*4.0,300.0);
        let area=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
            if transcript.is_empty() { ui.add(egui::Label::new("Type one command below, then choose Run command. All files stay inside this simulator.").wrap()); }
            let row_height = ui.text_style_height(&egui::TextStyle::Monospace).ceil();
            let size = egui::vec2(ui.available_width(),height + ui.spacing().scroll.allocated_width());
            controls::fixed_viewport(ui,size,|ui| {
                egui::ScrollArea::both().id_salt("output").max_height(size.y).min_scrolled_height(height)
                    .auto_shrink([false,false]).stick_to_bottom(true)
                    .show_rows(ui,row_height,output_lines(transcript).count(),|ui,range| {
                        for text in output_lines(transcript).skip(range.start).take(range.len()) {
                            ui.add(egui::Label::new(egui::RichText::new(text).monospace()).extend());
                        }
                    });
            });
        });metrics.output=Some(area.response.rect);
        ui.add_enabled_ui(active, |ui| {
            let field_id = ui.make_persistent_id("command");
            if new_generation {
                let mut state = egui::text_edit::TextEditState::default();
                state.set_undoer(egui::util::undoer::Undoer::with_settings(egui::util::undoer::Settings { max_undos: 16, ..Default::default() }));
                state.store(ui.ctx(), field_id);
            }
            let pending = active && entry_focus == Some(field_id) && ui.input(|i| i.events.iter().any(|e| match e {
                egui::Event::Text(text) => !text.is_empty() && text != "\n" && text != "\r",
                egui::Event::Paste(text) => !text.is_empty(),
                _ => false,
            }));
            // Only a real pending edit copies the bounded native Undo/cursor
            // state. Normal idle repaints do not clone it or the virtual Store.
            let editing_key = active && entry_focus == Some(field_id) && ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Key { pressed: true, .. } | egui::Event::Cut)));
            let prior_native = (pending || editing_key).then(|| egui::text_edit::TextEditState::load(ui.ctx(), field_id)
                .map(|state| { let undo = state.undoer(); NativeCheckpoint { state, undo, id: field_id } })).flatten();
            let mut buffer = CommandBuffer { text: command, rejected: false, received: false, before: None };
            let editor = controls::singleline(ui, &mut buffer)
                .id(field_id).font(egui::TextStyle::Monospace)
                .hint_text("Type one simulated command").desired_width(ui.available_width());
            let response = ui.add(editor);
            // Undo also calls TextBuffer::insert_text. It cannot serve as a
            // receipt for a later Paste that single-line Enter never delivers.
            let after_return = pending && ui.input(|input| {
                let mut returned = false;
                input.events.iter().any(|event| {
                    if matches!(event, egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers, .. }
                        if modifiers.matches_logically(egui::Modifiers::NONE)) {
                        returned = true;
                    }
                    returned && matches!(event, egui::Event::Text(t) | egui::Event::Paste(t)
                        if !t.is_empty() && t != "\n" && t != "\r")
                })
            });
            let dropped = pending && (!buffer.received || after_return);
            if buffer.rejected || dropped { buffer.rollback(); }
            metrics.input = Some((response.rect, response.id));
            metrics.input_clip = Some(ui.clip_rect());
            metrics.rejected = buffer.rejected;
            metrics.received = buffer.received && !after_return;
            metrics.dropped = dropped;
            if buffer.rejected || dropped { if let Some(prior) = prior_native { prior.restore(ui.ctx()); } } else { metrics.checkpoint = prior_native; }
            if focus && active { response.request_focus(); }
            name_control(&response, "Simulated backup terminal command");
            let run_enter = (response.has_focus() || response.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.horizontal_wrapped(|ui| {
                let run = ui.add(button("Run command")); metrics.run = Some(run.rect);
                ui.label(egui::RichText::new("Enter runs · No host commands").small());
                if active && !buffer.rejected && !dropped && (run.clicked() || run_enter) {
                    intent = Some(buffer.text.clone());
                }
            });
        });
    });
    intent
}

// Compact backup workspace; authored registration and native acceptance are separate gates.
#[derive(Default)]
struct Document {
    owner: Option<egui::Id>,
    value: String,
    field: Option<egui::Id>,
}
#[derive(Default)]
pub(super) struct Presentation {
    memo: CheckMemo,
    document: Document,
    notice: Option<String>,
    hint: usize,
    focus: bool,
    owner: Option<(String, u32)>,
    generation: u64,
    bound_generation: u64,
    advanced_frame: Option<u64>,
}
#[derive(Default)]
pub(super) struct BackupMetrics {
    pub heading: Option<egui::Rect>,
    pub goal: Option<(egui::Rect, f32)>,
    pub tabs: Vec<(teaching::View, egui::Rect)>,
    pub controls: Vec<(String, egui::Rect)>,
    pub inserts: Vec<(usize, egui::Rect)>,
    pub terminal: TerminalMetrics,
    pub rows: Vec<(usize, bool, egui::Rect)>,
    pub complete: bool,
}
fn action(ui: &mut egui::Ui, m: &mut BackupMetrics, key: &str, text: &str, main: bool) -> bool {
    let r = ui.add(if main {
        primary(ui, text)
    } else {
        button(text)
    });
    m.controls.push((key.into(), r.rect));
    r.clicked()
}
fn view(a: &Attempt) -> teaching::View {
    if a.challenge {
        teaching::View::Assignment
    } else {
        a.workspace
            .as_ref()
            .and_then(|w| w.file_backup_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
}
/// Caller supplies the actual pre-header focus; terminal receipts handle same-frame
/// edit+Run and dropped Paste. All navigation is deferred on pending text.
fn page_inner(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut BackupMetrics,
    active: bool,
    entry_focus: Option<egui::Id>,
) -> bool {
    if p.owner
        .as_ref()
        .is_none_or(|(id, version)| id != &s.id || *version != s.version)
    {
        p.owner = Some((s.id.clone(), s.version));
        p.notice = None;
        p.memo.notice = None;
        p.hint = 0;
        p.focus = false;
        p.generation = p.generation.wrapping_add(1);
    }
    if !source_shape(s) || a.ipv4.is_some() || a.workspace.as_ref().is_some_and(foreign) {
        ui.label("This saved workspace belongs to a different activity. Your work is retained.");
        return false;
    }
    let admitted = p.memo.ready(a, s).is_some()
        || p.memo
            .job
            .as_ref()
            .is_some_and(|j| same_source(&j.source, s) && same_attempt(&j.attempt, a));
    if !admitted {
        m.heading = Some(ui.heading(&s.title).rect);
        ui.label(
            "Open the current simulated workspace before continuing. Your saved work is retained.",
        );
        if active
            && action(ui, m, "admit", "Open workspace", true)
            && let Err(e) = p.memo.prepare(a, s, true)
        {
            p.notice = Some(e);
        }
        if let Some(notice) = &p.notice {
            ui.add(egui::Label::new(notice).wrap());
        }
        return false;
    }
    let Some(lesson) = &s.file_backup_teaching else {
        return false;
    };
    let current = view(a);
    let pending = p.document.field == entry_focus
        && entry_focus.is_some()
        && ui.input(|i| {
            i.events.iter().any(|e| match e {
                egui::Event::Text(t) => !t.is_empty() && t != "\n" && t != "\r",
                egui::Event::Paste(t) => !t.is_empty(),
                _ => false,
            })
        });
    let mut requested = None;
    m.heading = Some(ui.heading(&s.title).rect);
    if !a.challenge {
        ui.add_enabled_ui(active && !pending, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(current == v, label);
                    m.tabs.push((v, r.rect));
                    if r.clicked() {
                        requested = Some(v);
                    }
                }
            })
        });
    }
    let mut dirty = false;
    if current == teaching::View::Introduction {
        ui.add(egui::Label::new(&lesson.prerequisite_note).wrap());
        for c in &lesson.concepts {
            ui.label(RichText::new(&c.title).strong());
            ui.add(egui::Label::new(&c.text).wrap());
        }
        ui.add_enabled_ui(active && !pending, |ui| {
            ui.horizontal_wrapped(|ui| {
                if action(ui, m, "try-example", "Try the example", true) {
                    requested = Some(teaching::View::Example);
                }
                if action(ui, m, "assignment", "Go straight to assignment", false) {
                    requested = Some(teaching::View::Assignment);
                }
            })
        });
    } else {
        let practice = current == teaching::View::Example;
        let goal = ui.add(
            egui::Label::new(if practice {
                &lesson.example_description
            } else {
                &s.objective
            })
            .wrap(),
        );
        m.goal = Some((goal.rect, ui.text_style_height(&egui::TextStyle::Body)));
        if practice {
            ui.label(
                RichText::new("Practice only · Your assignment files and draft remain separate.")
                    .small(),
            );
        }
        let owner = egui::Id::new(("backup-command", &s.id, a.scenario_version, practice));
        let saved = a.workspace.as_ref().unwrap();
        let learning = saved.file_backup_learning.as_ref().unwrap();
        let command = if practice {
            &learning.command
        } else {
            &saved.command
        };
        let owner_changed = p.document.owner != Some(owner);
        let new_generation =
            owner_changed || p.document.value != *command || p.generation != p.bound_generation;
        if new_generation {
            if let Some(id) = p.document.field.take() {
                ui.ctx()
                    .data_mut(|data| data.remove::<egui::text_edit::TextEditState>(id));
            }
            p.document.owner = Some(owner);
            p.bound_generation = p.generation;
            p.document.value.clone_from(command);
            if owner_changed {
                p.notice = None;
                p.memo.notice = None;
                p.hint = 0;
                p.focus = false;
            }
        }
        let mut insert = None;
        if practice {
            controls::collapsing_header(ui, "Worked example commands")
                .default_open(true)
                .show(ui, |ui| {
                    for (n, command) in lesson.example_commands.iter().enumerate() {
                        ui.add(
                            egui::Label::new(RichText::new(&command.command).monospace()).wrap(),
                        );
                        ui.add(egui::Label::new(&command.explanation).wrap());
                        let r = ui.add_enabled(active && !pending, button("Insert command"));
                        m.inserts.push((n, r.rect));
                        if r.clicked() {
                            insert = Some(command.command.as_str());
                        }
                    }
                });
        }
        let (cwd, transcript) = if practice {
            (learning.state.cwd.as_str(), learning.transcript.as_slice())
        } else {
            let engine::State::FileBackup { simulation } = &a.state else {
                unreachable!()
            };
            (simulation.cwd.as_str(), a.transcript.as_slice())
        };
        let run = terminal(
            ui,
            owner,
            &mut p.document.value,
            TerminalView {
                transcript,
                cwd,
                active,
                focus: std::mem::take(&mut p.focus),
                entry_focus,
                new_generation,
            },
            &mut m.terminal,
        );
        p.document.field = m.terminal.input.map(|(_, id)| id);
        let mut edit_rejected = m.terminal.rejected || m.terminal.dropped;
        if m.terminal.rejected {
            p.notice = Some(
                "Use one command line of at most4096bytes; the earlier input is retained.".into(),
            );
        } else if m.terminal.dropped {
            p.notice = Some("Your text did not reach the command field. Focus it and retry; no command was run.".into());
        }
        if !edit_rejected && p.document.value != *command {
            let previous_command = command.clone();
            match present(
                a,
                s,
                practice,
                Edit {
                    command: Some(&p.document.value),
                    ..Default::default()
                },
            ) {
                Ok(()) => {
                    dirty = true;
                    p.notice = None;
                }
                Err(e) => {
                    p.document.value = previous_command;
                    if let Some(checkpoint) = m.terminal.checkpoint.take() {
                        checkpoint.restore(ui.ctx());
                    }
                    edit_rejected = true;
                    p.notice = Some(e);
                }
            }
        }
        if active && !edit_rejected {
            if let Some(input) = run {
                // Release checker snapshots before the bounded whole-attempt transaction.
                p.memo.job = None;
                let scope = if practice {
                    work::Scope::Practice
                } else {
                    work::Scope::Assignment
                };
                match work::perform(a, s, scope, &input) {
                    Ok(_) => {
                        dirty = true;
                        p.notice = None;
                    }
                    Err(e) => p.notice = Some(e),
                }
            }
            if let Some(input) = insert {
                match present(
                    a,
                    s,
                    true,
                    Edit {
                        command: Some(input),
                        ..Default::default()
                    },
                ) {
                    Ok(()) => {
                        dirty = true;
                        p.focus = true;
                        p.generation = p.generation.wrapping_add(1);
                    }
                    Err(e) => p.notice = Some(e),
                }
            }
        }
        // Keep a refused-edit message until the learner retries. A one-frame
        // label above Run moved its click target before the next real click.
        if let Some(notice) = p.notice.as_ref().or(p.memo.notice.as_ref()) {
            ui.add(egui::Label::new(notice).wrap());
        }
        let mut check = false;
        ui.add_enabled_ui(active && !edit_rejected && !pending, |ui| {
            ui.horizontal_wrapped(|ui| {
                check = action(
                    ui,
                    m,
                    "check",
                    if practice {
                        "Check example"
                    } else {
                        "Check work"
                    },
                    true,
                );
                if p.memo.job.is_some() && action(ui, m, "stop", "Stop checking", false) {
                    p.memo.cancel();
                }
            })
        });
        if check {
            let old_checked = a.workspace.as_ref().is_some_and(|w| {
                if practice {
                    w.file_backup_learning.as_ref().is_some_and(|l| l.checked)
                } else {
                    w.checked
                }
            });
            let result = present(
                a,
                s,
                practice,
                Edit {
                    checked: Some(true),
                    ..Default::default()
                },
            );
            match result {
                Ok(()) => match p.memo.prepare(a, s, true) {
                    Ok(()) => {
                        dirty = true;
                        p.notice = None;
                    }
                    Err(e) => {
                        // Checker admission cannot partially commit the saved
                        // checked flag. No other Attempt field was touched.
                        let w = a.workspace.as_mut().unwrap();
                        if practice {
                            w.file_backup_learning.as_mut().unwrap().checked = old_checked;
                        } else {
                            w.checked = old_checked;
                        }
                        p.notice = Some(e);
                    }
                },
                Err(e) => p.notice = Some(e),
            }
        }
        let checked = a.workspace.as_ref().is_some_and(|w| {
            if practice {
                w.file_backup_learning.as_ref().is_some_and(|l| l.checked)
            } else {
                w.checked
            }
        });
        if checked {
            if let Some(rows) = p.memo.rows(a, s, practice) {
                let passed = !rows.is_empty() && rows.iter().all(|r| r.0);
                ui.label(
                    RichText::new(if passed {
                        if practice {
                            "Example verified · Assignment credit is separate"
                        } else {
                            "Assignment complete"
                        }
                    } else {
                        "Some requirements still need attention"
                    })
                    .strong(),
                );
                for (n, (ok, text)) in rows.iter().enumerate() {
                    let r = ui.add(
                        egui::Label::new(format!(
                            "{} · {text}",
                            if *ok { "Pass" } else { "Needs work" }
                        ))
                        .wrap(),
                    );
                    m.rows.push((n, *ok, r.rect));
                }
                m.complete = !practice && passed;
            } else if p.memo.job.is_some() {
                ui.label("Checking the current files and independent recovery samples…");
            }
        }
        if practice {
            ui.add(
                egui::Label::new(format!("Try a change: {}", lesson.try_change.description)).wrap(),
            );
        }
        controls::collapsing_header(ui, "Help and command reference").show(ui, |ui| {
            ui.add(egui::Label::new(dlms_learning_lab::file_backup_terminal::HELP).wrap());
            if !practice {
                if p.hint > 0 && !s.hints.is_empty() {
                    ui.add(egui::Label::new(&s.hints[(p.hint - 1).min(s.hints.len() - 1)]).wrap());
                }
                if action(
                    ui,
                    m,
                    "hint",
                    if p.hint == 0 {
                        "Show a hint"
                    } else {
                        "Next hint"
                    },
                    false,
                ) {
                    p.hint = (p.hint + 1).min(s.hints.len());
                }
                controls::collapsing_header(ui, "Worked explanation · reveal when ready").show(
                    ui,
                    |ui| {
                        ui.add(egui::Label::new(&s.explanation).wrap());
                    },
                );
            }
        });
        controls::collapsing_header(ui, "References and simulation limits").show(ui, |ui| {
            ui.add(egui::Label::new(&s.simplifications).wrap());
            for line in &s.sources {
                ui.add(egui::Label::new(line).wrap());
            }
        });
    }
    if let Some(view) = requested {
        match present(
            a,
            s,
            false,
            Edit {
                view: Some(view),
                ..Default::default()
            },
        ) {
            Ok(()) => {
                dirty = true;
                p.notice = None;
                p.memo.notice = None;
            }
            Err(e) => p.notice = Some(e),
        }
    }
    if dirty
        && p.memo.ready(a, s).is_none()
        && !p
            .memo
            .job
            .as_ref()
            .is_some_and(|j| same_source(&j.source, s) && same_attempt(&j.attempt, a))
    {
        // A genuine user edit may resume fresh checking. Idle frames and Stop
        // alone never do; no old grade is attached to a changed full key.
        if let Err(e) = p.memo.prepare(a, s, true) {
            p.notice = Some(e);
        }
    }
    if current == teaching::View::Introduction
        && let Some(notice) = p.notice.as_ref().or(p.memo.notice.as_ref())
    {
        ui.add(egui::Label::new(notice).wrap());
    }
    dirty
}

fn page(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut BackupMetrics,
    active: bool,
    focus: Option<egui::Id>,
) -> bool {
    ui.add_enabled_ui(active, |ui| page_inner(ui, s, a, p, m, active, focus))
        .inner
}
impl Presentation {
    /// Borrowed observation of an actual explicit cancellation; no grade authority.
    pub(in crate::ui) fn explicitly_stopped(&self, id: &str) -> bool {
        self.memo.cancelled.contains(id) && self.memo.job.as_ref().is_none_or(|j| j.source.id != id)
    }
    pub(in crate::ui) fn checking(&self) -> bool {
        self.memo.job.is_some()
    }
    /// Called by Reset, restore/import and any external same-text replacement.
    /// The field ID survives until bind can remove native state with Context.
    fn external_replacement(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.notice = None;
        self.memo.notice = None;
        self.focus = false;
        self.hint = 0;
        self.memo.clear();
    }
    pub(super) fn clear(&mut self) {
        // Navigation clears transient editor state, not an explicit Stop intent.
        // Complete-key cache entries remain bounded and cannot authorize edits.
        self.generation = self.generation.wrapping_add(1);
        self.notice = None;
        self.memo.notice = None;
        self.focus = false;
        self.hint = 0;
        self.owner = None;
        self.document.owner = None;
        self.document.value.clear();
    }
    pub(super) fn reset_all(&mut self) {
        self.external_replacement();
        self.clear();
    }
    #[cfg(test)]
    pub(in crate::ui) fn observed(&self) -> (usize, usize, usize, bool, u64, u64) {
        (
            self.memo.counters.admissions,
            self.memo.counters.copies,
            self.memo.counters.ticks,
            self.memo.job.is_some(),
            self.generation,
            self.bound_generation,
        )
    }
    #[cfg(test)]
    pub(in crate::ui) fn document_value(&self) -> &str {
        &self.document.value
    }
    #[cfg(test)]
    pub(in crate::ui) fn last_notice(&self) -> Option<&str> {
        self.notice.as_deref().or(self.memo.notice.as_deref())
    }
}

// Check results are ephemeral; no memo boolean becomes saved progress.
pub(super) fn solved_with_memos(app: &LabApp, a: &Attempt, s: &Scenario) -> bool {
    if matches!(s.initial, engine::State::FileBackup { .. }) {
        app.backup_presentation.memo.passed(a, s).unwrap_or(false)
    } else {
        cron_workspace::solved_with_memos(
            &app.cron_presentation.memo,
            &app.ether_presentation.memo,
            &app.nfs_presentation.memo,
            &app.rpm_presentation.memo,
            &app.nat_presentation.memo,
            a,
            s,
        )
    }
}
impl LabApp {
    pub(super) fn prepare_backup(a: &mut Attempt, s: &Scenario, new: bool) {
        if matches!(s.initial, engine::State::FileBackup { .. })
            && let Some(lesson) = &s.file_backup_teaching
        {
            let _ = dlms_learning_lab::file_backup_teaching::prepare(a, lesson, new);
        }
    }
    pub(super) fn advance_backup(&mut self, ctx: &egui::Context) {
        // egui may discard a layout pass and render it again in the same frame.
        // Advance the bounded checker once per frame, not once per layout pass.
        let frame = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        if self.backup_presentation.advanced_frame == Some(frame) {
            return;
        }
        self.backup_presentation.advanced_frame = Some(frame);
        if self
            .backup_presentation
            .memo
            .advance_progress(&self.progress, &self.scenarios)
        {
            ctx.request_repaint();
        }
    }
    pub(super) fn backup_prepass(&mut self, ctx: &egui::Context) {
        // Restore/reset may remove complete attempts; retain no orphan authority.
        self.backup_presentation.memo.entries.retain(|entry| {
            self.progress.attempts.contains_key(&entry.source.id)
                && self.scenarios.iter().any(|s| s.id == entry.source.id)
        });
        self.backup_presentation.memo.cancelled.retain(|id| {
            self.progress.attempts.contains_key(id) && self.scenarios.iter().any(|s| &s.id == id)
        });

        let selected = self
            .selected
            .and_then(|i| self.scenarios.get(i))
            .map(|s| s.id.as_str());
        for selected_pass in [true, false] {
            for s in &self.scenarios {
                if matches!(s.initial, engine::State::FileBackup { .. })
                    && (selected == Some(s.id.as_str())) == selected_pass
                    && let Some(a) = self.progress.attempts.get(&s.id)
                    && let Err(e) = self.backup_presentation.memo.prepare(a, s, false)
                    && selected == Some(s.id.as_str())
                {
                    self.backup_presentation.notice = Some(e);
                }
            }
        }
        if self.backup_presentation.checking() {
            ctx.request_repaint();
        }
    }
    pub(super) fn backup_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let focus = ui.memory(|m| m.focused());
        let pending = self.backup_presentation.document.field == focus
            && focus.is_some()
            && ui.input(|input| {
                input.events.iter().any(|e| match e {
                    egui::Event::Text(t) => !t.is_empty() && t != "\n" && t != "\r",
                    egui::Event::Paste(t) => !t.is_empty(),
                    _ => false,
                })
            });
        ui.add_enabled_ui(!pending, |ui| self.workspace_header(ui, i));
        let active = self.selected == Some(i)
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let before = view(a);
        self.dirty |= card(ui)
            .show(ui, |ui| {
                page(
                    ui,
                    s,
                    a,
                    &mut self.backup_presentation,
                    &mut self.metrics.backup,
                    active,
                    focus,
                )
            })
            .inner;
        if before != view(a) {
            self.lesson_scroll_reset = true;
        }
        self.metrics.completion = self.metrics.backup.complete;
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}

#[cfg(test)]
mod output_tests {
    use super::*;
    #[test]
    fn bounded_utf8_chunks_preserve_every_byte_including_empty_and_long_lines() {
        for text in [
            "".to_owned(),
            "abc".into(),
            "ü".repeat(5000),
            "字".repeat(4096),
        ] {
            let chunks = LineChunks {
                remaining: &text,
                first: true,
            }
            .collect::<Vec<_>>();
            assert_eq!(chunks.concat(), text);
            assert!(chunks.iter().all(|s| s.len() <= 4096));
            assert!(!chunks.is_empty());
        }
    }
    #[test]
    fn output_line_virtualization_retains_newlines_and_only_borrows_source() {
        let text = vec!["first\n\nlast\n".into(), "second".into()];
        assert_eq!(
            output_lines(&text).collect::<Vec<_>>(),
            ["first", "", "last", "", "second"]
        );
        let many = vec!["\n".repeat(65536)];
        assert_eq!(output_lines(&many).count(), 65537);
        assert_eq!(output_lines(&many).skip(32000).take(8).count(), 8);
    }
}

#[cfg(test)]
mod stopped_admission_tests {
    use super::*;
    fn fixture(n: usize) -> (Scenario, Attempt) {
        let s = dlms_learning_lab::pack::bundled()
            .unwrap()
            .into_iter()
            .filter(|s| matches!(s.initial, State::FileBackup { .. }))
            .nth(n)
            .unwrap();
        let mut a = Attempt::new(&s, 1);
        dlms_learning_lab::workspace::prepare(&mut a);
        dlms_learning_lab::file_backup_teaching::prepare(
            &mut a,
            s.file_backup_teaching.as_ref().unwrap(),
            true,
        )
        .unwrap();
        (s, a)
    }
    fn cancelled(a: &Attempt, s: &Scenario) -> CheckMemo {
        let mut m = CheckMemo::default();
        m.prepare(a, s, true).unwrap();
        assert!(m.job.is_some());
        m.cancel();
        assert!(m.cancelled.contains(&s.id));
        m
    }
    #[test]
    fn stopped_recency_refresh_retains_full_key_without_grading_or_idle_copies() {
        let (s, mut a) = fixture(3);
        let mut m = cancelled(&a, &s);
        let before = m.counters;
        a.visited += 1;
        let raw = serde_json::to_vec(&a).unwrap();
        assert!(m.ready(&a, &s).is_none());
        m.prepare(&a, &s, false).unwrap();
        assert!(m.ready(&a, &s).is_some());
        assert!(m.cancelled.contains(&s.id));
        assert!(m.job.is_none());
        assert!(m.rows(&a, &s, false).is_none());
        assert!(m.passed(&a, &s).is_none());
        assert_eq!(m.counters.copies, before.copies + 1);
        assert_eq!(m.counters.ticks, before.ticks);
        let warm = m.counters;
        for _ in 0..24 {
            m.prepare(&a, &s, false).unwrap();
            assert!(!m.advance(&a, &s));
        }
        assert_eq!(m.counters.copies, warm.copies);
        assert_eq!(m.counters.ticks, warm.ticks);
        assert_eq!(serde_json::to_vec(&a).unwrap(), raw);
    }
    #[test]
    fn stopped_miss_still_validates_context_and_bounds_before_any_owned_copy() {
        let (s, a) = fixture(3);
        let mut m = cancelled(&a, &s);
        let counts = m.counters;
        let raw = serde_json::to_vec(&a).unwrap();
        for bad in 0..3 {
            let mut other = a.clone();
            let mut source = s.clone();
            other.visited += 1;
            match bad {
                0 => other.scenario_version += 1,
                1 => other.workspace.as_mut().unwrap().command = "x".repeat(4097),
                2 => source.steps.clear(),
                _ => unreachable!(),
            }
            let input = serde_json::to_vec(&other).unwrap();
            assert!(m.prepare(&other, &source, false).is_err());
            assert_eq!(m.counters.copies, counts.copies);
            assert_eq!(m.counters.ticks, counts.ticks);
            assert!(m.cancelled.contains(&s.id));
            assert!(m.job.is_none());
            assert_eq!(serde_json::to_vec(&other).unwrap(), input);
        }
        assert_eq!(serde_json::to_vec(&a).unwrap(), raw);
        assert!(m.ready(&a, &s).is_some());
    }
    #[test]
    fn stopped_refresh_keeps_another_job_and_all_existing_memory_allowances() {
        let (s, mut a) = fixture(3);
        let mut m = cancelled(&a, &s);
        let (other, active) = fixture(0);
        m.prepare(&active, &other, true).unwrap();
        let ticks = m.counters.ticks;
        let input = serde_json::to_vec(&a).unwrap();
        a.visited += 1;
        m.prepare(&a, &s, false).unwrap();
        assert_eq!(m.job.as_ref().unwrap().source.id, other.id);
        assert!(m.ready(&a, &s).unwrap().cancelled);
        assert!(m.cancelled.contains(&s.id));
        assert_eq!(m.counters.ticks, ticks);
        assert!(m.entries.len() <= 4);
        assert!(m.entries.iter().map(|e| e.charge).sum::<usize>() <= TOTAL_BYTES);
        assert!(
            m.entries.iter().map(|e| e.charge).sum::<usize>() + m.job.as_ref().unwrap().run_charge
                <= ALL_BYTES
        );
        for _ in 0..1000 {
            if !m.advance(&active, &other) {
                break;
            }
        }
        assert!(m.job.is_none());
        assert!(m.cancelled.contains(&s.id));
        assert!(m.passed(&a, &s).is_none());
        m.prepare(&a, &s, true).unwrap();
        assert!(!m.cancelled.contains(&s.id));
        assert_eq!(m.job.as_ref().unwrap().source.id, s.id);
        a.visited -= 1;
        assert_eq!(serde_json::to_vec(&a).unwrap(), input);
    }
}
