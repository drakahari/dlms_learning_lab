//! Native user data UI; file paths here are never obtained from lab/script input.
use super::*;
use dlms_learning_lab::backup;
use std::path::{Path, PathBuf};
#[derive(Default)]
pub(super) struct Tools {
    browser: Option<Browser>,
    inspection: Option<backup::Inspection>,
    candidate: Option<backup::Candidate>,
    message: Option<String>,
}
#[derive(Clone, Copy)]
enum Mode {
    Export,
    Restore,
}
impl Tools {
    #[cfg(test)]
    pub(super) fn restore_diagnostic(&self) -> String {
        format!(
            "message: {:?}; review still open: {}",
            self.message,
            self.candidate.is_some()
        )
    }
    pub(super) fn modal_open(&self) -> bool {
        self.browser.is_some() || self.inspection.is_some() || self.candidate.is_some()
    }
}
struct Browser {
    mode: Mode,
    folder: PathBuf,
    name: String,
    entries: Vec<(PathBuf, bool)>,
    error: Option<String>,
}
impl Browser {
    fn new(mode: Mode, folder: PathBuf) -> Self {
        let mut b = Self {
            mode,
            folder,
            name: if matches!(mode, Mode::Export) {
                "dlms-learning-backup.json".into()
            } else {
                String::new()
            },
            entries: Vec::new(),
            error: None,
        };
        b.refresh();
        b
    }
    fn refresh(&mut self) {
        self.entries.clear();
        self.error = None;
        match std::fs::read_dir(&self.folder) {
            Ok(entries) => {
                for entry in entries.take(1000).flatten() {
                    if let Ok(t) = entry.file_type()
                        && !t.is_symlink()
                        && (t.is_dir() || t.is_file())
                    {
                        self.entries.push((entry.path(), t.is_dir()));
                    }
                }
                self.entries
                    .sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            }
            Err(e) => self.error = Some(format!("Cannot read this folder: {e}")),
        }
    }
    fn target(&self) -> Result<PathBuf, String> {
        let name = Path::new(&self.name);
        if self.name.is_empty()
            || name.components().count() != 1
            || !matches!(
                name.components().next(),
                Some(std::path::Component::Normal(_))
            )
        {
            return Err("Choose a filename in the displayed folder".into());
        }
        Ok(self.folder.join(name))
    }
}
impl LabApp {
    pub(super) fn data_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Your data");
        ui.label("Progress, assignment drafts, examples and favorites stay on this computer. Backups contain your own work; keep them private. Themes stay unchanged when restoring progress.");
        ui.horizontal_wrapped(|ui| {
            let export = ui.add(button("Export backup…"));
            self.metrics.data_export = Some(export.rect);
            if export.clicked() {
                self.data_tools.browser = Some(Browser::new(Mode::Export, self.backup_folder()));
            }
            let restore = ui.add_enabled(
                self.profile_lock.is_some() || self.smoke,
                button("Restore backup…"),
            );
            self.metrics.data_restore = Some(restore.rect);
            if restore.clicked() {
                self.script_job = None;
                self.data_tools.browser = Some(Browser::new(Mode::Restore, self.backup_folder()));
            }
        });
        if let Some(message) = &self.data_tools.message {
            ui.add(egui::Label::new(message).wrap());
        }
        egui::CollapsingHeader::new("Storage & recovery details").show(ui,|ui|{ui.add(egui::Label::new(format!("Local progress: {}",self.path.display())).wrap());ui.label("Export creates a new file without replacing an existing backup. Restore checks the backup first, asks you to confirm, and saves an exact copy of the current profile before replacing it. Invalid backups make no changes. Close other copies of the app before restoring.");});
    }
    fn backup_folder(&self) -> PathBuf {
        if self.smoke {
            return self.path.parent().unwrap().into();
        }
        directories::UserDirs::new()
            .and_then(|d| d.document_dir().map(Path::to_path_buf))
            .or_else(|| self.path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."))
    }
    fn protect_session_before_restore(&mut self) -> Result<(), String> {
        if !self.read_only {
            self.save();
            if self.dirty {
                return Err("Current edits could not be saved; restore cancelled so they are not lost. Export them first.".into());
            }
        } else if self.dirty {
            let parent = self.path.parent().ok_or("Missing progress folder")?;
            let mut saved = false;
            for n in 1..=10_000 {
                let path = parent.join(format!("session-before-restore-{n}.json"));
                if !path.exists() {
                    if let Some(warning) = backup::export(&path, &self.progress, &self.scenarios)? {
                        return Err(warning);
                    }
                    saved = true;
                    break;
                }
            }
            if !saved {
                return Err("No unused session-backup filename available".into());
            }
        }
        Ok(())
    }
    pub(super) fn data_dialog(&mut self, ctx: &egui::Context) {
        if let Some(job) = &self.data_tools.inspection {
            if let Some(result) = job.poll() {
                self.data_tools.inspection = None;
                match result {
                    Ok(candidate) => self.data_tools.candidate = Some(candidate),
                    Err(e) => self.data_tools.message = Some(format!("Backup not restored: {e}")),
                }
            } else {
                let mut cancel = false;
                egui::Window::new("Checking backup").collapsible(false).show(ctx,|ui|{ui.label("Checking saved labs and re-running saved script results in the bounded simulator. Your current progress has not changed.");ui.label(format!("{} script/example results checked",job.checked.load(std::sync::atomic::Ordering::Relaxed)));cancel=ui.button("Cancel").clicked();});
                if cancel {
                    self.data_tools.inspection = None;
                    self.data_tools.message =
                        Some("Backup check cancelled; no changes made.".into());
                }
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
        if let Some(mut browser) = self.data_tools.browser.take() {
            let mut open = true;
            let mut chosen = false;
            let mut navigate = None;
            egui::Window::new(match browser.mode{Mode::Export=>"Export progress backup",Mode::Restore=>"Choose progress backup"}).open(&mut open).default_width(620.).default_height(360.).max_width((ctx.content_rect().width()-24.).max(200.)).max_height((ctx.content_rect().height()-24.).max(140.)).vscroll(true).resizable(true).show(ctx,|ui|{
    ui.label(match browser.mode{Mode::Export=>"Choose a folder and an unused filename. This saves your progress and both kinds of drafts.",Mode::Restore=>"Choose a DLMS backup or an earlier progress JSON file. You will review it before anything is replaced."});
    ui.horizontal_wrapped(|ui|{if ui.button("Up a folder").clicked(){navigate=browser.folder.parent().map(Path::to_path_buf);}
    if ui.button("Documents").clicked(){navigate=Some(self.backup_folder());}});
    ui.add(egui::Label::new(browser.folder.to_string_lossy()).wrap());
    egui::ScrollArea::vertical().id_salt("backup-files").max_height((ctx.content_rect().height()*0.40).clamp(100.,360.)).show(ui,|ui|{
     for (path,dir) in &browser.entries{let name=path.file_name().unwrap_or_default().to_string_lossy();let label=format!("{} {name}",if *dir{"Folder:"}else{"File:"});if ui.add(egui::Button::selectable(!*dir&&browser.name==name,label).wrap()).clicked(){if *dir{navigate=Some(path.clone());}else{browser.name=name.into_owned();}}}
     ui.label("Up to 1,000 entries shown. Symbolic links are excluded.");
    });
    ui.horizontal(|ui| {
        ui.label("Filename");
        let field=ui.add(egui::TextEdit::singleline(&mut browser.name).char_limit(240).desired_width((ui.available_width()-12.).max(40.)));
        self.metrics.data_filename=Some(field.rect);
    });
    if let Some(e)=&browser.error{ui.colored_label(palette(ui).error,e);}
    let action=ui.add(primary(ui,match browser.mode{Mode::Export=>"Save backup",Mode::Restore=>"Check backup"}));
    self.metrics.data_file_action=Some(action.rect);chosen=action.clicked();
   });
            if let Some(folder) = navigate {
                browser.folder = folder;
                browser.refresh();
            }
            if open && chosen {
                match browser.target() {
                    Err(e) => browser.error = Some(e),
                    Ok(path) => match browser.mode {
                        Mode::Export => {
                            match backup::check_destination(&path, &self.path).and_then(|_| {
                                backup::export(&path, &self.progress, &self.scenarios)
                            }) {
                                Ok(warning) => {
                                    self.data_tools.message = Some(format!(
                                        "Backup saved: {}. {}",
                                        path.display(),
                                        warning.unwrap_or_default()
                                    ));
                                    open = false;
                                }
                                Err(e) => browser.error = Some(e),
                            }
                        }
                        Mode::Restore => {
                            self.data_tools.inspection =
                                Some(backup::Inspection::start(path, self.scenarios.clone()));
                            open = false;
                        }
                    },
                }
            }
            if open {
                self.data_tools.browser = Some(browser);
            }
        }
        if let Some(candidate) = self.data_tools.candidate.take() {
            let mut confirm = false;
            let mut cancel = false;
            egui::Window::new("Review restore").collapsible(false).default_width(560.).default_height(240.).resizable(false).max_width((ctx.content_rect().width()-24.).max(200.)).max_height((ctx.content_rect().height()-24.).max(140.)).vscroll(true).show(ctx,|ui|{ui.heading("Replace this profile with the checked backup?");ui.label(candidate.summary(&self.scenarios));ui.label("This replaces your current lab progress, drafts and favorites. Before replacing them, the app saves an exact safety copy beside the current progress file. Themes are unchanged.");ui.horizontal_wrapped(|ui|{let r=ui.add(primary(ui,"Restore checked backup"));self.metrics.data_confirm=Some(r.rect);confirm=r.clicked();
        let r=ui.add(button("Cancel — keep current work"));self.metrics.data_cancel=Some(r.rect);cancel=r.clicked();});});
            if confirm {
                if self.profile_lock.is_none() && !self.smoke {
                    self.data_tools.message=Some("Restore unavailable: this app does not hold the profile's writer lock. Close other copies and reopen.".into());
                } else {
                    let protected = self.protect_session_before_restore();
                    let result = protected
                        .and_then(|_| backup::restore(&self.path, &candidate, &self.scenarios));
                    match result {
                        Ok(safety) => {
                            self.progress = candidate.progress;
                            self.selected = None;
                            self.script_job = None;
                            self.ipv4_view = None;
                            self.dirty = false;
                            self.storage_error = None;
                            self.read_only = false;
                            self.data_tools.message = Some(format!(
                                "Progress restored. {}",
                                safety.safety_copy.map_or(
                                    "There was no previous profile to back up.".into(),
                                    |p| format!("Previous profile kept at {}", p.display())
                                )
                            ));
                            if let Some(warning) = safety.durability_warning {
                                self.data_tools
                                    .message
                                    .as_mut()
                                    .unwrap()
                                    .push_str(&format!(". {warning}"));
                            }
                        }
                        Err(e) => {
                            self.data_tools.message =
                                Some(format!("Restore failed; current work retained: {e}"))
                        }
                    }
                }
            } else if !cancel {
                self.data_tools.candidate = Some(candidate);
            }
        }
    }
}
