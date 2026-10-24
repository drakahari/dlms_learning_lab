//! Fixed developer-only two-process reopen. Normal startup does all persistence.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
const PURPOSE: &str = "dlms-native-reopen-v1";
const COMPLETE_LAB: &str = "shell-case-interface";
const DRAFT_LAB: &str = "shell-case-publication";
const ASSIGNMENT_DRAFT: &str =
    "# Private cold-reopen assignment draft\nprintf 'unfinished assignment";
const EXAMPLE_DRAFT: &str = "# Independent private example draft\nprintf 'unfinished example";
const JSON_CAP: usize = 16 * 1024 * 1024;
const THEME: &str = "dlms-light";
const SCALE: f32 = 1.5;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Write,
    Read,
}
#[derive(Clone)]
pub(crate) struct Session {
    root: PathBuf,
    role: Role,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Marker {
    version: u8,
    purpose: String,
    root: PathBuf,
    temporary_root: PathBuf,
    nonce: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
struct Observed {
    progress_path: String,
    preferences_path: String,
    progress: serde_json::Value,
    preferences: serde_json::Value,
}
fn bounded_file(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let ceiling = u64::try_from(limit)
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or("Private evidence ceiling overflow")?;
    let before = fs::symlink_metadata(path).map_err(|_| "Missing private ordinary evidence")?;
    if !before.file_type().is_file() || before.len() > limit as u64 {
        return Err("Private evidence type/size ceiling".into());
    }
    let f = fs::File::open(path).map_err(|_| "Cannot open private evidence")?;
    let mut out = Vec::new();
    f.take(ceiling)
        .read_to_end(&mut out)
        .map_err(|_| "Cannot read private evidence")?;
    if out.len() > limit {
        return Err("Private evidence grew beyond ceiling".into());
    }
    Ok(out)
}
fn canonical_directory(path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
    {
        return Err("Expected canonical private directory".into());
    }
    let mut here = PathBuf::new();
    for part in path.components() {
        here.push(part);
        if !fs::symlink_metadata(&here)
            .map_err(|_| "Missing private directory")?
            .file_type()
            .is_dir()
        {
            return Err("Private directory symlink/type refused".into());
        }
    }
    if fs::canonicalize(path).map_err(|_| "Cannot resolve private directory")? != path {
        return Err("Private directory alias refused".into());
    }
    Ok(())
}
fn private_directory(path: &Path) -> Result<(), String> {
    canonical_directory(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(path)
            .map_err(|_| "Private metadata")?
            .permissions()
            .mode()
            & 0o777
            != 0o700
        {
            return Err("Private directory must be0700".into());
        }
    }
    Ok(())
}
fn contained_path(path: &Path, base: &Path) -> Result<(), String> {
    if !path.starts_with(base)
        || path
            .components()
            .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
    {
        return Err("Profile escaped private XDG directory".into());
    }
    let mut here = PathBuf::new();
    let total = path.components().count();
    for (index, part) in path.components().enumerate() {
        here.push(part);
        match fs::symlink_metadata(&here) {
            Ok(m)
                if (index + 1 == total && m.file_type().is_file())
                    || (index + 1 < total && m.file_type().is_dir()) => {}
            Ok(_) => return Err("Profile path symlink/type refused".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Cannot inspect private profile path".into()),
        }
    }
    Ok(())
}
impl Session {
    pub(crate) fn from_environment(smoke: bool, developer: bool) -> Result<Option<Self>, String> {
        if std::env::var_os("DLMS_NATIVE_REOPEN_ROOT").is_none()
            && std::env::var_os("DLMS_NATIVE_REOPEN_PHASE").is_none()
        {
            return Ok(None);
        }
        let keys = [
            "DLMS_NATIVE_REOPEN_ROOT",
            "DLMS_NATIVE_REOPEN_PHASE",
            "TMPDIR",
            "XDG_DATA_HOME",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
        ];
        let mut env = BTreeMap::new();
        for key in keys {
            if let Some(value) = std::env::var_os(key) {
                if value.len() > 4096 {
                    return Err("Developer reopen environment ceiling".into());
                }
                env.insert(
                    key.to_string(),
                    value
                        .into_string()
                        .map_err(|_| "Developer reopen paths must be UTF8")?,
                );
            }
        }
        Self::from_config(
            smoke,
            developer,
            &env,
            &std::env::current_dir().map_err(|_| "Current directory unavailable")?,
        )
    }
    fn from_config(
        smoke: bool,
        developer: bool,
        env: &BTreeMap<String, String>,
        cwd: &Path,
    ) -> Result<Option<Self>, String> {
        let root = env.get("DLMS_NATIVE_REOPEN_ROOT");
        let phase = env.get("DLMS_NATIVE_REOPEN_PHASE");
        if root.is_none() && phase.is_none() {
            return Ok(None);
        }
        if smoke || !developer {
            return Err("Reopen requires --developer without --ui-smoke".into());
        }
        if !cfg!(target_os = "linux") {
            return Err("Native reopen acceptance currently targets Linux".into());
        }
        let root = PathBuf::from(root.ok_or("Missing reopen root")?);
        let role = match phase.map(String::as_str) {
            Some("write") => Role::Write,
            Some("read") => Role::Read,
            _ => return Err("Fixed write/read role required".into()),
        };
        let tmp = PathBuf::from(env.get("TMPDIR").ok_or("Runner TMPDIR missing")?);
        canonical_directory(&tmp)?;
        if tmp.file_name() != Some(std::ffi::OsStr::new("tmp"))
            || tmp.parent().and_then(Path::file_name)
                != Some(std::ffi::OsStr::new(".build-artifacts"))
            || root.parent() != Some(tmp.as_path())
        {
            return Err("Use a direct private child of runner artifact TMPDIR".into());
        }
        let name = root
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or("Invalid reopen root name")?;
        if !name.starts_with("native-reopen-")
            || name.len() > 96
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("Invalid fixed reopen directory name".into());
        }
        private_directory(&root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let info = fs::symlink_metadata(root.join("REOPEN-SESSION.json"))
                .map_err(|_| "Missing reopen marker")?;
            let owner = fs::metadata(&root)
                .map_err(|_| "Private root metadata")?
                .uid();
            if info.uid() != owner || info.permissions().mode() & 0o777 != 0o600 {
                return Err("Private marker ownership/permissions refused".into());
            }
        }
        let marker: Marker =
            serde_json::from_slice(&bounded_file(&root.join("REOPEN-SESSION.json"), 4096)?)
                .map_err(|_| "Invalid bounded reopen marker")?;
        if marker.version != 1
            || marker.purpose != PURPOSE
            || marker.root != root
            || marker.temporary_root != tmp
            || marker.nonce.len() != 32
            || !marker
                .nonce
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err("Wrong reopen marker context".into());
        }
        for (key, leaf) in [
            ("XDG_DATA_HOME", "data"),
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_CACHE_HOME", "cache"),
        ] {
            let expected = root.join(leaf);
            if env.get(key).map(Path::new) != Some(expected.as_path()) {
                return Err("Private XDG context mismatch".into());
            }
            private_directory(&expected)?;
        }
        let work = root.join(if role == Role::Write { "write" } else { "read" });
        private_directory(&work)?;
        if cwd != work {
            return Err("Wrong private native working directory".into());
        }
        if role == Role::Write
            && fs::read_dir(root.join("data"))
                .map_err(|_| "Private data inspection")?
                .next()
                .is_some()
        {
            return Err("Writer requires an empty new private data directory".into());
        }
        if role == Role::Read {
            let _ = bounded_file(&root.join("writer-progress.raw"), 16_000_000)?;
            let _ = bounded_file(&root.join("writer-preferences.raw"), 4096)?;
            let _ = bounded_file(&root.join("writer-observed.json"), JSON_CAP)?;
        }
        Ok(Some(Self { root, role }))
    }
    pub(crate) fn verify_profile_paths(&self, path: &Path) -> Result<(), String> {
        if path.file_name() != Some(std::ffi::OsStr::new("progress-v1.json")) {
            return Err("Unexpected normal profile filename".into());
        }
        let base = self.root.join("data");
        for p in [
            path.to_path_buf(),
            path.with_file_name("preferences-v1.json"),
            path.with_extension("lock"),
        ] {
            contained_path(&p, &base)?;
        }
        Ok(())
    }
    fn write(&self, name: &str, value: &impl Serialize) -> Result<(), String> {
        let bytes =
            serde_json::to_vec_pretty(value).map_err(|_| "Cannot encode private observation")?;
        if bytes.len() > JSON_CAP {
            return Err("Private observation ceiling".into());
        }
        let path = self.root.join(name);
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut f = options
            .open(&path)
            .map_err(|_| "Private observation already exists or cannot be created")?;
        f.write_all(&bytes)
            .and_then(|_| f.sync_all())
            .map_err(|_| "Cannot persist private observation".to_string())
    }
    fn observed(&self, app: &LabApp) -> Observed {
        self.verify_profile_paths(&app.path)
            .expect("private normal path");
        let disk = progress::load(&app.path, &app.scenarios).expect("actual normal saved progress");
        let preferences = theme::Preferences::load(&app.path.with_file_name("preferences-v1.json"))
            .expect("actual saved appearance");
        assert_eq!(
            serde_json::to_value(&disk.attempts).unwrap(),
            serde_json::to_value(&app.progress.attempts).unwrap()
        );
        assert_eq!(disk.favorites, app.progress.favorites);
        Observed {
            progress_path: app.path.to_str().unwrap().into(),
            preferences_path: app
                .path
                .with_file_name("preferences-v1.json")
                .to_str()
                .unwrap()
                .into(),
            progress: serde_json::to_value(disk).unwrap(),
            preferences: serde_json::to_value(preferences).unwrap(),
        }
    }
}
impl LabApp {
    fn reopen_focus(&mut self, ctx: &egui::Context) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
        if phase >= 2 {
            let done = self.click_smoke_control(ctx, None);
            assert!(
                self.metrics
                    .focus
                    .is_some_and(|id| ctx.memory(|m| m.has_focus(id)))
            );
            return done;
        }
        let viewport = self
            .metrics
            .editor_viewport
            .expect("existing editor viewport");
        let height = ctx.style().spacing.interact_size.y.min(viewport.height());
        let target =
            egui::Rect::from_center_size(viewport.center(), egui::vec2(viewport.width(), height));
        self.reveal_smoke_control(ctx, target) && self.click_smoke_control(ctx, Some(target))
    }
    fn reopen_type(&mut self, text: &str) {
        assert!(text.len() <= shell::MAX_SOURCE);
        self.pending_events = [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            })
            .collect();
        self.pending_events.push(egui::Event::Text(text.into()));
    }
    fn reopen_wait(&self, ctx: &egui::Context) -> bool {
        if self.script_job.is_none() {
            return true;
        }
        let started = ctx.data_mut(|d| {
            let key = egui::Id::new(("reopen-wait", self.smoke_stage));
            let started = d.get_temp::<Instant>(key).unwrap_or_else(Instant::now);
            d.insert_temp(key, started);
            started
        });
        assert!(
            started.elapsed().as_secs() < 15,
            "private native VM deadline"
        );
        ctx.request_repaint();
        false
    }
    fn reopen_popup(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let rect = rect.expect("fixed popup target");
        assert!(
            ctx.content_rect().contains_rect(rect),
            "fixed item must be visible in the popup"
        );
        self.click_smoke_control(ctx, Some(rect))
    }
    fn reopen_filter(&mut self, id: &str) {
        let title = self
            .scenarios
            .iter()
            .find(|s| s.id == id)
            .unwrap()
            .title
            .clone();
        self.selected = None;
        self.filter = Filter::default();
        self.filter.search = title;
        self.filter.max_minutes = 120;
        self.page = 0;
    }
    fn reopen_geometry(&self, ctx: &egui::Context) -> bool {
        let rect = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let stable = ctx.data_mut(|d| {
            let key = egui::Id::new(("native-reopen-geometry", self.smoke_stage));
            let (previous, count, started) = d
                .get_temp::<(Option<egui::Rect>, usize, Instant)>(key)
                .unwrap_or((None, 0, Instant::now()));
            assert!(
                started.elapsed().as_secs() < 15,
                "private startup geometry deadline"
            );
            let count = if previous == Some(rect) && (zoom - self.scale).abs() < 0.01 {
                count + 1
            } else {
                0
            };
            d.insert_temp(key, (Some(rect), count, started));
            count >= 2
        });
        if !stable {
            ctx.request_repaint();
        }
        stable
    }
    fn reopen_assert_state(&self) {
        assert_eq!(self.progress.attempts.len(), 2);
        assert_eq!(
            self.progress.favorites,
            std::collections::BTreeSet::from([COMPLETE_LAB.to_string()])
        );
        let complete = self
            .scenarios
            .iter()
            .find(|s| s.id == COMPLETE_LAB)
            .unwrap();
        assert!(progress::current_solved(
            &self.progress.attempts[COMPLETE_LAB],
            complete
        ));
        let draft = &self.progress.attempts[DRAFT_LAB];
        let engine::State::Script { source, .. } = &draft.state else {
            panic!()
        };
        assert_eq!(source, ASSIGNMENT_DRAFT);
        assert!(!progress::current_solved(
            draft,
            self.scenarios.iter().find(|s| s.id == DRAFT_LAB).unwrap()
        ));
        let learning = draft
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap();
        assert_eq!(learning.example_source, EXAMPLE_DRAFT);
        assert!(learning.example_result.is_none());
        assert!(self.progress.favorites.contains(COMPLETE_LAB));
        assert_eq!(self.theme_id, THEME);
        assert_eq!(self.scale, SCALE);
    }
    pub(in crate::ui) fn reopen_tick(&mut self, ctx: &egui::Context) {
        if self.capture_pending || self.started.elapsed().as_millis() < 350 {
            return;
        }
        let session = self.reopen.as_ref().unwrap().clone();
        let phase = self.smoke_stage - 17000;
        let mut shot = None;
        let mut close = false;
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == COMPLETE_LAB)
            .unwrap();
        let j = self
            .scenarios
            .iter()
            .position(|s| s.id == DRAFT_LAB)
            .unwrap();
        if phase == 0 && !self.reopen_geometry(ctx) {
            return;
        }
        match session.role {
            Role::Write => match phase {
                0 => {
                    assert!(
                        !self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some()
                    );
                    assert!(!self.read_only && !self.preferences_read_only);
                    assert!(self.storage_error.is_none() && self.preferences_error.is_none());
                    assert!(self.progress.attempts.is_empty());
                    session.verify_profile_paths(&self.path).unwrap();
                    self.reopen_filter(COMPLETE_LAB);
                    shot = Some("reopen-write-library");
                }
                1 => {
                    assert_eq!(self.metrics.card_actions.len(), 1);
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics.card_actions.first().map(|r| r.1),
                    ) {
                        return;
                    }
                }
                2 => {
                    assert!(self.progress.favorites.contains(COMPLETE_LAB));
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics.card_actions.first().map(|r| r.0),
                    ) {
                        return;
                    }
                }
                3 => {
                    assert_eq!(self.selected, Some(i));
                    if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                        return;
                    }
                }
                4 | 14 | 18 => {
                    if !self.reopen_focus(ctx) {
                        return;
                    }
                }
                5 => self.reopen_type(&self.scenarios[i].solution[0].clone()),
                6 | 12 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                        return;
                    }
                }
                7 => {
                    if !self.reopen_wait(ctx) {
                        return;
                    }
                    assert!(progress::current_solved(
                        &self.progress.attempts[COMPLETE_LAB],
                        &self.scenarios[i]
                    ));
                    shot = Some("reopen-write-completed-run");
                }
                8 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                        return;
                    }
                }
                9 => self.reopen_filter(DRAFT_LAB),
                10 => {
                    assert_eq!(self.metrics.card_actions.len(), 1);
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics.card_actions.first().map(|r| r.0),
                    ) {
                        return;
                    }
                }
                11 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                        return;
                    }
                }
                13 => {
                    if !self.reopen_wait(ctx) {
                        return;
                    }
                    let lesson = self.scenarios[j].teaching.as_ref().unwrap();
                    let learning = self.progress.attempts[DRAFT_LAB]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .script_learning
                        .as_ref()
                        .unwrap();
                    assert!(
                        lesson
                            .fixture
                            .accepts(learning.example_result.as_ref().unwrap())
                    );
                    shot = Some("reopen-write-working-example");
                }
                15 => self.reopen_type(EXAMPLE_DRAFT),
                16 => {
                    assert_eq!(
                        self.progress.attempts[DRAFT_LAB]
                            .workspace
                            .as_ref()
                            .unwrap()
                            .script_learning
                            .as_ref()
                            .unwrap()
                            .example_source,
                        EXAMPLE_DRAFT
                    );
                    if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                        return;
                    }
                }
                17 => {}
                19 => self.reopen_type(ASSIGNMENT_DRAFT),
                20 => {
                    assert!(!progress::current_solved(
                        &self.progress.attempts[DRAFT_LAB],
                        &self.scenarios[j]
                    ));
                    shot = Some("reopen-write-unfinished-assignment");
                }
                21 => {
                    if !self.reopen_popup(ctx, self.metrics.reopen_scale) {
                        return;
                    }
                }
                22 => {
                    let r = self
                        .metrics
                        .reopen_scale_items
                        .iter()
                        .find(|(n, _)| *n == SCALE)
                        .map(|(_, r)| *r);
                    if !self.reopen_popup(ctx, r) {
                        return;
                    }
                }
                23 => {
                    assert_eq!(self.scale, SCALE);
                    if !self.reopen_popup(ctx, self.metrics.reopen_theme) {
                        return;
                    }
                }
                24 => {
                    let r = self
                        .metrics
                        .reopen_theme_items
                        .iter()
                        .find(|(id, _)| id == THEME)
                        .map(|(_, r)| *r);
                    if !self.reopen_popup(ctx, r) {
                        return;
                    }
                }
                25 => {
                    self.reopen_assert_state();
                    assert!(self.preferences_error.is_none());
                    shot = Some("reopen-write-saved-appearance-and-drafts");
                }
                26 => {
                    self.reopen_assert_state();
                    self.save();
                    assert!(!self.dirty && self.storage_error.is_none());
                    let observed = session.observed(self);
                    session.write("writer-observed.json", &observed).unwrap();
                    println!(
                        "PASS native reopen writer: actual VM completion and independent unfinished drafts saved with favorite and appearance"
                    );
                    close = true;
                }
                _ => panic!("Unexpected reopen writer phase{phase}"),
            },
            Role::Read => match phase {
                0 => {
                    assert!(
                        !self.smoke && self._smoke_profile.is_none() && self.profile_lock.is_some()
                    );
                    assert!(!self.read_only && !self.preferences_read_only);
                    assert!(self.storage_error.is_none() && self.preferences_error.is_none());
                    let expected: Observed = serde_json::from_slice(
                        &bounded_file(&session.root.join("writer-observed.json"), JSON_CAP)
                            .unwrap(),
                    )
                    .unwrap();
                    assert_eq!(
                        bounded_file(&self.path, 16_000_000).unwrap(),
                        bounded_file(&session.root.join("writer-progress.raw"), 16_000_000)
                            .unwrap()
                    );
                    assert_eq!(
                        bounded_file(&self.path.with_file_name("preferences-v1.json"), 4096)
                            .unwrap(),
                        bounded_file(&session.root.join("writer-preferences.raw"), 4096).unwrap()
                    );
                    assert_eq!(
                        serde_json::to_value(&self.progress).unwrap(),
                        expected.progress
                    );
                    assert_eq!(
                        serde_json::json!({"version":1,"theme":self.theme_id,"text_scale":self.scale}),
                        expected.preferences
                    );
                    self.reopen_assert_state();
                    let loaded = Observed {
                        progress_path: self.path.to_str().unwrap().into(),
                        preferences_path: self
                            .path
                            .with_file_name("preferences-v1.json")
                            .to_str()
                            .unwrap()
                            .into(),
                        progress: serde_json::to_value(&self.progress).unwrap(),
                        preferences: serde_json::json!({"version":1,"theme":self.theme_id,"text_scale":self.scale}),
                    };
                    assert_eq!(loaded, expected);
                    session.write("reader-loaded.json", &loaded).unwrap();
                    shot = Some("reopen-read-loaded-library");
                }
                1 => self.reopen_filter(DRAFT_LAB),
                2 => {
                    assert_eq!(self.metrics.card_actions.len(), 1);
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics.card_actions.first().map(|r| r.0),
                    ) {
                        return;
                    }
                }
                3 => {
                    assert_eq!(self.selected, Some(j));
                    assert_eq!(self.script_view(j), teaching::View::Assignment);
                    shot = Some("reopen-read-unfinished-assignment");
                }
                4 => {
                    if !self
                        .click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(1).copied())
                    {
                        return;
                    }
                }
                5 => {
                    assert_eq!(self.script_view(j), teaching::View::Example);
                    self.reopen_assert_state();
                    shot = Some("reopen-read-unfinished-independent-example");
                }
                6 => {
                    if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                        return;
                    }
                }
                7 => self.reopen_filter(COMPLETE_LAB),
                8 => {
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics.card_actions.first().map(|r| r.0),
                    ) {
                        return;
                    }
                }
                9 => {
                    self.reopen_assert_state();
                    assert!(self.metrics.completion);
                    if !self.reveal_smoke_control(ctx, self.metrics.script_feedback.unwrap()) {
                        return;
                    }
                    shot = Some("reopen-read-current-completion");
                }
                10 => {
                    self.reopen_assert_state();
                    self.save();
                    assert!(!self.dirty && self.storage_error.is_none());
                    println!(
                        "PASS native reopen reader: reacquired normal profile lock, exact cold-loaded records/drafts/favorite/theme/scale"
                    );
                    close = true;
                }
                _ => panic!("Unexpected reopen reader phase{phase}"),
            },
        }
        if let Some(name) = shot {
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                name.to_string(),
            )));
        }
        self.smoke_stage += 1;
        self.started = Instant::now();
        if close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    pub(in crate::ui) fn reopen_capture_events(&mut self, ctx: &egui::Context) {
        const NAMES: [&str; 9] = [
            "reopen-write-library",
            "reopen-write-completed-run",
            "reopen-write-working-example",
            "reopen-write-unfinished-assignment",
            "reopen-write-saved-appearance-and-drafts",
            "reopen-read-loaded-library",
            "reopen-read-unfinished-assignment",
            "reopen-read-unfinished-independent-example",
            "reopen-read-current-completion",
        ];
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot {
                image, user_data, ..
            } = event
            {
                let name = user_data
                    .data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<String>())
                    .expect("fixed private capture name");
                assert!(NAMES.contains(&name.as_str()));
                assert!(
                    image
                        .width()
                        .checked_mul(image.height())
                        .is_some_and(|n| n <= 4 * 1024 * 1024)
                );
                let session = self.reopen.as_ref().unwrap();
                let directory = session.root.join(if session.role == Role::Write {
                    "write"
                } else {
                    "read"
                });
                assert_eq!(std::env::current_dir().unwrap(), directory);
                let mut options = fs::OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                let f = options
                    .open(directory.join(format!("{name}.png")))
                    .expect("exclusive private screenshot");
                let mut encoder = png::Encoder::new(f, image.width() as u32, image.height() as u32);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(
                        &image
                            .pixels
                            .iter()
                            .flat_map(|p| p.to_array())
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
                self.capture_pending = false;
                println!("Captured {name}: {}x{}", image.width(), image.height());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_startup_stays_dormant_and_partial_developer_mode_is_rejected_before_profile_io() {
        assert!(
            Session::from_config(
                false,
                false,
                &BTreeMap::new(),
                Path::new("/nonexistent-unused")
            )
            .unwrap()
            .is_none()
        );
        for key in ["DLMS_NATIVE_REOPEN_ROOT", "DLMS_NATIVE_REOPEN_PHASE"] {
            let env = BTreeMap::from([(key.into(), "invalid".into())]);
            assert!(Session::from_config(false, false, &env, Path::new("/")).is_err());
            assert!(Session::from_config(true, true, &env, Path::new("/")).is_err());
            assert!(Session::from_config(false, true, &env, Path::new("/")).is_err());
        }
    }
    #[test]
    fn fixed_draft_sources_are_distinct_bounded_and_cannot_be_mistaken_for_authored_solutions() {
        assert_ne!(ASSIGNMENT_DRAFT, EXAMPLE_DRAFT);
        assert!(
            ASSIGNMENT_DRAFT.len() < shell::MAX_SOURCE && EXAMPLE_DRAFT.len() < shell::MAX_SOURCE
        );
        let all = pack::bundled().unwrap();
        for id in [COMPLETE_LAB, DRAFT_LAB] {
            let s = all.iter().find(|s| s.id == id).unwrap();
            assert!(s.teaching.is_some());
            assert!(
                s.solution
                    .iter()
                    .all(|source| source != ASSIGNMENT_DRAFT && source != EXAMPLE_DRAFT)
            );
        }
    }
    #[test]
    fn contained_profile_paths_reject_parent_escape_symlinks_and_nonordinary_files() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("data");
        fs::create_dir(&data).unwrap();
        assert!(contained_path(&data.join("future/progress-v1.json"), &data).is_ok());
        assert!(contained_path(&data.join("../outside"), &data).is_err());
        assert!(contained_path(&root.path().join("outside"), &data).is_err());
        #[cfg(unix)]
        {
            let target = root.path().join("outside");
            fs::write(&target, b"unchanged").unwrap();
            std::os::unix::fs::symlink(&target, data.join("progress-v1.json")).unwrap();
            assert!(contained_path(&data.join("progress-v1.json"), &data).is_err());
            assert_eq!(fs::read(target).unwrap(), b"unchanged");
        }
    }
    #[test]
    fn private_observations_do_not_overwrite_unknown_bytes_and_reader_is_finite() {
        let root = tempfile::tempdir().unwrap();
        let session = Session {
            root: root.path().into(),
            role: Role::Write,
        };
        let path = root.path().join("known");
        fs::write(&path, b"keep").unwrap();
        assert!(
            session
                .write("known", &serde_json::json!({"new":true}))
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), b"keep");
        assert!(bounded_file(&path, 3).is_err());
        assert_eq!(bounded_file(&path, 4).unwrap(), b"keep");
    }
}

#[cfg(all(test, target_os = "linux"))]
mod context_tests {
    use super::*;
    fn context() -> (tempfile::TempDir, BTreeMap<String, String>, PathBuf) {
        let root = tempfile::Builder::new()
            .prefix("native-reopen-")
            .tempdir()
            .unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        for name in ["data", "config", "cache", "write", "read"] {
            let path = root.path().join(name);
            fs::create_dir(&path).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let tmp = root.path().parent().unwrap().to_path_buf();
        let marker = Marker {
            version: 1,
            purpose: PURPOSE.into(),
            root: root.path().into(),
            temporary_root: tmp.clone(),
            nonce: "a".repeat(32),
        };
        let path = root.path().join("REOPEN-SESSION.json");
        fs::write(&path, serde_json::to_vec(&marker).unwrap()).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        let env = BTreeMap::from([
            (
                "DLMS_NATIVE_REOPEN_ROOT".into(),
                root.path().to_str().unwrap().into(),
            ),
            ("DLMS_NATIVE_REOPEN_PHASE".into(), "write".into()),
            ("TMPDIR".into(), tmp.to_str().unwrap().into()),
            (
                "XDG_DATA_HOME".into(),
                root.path().join("data").to_str().unwrap().into(),
            ),
            (
                "XDG_CONFIG_HOME".into(),
                root.path().join("config").to_str().unwrap().into(),
            ),
            (
                "XDG_CACHE_HOME".into(),
                root.path().join("cache").to_str().unwrap().into(),
            ),
        ]);
        let cwd = root.path().join("write");
        (root, env, cwd)
    }
    #[test]
    fn valid_private_context_uses_only_the_normal_contained_profile_names() {
        let (root, env, cwd) = context();
        let session = Session::from_config(false, true, &env, &cwd)
            .unwrap()
            .unwrap();
        session
            .verify_profile_paths(&root.path().join("data/dlmslearninglab/progress-v1.json"))
            .unwrap();
        assert!(
            session
                .verify_profile_paths(&root.path().join("progress-v1.json"))
                .is_err()
        );
        assert!(
            session
                .verify_profile_paths(&root.path().join("data/dlmslearninglab/another.json"))
                .is_err()
        );
    }
    #[test]
    fn exact_marker_purpose_version_paths_nonce_unknown_fields_and_byte_ceiling_are_enforced() {
        let (root, env, cwd) = context();
        let path = root.path().join("REOPEN-SESSION.json");
        let original: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        for which in 0..6 {
            let mut bad = original.clone();
            match which {
                0 => bad["version"] = 2.into(),
                1 => bad["purpose"] = "another-purpose".into(),
                2 => bad["root"] = "/not-private".into(),
                3 => bad["temporary_root"] = "/tmp".into(),
                4 => bad["nonce"] = "short".into(),
                _ => bad["unknown"] = true.into(),
            };
            fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
            assert!(Session::from_config(false, true, &env, &cwd).is_err());
        }
        fs::write(&path, vec![b' '; 4097]).unwrap();
        assert!(Session::from_config(false, true, &env, &cwd).is_err());
        assert!(
            fs::read_dir(root.path().join("data"))
                .unwrap()
                .next()
                .is_none()
        );
    }
    #[test]
    fn xdg_cwd_reader_preconditions_and_existing_writer_data_are_fail_closed() {
        let (root, env, cwd) = context();
        let mut wrong = env.clone();
        wrong.insert(
            "XDG_DATA_HOME".into(),
            root.path().join("cache").to_str().unwrap().into(),
        );
        assert!(Session::from_config(false, true, &wrong, &cwd).is_err());
        assert!(Session::from_config(false, true, &env, &root.path().join("read")).is_err());
        let mut read = env.clone();
        read.insert("DLMS_NATIVE_REOPEN_PHASE".into(), "read".into());
        assert!(Session::from_config(false, true, &read, &root.path().join("read")).is_err());
        let existing = root.path().join("data/unknown");
        fs::write(&existing, b"preserve").unwrap();
        assert!(Session::from_config(false, true, &env, &cwd).is_err());
        assert_eq!(fs::read(existing).unwrap(), b"preserve");
    }
    #[test]
    fn symlinked_marker_or_wrong_private_permissions_cannot_activate_the_mode() {
        use std::os::unix::fs::PermissionsExt;
        let (root, env, cwd) = context();
        let marker = root.path().join("REOPEN-SESSION.json");
        fs::set_permissions(&marker, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Session::from_config(false, true, &env, &cwd).is_err());
        fs::set_permissions(&marker, fs::Permissions::from_mode(0o600)).unwrap();
        let saved = root.path().join("saved-marker");
        fs::rename(&marker, &saved).unwrap();
        std::os::unix::fs::symlink(&saved, &marker).unwrap();
        assert!(Session::from_config(false, true, &env, &cwd).is_err());
    }
}
