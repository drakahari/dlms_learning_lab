//! Composition of the bounded environment VM and native virtual-file editor.
//! EDITOR is data in a simulated child's environment, never a host executable.
use crate::{environment_workspace as env, modal_workspace as files, script_teaching::View};
use serde::{Deserialize, Serialize};

pub const MAX_SESSION: usize = 3 * 1024 * 1024;
pub const HELP: &str = "Run setup evaluates the script with the existing bounded shell simulator against each supplied sample environment. The simulated handover application reads its child's EDITOR value. Only the exact names vi and vim select this activity's bounded vi-style editor; paths, flags, nano, Emacs and other programs are not launched or emulated. Other real applications can use VISUAL, EDITOR or their own settings and fallback rules. This exercise does not alter a login profile, host environment or default application. Edit and save only the supplied virtual files. Stop cancels the in-process setup run; it never signals another application.";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub version: u32,
    pub setup: env::Definition,
    pub editing: files::Definition,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub example_setup: String,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.id != self.editing.id
            || self.version != self.editing.version
            || self.setup.id != format!("{}-setup", self.id)
            || self.example_setup.trim().is_empty()
            || self.example_setup.len() > env::MAX_SOURCE
            || self.example_setup.contains('\0')
            || !serde_json::to_vec(self).is_ok_and(|b| b.len() <= MAX_SESSION / 4)
        {
            return Err(
                "Match the bounded editor-handover identity and separate setup/example tasks."
                    .into(),
            );
        }
        self.setup.validate()?;
        self.editing.validate()?;
        for t in [&self.setup.assignment, &self.setup.example] {
            if t.path_lookup.is_some()
                || t.cases.iter().any(|c| {
                    c.parent
                        .get("EDITOR")
                        .is_none_or(|v| !v.exported || !v.value.as_deref().is_some_and(supported))
                        || !c.input.files.is_empty()
                })
            {
                return Err("Editor setup needs exported vi/vim outcomes and independent environment-only samples.".into());
            }
        }
        Ok(())
    }
    pub fn validate_example(&self) -> Result<(), String> {
        self.validate()?;
        self.setup.assignment.validate_child_examples()?;
        self.setup.example.validate_child_examples()?;
        let mut s = self.new_session()?;
        s.edit_setup(self, true, self.example_setup.clone())?;
        let r = env::run(&self.setup.example, &self.example_setup)?;
        s.store_setup(self, true, r)?;
        s.open_editor(self, true)?;
        s.files.show_worked(&self.editing)?;
        s.files
            .practice
            .apply(&self.editing.example, &files::Event::Check {})?;
        if !s.checks(self, true).iter().all(|r| r.0) {
            return Err(
                "The worked setup and saved-file example must both reach their outcomes.".into(),
            );
        }
        Ok(())
    }
    pub fn new_session(&self) -> Result<Session, String> {
        self.validate()?;
        Ok(Session {
            id: self.id.clone(),
            version: self.version,
            view: View::Introduction,
            setup: env::Session::new(self.setup.clone()),
            files: self.editing.new_session()?,
            assignment_launch: None,
            example_launch: None,
        })
    }
    pub fn context(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.id != self.id || s.version != self.version {
            return Err("Saved editor handover belongs to another lesson or version.".into());
        }
        s.setup.validate(&self.setup)?;
        self.editing.context(&s.files)?;
        for practice in [false, true] {
            if let Some(launch) = s.launch(practice)
                && s.observed_editor(practice).as_ref() != Some(launch)
            {
                return Err(
                    "Saved editor choice does not match its actual child observation.".into(),
                );
            }
        }
        Ok(())
    }
}
fn supported(name: &str) -> bool {
    matches!(name, "vi" | "vim")
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub sample: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub editor: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub version: u32,
    pub view: View,
    pub setup: env::Session,
    pub files: files::Session,
    pub assignment_launch: Option<Launch>,
    pub example_launch: Option<Launch>,
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.id.is_empty()
            || self.id.len() > 80
            || !serde_json::to_vec(self).is_ok_and(|b| b.len() <= MAX_SESSION)
            || [&self.assignment_launch, &self.example_launch]
                .into_iter()
                .flatten()
                .any(|l| !supported(&l.editor) || l.sample.len() > 120)
        {
            return Err("Invalid or oversized editor-handover progress; original retained.".into());
        }
        self.setup.validate(&self.setup.definition)?;
        self.files.validate()
    }
    pub fn task<'a>(&self, d: &'a Definition, practice: bool) -> &'a env::Task {
        if practice {
            &d.setup.example
        } else {
            &d.setup.assignment
        }
    }
    pub fn draft(&self, practice: bool) -> &env::Draft {
        if practice {
            &self.setup.example
        } else {
            &self.setup.assignment
        }
    }
    pub fn launch(&self, practice: bool) -> &Option<Launch> {
        if practice {
            &self.example_launch
        } else {
            &self.assignment_launch
        }
    }
    fn invalidate(&mut self, practice: bool) {
        if practice {
            self.example_launch = None;
            self.files.practice.checked = false;
        } else {
            self.assignment_launch = None;
            self.files.assignment.checked = false;
        }
    }
    pub fn edit_setup(
        &mut self,
        d: &Definition,
        practice: bool,
        source: String,
    ) -> Result<(), String> {
        d.context(self)?;
        let draft = if practice {
            &mut self.setup.example
        } else {
            &mut self.setup.assignment
        };
        let changed = draft.source != source;
        draft.edit(source)?;
        if changed {
            self.invalidate(practice);
        }
        Ok(())
    }
    pub fn store_setup(
        &mut self,
        d: &Definition,
        practice: bool,
        record: env::Record,
    ) -> Result<(), String> {
        d.context(self)?;
        let (task, draft) = if practice {
            (&d.setup.example, &mut self.setup.example)
        } else {
            (&d.setup.assignment, &mut self.setup.assignment)
        };
        if !draft.store(record, task) {
            return Err("Setup result is stale or unverified; run the current draft again.".into());
        }
        self.invalidate(practice);
        Ok(())
    }
    pub fn begin_run(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.context(self)?;
        if practice {
            self.setup.example.result = None;
        } else {
            self.setup.assignment.result = None;
        }
        self.invalidate(practice);
        Ok(())
    }
    /// Reverify a decoded observation through a real current VM run. Keep an
    /// earlier launch/check only when every observed outcome is identical. A
    /// missing launch/check is never created by this path, and files stay intact.
    pub fn reverify_setup(
        &mut self,
        d: &Definition,
        practice: bool,
        record: env::Record,
    ) -> Result<(), String> {
        d.context(self)?;
        let draft = self.draft(practice);
        if !draft.needs_reverification(self.task(d, practice)) {
            return Err("Only a saved unverified setup observation can be rechecked automatically.".into());
        }
        let same = draft.result.as_ref().is_some_and(|old| {
            old.revision == record.revision && old.task == record.task
                && old.source == record.source && old.results == record.results
        });
        let launch = self.launch(practice).clone();
        let checked = if practice {self.files.practice.checked} else {self.files.assignment.checked};
        self.store_setup(d, practice, record)?;
        if same && self.ready(d, practice) && launch.is_some() && launch == self.observed_editor(practice) {
            if practice {self.example_launch = launch; self.files.practice.checked = checked;}
            else {self.assignment_launch = launch; self.files.assignment.checked = checked;}
        }
        Ok(())
    }
    fn observed_editor(&self, practice: bool) -> Option<Launch> {
        let r = self.draft(practice).result.as_ref()?;
        let row = r.results.first()?;
        let value = row.bindings.as_ref()?.child.get("EDITOR")?;
        let sample = r.task.cases.first()?.input.name.clone();
        supported(value).then(|| Launch {
            sample,
            editor: value.clone(),
        })
    }
    pub fn ready(&self, d: &Definition, practice: bool) -> bool {
        let checks = env::checks(self.task(d, practice), self.draft(practice));
        !checks.is_empty() && checks.iter().all(|r| r.0)
    }
    pub fn opened(&self, d: &Definition, practice: bool) -> bool {
        self.ready(d, practice)
            && self.launch(practice).is_some()
            && self.launch(practice) == &self.observed_editor(practice)
    }
    pub fn open_editor(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.context(self)?;
        if !self.ready(d, practice) {
            return Err("Run setup and repair the failed sample environments first. A private EDITOR value is not inherited by the handover application.".into());
        }
        let observed = self.observed_editor(practice).ok_or(
            "The simulated child has no supported EDITOR choice; no program was launched.",
        )?;
        if practice {
            self.example_launch = Some(observed);
        } else {
            self.assignment_launch = Some(observed);
        }
        Ok(())
    }
    pub fn apply_file(
        &mut self,
        d: &Definition,
        practice: bool,
        event: &files::Event,
    ) -> Result<(), String> {
        d.context(self)?;
        if !matches!(event, files::Event::Check {}) && !self.opened(d, practice) {
            return Err(
                "Configure and open the simulated editor before changing its virtual files.".into(),
            );
        }
        self.files.apply(&d.editing, practice, event)
    }
    pub fn checks(&self, d: &Definition, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = d.context(self) {
            return vec![(false, e)];
        }
        let mut rows = env::checks(self.task(d, practice), self.draft(practice));
        rows.push((self.opened(d, practice), if self.opened(d, practice) {
            format!("The handover application opened the bounded editor selected by its actual inherited EDITOR: {}.", self.launch(practice).as_ref().unwrap().editor)
        } else { "Open the virtual file with the verified inherited editor choice. Setup and saved-file work are both required.".into() }));
        rows.extend(d.editing.checks(&self.files, practice));
        rows
    }
    pub fn solved(&self, d: &Definition) -> bool {
        self.checks(d, false).iter().all(|r| r.0)
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.context(self)?;
        if practice {
            self.setup.example = env::Draft::new(&d.setup.example);
            self.files.reset_practice(&d.editing)?;
        } else {
            self.setup.assignment = env::Draft::new(&d.setup.assignment);
            self.files.reset_assignment(&d.editing)?;
        }
        self.invalidate(practice);
        Ok(())
    }
}
