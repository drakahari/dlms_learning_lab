use dlms_learning_lab::{
    catalog::{self, Filter},
    engine, ipv4,
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace as model,
};
use dlms_learning_lab::{course, script_teaching as teaching, shell, theme};
use eframe::egui::{self, RichText};
mod accounts_workspace;
mod acl_workspace;
mod address_learning;
mod apache_static_workspace;
mod client_learning;
mod completion_index;
mod controls;
mod cron_workspace;
mod data;
mod debian_packages_workspace;
mod dhcpv4_workspace;
mod discovery_workspace;
mod dns_workspace;
mod etherchannel_workspace;
mod file_backup_workspace;
mod legacy_learning;
mod lvm_workspace;
mod nat_workspace;
mod nfs_workspace;
mod one_off_boundary;
mod one_off_workspace;
mod ospf_workspace;
mod physical_link_workspace;
mod process_learning;
mod rpm_packages;
mod script_learning;
mod scripting;
mod seed_learning;
mod service_learning;
mod spanning_tree_workspace;
mod storage_workspace;
mod terminal_learning;
mod unit_manager_workspace;
use std::{path::PathBuf, time::Instant};
mod ipv4_routing_workspace;
mod ipv4_workspace;
mod ipv6_workspace;
mod mac_learning_workspace;
mod smoke;
mod switching_workspace;
#[cfg(test)]
mod tests;
mod workspace;
#[derive(Default)]
struct Metrics {
    cron: cron_workspace::CronMetrics,
    one_off: one_off_workspace::OneOffMetrics,
    backup: file_backup_workspace::BackupMetrics,
    nfs: nfs_workspace::NfsMetrics,
    ether: etherchannel_workspace::EtherMetrics,
    physical: physical_link_workspace::Metrics,
    lvm: lvm_workspace::Metrics,
    rpm: rpm_packages::RpmMetrics,
    nat_fields: Vec<(String, egui::Rect, egui::Id, bool, egui::Rect)>,
    nat_controls: Vec<(String, egui::Rect, bool, egui::Rect)>,
    nat_choices: Vec<(String, egui::Rect)>,
    nat_popup: Option<egui::Rect>,
    nat_example_picker: Option<(egui::Rect, egui::Rect)>,
    nat_example_popup: Option<egui::Rect>,
    nat_result_rows: Vec<(usize, bool, egui::Rect, egui::Rect)>,
    nat_static_viewport: Option<egui::Rect>,
    nat_static_scroll: Option<(egui::Id, egui::Vec2, egui::Vec2)>,
    nat_feedback: Option<egui::Rect>,
    nat_observation: Option<egui::Rect>,
    apache_controls: Vec<(String, egui::Rect, bool)>,
    apache_fields: Vec<(String, egui::Rect, egui::Id, bool)>,
    apache_editor: Option<(egui::Rect, egui::Rect, egui::Id, bool)>,
    apache_file_items: Vec<(String, egui::Rect)>,
    apache_action_items: Vec<(usize, egui::Rect)>,
    apache_popup: Option<(String, egui::Rect)>,
    apache_result: Option<egui::Rect>,
    apache_response: Option<egui::Rect>,
    apache_check_evaluated: bool,
    workspace_header_controls: Vec<(String, egui::Rect, egui::Id)>,
    main_viewport: Option<egui::Rect>,
    main_scroll_offset: egui::Vec2,
    main_content_size: egui::Vec2,
    main_content_top: f32,
    main_available_height: f32,
    header: Vec<egui::Rect>,
    lesson_tabs: Vec<egui::Rect>,
    example_action: Option<egui::Rect>,
    example_reset: Option<egui::Rect>,
    command_insert: Option<egui::Rect>,
    address_diagram: Option<egui::Rect>,
    acl_diagram: Option<egui::Rect>,
    acl_router: Option<egui::Rect>,
    stp_heading: Option<egui::Rect>,
    stp_result: Option<egui::Rect>,
    stp_bridge_popup: Option<egui::Rect>,
    stp_port_popup: Option<egui::Rect>,
    stp_bridge: Option<egui::Rect>,
    stp_port: Option<egui::Rect>,
    stp_link: Option<egui::Rect>,
    stp_bridge_items: Vec<(String, egui::Rect)>,
    stp_port_items: Vec<(String, egui::Rect)>,
    stp_link_items: Vec<(String, egui::Rect)>,
    stp_fields: Vec<(String, egui::Rect, egui::Id)>,
    stp_toggles: Vec<(String, egui::Rect)>,
    stp_apply: Option<egui::Rect>,
    ospf_router: Option<egui::Rect>,
    ospf_interface: Option<egui::Rect>,
    ospf_router_items: Vec<(String, egui::Rect)>,
    ospf_interface_items: Vec<(String, egui::Rect)>,
    ospf_fields: Vec<(String, egui::Rect, egui::Id)>,
    ospf_toggles: Vec<(String, egui::Rect)>,
    ospf_type: Option<egui::Rect>,
    ospf_diagram: Option<egui::Rect>,
    acl_router_items: Vec<(String, egui::Rect)>,
    assignment_action: Option<egui::Rect>,
    settings: Option<egui::Rect>,
    data_export: Option<egui::Rect>,
    data_file_cancel: Option<egui::Rect>,
    settings_tabs: Vec<(String, egui::Rect, egui::Rect)>,
    settings_samples: Option<(egui::Rect, egui::Rect)>,
    settings_close: Option<(egui::Rect, egui::Id, bool, egui::Rect)>,
    data_restore: Option<egui::Rect>,
    data_file_action: Option<egui::Rect>,
    data_file_viewport: Option<egui::Rect>,
    data_filename: Option<egui::Rect>,
    data_filename_id: Option<egui::Id>,
    data_confirm: Option<egui::Rect>,
    data_cancel: Option<egui::Rect>,
    course_start: Option<egui::Rect>,
    unit_picker: Option<egui::Rect>,
    unit_choices: Vec<(String, egui::Rect)>,
    unit_popup: Option<egui::Rect>,
    next_unit: Option<egui::Rect>,
    unit_back: Option<egui::Rect>,
    unit_complete: Option<egui::Rect>,
    filters: Vec<egui::Rect>,
    card_actions: Vec<(egui::Rect, egui::Rect)>,
    library_search: Option<egui::Rect>,
    card_verification: Vec<(String, Option<completion_index::Status>, egui::Rect)>,
    pagination: bool,
    inputs: Vec<egui::Rect>,
    seed_fields: Vec<(String, egui::Rect, egui::Id)>,
    seed_summary: Option<egui::Rect>,
    seed_explanation: Option<egui::Rect>,
    seed_feedback: Vec<(bool, egui::Rect)>,
    legacy_feedback: Option<egui::Rect>,
    vlan_fields: Vec<(egui::Id, egui::Rect)>,
    vlan_field_enabled: Vec<(egui::Id, bool)>,
    terminal: Option<egui::Rect>,
    terminal_run: Option<egui::Rect>,
    terminal_run_enabled: Option<bool>,
    terminal_example_picker: Option<egui::Rect>,
    terminal_example_popup: Option<egui::Rect>,
    terminal_example_choices: Vec<(usize, egui::Rect)>,
    accounts_run: Option<egui::Rect>,
    manager_slot: Option<egui::Rect>,
    manager_file: Option<egui::Rect>,
    manager_run: Option<egui::Rect>,
    manager_diagnostics: Vec<(egui::Rect, f32)>,
    manager_inspector: Option<egui::Rect>,
    manager_inspector_open: bool,
    manager_kind: Vec<(bool, egui::Rect)>,
    manager_items: Vec<(String, egui::Rect)>,
    manager_example_action: Option<egui::Rect>,
    manager_example_items: Vec<(usize, egui::Rect)>,
    manager_challenge: Option<egui::Rect>,
    manager_learn: Option<egui::Rect>,
    mode_learn: Option<egui::Rect>,
    mode_challenge: Option<egui::Rect>,
    reset_cancel: Option<egui::Rect>,
    accounts_output_scroll: Option<(f32, egui::Rect)>,
    accounts_latest_output: Option<egui::Rect>,
    output: Option<egui::Rect>,
    insert: Option<egui::Rect>,
    check: Option<egui::Rect>,
    focus: Option<egui::Id>,
    completion: bool,
    feedback: Option<egui::Rect>,
    guide: bool,
    editor_save: Option<egui::Rect>,
    storage_validate: Option<egui::Rect>,
    storage_test: Option<egui::Rect>,
    storage_command: Option<egui::Rect>,
    storage_run: Option<egui::Rect>,
    dns_reload: Option<egui::Rect>,
    dns_update: Option<egui::Rect>,
    dns_pickers: Vec<egui::Rect>,
    dns_editor_overflow: bool,
    dns_fields: Vec<(String, egui::Rect, egui::Id)>,
    dns_copy: Option<egui::Rect>,
    dns_type: Option<egui::Rect>,
    dns_popup: Option<egui::Rect>,
    dns_choices: Vec<(String, egui::Rect)>,
    dns_notice: Option<egui::Rect>,
    dns_result: Option<egui::Rect>,
    package_heading: Option<egui::Rect>,
    package_result: Option<egui::Rect>,
    package_command: Option<(egui::Rect, egui::Id, bool)>,
    package_run: Option<(egui::Rect, bool)>,
    package_path: Option<egui::Rect>,
    package_path_popup: Option<egui::Rect>,
    package_path_items: Vec<(String, egui::Rect)>,
    package_example: Option<egui::Rect>,
    package_example_popup: Option<egui::Rect>,
    package_example_items: Vec<(usize, egui::Rect)>,
    package_inspector: Option<egui::Rect>,
    package_diagnostics: Vec<(egui::Rect, f32)>,
    dhcp_fields: Vec<(String, egui::Rect, egui::Id)>,
    dhcp_field_clips: std::collections::BTreeMap<String, egui::Rect>,
    dhcp_controls: Vec<(String, egui::Rect, bool)>,
    dhcp_control_clips: std::collections::BTreeMap<String, egui::Rect>,
    dhcp_clients: Vec<(String, egui::Rect)>,
    dhcp_pools: Vec<(String, egui::Rect)>,
    dhcp_example_actions: Vec<(usize, egui::Rect)>,
    dhcp_client: Option<egui::Rect>,
    dhcp_pool: Option<egui::Rect>,
    dhcp_popup: Option<egui::Rect>,
    dhcp_topology: Option<egui::Rect>,
    dhcp_status: Option<egui::Rect>,
    dhcp_notice: Option<egui::Rect>,
    dhcp_result_summary: Option<egui::Rect>,
    discovery_send: Vec<(dlms_learning_lab::discovery::Endpoint, egui::Rect)>,
    discovery_toggles: Vec<(String, egui::Rect)>,
    discovery_time: Option<egui::Rect>,
    discovery_fields: Vec<(String, egui::Rect)>,
    discovery_picker: Option<egui::Rect>,
    discovery_options: Vec<(String, egui::Rect)>,
    editor_viewport: Option<egui::Rect>,
    probe: Option<egui::Rect>,
    routing_device: Option<egui::Rect>,
    routing_address_tab: Option<egui::Rect>,
    routing_device_choices: Vec<(String, egui::Rect)>,
    routing_forward: Option<egui::Rect>,
    routing_add: Option<egui::Rect>,
    routing_route_interface: Option<egui::Rect>,
    routing_picker_id: Option<egui::Id>,
    routing_port_options: Vec<(String, egui::Id)>,
    routing_load_example: Option<egui::Rect>,
    routing_router_options: Option<egui::Rect>,
    client_fields: Vec<(String, egui::Id, egui::Rect)>,
    client_action: Option<egui::Rect>,
    client_popup: Option<egui::Rect>,
    client_actions: Vec<(usize, egui::Rect)>,
    client_run: Option<egui::Rect>,
    mac_source: Option<egui::Rect>,
    mac_inspect: Option<egui::Rect>,
    mac_source_popup: Option<egui::Rect>,
    mac_action_popup: Option<egui::Rect>,
    mac_clear_popup: Option<egui::Rect>,
    vlan_switch_popup: Option<egui::Rect>,
    mac_sources: Vec<(String, egui::Rect)>,
    mac_destination: Option<egui::Rect>,
    mac_time: Option<egui::Rect>,
    mac_tools: Option<egui::Rect>,
    mac_settings: Option<egui::Rect>,
    mac_lifetime: Option<egui::Rect>,
    mac_aging: Option<egui::Rect>,
    mac_clear_switch: Option<egui::Rect>,
    mac_clear_choices: Vec<(String, egui::Rect)>,
    mac_clear_vlan: Option<egui::Rect>,
    vlan_switch_picker: Option<egui::Rect>,
    vlan_switch_choices: Vec<(String, egui::Rect)>,
    vlan_named_fields: Vec<(String, String, &'static str, egui::Rect)>,
    mac_example_action: Option<egui::Rect>,
    mac_example_choices: Vec<(usize, egui::Rect)>,
    mac_send: Option<egui::Rect>,
    mac_send_id: Option<egui::Id>,
    mac_send_enabled: Option<bool>,
    mac_advance: Option<egui::Rect>,
    mac_clear: Option<egui::Rect>,
    script_history: Option<egui::Rect>,
    script_diagnostic: Option<egui::Rect>,
    script_completion: Option<egui::Rect>,
    script_feedback: Option<egui::Rect>,
    script_case_rows: Vec<(usize, egui::Rect)>,
    script_case_status: Option<(usize, bool, egui::Rect)>,
    reopen_scale: Option<egui::Rect>,
    reopen_scale_items: Vec<(f32, egui::Rect)>,
    reopen_theme: Option<egui::Rect>,
    reopen_theme_items: Vec<(String, egui::Rect)>,
    physical_picker_choices: Vec<(String, egui::Rect, egui::Rect)>,
    stop: Option<egui::Rect>,
    hint: Option<egui::Rect>,
    why: Option<egui::Rect>,
    help_visible: bool,
    help: Option<egui::Rect>,
    reset: Option<egui::Rect>,
    back: Option<egui::Rect>,
    extras: Option<egui::Rect>,
    walkthrough: Option<egui::Rect>,
    confirm_reset: Option<egui::Rect>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistorySource {
    Writer7,
    Writer8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptLessonFlow {
    Case,
    Getopts,
}
pub struct LabApp {
    scenarios: Vec<Scenario>,
    units: Vec<course::Unit>,
    progress: Progress,
    path: PathBuf,
    // Keep the securely allocated synthetic profile alive only for this app instance.
    _smoke_profile: Option<tempfile::TempDir>,
    filter: Filter,
    selected: Option<usize>,
    input: String,
    checked: bool,
    reset_pending: bool,
    dirty: bool,
    storage_error: Option<String>,
    read_only: bool,
    synthetic: bool,
    synthetic_catalog: Vec<Scenario>,
    developer: bool,
    page: usize,
    scale: f32,
    theme_id: String,
    theme_preview: bool,
    settings_open: bool,
    settings_data: bool,
    data_tools: data::Tools,
    profile_lock: Option<progress::ProfileLock>,
    preferences_error: Option<String>,
    preferences_read_only: bool,
    mac_trace_reset: Option<usize>,
    script_job: Option<scripting::Job>,
    script_case: usize,
    focus_input: bool,
    scroll_command: bool,
    lesson_scroll_reset: bool,
    notice: Option<String>,
    metrics: Metrics,
    apache_presentation: apache_static_workspace::Presentation,
    nat_presentation: nat_workspace::Presentation,
    rpm_presentation: rpm_packages::Presentation,
    nfs_presentation: nfs_workspace::Presentation,
    ether_presentation: etherchannel_workspace::Presentation,
    physical_presentation: physical_link_workspace::Presentation,
    lvm_presentation: lvm_workspace::Presentation,
    completion_index: completion_index::Handle,
    cron_presentation: cron_workspace::Presentation,
    one_off_presentation: one_off_workspace::Presentation,
    backup_presentation: file_backup_workspace::Presentation,
    help_open: bool,
    extra_filters: bool,
    host_activity: bool,
    ipv4_view: Option<ipv4_workspace::Assignment>,
    address_example: Option<ipv4_workspace::Assignment>,
    address_worked: std::collections::BTreeMap<String, String>,
    smoke: bool,
    history_source: Option<HistorySource>,
    reopen: Option<smoke::reopen::Session>,
    started: Instant,
    smoke_stage: usize,
    pending_events: Vec<egui::Event>,
    capture_pending: bool,
}
impl LabApp {
    pub fn new(cc: &eframe::CreationContext<'_>, smoke: bool, developer: bool) -> Self {
        Self::from_context(&cc.egui_ctx, smoke, developer)
    }
    fn from_context(ctx: &egui::Context, smoke: bool, developer: bool) -> Self {
        let history_source =
            HistorySource::from_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref());
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12., 8.);
        style.spacing.button_padding = egui::vec2(14., 8.);

        style.spacing.combo_width = 160.;
        style.spacing.scroll.floating = false;
        for (kind, size) in [
            (egui::TextStyle::Body, 16.),
            (egui::TextStyle::Button, 16.),
            (egui::TextStyle::Small, 13.),
        ] {
            style
                .text_styles
                .insert(kind, egui::FontId::proportional(size));
        }
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(15.));
        ctx.set_style(style);
        #[cfg(not(test))]
        let scenarios = pack::bundled().expect("Bundled pack validates");
        #[cfg(not(test))]
        let units = course::bundled(&scenarios).expect("Bundled units validate");
        #[cfg(test)]
        let (scenarios, units) = tests::validated_ui_catalog();
        // Developer reopen validation runs before default path/fallback/profile I/O.
        let reopen = smoke::reopen::Session::from_environment(smoke, developer)
            .unwrap_or_else(|e| panic!("Developer reopen refused: {e}"));
        let is_reopen = reopen.is_some();
        let smoke_profile = smoke.then(|| {
            let mut builder = tempfile::Builder::new();
            builder.prefix("dlms-native-smoke-");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(std::fs::Permissions::from_mode(0o700));
            }
            builder.tempdir().expect("Create private synthetic profile")
        });
        let path = if let Some(profile) = &smoke_profile {
            profile.path().join("progress-v1.json")
        } else if is_reopen {
            progress::default_path()
                .expect("Private normal profile path must resolve without fallback")
        } else {
            progress::default_path()
                .unwrap_or_else(|_| PathBuf::from("local-data/progress-v1.json"))
        };
        if let Some(session) = &reopen {
            session
                .verify_profile_paths(&path)
                .expect("Private profile paths must be contained before any lock/load");
        }
        let fresh = || Progress {
            schema_version: 1,
            ..Default::default()
        };
        let lock = if smoke {
            Ok(None)
        } else {
            progress::lock_profile(&path).map(Some)
        };
        let lock_error = lock.as_ref().err().cloned();
        let profile_lock = lock.ok().flatten();
        let (progress, storage_error) = if smoke {
            (fresh(), None)
        } else {
            match progress::load(&path, &scenarios) {
                Ok(p) => (p, None),
                Err(e) => (fresh(), Some(e)),
            }
        };
        let storage_error = storage_error.or(lock_error.clone());
        let read_only = storage_error.is_some();
        let preferences = if smoke {
            Ok(theme::Preferences::default())
        } else {
            theme::Preferences::load(&path.with_file_name("preferences-v1.json"))
        };
        let preferences_error = preferences.as_ref().err().cloned();
        let preferences_read_only = preferences_error.is_some() || lock_error.is_some();
        let preferences = preferences.unwrap_or_default();
        theme::apply(ctx, &preferences.theme);
        let completion_index = completion_index::Handle::default();
        Self {
            scenarios,
            units,
            progress,
            path,
            _smoke_profile: smoke_profile,
            filter: Filter::default(),
            selected: None,
            input: String::new(),
            checked: false,
            reset_pending: false,
            dirty: false,
            storage_error,
            read_only,
            synthetic: false,
            synthetic_catalog: Vec::new(),
            developer,
            page: 0,
            scale: preferences.text_scale,
            theme_id: preferences.theme,
            theme_preview: false,
            settings_open: false,
            settings_data: false,
            data_tools: data::Tools::default(),
            profile_lock,
            preferences_error,
            preferences_read_only,
            mac_trace_reset: None,
            script_job: None,
            script_case: 0,
            focus_input: false,
            scroll_command: false,
            lesson_scroll_reset: false,
            notice: None,
            lvm_presentation: Default::default(),
            metrics: Metrics::default(),
            apache_presentation: apache_static_workspace::Presentation::default(),
            nat_presentation: nat_workspace::Presentation::default(),
            rpm_presentation: rpm_packages::Presentation::default(),
            nfs_presentation: nfs_workspace::Presentation::default(),
            ether_presentation: etherchannel_workspace::Presentation::default(),
            physical_presentation: physical_link_workspace::Presentation::with_index(
                completion_index.clone(),
            ),
            cron_presentation: cron_workspace::Presentation::default(),
            one_off_presentation: one_off_workspace::Presentation::with_index(
                completion_index.clone(),
            ),
            backup_presentation: file_backup_workspace::Presentation::with_index(
                completion_index.clone(),
            ),
            completion_index,
            help_open: false,
            extra_filters: false,
            host_activity: false,
            ipv4_view: None,
            address_example: None,
            address_worked: Default::default(),
            smoke,
            history_source,
            reopen,
            started: Instant::now(),
            smoke_stage: if is_reopen {
                17000
            } else if let Some(stage) = Self::one_off_lessons_stage(
                smoke,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if let Some(stage) = Self::physical_stage(
                smoke,
                developer,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if let Some(stage) =
                Self::etherchannel_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) = Self::file_backup_lessons_stage(
                smoke,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if let Some(stage) =
                Self::cron_lessons_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) =
                Self::nfs_alignment_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) = Self::script_feedback_stage(
                smoke,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if let Some(stage) =
                Self::nfs_lessons_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) =
                Self::rpm_lessons_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) =
                Self::nat_lessons_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) =
                Self::nat_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("apache-static-lessons")
            {
                36000
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("spanning-tree-lessons")
            {
                35000
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("spanning-tree-workspace")
            {
                33000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("package-lessons")
            {
                34000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("client-teaching")
            {
                23000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dhcpv4-lessons")
            {
                31000
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dhcpv4-workspace")
            {
                30000
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("legacy-terminal-teaching")
            {
                28000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dns-reverse") {
                27000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("seed-teaching") {
                25000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("find-lessons") {
                21000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("mac-layout") {
                20000
            } else if smoke
                && matches!(
                    std::env::var("DLMS_SMOKE_STAGE").as_deref(),
                    Ok("mac-lessons") | Ok("mac-handover")
                )
            {
                18000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("mac-learning") {
                16000
            } else if history_source.is_some() {
                15000
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("unit-manager-unit")
            {
                13000
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("unit-manager-workspace")
            {
                12000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ospf-layout") {
                11500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ospf-unit") {
                11000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ospf-workspace")
            {
                10000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("accounts-unit") {
                9000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("acl-unit-layout")
            {
                8700
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("acl-unit") {
                8200
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("accounts-workspace")
            {
                7800
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("accounts-output")
            {
                7900
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("acl-layout") {
                7660
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("acl-workspace") {
                7600
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("private-office")
            {
                72000
            } else if let Some(stage) =
                Self::lvm_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) =
                Self::loop_recovery_stage(smoke, std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
            {
                stage
            } else if let Some(stage) = Self::numeric_validation_stage(
                smoke,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if let Some(stage) = Self::optional_inputs_stage(
                smoke,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if let Some(stage) = Self::command_vectors_stage(
                smoke,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if let Some(stage) = Self::safe_automation_stage(
                smoke,
                std::env::var("DLMS_SMOKE_STAGE").ok().as_deref(),
            ) {
                stage
            } else if smoke
                && matches!(
                    std::env::var("DLMS_SMOKE_STAGE").as_deref(),
                    Ok("capture-lessons" | "case-lessons" | "getopts-lessons")
                )
            {
                7200
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("substitution") {
                7000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("startup") {
                4700
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("storage-workspace")
            {
                6200
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("storage-layout")
            {
                6270
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("storage-unit") {
                6400
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("arrays") {
                4800
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("arrays-stop") {
                4920
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("arrays-library")
            {
                4950
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("process-workspace")
            {
                5000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("discovery-unit")
            {
                5600
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("discovery-workspace")
            {
                5500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("process-unit") {
                5200
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dns-unit") {
                4500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dns-layout") {
                4250
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("dns-workspace") {
                4200
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("addressing-layout")
            {
                2940
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("alignment") {
                3100
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("service-teaching")
            {
                3000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("addressing") {
                2900
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("file-batches") {
                2500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("routing-v4") {
                2600
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ipv4-unit") {
                2700
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ipv4-unit-layout")
            {
                2808
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("expansion") {
                2400
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("vlan-resume") {
                2320
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("vlans") {
                2200
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("switching") {
                2100
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("files-layout") {
                1209
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("scope-recovery")
            {
                1500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("access-library")
            {
                2090
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("access") {
                2000
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("counts-stop") {
                1980
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("counts") {
                1900
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("arithmetic") {
                1800
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("text") {
                1700
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("scope") {
                1600
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ipv6-layout") {
                1408
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("ipv6") {
                1300
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("files") {
                1100
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("reusable") {
                900
            } else if smoke
                && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("composition-layout")
            {
                772
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("composition") {
                700
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("courses") {
                600
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("recovery") {
                500
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("teaching-layout")
            {
                400
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("teaching") {
                250
            } else if smoke && std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("scripts") {
                176
            } else {
                0
            },
            pending_events: Vec::new(),
            capture_pending: false,
        }
    }
    fn launch(&mut self, i: usize) {
        let source = &self.scenarios[i];
        let previous = self.progress.attempts.get(&source.id);
        if matches!(source.initial, engine::State::Lvm { .. })
            || matches!(source.goal, engine::Goal::Lvm { .. })
            || previous.is_some_and(|a| matches!(a.state, engine::State::Lvm { .. }))
        {
            let admission = dlms_learning_lab::lvm_workspace::source(source).and_then(|()| {
                previous.map_or(Ok(()), |a| {
                    dlms_learning_lab::lvm_workspace::context(a, source)
                })
            });
            if let Err(e) = admission {
                self.notice = Some(e);
                return;
            }
        }
        self.invalidate_apache_presentation();
        self.nat_presentation.clear();
        self.rpm_presentation.clear();
        self.nfs_presentation.clear();
        self.ether_presentation.clear();
        self.physical_presentation.clear();
        self.cron_presentation.clear();
        self.one_off_presentation.clear();
        self.backup_presentation.clear();
        self.script_job = None;
        self.script_case = 0;
        self.progress.clock = self.progress.clock.saturating_add(1);
        let clock = self.progress.clock;
        let s = &self.scenarios[i];
        let new_attempt = !self.progress.attempts.contains_key(&s.id);
        let a = self
            .progress
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, clock));
        model::prepare(a);
        Self::prepare_apache_static(a, s, new_attempt);
        Self::prepare_rpm(a, s, new_attempt);
        Self::prepare_nfs(a, s, new_attempt);
        Self::prepare_ether(a, s, new_attempt);
        Self::prepare_cron(a, s);
        Self::prepare_one_off(a, s);
        Self::prepare_physical(a, s, new_attempt);
        Self::prepare_backup(a, s, new_attempt);
        if let Some(lesson) = &s.acl_teaching {
            dlms_learning_lab::acl_teaching::prepare(a, lesson, new_attempt)
                .expect("validated ACL lesson");
        }
        if let Some(lesson) = &s.storage_teaching {
            dlms_learning_lab::storage_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.discovery_teaching {
            discovery_workspace::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.teaching {
            teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.client_teaching {
            dlms_learning_lab::client_teaching::prepare(a, lesson, new_attempt)
                .expect("validated client teaching");
        }
        if let Some(lesson) = &s.seed_dns_teaching {
            dlms_learning_lab::seed_dns_teaching::prepare(a, lesson, new_attempt)
                .expect("validated seed DNS teaching");
        }
        if let Some(lesson) = &s.seed_mount_teaching {
            dlms_learning_lab::seed_mount_teaching::prepare(a, lesson, new_attempt)
                .expect("validated seed mount teaching");
        }
        if let Some(lesson) = &s.legacy_access_teaching {
            dlms_learning_lab::legacy_access_teaching::prepare(a, lesson, new_attempt)
                .expect("validated access lesson");
        }
        if let Some(lesson) = &s.legacy_logs_teaching {
            dlms_learning_lab::legacy_logs_teaching::prepare(a, lesson, new_attempt)
                .expect("validated log lesson");
        }
        if let Some(lesson) = &s.service_teaching {
            dlms_learning_lab::service_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.dns_teaching {
            dlms_learning_lab::dns_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.address_teaching {
            dlms_learning_lab::address_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.terminal_teaching {
            dlms_learning_lab::terminal_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.process_teaching {
            dlms_learning_lab::process_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.debian_packages_teaching {
            dlms_learning_lab::debian_packages_teaching::prepare(a, lesson, new_attempt)
                .expect("validated package lesson");
        }
        if let Some(lesson) = &s.nat_teaching {
            dlms_learning_lab::nat_teaching::prepare(a, lesson, new_attempt)
                .expect("validated NAT teaching");
        }
        if let Some(lesson) = &s.dhcpv4_teaching {
            dlms_learning_lab::dhcpv4_teaching::prepare(a, lesson, new_attempt)
                .expect("validated DHCP teaching");
            if new_attempt {
                Self::dhcpv4_initial_selection(a, s);
            }
        }
        if let Some(lesson) = &s.spanning_tree_teaching {
            dlms_learning_lab::spanning_tree_teaching::prepare(a, lesson)
                .expect("validated spanning-tree lesson");
            if new_attempt {
                Self::spanning_tree_initial_selection(a, s);
            }
        }
        if let Some(lesson) = &s.ospf_teaching {
            dlms_learning_lab::ospf_teaching::prepare(a, lesson, new_attempt)
                .expect("validated OSPF lesson");
            if new_attempt {
                Self::ospf_initial_selection(a, s);
            }
        }
        if let Some(lesson) = &s.accounts_teaching {
            dlms_learning_lab::accounts_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.unit_manager_teaching {
            dlms_learning_lab::unit_manager_teaching::prepare(a, lesson, new_attempt)
                .expect("validated unit-manager lesson");
        }
        if let Some(lesson) = &s.mac_teaching {
            dlms_learning_lab::mac_learning_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.switching_teaching {
            dlms_learning_lab::switching_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.ipv6_teaching {
            dlms_learning_lab::ipv6_teaching::prepare(a, lesson, new_attempt);
        }
        if let Some(lesson) = &s.ipv4_routing_teaching {
            dlms_learning_lab::ipv4_routing_teaching::prepare(a, lesson, new_attempt);
        }
        if new_attempt
            && let engine::Goal::Ipv4Routing { requirements } = &s.goal
            && let Some(device) =
                dlms_learning_lab::ipv4_routing_teaching::preferred_device(requirements)
        {
            a.workspace
                .as_mut()
                .unwrap()
                .routing_v4_draft
                .as_mut()
                .unwrap()
                .selected_device = device;
        }
        if new_attempt
            && let engine::Goal::Ipv6 { requirements } = &s.goal
            && let Some(device) = dlms_learning_lab::ipv6_teaching::preferred_device(requirements)
        {
            a.workspace
                .as_mut()
                .unwrap()
                .ipv6_draft
                .as_mut()
                .unwrap()
                .selected_device = device;
        }
        let nfs_recency_notice = if matches!(a.state, engine::State::PhysicalLink { .. }) {
            self.input.clear();
            self.checked = false;
            dlms_learning_lab::physical_link_workspace::record_visit(a, s, clock).err()
        } else if matches!(a.state, engine::State::EtherChannel { .. }) {
            self.input.clear();
            self.checked = false;
            dlms_learning_lab::etherchannel_workspace::record_visit(a, s, clock).err()
        } else if matches!(a.state, engine::State::FileBackup { .. }) {
            self.input.clear();
            self.checked = false;
            dlms_learning_lab::file_backup_workspace::presentation::record_visit(a, s, clock).err()
        } else if matches!(a.state, engine::State::OneOff { .. }) {
            self.input.clear();
            self.checked = false;
            dlms_learning_lab::one_off_workspace::budget::record_visit(a, s, clock).err()
        } else if matches!(a.state, engine::State::Cron { .. }) {
            self.input.clear();
            self.checked = false;
            dlms_learning_lab::cron_workspace::budget::record_visit(a, s, clock).err()
        } else if matches!(a.state, engine::State::Nfs { .. }) {
            self.input.clear();
            self.checked = false;
            dlms_learning_lab::nfs_workspace::record_visit(a, s, clock).err()
        } else {
            a.visited = clock;
            let data = a.workspace.as_ref().unwrap();
            self.input = data.command.clone();
            self.checked = data.checked;
            None
        };
        if nfs_recency_notice.is_none() {
            let _ = self.completion_index.rekey_metadata(a, s);
        }
        self.selected = Some(i);
        self.reset_pending = false;
        self.focus_input = true;
        self.scroll_command = false;
        self.notice = nfs_recency_notice;
        self.help_open = false;
        self.host_activity = false;
        self.load_ipv4_view(i);
        self.load_address_example(i);
        self.dirty = true;
    }
    fn load_ipv4_view(&mut self, i: usize) {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        self.ipv4_view = if let Some(original) = ipv4::Exercise::from_state(&a.state) {
            let practice = a.ipv4.as_ref().is_some_and(|l| l.use_variant);
            let e = if practice {
                a.ipv4
                    .as_ref()
                    .and_then(|l| l.variant.clone())
                    .unwrap_or(original)
            } else {
                original
            };
            let w = a.workspace.as_ref().unwrap();
            Some(ipv4_workspace::Assignment::from_saved(
                e,
                if practice {
                    w.practice_check.clone()
                } else {
                    w.ipv4_check.clone()
                },
                a.hints,
            ))
        } else {
            None
        };
    }
    fn reset_attempt(&mut self, i: usize) {
        if matches!(self.scenarios[i].initial, engine::State::Lvm { .. }) {
            self.lvm_presentation.reset_all();
        }
        self.invalidate_apache_presentation();
        self.nat_presentation.clear();
        self.rpm_presentation.clear();
        self.nfs_presentation.clear();
        self.ether_presentation.clear();
        self.physical_presentation.clear();
        self.cron_presentation.clear();
        self.one_off_presentation.clear();
        self.backup_presentation.reset_all();
        self.physical_presentation.reset_all();
        let s = &self.scenarios[i];
        self.completion_index.forget(&s.id);
        if matches!(s.initial, engine::State::OneOff { .. }) {
            if let Some(a) = self.progress.attempts.get_mut(&s.id)
                && let Err(error) =
                    dlms_learning_lab::one_off_workspace::budget::reset_attempt(a, s)
            {
                self.notice = Some(error);
                return;
            }
            self.one_off_presentation.memo.changed(&s.id);
            self.dirty = true;
            self.reset_pending = false;
            self.lesson_scroll_reset = true;
            return;
        }
        self.address_worked.remove(&s.id);
        if matches!(s.initial, engine::State::MacLearning { .. }) {
            self.mac_trace_reset = Some(i);
        }
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        *a = Attempt::new(s, a.visited);
        model::prepare(a);
        Self::prepare_apache_static(a, s, true);
        Self::prepare_rpm(a, s, true);
        Self::prepare_nfs(a, s, true);
        Self::prepare_ether(a, s, true);
        Self::prepare_cron(a, s);
        Self::prepare_one_off(a, s);
        Self::prepare_physical(a, s, true);
        Self::prepare_backup(a, s, true);
        if let Some(lesson) = &s.acl_teaching {
            dlms_learning_lab::acl_teaching::prepare(a, lesson, true)
                .expect("validated ACL lesson");
        }
        if let Some(lesson) = &s.storage_teaching {
            dlms_learning_lab::storage_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.discovery_teaching {
            discovery_workspace::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.teaching {
            teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.client_teaching {
            dlms_learning_lab::client_teaching::prepare(a, lesson, true)
                .expect("validated client teaching");
        }
        if let Some(lesson) = &s.seed_dns_teaching {
            dlms_learning_lab::seed_dns_teaching::prepare(a, lesson, true)
                .expect("validated seed DNS teaching");
        }
        if let Some(lesson) = &s.seed_mount_teaching {
            dlms_learning_lab::seed_mount_teaching::prepare(a, lesson, true)
                .expect("validated seed mount teaching");
        }
        if let Some(lesson) = &s.legacy_access_teaching {
            dlms_learning_lab::legacy_access_teaching::prepare(a, lesson, true)
                .expect("validated access lesson");
        }
        if let Some(lesson) = &s.legacy_logs_teaching {
            dlms_learning_lab::legacy_logs_teaching::prepare(a, lesson, true)
                .expect("validated log lesson");
        }
        if let Some(lesson) = &s.service_teaching {
            dlms_learning_lab::service_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.dns_teaching {
            dlms_learning_lab::dns_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.address_teaching {
            dlms_learning_lab::address_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.terminal_teaching {
            dlms_learning_lab::terminal_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.process_teaching {
            dlms_learning_lab::process_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.debian_packages_teaching {
            dlms_learning_lab::debian_packages_teaching::prepare(a, lesson, true)
                .expect("validated package lesson");
        }
        if let Some(lesson) = &s.nat_teaching {
            dlms_learning_lab::nat_teaching::prepare(a, lesson, true)
                .expect("validated NAT teaching");
        }
        if let Some(lesson) = &s.dhcpv4_teaching {
            dlms_learning_lab::dhcpv4_teaching::prepare(a, lesson, true)
                .expect("validated DHCP teaching");
            Self::dhcpv4_initial_selection(a, s);
        }
        if let Some(lesson) = &s.spanning_tree_teaching {
            dlms_learning_lab::spanning_tree_teaching::prepare(a, lesson)
                .expect("validated spanning-tree lesson");
            Self::spanning_tree_initial_selection(a, s);
        }
        if let Some(lesson) = &s.ospf_teaching {
            dlms_learning_lab::ospf_teaching::prepare(a, lesson, true)
                .expect("validated OSPF lesson");
            Self::ospf_initial_selection(a, s);
        }
        if let Some(lesson) = &s.accounts_teaching {
            dlms_learning_lab::accounts_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.unit_manager_teaching {
            dlms_learning_lab::unit_manager_teaching::prepare(a, lesson, true)
                .expect("validated unit-manager lesson");
        }
        if let Some(lesson) = &s.mac_teaching {
            dlms_learning_lab::mac_learning_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.switching_teaching {
            dlms_learning_lab::switching_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.ipv6_teaching {
            dlms_learning_lab::ipv6_teaching::prepare(a, lesson, true);
        }
        if let Some(lesson) = &s.ipv4_routing_teaching {
            dlms_learning_lab::ipv4_routing_teaching::prepare(a, lesson, true);
        }
        if let engine::Goal::Ipv4Routing { requirements } = &s.goal
            && let Some(device) =
                dlms_learning_lab::ipv4_routing_teaching::preferred_device(requirements)
        {
            a.workspace
                .as_mut()
                .unwrap()
                .routing_v4_draft
                .as_mut()
                .unwrap()
                .selected_device = device;
        }
        if let engine::Goal::Ipv6 { requirements } = &s.goal
            && let Some(device) = dlms_learning_lab::ipv6_teaching::preferred_device(requirements)
        {
            a.workspace
                .as_mut()
                .unwrap()
                .ipv6_draft
                .as_mut()
                .unwrap()
                .selected_device = device;
        }
        self.script_job = None;
        self.input.clear();
        self.checked = false;
        self.reset_pending = false;
        self.notice = None;
        self.focus_input = true;
        self.scroll_command = false;
        self.help_open = false;
        self.host_activity = false;
        self.load_ipv4_view(i);
        self.load_address_example(i);
        self.dirty = true;
    }
    fn save(&mut self) {
        if self.dirty && !self.read_only {
            match progress::save_with_catalog(&self.path, &self.progress, &self.scenarios) {
                Ok(saved) => {
                    self.dirty = false;
                    self.storage_error = saved.durability_warning
                }
                Err(e) => self.storage_error = Some(format!("Progress not saved: {e}")),
            }
        }
    }
    fn text_size_picker(&mut self, ui: &mut egui::Ui) {
        let observe_picker = self.reopen.is_some()
            || self.physical_capture_active()
            || self.one_off_capture_active()
            || self.lvm_capture_active();
        ui.label(RichText::new("Text size").small().color(palette(ui).muted));
        let r = egui::ComboBox::from_id_salt("text-size")
            .width(84.)
            .selected_text(format!("{:.0}%", self.scale * 100.))
            .show_ui(ui, |ui| {
                for scale in [1., 1.15, 1.25, 1.5, 2.] {
                    let item = ui.selectable_value(
                        &mut self.scale,
                        scale,
                        format!("{:.0}%", scale * 100.),
                    );
                    if observe_picker {
                        self.metrics.reopen_scale_items.push((scale, item.rect));
                        self.metrics.physical_picker_choices.push((
                            format!("scale:{scale:.2}"),
                            item.rect,
                            ui.clip_rect(),
                        ));
                    }
                }
            });
        name_control(&r.response, "Text size");
        if observe_picker {
            self.metrics.reopen_scale = Some(r.response.rect);
        }
        self.metrics.header.push(r.response.rect);
    }
    fn theme_picker(&mut self, ui: &mut egui::Ui) {
        let reserve = controls::button_width(ui, "Settings") + controls::GAP;
        let minimum =
            ui.text_style_height(&egui::TextStyle::Body) * 4. + 2. * controls::PAD_X as f32;
        let width = (ui.available_size_before_wrap().x - reserve).clamp(minimum, 195.);
        self.theme_dropdown_width(ui, width);
        let r = ui.button("Settings");
        self.metrics.settings = Some(r.rect);
        if r.clicked() {
            self.settings_open = !self.settings_open;
        }
    }
    fn theme_dropdown(&mut self, ui: &mut egui::Ui) {
        self.theme_dropdown_width(ui, 195.);
    }
    fn theme_dropdown_width(&mut self, ui: &mut egui::Ui, width: f32) {
        let observe_picker = self.reopen.is_some()
            || self.physical_capture_active()
            || self.one_off_capture_active()
            || self.lvm_capture_active();
        let r = controls::cell(ui, width, |ui| {
            egui::ComboBox::from_id_salt("native-theme-picker")
                .width(width)
                .truncate()
                .selected_text(theme::find(&self.theme_id).name.clone())
                .show_ui(ui, |ui| {
                    for t in theme::inventory() {
                        let item = ui.selectable_value(
                            &mut self.theme_id,
                            t.id.clone(),
                            format!("{} · {}", t.group, t.name),
                        );
                        if observe_picker {
                            self.metrics
                                .reopen_theme_items
                                .push((t.id.clone(), item.rect));
                            self.metrics.physical_picker_choices.push((
                                format!("theme:{}", t.id),
                                item.rect,
                                ui.clip_rect(),
                            ));
                        }
                    }
                })
        })
        .inner;
        name_control(&r.response, "Application theme");
        if observe_picker {
            self.metrics.reopen_theme = Some(r.response.rect);
        }
        self.metrics.header.push(r.response.rect);
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("brand")
            .frame(
                egui::Frame::new()
                    .fill(theme::roles(ctx).panel)
                    .inner_margin(egui::Margin::symmetric(20, 10)),
            )
            .show(ctx, |ui| {
                let narrow = ui.available_width() < 1050.;
                ui.horizontal_wrapped(|ui| {
                    self.metrics.header.push(
                        ui.label(
                            RichText::new("DLMS Learning Lab")
                                .strong()
                                .size(20.)
                                .color(palette(ui).accent),
                        )
                        .rect,
                    );
                    self.metrics.header.push(
                        ui.label(
                            RichText::new("Offline simulation")
                                .small()
                                .color(palette(ui).muted),
                        )
                        .rect,
                    );
                    if !narrow {
                        self.text_size_picker(ui);
                        self.theme_picker(ui);
                    }
                });
                if narrow {
                    ui.horizontal_wrapped(|ui| {
                        self.text_size_picker(ui);
                        self.theme_picker(ui);
                    });
                }
            });
    }
    fn sync_appearance(&mut self, ctx: &egui::Context, previous: (String, f32)) {
        if previous != (self.theme_id.clone(), self.scale) {
            theme::apply(ctx, &self.theme_id);
            if !self.preferences_read_only {
                self.preferences_error = theme::Preferences {
                    version: 1,
                    theme: self.theme_id.clone(),
                    text_scale: self.scale,
                }
                .save(&self.path.with_file_name("preferences-v1.json"))
                .err();
            }
        }
    }
    fn course_entry(&mut self, ui: &mut egui::Ui) {
        if let Some(unit) = self
            .units
            .iter()
            .find(|u| u.id == self.filter.unit)
            .cloned()
        {
            let done = unit
                .labs
                .iter()
                .filter(|id| {
                    self.progress.attempts.get(*id).is_some_and(|a| {
                        self.scenarios
                            .iter()
                            .find(|s| &s.id == *id)
                            .is_some_and(|s| self.nfs_visible_solved(a, s))
                    })
                })
                .count();
            let earlier = unit
                .labs
                .iter()
                .filter(|id| {
                    self.progress.attempts.get(*id).is_some_and(|a| {
                        self.scenarios
                            .iter()
                            .find(|s| &s.id == *id)
                            .is_some_and(|s| {
                                !self.nfs_visible_solved(a, s)
                                    && progress::legacy_script_credit(a, s)
                            })
                    })
                })
                .count();
            let pending = unit
                .labs
                .iter()
                .filter(|id| {
                    self.progress.attempts.get(*id).is_some_and(|a| {
                        self.scenarios
                            .iter()
                            .find(|s| &s.id == *id)
                            .is_some_and(|s| {
                                self.completion_index.status(a, s)
                                    == Some(completion_index::Status::Pending)
                            })
                    })
                })
                .count();
            ui.label(if pending > 0 {
                format!(
                    "{} · {done}/{} verified complete · {pending} pending verification{}",
                    unit.title,
                    unit.labs.len(),
                    if earlier > 0 {
                        format!(" · {earlier} earlier completions retained")
                    } else {
                        String::new()
                    }
                )
            } else if earlier > 0 {
                format!(
                    "{} · {done}/{} currently verified · {earlier} earlier completions retained",
                    unit.title,
                    unit.labs.len()
                )
            } else {
                format!("{} · {done}/{} labs complete", unit.title, unit.labs.len())
            });
            ui.label(&unit.summary);
            if !unit.prerequisites.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "Suggested preparation: {}. You can still open any lab.",
                        unit.prerequisites
                            .iter()
                            .filter_map(|id| self.units.iter().find(|u| &u.id == id))
                            .map(|u| u.title.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .small()
                    .color(palette(ui).muted),
                );
            }
        } else if !self.filter.track.is_empty()
            && self.filter.search.is_empty()
            && !self.filter.favorites
            && !self.filter.recent
        {
            let next = self
                .units
                .iter()
                .filter(|u| u.track == self.filter.track)
                .flat_map(|u| u.labs.iter())
                .find_map(|id| {
                    self.scenarios.iter().position(|s| {
                        &s.id == id
                            && self
                                .progress
                                .attempts
                                .get(id)
                                .is_none_or(|a| !self.nfs_visible_solved(a, s))
                    })
                });
            ui.horizontal_wrapped(|ui| {
                ui.label("Follow the course order, or choose any lab below.");
                if let Some(i) = next {
                    let r = ui.add(button(
                        if self
                            .progress
                            .attempts
                            .get(&self.scenarios[i].id)
                            .is_some_and(|a| progress::legacy_script_credit(a, &self.scenarios[i]))
                        {
                            "Verify earlier run"
                        } else if self.progress.attempts.contains_key(&self.scenarios[i].id) {
                            "Continue course"
                        } else {
                            "Start course"
                        },
                    ));
                    self.metrics.course_start = Some(r.rect);
                    if r.clicked() {
                        self.launch(i);
                    }
                }
            });
        }
    }
    fn next_unit_lab(&mut self, ui: &mut egui::Ui, i: usize) {
        let Some(unit) = self.units.iter().find(|u| u.id == self.scenarios[i].unit) else {
            return;
        };
        let next = if matches!(
            self.scenarios[i].initial,
            engine::State::Cron { .. }
                | engine::State::FileBackup { .. }
                | engine::State::PhysicalLink { .. }
                | engine::State::OneOff { .. }
        ) {
            let position = unit
                .labs
                .iter()
                .position(|id| id == &self.scenarios[i].id)
                .unwrap();
            unit.labs
                .iter()
                .skip(position + 1)
                .chain(unit.labs.iter().take(position))
                .filter_map(|id| self.scenarios.iter().position(|s| &s.id == id))
                .find(|at| {
                    self.progress
                        .attempts
                        .get(&self.scenarios[*at].id)
                        .is_none_or(|a| !self.nfs_visible_solved(a, &self.scenarios[*at]))
                })
        } else {
            course::next(unit, &self.scenarios[i].id, &self.scenarios, &self.progress)
        };
        if let Some(next) = next {
            let r = ui.link(format!("Next in unit: {}", self.scenarios[next].title));
            self.metrics.next_unit = Some(r.rect);
            if r.clicked() {
                self.launch(next);
            }
        } else {
            let r = ui.label("Unit complete · All its current assignments pass.");
            self.metrics.unit_complete = Some(r.rect);
            let r = ui.link("Back to this unit");
            self.metrics.unit_back = Some(r.rect);
            if r.clicked() {
                self.filter.track = unit.track.clone();
                self.filter.unit = unit.id.clone();
                self.filter.topic.clear();
                self.filter.search.clear();
                self.selected = None;
                self.invalidate_apache_presentation();
                self.nat_presentation.clear();
                self.rpm_presentation.clear();
                self.nfs_presentation.clear();
                self.ether_presentation.clear();
                self.physical_presentation.clear();
                self.cron_presentation.clear();
                self.one_off_presentation.clear();
                self.backup_presentation.clear();
            }
        }
    }
    fn library(&mut self, ui: &mut egui::Ui) {
        self.nat_library_prepass();
        ui.heading("Practice library");
        ui.add_space(2.);
        ui.horizontal_wrapped(|ui| {
            for (label, value) in [
                ("All tracks", ""),
                ("CCNA", "CCNA"),
                ("LPIC-1", "LPIC-1"),
                ("LPIC-2", "LPIC-2"),
                ("Bash/Shell", "Bash/Shell"),
            ] {
                if ui
                    .selectable_label(self.filter.track == value, label)
                    .clicked()
                {
                    if self.filter.track != value {
                        self.filter.unit.clear();
                        self.filter.topic.clear();
                    }
                    self.filter.track = value.into();
                    self.page = 0;
                }
            }
            if ui
                .toggle_value(&mut self.filter.favorites, "Favorites")
                .changed()
            {
                self.page = 0;
            }
            if ui
                .toggle_value(&mut self.filter.recent, "Recent / continue")
                .changed()
            {
                self.page = 0;
            }
        });
        let search = ui.add_sized(
            [ui.available_width(), controls::height(ui)],
            controls::singleline(ui, &mut self.filter.search)
                .hint_text("Search titles, goals or topics…")
                .char_limit(200),
        );
        name_control(&search, "Search labs by title, goal or topic");
        self.metrics.library_search = Some(search.rect);
        if search.changed() {
            self.page = 0;
        }
        self.course_entry(ui);
        ui.horizontal_wrapped(|ui| {
            let previous = self.filter.unit.clone();
            let chosen = self
                .units
                .iter()
                .find(|u| u.id == self.filter.unit)
                .map_or("Any unit", |u| u.title.as_str());
            let unit_width = 220_f32.min(ui.available_width());
            let r = ui
                .allocate_ui_with_layout(
                    egui::vec2(unit_width, controls::height(ui)),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_width(unit_width);
                        egui::ComboBox::from_id_salt("unit-filter")
                            .width(220_f32.min(ui.available_width()))
                            .truncate()
                            .selected_text(chosen)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.filter.unit,
                                    String::new(),
                                    "Any unit",
                                );
                                for u in self.units.iter().filter(|u| {
                                    self.filter.track.is_empty() || u.track == self.filter.track
                                }) {
                                    let r = ui.selectable_value(
                                        &mut self.filter.unit,
                                        u.id.clone(),
                                        &u.title,
                                    );
                                    self.metrics.unit_choices.push((u.id.clone(), r.rect));
                                }
                                self.metrics.unit_popup = Some(controls::popup_content_clip(ui));
                            })
                    },
                )
                .inner;
            self.metrics.unit_picker = Some(r.response.rect);
            name_control(&r.response, "Course unit filter");
            self.metrics.filters.push(r.response.rect);
            if previous != self.filter.unit {
                self.filter.topic.clear();
                self.page = 0;
            }
            let topics: std::collections::BTreeSet<_> = self
                .scenarios
                .iter()
                .filter(|s| {
                    (self.filter.track.is_empty() || s.track == self.filter.track)
                        && (self.filter.unit.is_empty() || s.unit == self.filter.unit)
                })
                .map(|s| s.topic.clone())
                .collect();
            let before = self.filter.topic.clone();
            let r = combo(
                ui,
                "topic",
                "Any topic",
                &mut self.filter.topic,
                topics.iter().map(String::as_str),
                220_f32.min(ui.available_width()),
            );
            self.metrics.filters.push(r.rect);
            if before != self.filter.topic {
                self.page = 0;
            }
            let count = usize::from(!self.filter.difficulty.is_empty())
                + usize::from(!self.filter.status.is_empty())
                + usize::from(self.filter.max_minutes > 0);
            let r = ui.add(button(&if count > 0 {
                format!("More filters ({count})")
            } else {
                "More filters".into()
            }));
            self.metrics.extras = Some(r.rect);
            if r.clicked() {
                self.extra_filters = !self.extra_filters;
            }
            if !self.extra_filters
                && (!self.filter.search.is_empty()
                    || !self.filter.track.is_empty()
                    || !self.filter.topic.is_empty()
                    || !self.filter.unit.is_empty()
                    || count > 0
                    || self.filter.favorites
                    || self.filter.recent)
                && ui.add(button("Clear filters")).clicked()
            {
                self.filter = Filter::default();
                self.page = 0;
            }
        });
        if self.extra_filters {
            self.filters(ui);
        } else {
            let mut active = Vec::new();
            if !self.filter.difficulty.is_empty() {
                active.push(self.filter.difficulty.clone());
            }
            if !self.filter.status.is_empty() {
                active.push(self.filter.status.clone());
            }
            if self.filter.max_minutes > 0 {
                active.push(format!("Up to {} min", self.filter.max_minutes));
            }
            if !active.is_empty() {
                ui.label(
                    RichText::new(active.join(" · "))
                        .small()
                        .color(palette(ui).muted),
                );
            }
        }
        ui.add_space(4.);
        ui.separator();
        if self.developer && self.synthetic && self.synthetic_catalog.is_empty() {
            self.synthetic_catalog = catalog::synthetic_catalog(&self.scenarios, 900);
        }
        let source = if self.developer && self.synthetic {
            &self.synthetic_catalog
        } else {
            &self.scenarios
        };
        let mut items: Vec<_> = source
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                let a = self.progress.attempts.get(&s.id);
                catalog::matches_verification(
                    s,
                    &self.filter,
                    a.is_some(),
                    a.and_then(|a| {
                        self.completion_index
                            .status(a, s)
                            .map(|v| v.result())
                            .unwrap_or_else(|| Some(self.nfs_visible_solved(a, s)))
                    }),
                    self.progress.favorites.contains(&s.id),
                )
            })
            .collect();
        if self.filter.recent {
            items.sort_by_key(|(_, s)| {
                std::cmp::Reverse(self.progress.attempts.get(&s.id).map_or(0, |a| a.visited))
            });
        }
        self.completion_index.page(
            source
                .iter()
                .filter(|s| {
                    catalog::matches_filters(
                        s,
                        &self.filter,
                        self.progress.attempts.contains_key(&s.id),
                        self.progress.favorites.contains(&s.id),
                    )
                })
                .filter(|s| {
                    self.completion_index.can_queue(&s.id)
                        && self.progress.attempts.get(&s.id).is_some_and(|a| {
                            self.completion_index.status(a, s)
                                == Some(completion_index::Status::Pending)
                        })
                })
                .take(12)
                .map(|s| s.id.clone()),
        );
        if !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open()
            && self
                .completion_index
                .pending(&self.progress, &self.scenarios)
                .is_some()
        {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        self.page = self.page.min(items.len().saturating_sub(1) / 12);
        self.metrics.pagination = items.len() > 12;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    items.len(),
                    if self.synthetic {
                        "synthetic previews"
                    } else {
                        if items.len() == 1 { "lab" } else { "labs" }
                    }
                ))
                .color(palette(ui).muted),
            );
            if self.metrics.pagination {
                if ui
                    .add_enabled(self.page > 0, button("Previous page"))
                    .clicked()
                {
                    self.page -= 1;
                }
                ui.label(format!(
                    "Page {} / {}",
                    self.page + 1,
                    items.len().div_ceil(12)
                ));
                if ui
                    .add_enabled((self.page + 1) * 12 < items.len(), button("Next page"))
                    .clicked()
                {
                    self.page += 1;
                }
            }
        });
        if items.is_empty() {
            ui.label("No labs match. Clear a filter or try another search.");
        }
        let columns = if ui.available_width() >= 980. { 2 } else { 1 };
        let mut launch = None;
        // Read current results for only this page before mutable card controls.
        // This keeps the complete memo key without borrowing the entire app
        // from inside a closure that also edits favorites/navigation.
        let page_items = items
            .iter()
            .skip(self.page * 12)
            .take(12)
            .map(|(i, s)| {
                let complete = self
                    .progress
                    .attempts
                    .get(&s.id)
                    .is_some_and(|a| file_backup_workspace::solved_with_memos(self, a, s));
                let verification = self
                    .progress
                    .attempts
                    .get(&s.id)
                    .and_then(|a| self.completion_index.status(a, s));
                (*i, *s, complete, verification)
            })
            .collect::<Vec<_>>();
        for row in page_items.chunks(columns) {
            ui.columns(columns, |uis| {
                for (col, (i, s, complete, verification)) in row.iter().enumerate() {
                    let ui = &mut uis[col];
                    card(ui).show(ui, |ui| {
                        ui.set_min_width((ui.available_width() - 1.).max(0.));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(format!("{} / {}", s.track, s.topic))
                                    .small()
                                    .color(palette(ui).accent),
                            );
                            ui.label(
                                RichText::new(format!("{} · {} min", s.difficulty, s.minutes))
                                    .small()
                                    .color(palette(ui).muted),
                            );
                        });
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 28.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_min_height(28.);
                                ui.label(RichText::new(&s.title).strong().size(21.));
                            },
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 42.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_min_height(42.);
                                ui.label(&s.summary);
                            },
                        );
                        let a = self.progress.attempts.get(&s.id);
                        let complete = *complete;
                        let favorite = self.progress.favorites.contains(&s.id);
                        let status = if *verification == Some(completion_index::Status::Pending) {
                            "Verification pending · Open to check saved work"
                        } else if *verification == Some(completion_index::Status::Incomplete) {
                            "Incomplete · Continue working"
                        } else if a.is_some_and(|a| progress::legacy_script_credit(a, s))
                            && !complete
                        {
                            "Earlier run complete · Rerun to verify"
                        } else if a.is_some_and(progress::legacy_credit) && !complete {
                            "Earlier analysis complete · New requirement remaining"
                        } else if complete {
                            "Complete"
                        } else if a.is_some() {
                            "In progress"
                        } else {
                            "Not started"
                        };
                        let mut rects = None;
                        ui.horizontal_wrapped(|ui| {
                            let start = ui.add_enabled(
                                !self.synthetic,
                                primary(
                                    ui,
                                    if self.synthetic {
                                        "Preview only"
                                    } else if a.is_none() {
                                        "Start lab"
                                    } else {
                                        "Continue"
                                    },
                                )
                                .min_size(egui::vec2(108., controls::height(ui))),
                            );
                            name_control(
                                &start,
                                format!(
                                    "{}: {}",
                                    if self.synthetic {
                                        "Preview only"
                                    } else if a.is_none() {
                                        "Start lab"
                                    } else {
                                        "Continue lab"
                                    },
                                    s.title
                                ),
                            );
                            if start.clicked() {
                                launch = Some(*i);
                            }
                            let fav = ui.add_enabled(
                                !self.synthetic,
                                button(if favorite { "Favorited" } else { "Favorite" })
                                    .min_size(egui::vec2(108., controls::height(ui))),
                            );
                            name_control(
                                &fav,
                                format!(
                                    "{}: {}",
                                    if favorite {
                                        "Remove favorite"
                                    } else {
                                        "Add favorite"
                                    },
                                    s.title
                                ),
                            );
                            if fav.clicked() {
                                if favorite {
                                    self.progress.favorites.remove(&s.id);
                                } else {
                                    self.progress.favorites.insert(s.id.clone());
                                }
                                self.dirty = true;
                            }
                            rects = Some((start.rect, fav.rect));
                            let status_response =
                                ui.label(RichText::new(status).small().color(if complete {
                                    palette(ui).success
                                } else {
                                    palette(ui).muted
                                }));
                            self.metrics.card_verification.push((
                                s.id.clone(),
                                *verification,
                                status_response.rect,
                            ));
                        });
                        if let Some(rects) = rects {
                            self.metrics.card_actions.push(rects);
                        }
                    });
                    ui.add_space(4.);
                }
            });
        }
        if let Some(i) = launch {
            self.launch(i);
        }
        ui.label(
            RichText::new("Illustrative practice • not full exam coverage")
                .small()
                .color(palette(ui).muted),
        );
        if self.developer {
            egui::CollapsingHeader::new("Developer options").show(ui, |ui| {
                if ui
                    .checkbox(
                        &mut self.synthetic,
                        "900 synthetic catalog previews (not playable labs)",
                    )
                    .changed()
                {
                    self.page = 0;
                }
            });
        }
    }
    fn filters(&mut self, ui: &mut egui::Ui) {
        let old = (
            self.filter.topic.clone(),
            self.filter.difficulty.clone(),
            self.filter.status.clone(),
            self.filter.max_minutes,
        );
        let columns = (((ui.available_width() + 10.) / 174.).floor() as usize).clamp(1, 4);
        let width =
            ((ui.available_width() - 10. * (columns - 1) as f32) / columns as f32).min(240.);
        egui::Grid::new("library-filters")
            .num_columns(columns)
            .spacing([10., 10.])
            .show(ui, |ui| {
                for index in 0..4 {
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, controls::height(ui)),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let response = match index {
                                0 => combo(
                                    ui,
                                    "difficulty",
                                    "Any difficulty",
                                    &mut self.filter.difficulty,
                                    pack::DIFFICULTIES.into_iter(),
                                    width,
                                ),
                                1 => combo(
                                    ui,
                                    "status",
                                    "Any status",
                                    &mut self.filter.status,
                                    [
                                        "Not started",
                                        "In progress",
                                        "Complete",
                                        "Verification pending",
                                    ]
                                    .into_iter(),
                                    width,
                                ),
                                2 => {
                                    egui::ComboBox::from_id_salt("duration")
                                        .width(width)
                                        .selected_text(if self.filter.max_minutes == 0 {
                                            "Any duration".into()
                                        } else {
                                            format!("Up to {} min", self.filter.max_minutes)
                                        })
                                        .show_ui(ui, |ui| {
                                            for (v, label) in [
                                                (0, "Any duration"),
                                                (6, "Up to 6 min"),
                                                (8, "Up to 8 min"),
                                                (10, "Up to 10 min"),
                                                (20, "Up to 20 min"),
                                                (30, "Up to 30 min"),
                                                (60, "Up to 60 min"),
                                                (120, "Up to 120 min"),
                                                (180, "Up to 180 min"),
                                            ] {
                                                ui.selectable_value(
                                                    &mut self.filter.max_minutes,
                                                    v,
                                                    label,
                                                );
                                            }
                                        })
                                        .response
                                }
                                _ => {
                                    let r = ui.add_sized(
                                        [width, controls::height(ui)],
                                        button("Clear filters"),
                                    );
                                    if r.clicked() {
                                        self.filter = Filter::default();
                                        self.page = 0;
                                    }
                                    r
                                }
                            };
                            if index == 2 {
                                name_control(&response, "Estimated time filter");
                            }
                            self.metrics.filters.push(response.rect);
                        },
                    );
                    if (index + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
        if old
            != (
                self.filter.topic.clone(),
                self.filter.difficulty.clone(),
                self.filter.status.clone(),
                self.filter.max_minutes,
            )
        {
            self.page = 0;
        }
    }
    fn render(&mut self, ctx: &egui::Context) {
        self.metrics = Metrics::default();
        let completion_root_frame = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        self.completion_index.begin_frame(completion_root_frame);
        self.nat_presentation.memo.begin_frame();
        self.rpm_presentation.memo.begin_frame();
        self.nfs_presentation.memo.begin_frame();
        self.ether_presentation.memo.begin_frame();
        self.cron_presentation.memo.begin_frame();
        ctx.set_zoom_factor(self.scale);
        controls::configure(ctx);
        self.advance_scripts(ctx);
        let previous = (self.theme_id.clone(), self.scale);
        self.header(ctx);
        self.appearance_preview(ctx);
        let nfs_selected_before_restore = self.selected;
        self.data_dialog(ctx);
        if nfs_selected_before_restore.is_some() && self.selected.is_none() {
            self.completion_index.clear();
            self.nfs_presentation.clear();
            self.ether_presentation.clear();
            self.physical_presentation.clear();
            self.cron_presentation.clear();
            self.one_off_presentation.clear();
            self.backup_presentation.reset_all();
            self.physical_presentation.reset_all();
        }
        self.sync_appearance(ctx, previous);
        self.rpm_prepass();
        self.nfs_prepass();
        self.ether_prepass();
        self.advance_cron(ctx);
        self.cron_prepass(ctx);
        self.one_off_prepass(ctx);
        self.advance_backup(ctx);
        self.backup_prepass(ctx);
        self.physical_library_prepass(ctx);
        if let Some(error) = &self.storage_error {
            egui::TopBottomPanel::bottom("storage-error").show(ctx, |ui| {
                ui.colored_label(palette(ui).error, error);
                ui.label(if self.read_only {
                    "Original progress retained; this session cannot overwrite it."
                } else {
                    "Changes remain in memory until saved."
                });
                if !self.read_only && ui.button("Retry save").clicked() {
                    self.dirty = true;
                }
            });
        }
        // Preserve a full native control row when a short window and large text
        // leave little room below the toolbar. Font sizes/control heights stay
        // unchanged; normal windows retain the existing generous margins.
        let control_height = ctx.style().spacing.interact_size.y;
        let vertical_margin = ((ctx.available_rect().height() - control_height) / 2.)
            .clamp(0., 18.)
            .floor() as i8;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::roles(ctx).background)
                    .inner_margin(egui::Margin::symmetric(18, vertical_margin)),
            )
            .show(ctx, |ui| {
                let mut area = egui::ScrollArea::vertical()
                    .id_salt(("main", self.selected))
                    .min_scrolled_height(0.);
                self.metrics.main_available_height = ui.available_height();
                if std::mem::take(&mut self.lesson_scroll_reset) {
                    area = area.vertical_scroll_offset(0.);
                }
                let scroll = area.auto_shrink([false, false]).show(ui, |ui| {
                    self.metrics.main_content_top = ui.cursor().top();
                    let width = ui.available_width().min(if self.selected.is_some() {
                        1060.
                    } else {
                        1280.
                    });
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(width, 0.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(width);
                                ui.add_enabled_ui(!self.data_tools.modal_open(), |ui| {
                                    if let Some(i) = self.selected {
                                        self.workspace(ui, i)
                                    } else {
                                        self.library(ui)
                                    }
                                });
                            },
                        );
                    });
                });
                self.metrics.main_viewport = Some(scroll.inner_rect);
                self.metrics.main_scroll_offset = scroll.state.offset;
                self.metrics.main_content_size = scroll.content_size;
            });
        if self.host_activity
            && let Some(i) = self.selected
        {
            self.host_bits(ctx, i);
        }
        // egui shares selection foreground with TextEdit's built-in focus border.
        // Paint a separate contrasting native focus ring so both remain readable.
        if let Some(id) = ctx.memory(|m| m.focused())
            && let Some(response) = ctx.read_response(id)
        {
            ctx.layer_painter(response.layer_id).rect_stroke(
                response.interact_rect,
                4_u8,
                egui::Stroke::new(2_f32, theme::roles(ctx).accent),
                egui::StrokeKind::Inside,
            );
        }
    }
}
fn card(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(palette(ui).panel)
        .stroke(egui::Stroke::new(1_f32, palette(ui).border))
        .inner_margin(16.)
        .corner_radius(10.)
}
fn button(text: &str) -> egui::Button<'_> {
    egui::Button::new(text).corner_radius(6.)
}
fn primary<'a>(ui: &egui::Ui, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(RichText::new(text).color(palette(ui).on_accent))
        .corner_radius(6.)
        .fill(palette(ui).accent)
        .stroke(egui::Stroke::new(1_f32, palette(ui).border))
}
fn combo<'a>(
    ui: &mut egui::Ui,
    id: &str,
    any: &str,
    value: &mut String,
    options: impl Iterator<Item = &'a str>,
    width: f32,
) -> egui::Response {
    let response = ui
        .allocate_ui_with_layout(
            egui::vec2(width, controls::height(ui)),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_width(width);
                egui::ComboBox::from_id_salt(id)
                    .width(width)
                    .truncate()
                    .selected_text(if value.is_empty() {
                        any.to_owned()
                    } else {
                        value.clone()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(value, String::new(), any);
                        for option in options {
                            ui.selectable_value(value, option.to_owned(), option);
                        }
                    })
                    .response
            },
        )
        .inner;
    name_control(
        &response,
        match id {
            "topic" => "Topic filter",
            "difficulty" => "Difficulty filter",
            "status" => "Completion status filter",
            _ => "Catalog filter",
        },
    );
    response
}
impl eframe::App for LabApp {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        if self.smoke || self.reopen.is_some() {
            if let Some(modifiers) = self.pending_events.iter().rev().find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            }) {
                input.modifiers = modifiers;
            }
            input.events.append(&mut self.pending_events);
        }
    }
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        crate::startup_trace::frame(ctx, false);
        if self.reopen.is_some() {
            self.reopen_capture_events(ctx);
        } else if self.smoke {
            self.capture_events(ctx);
        }
        self.render(ctx);
        self.save();
        crate::startup_trace::frame(ctx, true);
        if self.reopen.is_some() {
            self.reopen_tick(ctx);
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        } else if self.smoke {
            self.smoke_tick(ctx);
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}

/// Add a contextual native name without replacing the widget's role, value or state.
fn name_control(response: &egui::Response, label: impl Into<String>) {
    response
        .ctx
        .accesskit_node_builder(response.id, |node| node.set_label(label.into()));
}
fn palette(ui: &egui::Ui) -> theme::Roles {
    theme::roles(ui.ctx())
}

impl LabApp {
    fn prepare_physical(a: &mut Attempt, s: &Scenario, new: bool) {
        if matches!(s.initial, engine::State::PhysicalLink { .. }) {
            let _ = dlms_learning_lab::physical_link_workspace::prepare_attempt(a, s, new);
        }
    }
}
