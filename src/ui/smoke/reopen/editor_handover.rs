//! One closed two-process handover fixture. All answers enter actual widgets;
//! these observers only inspect normal loaded/saved state and runtime proof.
use super::*;
use dlms_learning_lab::{editor_handover as work, editor_handover_source as binding, modal_editing::Mode};
const ID: &str = "lpic1-editor-handover";
const DRAFT: &str = "# Independent practice draft\nEDITOR='unfinished";
fn state(a: &Attempt) -> &work::Session {
    let engine::State::EditorHandover { simulation } = &a.state else { panic!("fixed handover state") };
    simulation
}
impl LabApp {
    fn handover_reopen_private(&self, session: &Session) {
        assert_eq!(session.case, Case::EditorHandover);
        assert!(!self.smoke && self._smoke_profile.is_none());
        assert!(self.profile_lock.is_some());
        assert!(!self.read_only && !self.preferences_read_only);
        assert!(self.storage_error.is_none() && self.preferences_error.is_none());
        session.verify_profile_paths(&self.path).unwrap();
        assert_eq!(shell::EVALUATOR_REVISION,19);
    }
    fn handover_reopen_saved(&self, verified: bool, repaired: bool) {
        assert_eq!(self.progress.attempts.len(),1);
        assert_eq!(self.progress.favorites, std::collections::BTreeSet::from([ID.to_owned()]));
        let source=self.scenarios.iter().find(|s|s.id==ID).unwrap();
        let attempt=&self.progress.attempts[ID];
        binding::context(attempt,source).unwrap();
        let s=state(attempt);
        let d=binding::source(source).unwrap();
        let run=s.setup.assignment.result.as_ref().unwrap();
        assert_eq!(run.verified(),verified);
        assert_eq!(run.revision,shell::EVALUATOR_REVISION);
        assert_eq!(run.results.len(),d.setup.assignment.cases.len());
        assert_eq!(progress::current_solved(attempt,source),verified);
        assert!(s.assignment_launch.is_some());
        assert!(s.files.assignment.checked);
        if repaired {
            assert!(s.checks(d,true).iter().all(|r|r.0));
            assert!(!s.files.practice.editor().modified());
            assert!(!s.files.practice.editor().contents().contains("DRAFT"));
            assert_eq!(s.files.practice.editor().mode(),Mode::Normal);
        } else {
            assert_eq!(s.setup.example.source,DRAFT);
            assert!(s.setup.example.result.is_none());
            assert!(s.example_launch.is_none());
            assert!(s.files.practice.editor().modified());
            assert!(s.files.practice.editor().contents().contains("DRAFT"));
            assert_eq!(s.files.practice.editor().mode(),Mode::Insert);
            assert!(!s.files.practice.checked);
        }
        assert_eq!(self.scale,1.5);
        assert!(theme::inventory().iter().find(|t|t.id==self.theme_id).unwrap().light);
    }
    /// Called before the first render/VM slice. Never grants a runtime token or
    /// edits a state, selection, draft, completion marker or provenance receipt.
    pub(in crate::ui) fn reopen_editor_handover_loaded(&self, session: &Session) {
        if session.case!=Case::EditorHandover || session.role!=Role::Read { return; }
        self.handover_reopen_private(session);
        self.handover_reopen_saved(false,false);
        let s=state(&self.progress.attempts[ID]);
        let source=self.scenarios.iter().find(|s|s.id==ID).unwrap();
        assert!(s.setup.assignment.needs_reverification(&binding::source(source).unwrap().setup.assignment));
        let expected:Observed=serde_json::from_slice(&bounded_file(&session.root.join("writer-observed.json"),JSON_CAP).unwrap()).unwrap();
        let (loaded,raw,prefs)=session.observed(self);
        assert_eq!(raw,bounded_file(&session.root.join("writer-progress.raw"),16_000_000).unwrap());
        assert_eq!(prefs,bounded_file(&session.root.join("writer-preferences.raw"),4096).unwrap());
        assert_eq!(loaded,expected,"ordinary cold load preserves every raw/portable/provenance/appearance field");
        session.write("reader-loaded.json",&loaded).unwrap();
        session.write("reader-before-replay.json",&serde_json::json!({
            "evaluator_revision":shell::EVALUATOR_REVISION,
            "saved_record_verified":s.setup.assignment.result.as_ref().unwrap().verified(),
            "current_assignment_credit":progress::current_solved(&self.progress.attempts[ID],source),
            "drafts_preserved":true,
            "note":"Observed ordinary load before the first bounded replay slice; draft assertions above include Insert mode and unsaved bytes."
        })).unwrap();
    }
    pub(in crate::ui) fn editor_handover_replayed(&self, ctx:&egui::Context) -> bool {
        let session=self.reopen.as_ref().expect("private handover reopen");
        self.handover_reopen_private(session);
        assert_eq!(session.role,Role::Read);
        if self.editor_handover_run.is_some() { ctx.request_repaint();return false; }
        self.handover_reopen_saved(true,false);
        let expected:Observed=serde_json::from_slice(&bounded_file(&session.root.join("writer-observed.json"),JSON_CAP).unwrap()).unwrap();
        let (observed,_,_)=session.observed(self);
        assert_eq!(observed,expected,"fresh replay changes no portable learning data or receipts");
        session.write("reader-replayed.json",&serde_json::json!({"observed":observed,
            "record_verified":state(&self.progress.attempts[ID]).setup.assignment.result.as_ref().unwrap().verified(),
            "current_assignment_credit":progress::current_solved(&self.progress.attempts[ID],self.scenarios.iter().find(|s|s.id==ID).unwrap())})).unwrap();
        true
    }
    pub(in crate::ui) fn finish_editor_handover_reopen(&mut self,ctx:&egui::Context) {
        let session=self.reopen.as_ref().unwrap().clone();
        self.handover_reopen_private(&session);
        let key=egui::Id::new("handover-reopen-finished");
        if !ctx.data(|d|d.get_temp::<bool>(key)).unwrap_or(false) {
            self.handover_reopen_saved(true,session.role==Role::Read);
            self.save();
            assert!(!self.dirty && self.storage_error.is_none());
            let (observed,raw,prefs)=session.observed(self);
            let (progress,preferences,observation)=if session.role==Role::Write {
                ("writer-progress.raw","writer-preferences.raw","writer-observed.json")
            }else {("reader-progress.raw","reader-preferences.raw","reader-repaired.json")};
            session.write_raw(progress,&raw,16_000_000).unwrap();
            session.write_raw(preferences,&prefs,4096).unwrap();
            session.write(observation,&observed).unwrap();
            ctx.data_mut(|d|d.insert_temp(key,true));
            println!("PASS native reopen {}: inherited-editor actual setup, saved file, independent Insert draft and bounded replay",if session.role==Role::Write{"writer"}else{"reader"});
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
