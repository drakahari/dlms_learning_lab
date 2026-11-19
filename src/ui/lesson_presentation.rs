//! Shared instructional hierarchy; rendering only, never simulation or grading.
use super::*;
use std::collections::BTreeMap;
pub(super) mod paint;

#[derive(Clone, serde::Deserialize)]
pub(super) struct PracticeStep {
    pub title: String,
    pub action: String,
    pub observe: String,
}
#[derive(Clone, serde::Deserialize)]
pub(super) struct Entry {
    pub id: String,
    pub goal: String,
    pub requirements: Vec<String>,
    pub practice: Vec<PracticeStep>,
    #[serde(default)]
    pub worked: Vec<WorkedRow>,
    #[serde(default)]
    pub concepts: Vec<Concept>,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub prerequisite_sources: Vec<String>,
    /// Opt-in compact worked-command browser; never executes or fills on selection.
    #[serde(default)]
    pub command_picker: bool,
}
#[derive(Clone, serde::Deserialize)]
pub(super) struct Concept {
    pub title: String,
    pub text: String,
}
#[derive(Clone, serde::Deserialize)]
pub(super) struct WorkedRow {
    pub label: String,
    pub value: String,
    pub explanation: String,
}
pub(super) fn entries() -> &'static BTreeMap<String, Entry> {
    static ENTRIES: std::sync::OnceLock<BTreeMap<String, Entry>> = std::sync::OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut out = BTreeMap::new();
        for data in [
            include_str!("../../scenarios/presentation/ccna.json"),
            include_str!("../../scenarios/presentation/lpic1.json"),
            include_str!("../../scenarios/presentation/lpic2.json"),
            include_str!("../../scenarios/presentation/bash.json"),
        ] {
            for entry in
                serde_json::from_str::<Vec<Entry>>(data).expect("reviewed presentation data")
            {
                assert!(
                    out.insert(entry.id.clone(), entry).is_none(),
                    "duplicate presentation ID"
                );
            }
        }
        apply_foundation_notes(&mut out);
        let raw = include_str!("../../scenarios/presentation/lpic1_direct_dns.json");
        assert!(raw.len() <= 16384);
        let expanded: Entry = serde_json::from_str(raw).expect("reviewed direct DNS teaching");
        assert_eq!(expanded.id, "lpic1-host-resolver");
        let old = out.get(&expanded.id).expect("Existing resolver lesson");
        assert_eq!(old.requirements, expanded.requirements, "No expanded grading requirements");
        out.insert(expanded.id.clone(), expanded);
        let raw=include_str!("../../scenarios/presentation/lpic2_dns_queries.json");
        assert!(raw.len()<=40000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed authoritative query teaching");
        assert_eq!(additions.len(),3);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-dns-dualstack"|"lpic2-dns-aliases"|"lpic2-dns-secondary")&&seen.insert(entry.id.clone()));
            assert_eq!(out.get(&entry.id).expect("Existing DNS lesson").requirements,entry.requirements,"No added independent requirements");
            out.insert(entry.id.clone(),entry);
        }
        // Two existing investigations gain another supported diagnostic tool.
        // Original task/goal values and all independent clauses stay unchanged.
        let raw = include_str!("../../scenarios/presentation/lpic2_port_probes.json");
        assert!(raw.len() <= 20000);
        let additions: Vec<Entry> = serde_json::from_str(raw).expect("reviewed numeric TCP probe teaching");
        assert_eq!(additions.len(), 2);
        let mut seen = std::collections::BTreeSet::new();
        for entry in additions {
            assert!(matches!(entry.id.as_str(), "lpic2-exposure-client-policy" | "lpic2-exposure-listener-retirement") && seen.insert(entry.id.clone()));
            assert_eq!(out.get(&entry.id).expect("Existing exposure lesson").requirements, entry.requirements, "No changed independent requirements");
            out.insert(entry.id.clone(), entry);
        }
        let raw = include_str!("../../scenarios/presentation/lpic2_telnet_transport.json");
        assert!(raw.len() <= 20000);
        let additions: Vec<Entry> = serde_json::from_str(raw).expect("reviewed transport client teaching");
        assert_eq!(additions.len(), 2);
        let mut seen = std::collections::BTreeSet::new();
        for entry in additions {
            assert!(matches!(entry.id.as_str(), "lpic2-exposure-client-policy" | "lpic2-exposure-listener-retirement") && seen.insert(entry.id.clone()));
            assert_eq!(out.get(&entry.id).expect("Existing exposure investigation").requirements, entry.requirements, "No changed independent requirements");
            out.insert(entry.id.clone(), entry);
        }
        let raw = include_str!("../../scenarios/presentation/ccna_poe.json");
        assert!(raw.len() <= 20000);
        let additions: Vec<Entry> = serde_json::from_str(raw).expect("reviewed power-plan teaching");
        assert_eq!(additions.len(), 2);
        let mut seen = std::collections::BTreeSet::new();
        for entry in additions {
            assert!(matches!(entry.id.as_str(), "ccna-poe-compatible-ports" | "ccna-poe-budget-recovery") && seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(), entry).is_none(), "New power lessons must not replace another lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_kernel_build.json");
        assert!(raw.len()<=24000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed kernel-build teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-kernel-root-build"|"lpic2-kernel-staged-handover")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New kernel lessons must not replace another lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_dkms.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed DKMS teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-dkms-kernel-recovery"|"lpic2-dkms-version-handover")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New DKMS lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_initramfs.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed early-userspace teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-initramfs-root-drivers"|"lpic2-initramfs-kernel-handover")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New early-userspace lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_pxe.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed network boot teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-pxe-firmware-selection"|"lpic2-pxelinux-client-lookup")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New network boot lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_iscsi.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed iSCSI teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-iscsi-target-identity"|"lpic2-iscsi-startup-policy")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New iSCSI lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_ata_smart.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed ATA SMART teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-smart-short-diagnosis"|"lpic2-smart-monitor-identity")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New ATA SMART lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_apache_https.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed Apache HTTPS teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic2-apache-https-chain"|"lpic2-apache-https-sni")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New Apache HTTPS lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic1_desktop_profile.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed desktop profile teaching");
        assert_eq!(additions.len(),3);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic1-xorg-keyboard-scope"|"lpic1-remote-desktop-arrangement"|"lpic1-accessible-input-profile")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New desktop profile lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/ccna_security_investigation.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed security investigation teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"ccna-management-change-evidence"|"ccna-facility-access-plan")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New security investigation lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/ccna_credentials_vpn.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed credentials and VPN teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"ccna-credential-policy-repair"|"ccna-ipsec-design-review")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New credentials and VPN lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/ccna_staged_network.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed staged network changes teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"ccna-controller-pilot-rollout"|"ccna-controller-intent-path")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New staged network changes lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/ccna_network_evidence_review.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed network evidence review teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"ccna-review-ai-operations-claim"|"ccna-review-declarative-change")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New network evidence review lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/ccna_site_connectivity.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed site connectivity teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"ccna-diverse-branch-paths"|"ccna-local-and-hosted-connectivity")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New site connectivity lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic_host_dns_security.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed host and DNS security teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"lpic1-retire-socket-activation"|"lpic2-authenticated-zone-handover")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New host and DNS security lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/ccna_vlan_gateway.json");
        assert!(raw.len()<=20000);
        let additions:Vec<Entry>=serde_json::from_str(raw).expect("reviewed VLAN gateway repair teaching");
        assert_eq!(additions.len(),2);
        let mut seen=std::collections::BTreeSet::new();
        for entry in additions{
            assert!(matches!(entry.id.as_str(),"ccna-default-access-membership"|"ccna-switched-gateway-repair")&&seen.insert(entry.id.clone()));
            assert!(out.insert(entry.id.clone(),entry).is_none(),"New VLAN gateway repair lessons must not replace a prior lesson");
        }
        let raw=include_str!("../../scenarios/presentation/lpic2_nfs_export_operations.json");
        assert!(raw.len()<=20000);
        let [entry]:[Entry;1]=serde_json::from_str(raw).expect("reviewed NFS export operations teaching");
        assert_eq!(entry.id,"lpic2-withdraw-obsolete-nfs-export");
        assert!(out.insert(entry.id.clone(),entry).is_none(),"New NFS export operations lesson must keep earlier entries");
        let raw=include_str!("../../scenarios/presentation/ccna_route_reading.json");
        assert!(raw.len()<=20000);
        let [entry]:[Entry;1]=serde_json::from_str(raw).expect("reviewed installed route reading teaching");
        assert_eq!(entry.id,"ccna-route-reading-backup");
        assert!(out.insert(entry.id.clone(),entry).is_none(),"New installed route reading lesson must keep earlier entries");
        let raw=include_str!("../../scenarios/presentation/ccna_ospf_broadcast.json");
        assert!(raw.len()<=20000);
        let [entry]:[Entry;1]=serde_json::from_str(raw).expect("reviewed broadcast designated roles teaching");
        assert_eq!(entry.id,"ccna-ospf-broadcast-handover");
        assert!(out.insert(entry.id.clone(),entry).is_none(),"New broadcast designated roles lesson must keep earlier entries");
        out
    })
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FoundationNotes {
    version: u8,
    lessons: Vec<FoundationNote>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FoundationNote {
    id: String,
    concepts: Vec<Concept>,
    worked: Vec<WorkedRow>,
    introduction_review: String,
    sources: Vec<String>,
}
fn apply_foundation_notes(out: &mut BTreeMap<String, Entry>) {
    let raw = include_str!("../../scenarios/presentation/lpic1_foundations.json");
    assert!(raw.len() <= 8192, "Bounded source-owned foundation notes");
    let notes: FoundationNotes =
        serde_json::from_str(raw).expect("reviewed foundation prerequisites");
    assert_eq!(notes.version, 1);
    assert_eq!(notes.lessons.len(), 2);
    let mut seen = std::collections::BTreeSet::new();
    for note in notes.lessons {
        assert!(
            matches!(
                note.id.as_str(),
                "lpic1-host-address" | "lpic1-host-routing"
            ) && seen.insert(note.id.clone())
        );
        assert_eq!(note.concepts.len(), 1);
        assert_eq!(note.worked.len(), 1);
        assert!(note.introduction_review.len() > 40);
        let entry = out
            .get_mut(&note.id)
            .expect("Notes must belong to a registered original lesson");
        let mut concepts = note.concepts;
        concepts.append(&mut entry.concepts);
        entry.concepts = concepts;
        let mut worked = note.worked;
        worked.append(&mut entry.worked);
        entry.worked = worked;
        assert!(
            note.sources.len() <= 3
                && note
                    .sources
                    .iter()
                    .all(|s| s.starts_with("https://www.rfc-editor.org/rfc/") && s.len() < 160)
        );
        entry.prerequisite_sources = note.sources;
    }
}
pub(super) fn entry(s: &Scenario) -> Option<&'static Entry> {
    entries().get(&s.id)
}
pub(super) fn lead(ui: &mut egui::Ui, eyebrow: &str, text: &str) -> egui::Response {
    lead_layout(ui, eyebrow, text).response
}
fn lead_layout(ui: &mut egui::Ui, eyebrow: &str, text: &str) -> paint::TextLayout {
    ui.add_space(8.);
    rich(
        ui,
        RichText::new(eyebrow)
            .small()
            .strong()
            .color(palette(ui).accent),
    );
    let job = egui::WidgetText::from(RichText::new(text).strong().size(21.)).into_layout_job(
        ui.style(),
        egui::FontSelection::Default,
        egui::Align::Center,
    );
    let layout = left_galley_layout(ui, std::sync::Arc::unwrap_or_clone(job), true);
    ui.add_space(6.);
    layout
}
pub(super) fn rich(ui: &mut egui::Ui, text: RichText) -> egui::Response {
    let job = egui::WidgetText::from(text).into_layout_job(
        ui.style(),
        egui::FontSelection::Default,
        egui::Align::Center,
    );
    left_galley(ui, std::sync::Arc::unwrap_or_clone(job), true)
}
pub(super) fn prose(ui: &mut egui::Ui, text: &str) -> egui::Response {
    prose_layout(ui, text).response
}
fn prose_layout(ui: &mut egui::Ui, text: &str) -> paint::TextLayout {
    let mut job = egui::text::LayoutJob::default();
    for token in text.split_inclusive(char::is_whitespace) {
        let word = token.trim_end_matches(char::is_whitespace);
        let mono = word.chars().any(|c| c.is_ascii_digit())
            && (word.contains('.') || word.contains('/') || word.contains(':'));
        job.append(
            word,
            0.,
            egui::TextFormat {
                font_id: if mono {
                    egui::TextStyle::Monospace
                } else {
                    egui::TextStyle::Body
                }
                .resolve(ui.style()),
                color: palette(ui).text,
                ..Default::default()
            },
        );
        job.append(
            &token[word.len()..],
            0.,
            egui::TextFormat {
                font_id: egui::TextStyle::Body.resolve(ui.style()),
                color: palette(ui).text,
                ..Default::default()
            },
        );
    }
    left_galley_layout(ui, job, true)
}
fn left_galley(ui: &mut egui::Ui, job: egui::text::LayoutJob, wrap: bool) -> egui::Response {
    left_galley_layout(ui, job, wrap).response
}
fn left_galley_layout(
    ui: &mut egui::Ui,
    mut job: egui::text::LayoutJob,
    wrap: bool,
) -> paint::TextLayout {
    job.wrap.max_width = if wrap {
        ui.available_width()
    } else {
        f32::INFINITY
    };
    job.halign = egui::Align::LEFT;
    job.justify = false;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let response = ui.add(egui::Label::new(galley.clone()));
    paint::TextLayout {
        response,
        galley,
        clip: ui.clip_rect(),
    }
}
pub(super) fn monospace(ui: &mut egui::Ui, text: &str, wrap: bool) -> egui::Response {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        text,
        0.,
        egui::TextFormat {
            font_id: egui::TextStyle::Monospace.resolve(ui.style()),
            color: palette(ui).text,
            ..Default::default()
        },
    );
    left_galley(ui, job, wrap)
}
/// Responses belong to the actual rendered objective and prerequisite context.
/// Capture adapters must not reconstruct their geometry from text or scopes.
pub(super) struct IntroResponses {
    pub objective: egui::Response,
    pub context: egui::Response,
}
pub(super) fn concepts(
    ui: &mut egui::Ui,
    s: &Scenario,
    note: &str,
    items: impl IntoIterator<Item = (String, String)>,
) -> IntroResponses {
    let objective = lead(
        ui,
        "WHAT YOU’LL LEARN",
        entry(s).map_or(s.summary.as_str(), |e| e.goal.as_str()),
    );
    let e = entry(s);
    let context = prose(
        ui,
        e.filter(|e| !e.context.is_empty())
            .map_or(note, |e| e.context.as_str()),
    );
    ui.add_space(6.);
    ui.separator();
    let original = items.into_iter().collect::<Vec<_>>();
    let curated = e.filter(|e| !e.concepts.is_empty()).map(|e| {
        e.concepts
            .iter()
            .map(|c| (c.title.clone(), c.text.clone()))
            .collect::<Vec<_>>()
    });
    for (i, (title, text)) in curated.as_ref().unwrap_or(&original).iter().enumerate() {
        section(ui, i + 1, title, text);
    }
    if curated.is_some() && !original.is_empty() {
        controls::collapsing_header(ui, "More detail · original explanations").show(ui, |ui| {
            for (title, text) in &original {
                ui.strong(title);
                prose(ui, text);
            }
        });
    }
    IntroResponses { objective, context }
}
pub(super) fn section(ui: &mut egui::Ui, n: usize, title: &str, text: &str) {
    ui.add_space(9.);
    rich(
        ui,
        RichText::new(format!("{n}. {title}")).strong().size(19.),
    );
    prose(ui, text);
    ui.add_space(4.);
}
pub(super) fn worked(ui: &mut egui::Ui, description: &str) {
    ui.add_space(8.);
    ui.separator();
    ui.add_space(6.);
    ui.strong("Worked example");
    prose(ui, description);
}
/// Opt-in long command rows: wrap complete syntax instead of hiding flags horizontally.
/// New terminal lessons can use this; established worked layouts remain unchanged.
pub(super) fn wrapped_worked_commands(ui: &mut egui::Ui, s: &Scenario) {
    let Some(e) = entry(s) else {
        return;
    };
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 38.;
    for row in &e.worked {
        if wide {
            ui.columns(2, |columns| {
                rich(&mut columns[0], RichText::new(&row.label).strong());
                prose(&mut columns[0], &row.explanation);
                monospace(&mut columns[1], &row.value, true);
            });
        } else {
            rich(ui, RichText::new(&row.label).strong());
            monospace(ui, &row.value, true);
            prose(ui, &row.explanation);
        }
        ui.add_space(8.);
    }
}
pub(super) fn worked_rows(ui: &mut egui::Ui, s: &Scenario) {
    let Some(e) = entry(s) else {
        return;
    };
    if e.worked.is_empty() {
        return;
    }
    if ui.available_width() >= 720. {
        let column = ((ui.available_width() - 36.) / 3.).max(1.);
        egui::Grid::new(ui.id().with("worked-example-rows"))
            .num_columns(3)
            .max_col_width(column)
            .spacing([18., 8.])
            .show(ui, |ui| {
                ui.strong("Action / setting");
                ui.strong("Example value");
                ui.strong("Why / what to observe");
                ui.end_row();
                for (n, r) in e.worked.iter().enumerate() {
                    ui.add(egui::Label::new(&r.label).wrap());
                    code_value(ui, ("worked-value", &s.id, n), &r.value, column);
                    prose(ui, &r.explanation);
                    ui.end_row();
                }
            });
    } else {
        for (n, r) in e.worked.iter().enumerate() {
            rich(ui, RichText::new(&r.label).strong());
            code_value(
                ui,
                ("worked-value", &s.id, n),
                &r.value,
                ui.available_width(),
            );
            prose(ui, &r.explanation);
            ui.add_space(6.);
        }
    }
}
/// Opt-in for command-heavy lessons: keep complete commands visible and wrap
/// their visual layout without changing the underlying one-line command text.
/// Existing calculation/configuration tables retain their approved renderer.
pub(super) fn worked_command_rows(ui: &mut egui::Ui, s: &Scenario) -> Vec<egui::Response> {
    let Some(entry) = entry(s) else {
        return vec![];
    };
    let mut responses = Vec::new();
    for (number, row) in entry.worked.iter().enumerate() {
        ui.push_id(("worked-command", &s.id, number), |ui| {
            rich(ui, RichText::new(&row.label).strong());
            responses.push(monospace(ui, &row.value, true));
            prose(ui, &row.explanation);
            ui.add_space(4.);
        });
    }
    responses
}
pub(super) fn code_value(ui: &mut egui::Ui, id: impl std::hash::Hash, text: &str, width: f32) {
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let size = ui.fonts_mut(|f| f.layout_no_wrap(text.into(), font, color).size());
    if size.x > width {
        // Reserve the glyph height AND scrollbar using native metrics. In a
        // vertically centered Grid, an unbounded horizontal area can clip the
        // top of a long one-line command while its neighboring prose is taller.
        let scroll = ui.spacing().scroll;
        let height = size.y + scroll.bar_width + scroll.bar_inner_margin + scroll.bar_outer_margin;
        ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                controls::nested_scroll(
                    ui,
                    egui::ScrollArea::horizontal()
                        .id_salt(id)
                        .max_width(width)
                        .min_scrolled_height(size.y)
                        .max_height(height)
                        .auto_shrink([false, false]),
                    |ui| {
                        monospace(ui, text, false);
                    },
                );
            },
        );
    } else {
        monospace(ui, text, false);
    }
}
pub(super) fn command(ui: &mut egui::Ui, command: &str, explanation: &str) {
    monospace(ui, command, true);
    if !explanation.is_empty() {
        prose(ui, explanation);
    }
    ui.add_space(4.);
}
/// Equal-purpose diagram cells share width, padding and text baseline. Wrap
/// whole cells at narrow widths; preserve the labels as native accessible text.
pub(super) fn range_blocks(ui: &mut egui::Ui, labels: &[(String, bool)]) -> Vec<egui::Rect> {
    let per_row = if ui.available_width() >= 480. {
        labels.len()
    } else {
        labels.len().min(2)
    }
    .max(1);
    let mut rects = Vec::new();
    for row in labels.chunks(per_row) {
        ui.columns(per_row, |columns| {
            for (column, (label, selected)) in columns.iter_mut().zip(row) {
                let width = column.available_width();
                let r = egui::Frame::new()
                    .stroke(egui::Stroke::new(
                        1_f32,
                        if *selected {
                            palette(column).accent
                        } else {
                            palette(column).muted
                        },
                    ))
                    .inner_margin(egui::Margin::symmetric(6, 6))
                    .show(column, |ui| {
                        ui.set_width((width - 14.).max(1.));
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add(
                                egui::Label::new(RichText::new(label).monospace().strong()).wrap(),
                            );
                        });
                    });
                rects.push(r.response.rect);
            }
        });
        ui.add_space(4.);
    }
    rects
}
/// Put a reviewed, subject-specific action beside its related native control.
pub(super) fn practice_step(ui: &mut egui::Ui, s: &Scenario, n: usize) {
    if let Some(step) = entry(s).and_then(|e| e.practice.get(n)) {
        ui.add_space(6.);
        rich(
            ui,
            RichText::new(format!("{}. {}", n + 1, step.title)).strong(),
        );
        prose(ui, &step.action);
        if !step.observe.is_empty() {
            prose(ui, &format!("Observe: {}", step.observe));
        }
    }
}
pub(super) fn practice_step_commands(ui: &mut egui::Ui, scenario: &Scenario, number: usize) {
    let Some(step) = entry(scenario).and_then(|entry| entry.practice.get(number))
    else {
        return;
    };
    ui.strong(format!("{}. {}", number + 1, step.title));
    for line in step.action.lines() {
        if let Some(command) = line.strip_prefix("$ ") {
            monospace(ui, command, true);
        } else {
            prose(ui, line);
        }
    }
    prose(ui, &format!("Observe: {}", step.observe));
}

/// Exact learner-facing assignment text, in rendered order; headings are excluded.
pub(super) fn assignment_texts(s: &Scenario) -> Vec<&str> {
    let e = entry(s);
    let mut texts = vec![e.map_or(s.summary.as_str(), |e| e.goal.as_str())];
    if let Some(e) = e.filter(|e| !e.requirements.is_empty()) {
        texts.extend(e.requirements.iter().map(String::as_str));
    } else {
        texts.extend(s.objective.split(". "));
    }
    texts
}
/// Real native responses for distinct navigation/capture purposes.
/// `goal` is the lead Label only; `whole` retains the complete task extent.
pub(super) struct AssignmentResponses {
    pub goal: egui::Response,
    pub whole: egui::Response,
}
pub(super) fn assignment(ui: &mut egui::Ui, s: &Scenario) -> egui::Response {
    assignment_responses(ui, s).whole
}
pub(super) fn assignment_responses(ui: &mut egui::Ui, s: &Scenario) -> AssignmentResponses {
    let capture = paint::requested(ui.ctx(), s);
    let mut parts = Vec::new();
    let lead = lead_layout(
        ui,
        "YOUR TASK",
        entry(s).map_or(s.summary.as_str(), |e| e.goal.as_str()),
    );
    let goal = lead.response.clone();
    let mut rect = goal.rect;
    if capture {
        parts.push(lead);
    }
    ui.strong("Requirements");
    let fallback: Vec<String> = s.objective.split(". ").map(str::to_owned).collect();
    let requirements = entry(s)
        .filter(|e| !e.requirements.is_empty())
        .map_or(fallback.as_slice(), |e| e.requirements.as_slice());
    for text in requirements {
        let r = ui
            .horizontal_top(|ui| {
                ui.label("•");
                let label = prose_layout(ui, text);
                if capture {
                    parts.push(label);
                }
            })
            .response;
        rect = rect.union(r.rect);
    }
    if capture {
        paint::record(ui, s, parts);
    }
    ui.add_space(6.);
    ui.separator();
    ui.add_space(6.);
    let whole = ui.interact(
        rect,
        ui.id().with("lesson-requirements"),
        egui::Sense::hover(),
    );
    AssignmentResponses { goal, whole }
}

#[cfg(test)]
mod clipping_tests {
    use super::*;
    #[test]
    fn worked_command_glyphs_fit_their_scroll_clip_in_tall_grid_rows_at_all_text_scales() {
        let command = "ln /practice/in/brief /practice/out/live";
        for scale in [1., 1.5, 2.] {
            let ctx = egui::Context::default();
            ctx.set_zoom_factor(scale);
            for _ in 0..4 {
                let output=ctx.run(egui::RawInput{screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(1000.,700.))),..Default::default()},|ctx|{
     egui::CentralPanel::default().show(ctx,|ui|{egui::Grid::new("clipped-worked-values").num_columns(3).show(ui,|ui|{
      ui.label("Action");code_value(ui,"long-command",command,150.);ui.add(egui::Label::new("Several lines of explanation beside a complete command.\nThe command must retain its full glyph height.\nScrolling changes X only.").wrap());ui.end_row();
     });});
    });
                let text = output
                    .shapes
                    .iter()
                    .find_map(|s| {
                        if let egui::Shape::Text(t) = &s.shape
                            && t.galley.job.text == command
                        {
                            Some((
                                s.clip_rect,
                                egui::Rect::from_min_size(t.pos, t.galley.size()),
                            ))
                        } else {
                            None
                        }
                    })
                    .expect("actual worked command painted");
                assert!(
                    text.1.top() >= text.0.top() - 0.5 && text.1.bottom() <= text.0.bottom() + 0.5,
                    "scale{scale}: clip={:?} glyph={:?}",
                    text.0,
                    text.1
                );
            }
        }
    }
}

#[cfg(test)]
mod assignment_anchor_tests {
    use super::*;

    // Test fixture only: no app, simulation, grading, progress or solution replay.
    fn source() -> Scenario {
        let pack: pack::Pack =
            serde_json::from_str(include_str!("../../scenarios/lvm-growth-v1.json")).unwrap();
        let mut source = pack.scenarios.into_iter().next().unwrap();
        source.id = "test-only-assignment-anchor".into();
        source.summary = "Repair the sample storage layout.".into();
        source.objective = (0..12).map(|n| format!(
            "Requirement {n}: preserve the supplied sample files while changing only the requested storage setting"
        )).collect::<Vec<_>>().join(". ");
        source
    }

    fn render(
        source: &Scenario,
        width: f32,
        height: f32,
        scale: f32,
    ) -> (
        egui::Context,
        egui::FullOutput,
        egui::Id,
        egui::Rect,
        egui::Rect,
        paint::Rows,
    ) {
        let ctx = egui::Context::default();
        ctx.set_zoom_factor(scale);
        paint::request(&ctx, source);
        let mut last = None;
        for _ in 0..4 {
            let mut observed = None;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width / scale, height / scale),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let visible = ui.clip_rect();
                            let responses = assignment_responses(ui, source);
                            if let Some(rows) = paint::collect(ctx, source, visible) {
                                observed = Some((
                                    responses.goal.id,
                                    responses.goal.rect,
                                    responses.whole.rect,
                                    rows,
                                ));
                            }
                        });
                    });
                },
            );
            last = observed.map(|observed| (output, observed));
        }
        let (output, (id, goal, whole, rows)) =
            last.expect("Actual completed, nondiscarded render");
        (ctx, output, id, goal, whole, rows)
    }

    #[test]
    fn assignment_lead_anchor_is_a_real_label_and_whole_bounds_keep_every_requirement() {
        let source = source();
        let (ctx, output, id, goal, whole, rows) = render(&source, 1200., 850., 1.);
        let actual = ctx
            .viewport(|v| v.prev_pass.widgets.get(id).copied())
            .expect("Latest completed pass registered the actual lead Label");
        assert_eq!(
            actual.rect, goal,
            "No scope rectangle or reconstructed label geometry"
        );
        assert_ne!(goal, whole);
        assert!(whole.contains_rect(goal));
        let tolerance = 1. / output.pixels_per_point;
        for row in &rows.rows {
            assert!(whole.expand(tolerance).contains_rect(*row));
        }
        let painted = paint::output_evidence(&ctx, &source, &rows, &output);
        assert_eq!(
            painted
                .keys
                .iter()
                .map(|k| k.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            13
        );
        assert!(
            painted.all_witnessed(),
            "Normal-sized whole task still actually paints all requirements"
        );
    }

    #[test]
    fn tall_200_percent_task_keeps_full_extent_but_geometry_targets_the_real_lead_only() {
        let source = source();
        let (ctx, output, _, goal, whole, rows) = render(&source, 760., 600., 2.);
        assert!(
            whole.height() > rows.clip.height(),
            "Reproduce a genuinely oversized assignment"
        );
        assert!(!crate::ui::smoke::smoke_control_visible(
            whole,
            rows.clip,
            output.pixels_per_point
        ));
        assert!(crate::ui::smoke::smoke_control_visible(
            goal,
            rows.clip,
            output.pixels_per_point
        ));
        let painted = paint::output_evidence(&ctx, &source, &rows, &output);
        assert!(
            painted
                .keys
                .iter()
                .zip(&painted.witnessed)
                .filter(|(key, _)| key.0 == 0)
                .all(|(_, seen)| *seen),
            "Actual goal glyphs remain fully painted, at the unchanged 21 point font"
        );
        assert!(
            !painted.all_witnessed(),
            "A lead anchor must never fabricate whole-assignment capture credit"
        );
        assert_eq!(
            painted
                .keys
                .iter()
                .map(|k| k.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            13,
            "All offscreen requirements remain in the actual layout inventory"
        );
    }
}
