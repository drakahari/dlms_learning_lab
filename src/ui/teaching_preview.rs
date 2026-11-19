//! Fixed rollout screenshots on private smoke state. No desktop/profile changes.
use super::*;
#[derive(Clone, Default)]
struct Capture {
    index: usize,
    phase: usize,
    reveal_after: f64,
    reveal_deadline: f64,
}
fn enlarged_view_slots(all_views: bool) -> &'static [usize] {
    if all_views { &[0, 1, 2] } else { &[1, 2] }
}
impl LabApp {
    fn teaching_preview_view(&mut self, i: usize, view: teaching::View) {
        let s = self.scenarios[i].clone();
        match &s.initial {
            engine::State::PartitionFs { .. } => {
                dlms_learning_lab::partition_fs_workspace::binding::set_view(
                    self.progress.attempts.get_mut(&s.id).unwrap(),
                    &s,
                    view,
                )
                .expect("fixed partition review context");
            }
            engine::State::Lvm { .. } => {
                let engine::Goal::Lvm { definition } = &s.goal else {
                    unreachable!()
                };
                let engine::State::Lvm { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation
                    .set_view(definition, view)
                    .expect("fixed LVM review context");
            }
            engine::State::PersistentMounts { .. } => self.set_storage_view(i, view),
            engine::State::KernelModules { .. } => {
                let engine::State::KernelModules { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::RecursiveTree { .. } => {
                let engine::State::RecursiveTree { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Transport { .. } => {
                let engine::State::Transport { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::MagicSubnet { .. } => {
                let engine::State::MagicSubnet { session } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                session.view = view;
            }
            engine::State::DebianPackages { .. } => self.set_debian_packages_view(i, view),
            engine::State::Raid1 { .. } => {
                let engine::State::Raid1 { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::StartupFiles { .. } => {
                let engine::State::StartupFiles { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::FsIntegrity { .. } => {
                let engine::State::FsIntegrity { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::UserMounts { .. } => {
                let engine::State::UserMounts { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Environment { .. } => {
                let engine::State::Environment { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::DebianSources { .. } => {
                let engine::State::DebianSources { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::ClientNames { .. } => {
                let engine::State::ClientNames { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::RemovableMounts { .. } => {
                let engine::State::RemovableMounts { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::NmProfiles { .. } => {
                let engine::State::NmProfiles { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::PamStack { .. } => {
                let engine::State::PamStack { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Aging { .. } => {
                let engine::State::Aging { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::PortSecurity { .. } => {
                let engine::State::PortSecurity { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::DhcpSnooping { .. } => {
                let engine::State::DhcpSnooping { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::ArpInspection { .. } => {
                let engine::State::ArpInspection { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::DhcpConfig { .. } => {
                let engine::State::DhcpConfig { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Qos { .. } => {
                let engine::State::Qos { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::FileBackup { .. } => {
                dlms_learning_lab::file_backup_workspace::presentation::present(
                    self.progress.attempts.get_mut(&s.id).unwrap(),
                    &s,
                    false,
                    dlms_learning_lab::file_backup_workspace::presentation::Edit {
                        view: Some(view),
                        ..Default::default()
                    },
                )
                .expect("fixed backup review context");
            }
            engine::State::UnitManager { .. } => self.set_unit_manager_view(i, view),
            engine::State::Switching { .. } => self.set_switching_view(i, view),
            engine::State::Ospf { .. } => self.set_ospf_view(i, view),
            engine::State::Processes { .. } => self.set_process_view(i, view),
            engine::State::Filesystem { .. } => self.set_terminal_view(i, view),
            engine::State::Accounts { .. } => self.set_accounts_view(i, view),
            engine::State::Service { .. } => self.set_service_view(i, view),
            engine::State::Script { .. } => self.set_script_view(i, view),
            engine::State::Hsrp { .. } => {
                dlms_learning_lab::hsrp_workspace::binding::change(
                    self.progress.attempts.get_mut(&s.id).unwrap(),
                    &s,
                    dlms_learning_lab::hsrp_workspace::binding::Change::View(view),
                )
                .expect("fixed gateway review context");
            }
            engine::State::LinuxLogging { .. } => {
                let engine::State::LinuxLogging { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::HostNetwork { .. } => {
                let engine::State::HostNetwork { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Repositories { .. } => {
                let engine::State::Repositories { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Capacity { .. } => {
                let engine::State::Capacity { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Syslog { .. } => {
                let engine::State::Syslog { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Links { .. } => {
                let engine::State::Links { simulation } =
                    &mut self.progress.attempts.get_mut(&s.id).unwrap().state
                else {
                    unreachable!()
                };
                simulation.view = view;
            }
            engine::State::Nfs { .. } => {
                dlms_learning_lab::nfs_workspace::set_practice_presentation(
                    self.progress.attempts.get_mut(&s.id).unwrap(),
                    &s,
                    Some(view),
                    None,
                )
                .expect("fixed native review context");
            }
            _ => panic!("unexpected fixed review family"),
        }
    }
    pub(super) fn preview_view_ready(&self, i: usize, view: teaching::View) -> bool {
        if self.scenarios[i]
            .accounts_teaching
            .as_ref()
            .is_some_and(|l| l.template_teaching.is_some())
        {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && self.accounts_view(i) == view
        } else if matches!(
            self.scenarios[i].initial,
            engine::State::PersistentMounts { .. }
        ) {
            !self.progress.attempts[&self.scenarios[i].id].challenge && self.storage_view(i) == view
        } else if matches!(self.scenarios[i].initial, engine::State::PartitionFs { .. }) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::PartitionFs{simulation}if simulation.view==view)
        } else if matches!(self.scenarios[i].initial, engine::State::Lvm { .. }) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::Lvm{simulation}if simulation.view==view)
        } else if matches!(self.scenarios[i].initial, engine::State::PamStack { .. }) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::PamStack{simulation}if simulation.view==view)
        } else if matches!(self.scenarios[i].initial, engine::State::Aging { .. }) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::Aging{simulation}if simulation.view==view)
        } else if matches!(
            self.scenarios[i].initial,
            engine::State::PortSecurity { .. }
        ) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::PortSecurity{simulation}if simulation.view==view)
        } else if matches!(
            self.scenarios[i].initial,
            engine::State::DhcpSnooping { .. }
        ) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::DhcpSnooping{simulation}if simulation.view==view)
        } else if matches!(
            self.scenarios[i].initial,
            engine::State::ArpInspection { .. }
        ) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::ArpInspection{simulation}if simulation.view==view)
        } else if matches!(self.scenarios[i].initial, engine::State::Processes { .. }) {
            !self.progress.attempts[&self.scenarios[i].id].challenge && self.process_view(i) == view
        } else if matches!(self.scenarios[i].initial, engine::State::Transport { .. }) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state, engine::State::Transport { simulation } if simulation.view==view)
        } else if matches!(self.scenarios[i].initial, engine::State::Environment { .. }) {
            !self.progress.attempts[&self.scenarios[i].id].challenge
                && matches!(&self.progress.attempts[&self.scenarios[i].id].state, engine::State::Environment { simulation } if simulation.view == view)
        } else {
            true
        }
    }
    pub(super) fn preview_action_rect(&self) -> Option<egui::Rect> {
        if let Some(i) = self.selected
            && let engine::State::Aging { simulation } =
                &self.progress.attempts[&self.scenarios[i].id].state
        {
            let key = if !self.progress.attempts[&self.scenarios[i].id].challenge
                && simulation.view == teaching::View::Introduction
            {
                "try"
            } else {
                "check"
            };
            return self
                .metrics
                .aging
                .controls
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1);
        }
        if let Some(i) = self.selected
            && let engine::State::PortSecurity { simulation } =
                &self.progress.attempts[&self.scenarios[i].id].state
        {
            let key = if !self.progress.attempts[&self.scenarios[i].id].challenge
                && simulation.view == teaching::View::Introduction
            {
                "try"
            } else {
                "check"
            };
            return self
                .metrics
                .port_security
                .controls
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1);
        }
        if let Some(i) = self.selected
            && let engine::State::DhcpSnooping { simulation } =
                &self.progress.attempts[&self.scenarios[i].id].state
        {
            let key = if !self.progress.attempts[&self.scenarios[i].id].challenge
                && simulation.view == teaching::View::Introduction
            {
                "try"
            } else {
                "check"
            };
            return self
                .metrics
                .dhcp_snooping
                .controls
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1);
        }
        if let Some(i) = self.selected
            && let engine::State::ArpInspection { simulation } =
                &self.progress.attempts[&self.scenarios[i].id].state
        {
            let key = if !self.progress.attempts[&self.scenarios[i].id].challenge
                && simulation.view == teaching::View::Introduction
            {
                "try"
            } else {
                "check"
            };
            return self
                .metrics
                .arp_inspection
                .controls
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1);
        }
        if let Some(i) = self.selected
            && let engine::State::PartitionFs { simulation } =
                &self.progress.attempts[&self.scenarios[i].id].state
        {
            let key = if !self.progress.attempts[&self.scenarios[i].id].challenge
                && simulation.view == teaching::View::Introduction
            {
                "practice"
            } else {
                "check"
            };
            return self
                .metrics
                .partition_fs
                .controls
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1);
        }
        if let Some(i) = self.selected
            && let engine::State::Lvm { simulation } =
                &self.progress.attempts[&self.scenarios[i].id].state
        {
            let key = if !self.progress.attempts[&self.scenarios[i].id].challenge
                && simulation.view == teaching::View::Introduction
            {
                if self
                    .metrics
                    .lvm
                    .controls
                    .iter()
                    .any(|r| r.0 == "practice-start")
                {
                    "practice-start"
                } else {
                    "example"
                }
            } else {
                "check"
            };
            return self
                .metrics
                .lvm
                .controls
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1);
        }
        if let Some(i) = self.selected
            && self.scenarios[i]
                .accounts_teaching
                .as_ref()
                .is_some_and(|l| l.template_teaching.is_some())
        {
            return match self.accounts_view(i) {
                teaching::View::Introduction => self.metrics.example_action,
                teaching::View::Example => self.metrics.accounts_run,
                teaching::View::Assignment => self.metrics.check,
            };
        }
        if let Some(i) = self.selected
            && matches!(self.scenarios[i].initial, engine::State::Processes { .. })
        {
            return match self.process_view(i) {
                teaching::View::Introduction => self.metrics.example_action,
                teaching::View::Example => self.metrics.terminal_run,
                teaching::View::Assignment => self.metrics.check,
            };
        }
        if let Some(i) = self.selected
            && let engine::State::HostNetwork { simulation } =
                &self.progress.attempts[&self.scenarios[i].id].state
        {
            let introduction = !self.progress.attempts[&self.scenarios[i].id].challenge
                && simulation.view == teaching::View::Introduction;
            let key = if introduction { "try" } else { "check" };
            return self
                .metrics
                .host
                .controls
                .iter()
                .find(|c| c.0 == key)
                .map(|c| c.1);
        }
        let introduction=self.selected.is_some_and(|i| match self.scenarios[i].initial {
            engine::State::KernelModules{..} => matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::KernelModules{simulation} if !self.progress.attempts[&self.scenarios[i].id].challenge && simulation.view==teaching::View::Introduction),
            engine::State::RecursiveTree{..} => matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::RecursiveTree{simulation} if !self.progress.attempts[&self.scenarios[i].id].challenge && simulation.view==teaching::View::Introduction),
            engine::State::Transport{..} => matches!(&self.progress.attempts[&self.scenarios[i].id].state,engine::State::Transport{simulation} if !self.progress.attempts[&self.scenarios[i].id].challenge && simulation.view==teaching::View::Introduction),
            engine::State::PersistentMounts{..} => self.storage_view(i)==teaching::View::Introduction,
            engine::State::Script{..} => self.script_view(i)==teaching::View::Introduction,
            engine::State::Environment{..} => matches!(&self.progress.attempts[&self.scenarios[i].id].state, engine::State::Environment { simulation } if !self.progress.attempts[&self.scenarios[i].id].challenge && simulation.view==teaching::View::Introduction),
            engine::State::Processes{..} => self.process_view(i)==teaching::View::Introduction,
            _ => false,
        });
        if introduction {
            self.metrics.example_action
        } else {
            self.metrics.check
        }
    }
    fn reveal_preview_check(&mut self, ctx: &egui::Context) -> bool {
        let rect = self
            .preview_action_rect()
            .expect("The preview has a relevant lesson action");
        self.reveal_smoke_control(ctx, rect)
    }
    fn preview_check_ready(&mut self, ctx: &egui::Context, c: &mut Capture) -> bool {
        let now = ctx.input(|i| i.time);
        if c.reveal_deadline == 0. {
            c.reveal_deadline = now + 5.;
        }
        let visible = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let action = self
            .preview_action_rect()
            .expect("The rendered lesson exposes its primary action");
        let reached = smoke::smoke_control_visible(action, visible, ctx.pixels_per_point());
        if reached && now >= c.reveal_after {
            return true;
        }
        assert!(
            now < c.reveal_deadline,
            "Controls screenshot could not reach lesson action: {action:?}; viewport {visible:?}"
        );
        if now >= c.reveal_after {
            self.reveal_preview_check(ctx);
            c.reveal_after = now + 0.2;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
        false
    }
    pub(super) fn teaching_preview_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("teaching-rollout-capture");
        let mut c = ctx.data_mut(|d| d.get_temp::<Capture>(key).unwrap_or_default());
        let debian_sources =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("debian-sources-batch");
        let debian_contents =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("debian-contents-batch");
        let resolver_search =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("resolver-search-batch");
        let client_names = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("client-names-batch");
        let optional_mount =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("optional-mount-batch");
        let startup_files =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("startup-files-batch");
        let fs_integrity = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("fs-integrity-batch");
        let nm_bridge = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-bridge-batch");
        let aging = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("aging-batch");
        let port_security =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("port-security-batch");
        let dhcp_snooping =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dhcp-snooping-batch");
        let arp_inspection =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("arp-inspection-batch");
        let pam_auth = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("pam-auth-batch");
        let nm_bond = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-bond-batch");
        let nm_vlan = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-vlan-batch");
        let nm_dhcp = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-dhcp-batch");
        let nm_profiles = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("nm-profiles-batch");
        let dhcp_config = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dhcp-config-batch");
        let backup_selection =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("backup-selection-batch");
        let qos = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("qos-batch");
        let startup = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("startup-batch");
        let linux_logging =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("linux-logging-batch");
        let debian_conflicts =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("debian-conflicts-batch");
        let user_mounts = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("user-mounts-batch");
        let environment = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("environment-batch");
        let conffiles = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("conffile-batch");
        let raid1 = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("raid1-batch");
        let file_lifecycle =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("file-lifecycle-batch");
        let skeleton = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("account-skeleton-batch");
        let lookup = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("path-lookup-batch");
        let listener = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("listener-batch");
        let triage = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("log-triage-batch");
        let mount_labs = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-mount-batch");
        let config_scan = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-config-batch");
        let references = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-reference-batch");
        let cow = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-cow-batch");
        let mbr = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("mbr-layout-batch");
        let physical = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-pv-capacity-batch");
        let renaming = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-rename-batch");
        let availability =
            std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-activation-batch");
        let retirement = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-retirement-batch");
        let raw_lvm = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("lvm-provisioning-batch");
        let swap_policy = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("swap-policy-batch");
        let host6 = matches!(
            std::env::var("DLMS_SMOKE_STAGE").as_deref(),
            Ok("host-ipv6-batch" | "host-ipv6-previews")
        );
        let managed = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("managed-config-batch");
        let transport = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("transport-batch");
        let backups = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("content-backup-batch");
        let maps = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("associative-batch");
        let tracing = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("tracing-batch");
        let debug = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("debug-recovery-batch");
        let host = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("host-batch");
        let repository = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("repository-batch");
        let capacity = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("capacity-batch");
        let gateway = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("gateway-batch");
        let links = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("links-batch");
        let syslog = std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("syslog-batch")
            || std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("syslog-previews");
        let normal: &[&str] = if arp_inspection {
            &smoke::arp_inspection::IDS
        } else if dhcp_snooping {
            &smoke::dhcp_snooping::IDS
        } else if port_security {
            &smoke::port_security::IDS
        } else if aging {
            &smoke::aging::IDS
        } else if pam_auth {
            &["lpic2-pam-auth-bypass"]
        } else if nm_bridge {
            &["lpic2-bridge-lan-commission"]
        } else if nm_bond {
            &["lpic2-bonded-vlan-handover"]
        } else if nm_vlan {
            &[
                "lpic2-vlan-profile-commission",
                "lpic2-vlan-return-handover",
            ]
        } else if mount_labs {
            &smoke::lvm_provisioning::mounts::IDS
        } else if config_scan {
            &smoke::lvm_provisioning::config_scan::IDS
        } else if references {
            &smoke::lvm_provisioning::references::IDS
        } else if cow {
            &smoke::lvm_provisioning::cow::IDS
        } else if mbr {
            &smoke::mbr_layouts::IDS
        } else if physical {
            &smoke::lvm_provisioning::PV_IDS
        } else if renaming {
            &smoke::lvm_provisioning::RENAME_IDS
        } else if availability {
            &smoke::lvm_provisioning::ACTIVE_IDS
        } else if retirement {
            &smoke::lvm_provisioning::RETIRE_IDS
        } else if raw_lvm {
            &smoke::lvm_provisioning::IDS
        } else if swap_policy {
            &[
                "lpic1-swap-prepared-activation",
                "lpic1-swap-replacement-handover",
            ]
        } else if host6 {
            &[
                "lpic2-host-ipv6-addition",
                "lpic2-host-ipv6-gateway-scope",
                "lpic2-host-dual-stack-handover",
            ]
        } else if triage {
            &["shell-log-owner-triage"]
        } else if transport {
            &[
                "ccna-transport-port-pairing",
                "ccna-transport-return-policy",
            ]
        } else if backups {
            &["shell-backup-content-set", "shell-restore-validated-set"]
        } else if skeleton {
            &["lpic1-skeleton-handover"]
        } else if lookup {
            &["lpic1-env-tool-path"]
        } else if listener {
            &["lpic1-listener-audit"]
        } else if managed {
            &["shell-managed-config"]
        } else if maps {
            &["shell-map-lookup", "shell-map-reconcile"]
        } else if raid1 {
            &["lpic2-raid1-replace", "lpic2-raid1-handover"]
        } else if conffiles {
            &[
                "lpic1-conffile-retain",
                "lpic1-conffile-adopt",
                "lpic1-conffile-handover",
            ]
        } else if file_lifecycle {
            &[
                "shell-stage-report",
                "shell-cleanup-preview",
                "shell-recover-publication",
            ]
        } else if startup_files {
            &["lpic1-shell-startup-priority", "lpic1-shell-startup-quiet"]
        } else if fs_integrity {
            &["lpic1-fs-offline-accounting", "lpic1-fs-retained-recovery"]
        } else if nm_dhcp {
            &["lpic1-nm-dhcp-startup", "lpic1-nm-dhcp-local-only"]
        } else if debian_conflicts {
            &[
                "lpic1-package-conflict-retirement",
                "lpic1-package-conflict-upgrade",
            ]
        } else if user_mounts {
            &[
                "lpic1-removable-personal-session",
                "lpic1-removable-shared-handover",
            ]
        } else if environment {
            &["lpic1-env-child-settings", "lpic1-env-parent-note"]
        } else if tracing {
            &["shell-trace-delivery", "shell-trace-fallback"]
        } else if debian_sources {
            &[
                "lpic1-package-source-selection",
                "lpic1-package-source-recovery",
            ]
        } else if debian_contents {
            &["lpic1-package-file-provider"]
        } else if resolver_search {
            &["lpic1-dns-search-suffixes", "lpic1-dns-absolute-precedence"]
        } else if client_names {
            &["lpic1-local-host-alias", "lpic1-name-source-fallback"]
        } else if optional_mount {
            &["lpic1-removable-startup-policy"]
        } else if nm_profiles {
            &[
                "lpic1-nm-persistent-profile",
                "lpic1-nm-autoconnect-choice",
                "lpic1-nm-uplink-handover",
            ]
        } else if dhcp_config {
            &["lpic2-dhcp-dynamic-ranges", "lpic2-dhcp-option-scope"]
        } else if backup_selection {
            &["lpic2-backup-directory-selection"]
        } else if qos {
            &[
                "ccna-qos-mark-boundary",
                "ccna-qos-queue-bursts",
                "ccna-qos-trusted-handover",
            ]
        } else if startup {
            &[
                "lpic1-startup-current-future",
                "lpic1-startup-default-target",
                "lpic1-startup-install-links",
            ]
        } else if linux_logging {
            &[
                "lpic1-syslog-threshold",
                "lpic1-syslog-exact",
                "lpic1-syslog-privacy",
                "lpic1-syslog-handover",
            ]
        } else if debug {
            &["shell-debug-status", "shell-recovery-ledger"]
        } else if host {
            &[
                "lpic1-host-address",
                "lpic1-host-routing",
                "lpic1-host-resolver",
                "lpic1-host-handover",
            ]
        } else if repository {
            &[
                "lpic1-repo-visibility",
                "lpic1-repo-provenance",
                "lpic1-repo-offline-evidence",
                "lpic1-repo-handover",
            ]
        } else if capacity {
            &[
                "lpic2-capacity-inodes",
                "lpic2-capacity-blocks",
                "lpic2-capacity-snapshot",
                "lpic2-capacity-handover",
            ]
        } else if syslog {
            &[
                "ccna-syslog-local-remote",
                "ccna-syslog-export-path",
                "ccna-syslog-facility-routing",
                "ccna-syslog-policy-handover",
                "lpic1-links-identity",
            ]
        } else if links {
            &[
                "lpic1-links-identity",
                "lpic1-links-relative",
                "lpic1-links-move",
                "lpic1-links-handover",
            ]
        } else if gateway {
            &[
                "ccna-hsrp-virtual-gateway",
                "ccna-hsrp-incumbent-return",
                "ccna-hsrp-wan-tracking",
                "ccna-hsrp-resilient-handover",
            ]
        } else {
            &[
                "ccna-magic-blocks",
                "ccna-vlan-trunk",
                "ccna-ospf-directed-ecmp",
                "lpic1-file-navigation",
                "lpic1-account-onboarding",
                "lpic2-service",
                "lpic2-nfs-reference",
                "shell-functions",
            ]
        };
        let enlarged: &[&str] = if arp_inspection {
            &smoke::arp_inspection::IDS
        } else if dhcp_snooping {
            &smoke::dhcp_snooping::IDS
        } else if port_security {
            &["ccna-security-identity", "ccna-security-recover"]
        } else if aging {
            &["lpic1-aging-rotation", "lpic1-aging-contract-reset"]
        } else if pam_auth {
            &["lpic2-pam-auth-bypass"]
        } else if nm_bridge {
            &["lpic2-bridge-lan-commission"]
        } else if nm_bond {
            &["lpic2-bonded-vlan-handover"]
        } else if nm_vlan {
            &[
                "lpic2-vlan-profile-commission",
                "lpic2-vlan-return-handover",
            ]
        } else if mount_labs {
            &smoke::lvm_provisioning::mounts::IDS
        } else if config_scan {
            &smoke::lvm_provisioning::config_scan::IDS
        } else if references {
            &smoke::lvm_provisioning::references::IDS
        } else if cow {
            &smoke::lvm_provisioning::cow::IDS
        } else if mbr {
            &smoke::mbr_layouts::IDS
        } else if physical {
            &smoke::lvm_provisioning::PV_IDS
        } else if renaming {
            &smoke::lvm_provisioning::RENAME_IDS
        } else if availability {
            &smoke::lvm_provisioning::ACTIVE_IDS
        } else if retirement {
            &smoke::lvm_provisioning::RETIRE_IDS
        } else if raw_lvm {
            &smoke::lvm_provisioning::IDS
        } else if swap_policy {
            &[
                "lpic1-swap-prepared-activation",
                "lpic1-swap-replacement-handover",
            ]
        } else if host6 {
            &[
                "lpic2-host-ipv6-addition",
                "lpic2-host-ipv6-gateway-scope",
                "lpic2-host-dual-stack-handover",
            ]
        } else if triage {
            &["shell-log-owner-triage"]
        } else if transport {
            &[
                "ccna-transport-port-pairing",
                "ccna-transport-return-policy",
            ]
        } else if backups {
            &["shell-backup-content-set", "shell-restore-validated-set"]
        } else if skeleton {
            &["lpic1-skeleton-handover"]
        } else if lookup {
            &["lpic1-env-tool-path"]
        } else if listener {
            &["lpic1-listener-audit"]
        } else if managed {
            &["shell-managed-config"]
        } else if maps {
            &["shell-map-lookup", "shell-map-reconcile"]
        } else if raid1 {
            &["lpic2-raid1-replace", "lpic2-raid1-handover"]
        } else if conffiles {
            &[
                "lpic1-conffile-retain",
                "lpic1-conffile-adopt",
                "lpic1-conffile-handover",
            ]
        } else if file_lifecycle {
            &[
                "shell-stage-report",
                "shell-cleanup-preview",
                "shell-recover-publication",
            ]
        } else if startup_files {
            &["lpic1-shell-startup-priority", "lpic1-shell-startup-quiet"]
        } else if fs_integrity {
            &["lpic1-fs-offline-accounting", "lpic1-fs-retained-recovery"]
        } else if nm_dhcp {
            &["lpic1-nm-dhcp-startup", "lpic1-nm-dhcp-local-only"]
        } else if debian_conflicts {
            &[
                "lpic1-package-conflict-retirement",
                "lpic1-package-conflict-upgrade",
            ]
        } else if user_mounts {
            &[
                "lpic1-removable-personal-session",
                "lpic1-removable-shared-handover",
            ]
        } else if environment {
            &["lpic1-env-child-settings", "lpic1-env-parent-note"]
        } else if tracing {
            &["shell-trace-delivery", "shell-trace-fallback"]
        } else if debian_sources {
            &[
                "lpic1-package-source-selection",
                "lpic1-package-source-recovery",
            ]
        } else if debian_contents {
            &["lpic1-package-file-provider"]
        } else if resolver_search {
            &["lpic1-dns-search-suffixes", "lpic1-dns-absolute-precedence"]
        } else if client_names {
            &["lpic1-local-host-alias", "lpic1-name-source-fallback"]
        } else if optional_mount {
            &["lpic1-removable-startup-policy"]
        } else if nm_profiles {
            &[
                "lpic1-nm-persistent-profile",
                "lpic1-nm-autoconnect-choice",
                "lpic1-nm-uplink-handover",
            ]
        } else if dhcp_config {
            &["lpic2-dhcp-dynamic-ranges", "lpic2-dhcp-option-scope"]
        } else if backup_selection {
            &["lpic2-backup-directory-selection"]
        } else if qos {
            &["ccna-qos-mark-boundary", "ccna-qos-trusted-handover"]
        } else if startup {
            &[
                "lpic1-startup-current-future",
                "lpic1-startup-install-links",
            ]
        } else if linux_logging {
            &["lpic1-syslog-exact", "lpic1-syslog-handover"]
        } else if debug {
            &["shell-debug-status", "shell-recovery-ledger"]
        } else if host {
            &["lpic1-host-routing", "lpic1-host-handover"]
        } else if repository {
            &["lpic1-repo-provenance", "lpic1-repo-handover"]
        } else if capacity {
            &["lpic2-capacity-inodes", "lpic2-capacity-handover"]
        } else if syslog {
            &[
                "ccna-syslog-facility-routing",
                "ccna-syslog-policy-handover",
            ]
        } else if links {
            &["lpic1-links-identity", "lpic1-links-handover"]
        } else if gateway {
            &["ccna-hsrp-wan-tracking", "ccna-hsrp-resilient-handover"]
        } else {
            &[
                "ccna-magic-blocks",
                "lpic1-file-navigation",
                "lpic2-nfs-reference",
                "shell-functions",
            ]
        };
        let light = c.index >= normal.len() * 3;
        let enlarged_slots = enlarged_view_slots(
            arp_inspection
                || dhcp_snooping
                || port_security
                || mount_labs
                || config_scan
                || references
                || cow
                || mbr
                || physical
                || renaming
                || availability
                || retirement
                || raw_lvm
                || swap_policy
                || host6
                || triage
                || transport
                || backups
                || skeleton
                || lookup
                || maps
                || managed
                || listener,
        );
        let enlarged_views = enlarged_slots.len();
        if c.index >= normal.len() * 3 + enlarged.len() * enlarged_views {
            println!("TEACHING_ROLLOUT_PREVIEWS_COMPLETE");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let (id, v) = if !light {
            (normal[c.index / 3], c.index % 3)
        } else {
            let n = c.index - normal.len() * 3;
            (
                enlarged[n / enlarged_views],
                enlarged_slots[n % enlarged_views],
            )
        };
        let view = match v {
            0 => teaching::View::Introduction,
            1 => teaching::View::Example,
            _ => teaching::View::Assignment,
        };
        if c.phase == 0 {
            self.smoke_stage = 200000 + c.index;
            c.reveal_after = 0.;
            c.reveal_deadline = 0.;
            let i = self
                .scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("registered native sample");
            self.launch(i);
            self.teaching_preview_view(i, view);
            self.scale = if light { 2. } else { 1. };
            ctx.set_zoom_factor(self.scale);
            self.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|t| t.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!light));
            if light {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100., 850.)));
            }
            self.lesson_scroll_reset = true;
        }
        if c.phase == 14 {
            assert!(
                self.preview_view_ready(self.selected.expect("actual selected preview"), view),
                "Refuse a mislabeled teaching screenshot: {id} requested {view:?}"
            );
            println!(
                "TEACHING_PREVIEW {id} view={v} light={light} geometry={:?}",
                ctx.content_rect()
            );
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!(
                    "teaching-{id}-{}-{}",
                    ["introduction", "example", "assignment"][v],
                    if light { "light-200" } else { "dark-100" }
                ),
            )));
        }
        if light && c.phase == 16 {
            let r = self.metrics.main_viewport.unwrap();
            self.pending_events = vec![
                egui::Event::PointerMoved(egui::pos2(r.left() + 1., r.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -500.),
                    modifiers: Default::default(),
                },
            ];
        }
        if light && c.phase == 50 && !self.preview_check_ready(ctx, &mut c) {
            ctx.data_mut(|d| d.insert_temp(key, c));
            return;
        }
        if light && (c.phase == 32 || c.phase == 50) {
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!(
                    "teaching-{id}-{}-light-200-{}",
                    ["introduction", "example", "assignment"][v],
                    if c.phase == 32 {
                        "scrolled"
                    } else {
                        "controls"
                    }
                ),
            )));
        }
        if c.phase > if light { 50 } else { 14 } && !self.capture_pending {
            c.index += 1;
            c.phase = 0;
        } else {
            c.phase += 1;
        }
        ctx.data_mut(|d| d.insert_temp(key, c));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn environment_intro_preview_reaches_real_example_action_and_refuses_wrong_view() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-env-tool-path")
            .unwrap();
        a.launch(i);
        for v in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.teaching_preview_view(i, v);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 6000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            assert!(a.preview_view_ready(i, v));
            assert_eq!(
                a.preview_action_rect(),
                if v == teaching::View::Introduction {
                    a.metrics.example_action
                } else {
                    a.metrics.check
                }
            );
            assert!(a.preview_action_rect().is_some());
            a.progress
                .attempts
                .get_mut("lpic1-env-tool-path")
                .unwrap()
                .challenge = true;
            assert!(!a.preview_view_ready(i, v));
            a.progress
                .attempts
                .get_mut("lpic1-env-tool-path")
                .unwrap()
                .challenge = false;
        }
        a.teaching_preview_view(i, teaching::View::Introduction);
        a.scale = 2.;
        ctx.set_zoom_factor(2.);
        a.smoke_stage = 245003;
        let before = serde_json::to_value(&a.progress).unwrap();
        let mut capture = Capture::default();
        let mut ready = false;
        let began = ctx.input(|i| i.time);
        for n in 0..100 {
            let events = std::mem::take(&mut a.pending_events);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(634., 316.),
                    )),
                    time: Some(began + n as f64 * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    a.render(ctx);
                    ready = a.preview_check_ready(ctx, &mut capture);
                },
            );
            if ready {
                break;
            }
        }
        assert!(
            ready,
            "actual introduction action must be reachable by normal scrolling"
        );
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    }
    #[test]
    fn enlarged_view_plan_has_only_real_labels_and_visits_each_requested_screen_once() {
        for all in [false, true] {
            let slots = enlarged_view_slots(all);
            assert_eq!(slots, if all { &[0, 1, 2][..] } else { &[1, 2][..] });
            let actual: Vec<_> = (0..slots.len())
                .map(|n| ["introduction", "example", "assignment"][slots[n % slots.len()]])
                .collect();
            assert_eq!(actual.len(), if all { 3 } else { 2 });
            assert_eq!(
                actual
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                slots.len()
            );
        }
    }

    #[test]
    fn listener_example_reaches_actual_run_without_inventing_check_button() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        a.scale = 2.;
        ctx.set_zoom_factor(2.);
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-listener-audit")
            .unwrap();
        a.launch(i);
        a.teaching_preview_view(i, teaching::View::Example);
        a.smoke_stage = 245002;
        let before = serde_json::to_value(&a.progress).unwrap();
        let mut capture = Capture::default();
        let mut ready = false;
        for n in 0..40 {
            let events = std::mem::take(&mut a.pending_events);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(634., 316.),
                    )),
                    time: Some(n as f64 * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    a.render(ctx);
                    assert!(a.metrics.check.is_none());
                    assert!(a.metrics.terminal_run.is_some());
                    ready = a.preview_check_ready(ctx, &mut capture);
                },
            );
            if ready {
                break;
            }
        }
        assert!(ready);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    }
    #[test]
    fn listener_preview_refuses_challenge_assignment_as_introduction_or_example() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-listener-audit")
            .unwrap();
        a.launch(i);
        a.progress
            .attempts
            .get_mut("lpic1-listener-audit")
            .unwrap()
            .challenge = true;
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.teaching_preview_view(i, view);
            assert!(!a.preview_view_ready(i, view));
        }
        a.progress
            .attempts
            .get_mut("lpic1-listener-audit")
            .unwrap()
            .challenge = false;
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.teaching_preview_view(i, view);
            assert!(a.preview_view_ready(i, view));
        }
    }
    #[test]
    fn preview_waits_for_visibility_and_scroll_settling_without_repeating_same_frame_input() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = false;
        app.scale = 2.;
        ctx.set_zoom_factor(2.);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-syslog-policy-handover")
            .unwrap();
        app.launch(i);
        app.teaching_preview_view(i, teaching::View::Assignment);
        app.smoke_stage = 245002;
        let before = serde_json::to_value(&app.progress).unwrap();
        let mut capture = Capture::default();
        let mut ready = false;
        for n in 0..30 {
            let events = std::mem::take(&mut app.pending_events);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(634., 316.),
                    )),
                    time: Some(n as f64 * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    app.render(ctx);
                    ready = app.preview_check_ready(ctx, &mut capture);
                    let queued = app.pending_events.len();
                    assert_eq!(ready, app.preview_check_ready(ctx, &mut capture));
                    assert_eq!(
                        app.pending_events.len(),
                        queued,
                        "Do not queue another wheel event before the current scroll settles"
                    );
                },
            );
            if ready {
                assert!(ctx.input(|i| i.time) >= capture.reveal_after);
                break;
            }
        }
        assert!(ready);
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    }

    #[test]
    fn enlarged_script_introduction_reaches_example_action_without_assuming_run_exists() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = false;
        app.scale = 2.;
        ctx.set_zoom_factor(2.);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "shell-map-lookup")
            .unwrap();
        app.launch(i);
        app.teaching_preview_view(i, teaching::View::Introduction);
        app.smoke_stage = 245002;
        let before = serde_json::to_value(&app.progress).unwrap();
        let mut capture = Capture::default();
        let mut ready = false;
        for n in 0..30 {
            let events = std::mem::take(&mut app.pending_events);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(634., 316.),
                    )),
                    time: Some(n as f64 * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    app.render(ctx);
                    assert!(app.metrics.check.is_none());
                    assert!(app.metrics.example_action.is_some());
                    ready = app.preview_check_ready(ctx, &mut capture);
                    let queued = app.pending_events.len();
                    assert_eq!(ready, app.preview_check_ready(ctx, &mut capture));
                    assert_eq!(
                        app.pending_events.len(),
                        queued,
                        "Do not queue another wheel event before the current scroll settles"
                    );
                },
            );
            if ready {
                assert!(ctx.input(|i| i.time) >= capture.reveal_after);
                break;
            }
        }
        assert!(ready);
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    }

    #[test]
    fn enlarged_preview_scroll_reaches_check_without_changing_work() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = false;
        app.scale = 2.;
        ctx.set_zoom_factor(2.);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-syslog-policy-handover")
            .unwrap();
        app.launch(i);
        app.teaching_preview_view(i, teaching::View::Assignment);
        app.smoke_stage = 245001;
        let before = serde_json::to_value(&app.progress).unwrap();
        let mut reached = false;
        for n in 0..20 {
            let events = std::mem::take(&mut app.pending_events);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(634., 316.),
                    )),
                    time: Some(n as f64 * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
            if app.reveal_preview_check(&ctx) {
                reached = true;
                break;
            }
            assert!(
                app.pending_events
                    .iter()
                    .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
            );
        }
        assert!(
            reached,
            "actual Check must be reachable in enlarged narrow preview"
        );
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    }
    #[test]
    fn mbr_preview_uses_actual_partition_views_and_rejects_challenge_mislabeling() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        for id in smoke::mbr_layouts::IDS
            .into_iter()
            .chain(["lpic1-disk-safe-gap"])
        {
            let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
            a.launch(i);
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                a.teaching_preview_view(i, view);
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1500., 1800.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                assert!(a.preview_view_ready(i, view));
                assert!(a.preview_action_rect().is_some());
                let engine::State::PartitionFs { simulation } = &a.progress.attempts[id].state
                else {
                    panic!()
                };
                assert_eq!(simulation.view, view);
                assert_eq!(simulation.current, simulation.origin.initial);
                a.progress.attempts.get_mut(id).unwrap().challenge = true;
                assert!(!a.preview_view_ready(i, view));
                a.progress.attempts.get_mut(id).unwrap().challenge = false;
            }
        }
    }

    #[test]
    fn aging_preview_uses_the_actual_action_in_each_view_and_rejects_challenge_labels() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-aging-rotation")
            .unwrap();
        a.launch(i);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.teaching_preview_view(i, view);
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1268., 1308.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
            }
            let key = if view == teaching::View::Introduction {
                "try"
            } else {
                "check"
            };
            let actual = a
                .metrics
                .aging
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1;
            assert_eq!(a.preview_action_rect(), Some(actual));
            assert!(a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("lpic1-aging-rotation")
                .unwrap()
                .challenge = true;
            assert!(!a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("lpic1-aging-rotation")
                .unwrap()
                .challenge = false;
        }
    }

    #[test]
    fn port_security_preview_uses_the_actual_action_in_each_view_and_rejects_challenge_labels() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-security-identity")
            .unwrap();
        a.launch(i);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.teaching_preview_view(i, view);
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1268., 1308.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
            }
            let key = if view == teaching::View::Introduction {
                "try"
            } else {
                "check"
            };
            let actual = a
                .metrics
                .port_security
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1;
            assert_eq!(a.preview_action_rect(), Some(actual));
            assert!(a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("ccna-security-identity")
                .unwrap()
                .challenge = true;
            assert!(!a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("ccna-security-identity")
                .unwrap()
                .challenge = false;
        }
    }

    #[test]
    fn dhcp_snooping_preview_uses_the_actual_action_in_each_view_and_rejects_challenge_labels() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-dhcp-server-boundary")
            .unwrap();
        a.launch(i);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.teaching_preview_view(i, view);
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1268., 1308.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
            }
            let key = if view == teaching::View::Introduction {
                "try"
            } else {
                "check"
            };
            let actual = a
                .metrics
                .dhcp_snooping
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1;
            assert_eq!(a.preview_action_rect(), Some(actual));
            assert!(a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("ccna-dhcp-server-boundary")
                .unwrap()
                .challenge = true;
            assert!(!a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("ccna-dhcp-server-boundary")
                .unwrap()
                .challenge = false;
        }
    }

    #[test]
    fn arp_inspection_preview_uses_the_actual_action_in_each_view_and_rejects_challenge_labels() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-arp-sender-binding")
            .unwrap();
        a.launch(i);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.teaching_preview_view(i, view);
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1268., 1308.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
            }
            let key = if view == teaching::View::Introduction {
                "try"
            } else {
                "check"
            };
            let actual = a
                .metrics
                .arp_inspection
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1;
            assert_eq!(a.preview_action_rect(), Some(actual));
            assert!(a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("ccna-arp-sender-binding")
                .unwrap()
                .challenge = true;
            assert!(!a.preview_view_ready(i, view));
            a.progress
                .attempts
                .get_mut("ccna-arp-sender-binding")
                .unwrap()
                .challenge = false;
        }
    }
}
