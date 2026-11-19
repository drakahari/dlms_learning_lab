//! Fixed normal-profile administration reopen. Only native controls create work.
use super::*;
use crate::ui::smoke::mailbox::{Op, translate};
use dlms_learning_lab::{mailbox_source as binding, mailbox_workspace as work};
const ID: &str = "lpic2-mailbox-administration-recovery";
pub(super) const CAPTURES: [&str; 6] = [
    "reopen-mail-admin-writer-example", "reopen-mail-admin-writer-assignment",
    "reopen-mail-admin-reader-assignment", "reopen-mail-admin-reader-example",
    "reopen-mail-admin-repaired-assignment", "reopen-mail-admin-repaired-example",
];
fn actions(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    if practice { serde_json::from_str(include_str!("../../../../tests/fixtures/mailbox-administration-worked-v1.json")).unwrap() }
    else { source.solution.iter().map(|s|serde_json::from_str(s.strip_prefix(binding::PREFIX).unwrap()).unwrap()).collect() }
}
fn pending(source: &pack::Scenario, practice: bool) -> Vec<work::Action> {
    let mut out=vec![];
    let all=actions(source,practice);
    let config=all.iter().find_map(|a|if let work::Action::Edit{text}=a{Some(text)}else{None}).unwrap();
    let unsent=all.iter().find_map(|a|if let work::Action::Admin{line}=a{(line.starts_with("doveadm expunge ")&&line.contains("mailbox Archive")).then_some(line)}else{None}).unwrap();
    for a in &all {
        out.push(a.clone());
        if matches!(a,work::Action::Admin{line}if line.starts_with("doveadm expunge ")) { break; }
    }
    out.extend([
        work::Action::Check,
        work::Action::Input{text:unsent.clone()},
        work::Action::Edit{text:format!("# {} cleanup in progress; Archive command is not run yet\n{config}",if practice{"Guided"}else{"Assignment"})},
    ]);
    out
}
fn verify_partial(a:&LabApp) {
    let source=a.scenarios.iter().find(|s|s.id==ID).unwrap();
    let attempt=&a.progress.attempts[ID];binding::context(attempt,source).unwrap();
    let d=binding::source(source).unwrap();
    let engine::State::MailboxAdministration{simulation}=&attempt.state else{panic!()};
    for (scope,t,practice)in[(&simulation.assignment,&d.assignment,false),(&simulation.practice,&d.example,true)] {
        let mut expected=work::Scope::new(t).unwrap();
        for action in pending(source,practice){expected.apply(t,&action).unwrap();}
        assert_eq!(scope,&expected);
        assert!(!scope.checked);
        let checks=scope.checks(t);
        assert!(checks.iter().any(|r|r.passed));assert!(checks.iter().any(|r|!r.passed));
        assert!(scope.input.contains("mailbox Archive"));assert!(scope.connection.is_none());
    }
    assert!(!progress::current_solved(attempt,source));
}
fn plan(read:bool)->&'static Vec<Op>{
    static W:std::sync::OnceLock<Vec<Op>>=std::sync::OnceLock::new();
    static R:std::sync::OnceLock<Vec<Op>>=std::sync::OnceLock::new();
    (if read{&R}else{&W}).get_or_init(||{
        let labs=pack::current().unwrap();let source=labs.iter().find(|s|s.id==ID).unwrap();let mut p=vec![];
        if !read{p.push(Op::ColdAppearance);}
        p.push(Op::Open(ID.into()));
        for practice in if read{[false,true]}else{[true,false]}{
            p.push(Op::Click(if practice{"example"}else{"assignment"}.into()));
            if read{
                p.push(Op::Shot(CAPTURES[if practice{3}else{2}].into(),"command".into()));
                p.extend([Op::Click("run-admin".into()),Op::Click("save".into()),Op::Click("restart".into()),Op::Click("check".into()),Op::Check(true,practice),Op::Shot(CAPTURES[if practice{5}else{4}].into(),"result".into())]);
            }else{
                for action in pending(source,practice){translate(&mut p,&action);}
                p.extend([Op::NotRun(practice),Op::Shot(CAPTURES[if practice{0}else{1}].into(),"command".into())]);
            }
        }
        p.push(Op::ColdFinish);p
    })
}
impl LabApp{
    pub(in crate::ui) fn finish_mailbox_cold_reopen(&mut self,ctx:&egui::Context){
        if self.reopen.as_ref().unwrap().case==Case::MailboxAdministration{self.finish_mail_admin_reopen(ctx);}else{self.finish_mailbox_reopen(ctx);}
    }
    pub(super) fn reopen_mail_admin_tick(&mut self,ctx:&egui::Context,session:&Session){
        assert_eq!(session.case,Case::MailboxAdministration);
        assert!(!self.smoke&&self._smoke_profile.is_none()&&self.profile_lock.is_some());
        assert!(!self.read_only&&!self.preferences_read_only);session.verify_profile_paths(&self.path).unwrap();
        if session.role==Role::Read{
            let key=egui::Id::new("mail-admin-normal-load-observed");
            if !ctx.data(|d|d.get_temp::<bool>(key)).unwrap_or(false){
                let expected:Observed=serde_json::from_slice(&bounded_file(&session.root.join("writer-observed.json"),JSON_CAP).unwrap()).unwrap();
                let(loaded,raw,prefs)=session.observed(self);assert_eq!(loaded,expected);
                assert_eq!(raw,bounded_file(&session.root.join("writer-progress.raw"),16_000_000).unwrap());
                assert_eq!(prefs,bounded_file(&session.root.join("writer-preferences.raw"),4096).unwrap());
                verify_partial(self);assert_eq!(self.scale,SCALE);assert_eq!(self.theme_id,THEME);
                session.write("reader-loaded.json",&loaded).unwrap();ctx.data_mut(|d|d.insert_temp(key,true));
            }
        }
        self.mailbox_flow(ctx,plan(session.role==Role::Read));
    }
    pub(in crate::ui) fn finish_mail_admin_reopen(&mut self,ctx:&egui::Context){
        let session=self.reopen.as_ref().unwrap().clone();assert_eq!(session.case,Case::MailboxAdministration);
        if session.role==Role::Write{verify_partial(self);}else{
            let source=self.scenarios.iter().find(|s|s.id==ID).unwrap();let a=&self.progress.attempts[ID];let d=binding::source(source).unwrap();
            let engine::State::MailboxAdministration{simulation}=&a.state else{panic!()};
            assert!(progress::current_solved(a,source));assert!(simulation.practice.checks(&d.example).iter().all(|r|r.passed));
        }
        let key=egui::Id::new("mail-admin-cold-finished");
        if !ctx.data(|d|d.get_temp::<bool>(key)).unwrap_or(false){
            self.save();assert!(!self.dirty&&self.storage_error.is_none());let(observed,raw,prefs)=session.observed(self);
            let names=if session.role==Role::Write{("writer-progress.raw","writer-preferences.raw","writer-observed.json")}else{("reader-progress.raw","reader-preferences.raw","reader-repaired.json")};
            session.write_raw(names.0,&raw,16_000_000).unwrap();session.write_raw(names.1,&prefs,4096).unwrap();session.write(names.2,&observed).unwrap();
            ctx.data_mut(|d|d.insert_temp(key,true));println!("PASS native reopen {}: separate admin drafts, partial cleanup and unsent command restored before native repair",if session.role==Role::Write{"writer"}else{"reader"});
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
#[test]
fn mailbox_admin_cold_plan_restores_partial_work_before_running_saved_input(){
    for read in[false,true]{let p=plan(read);assert!(matches!(p.last(),Some(Op::ColdFinish)));assert_eq!(p.iter().filter(|o|matches!(o,Op::ColdAppearance)).count(),usize::from(!read));assert!(!p.iter().any(|o|matches!(o,Op::Style(..))));}
    let labs=pack::current().unwrap();let source=labs.iter().find(|s|s.id==ID).unwrap();let d=binding::source(source).unwrap();
    for(practice,t)in[(false,&d.assignment),(true,&d.example)]{let mut scope=work::Scope::new(t).unwrap();for a in pending(source,practice){scope.apply(t,&a).unwrap();}assert!(!scope.checked);assert!(scope.checks(t).iter().any(|r|!r.passed));let line=scope.input.clone();for a in[work::Action::Admin{line},work::Action::Save,work::Action::Restart,work::Action::Check]{scope.apply(t,&a).unwrap();}assert!(scope.checks(t).iter().all(|r|r.passed));}
}
