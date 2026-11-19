use dlms_learning_lab::{editor_handover::Session, editor_handover_source as binding,
    engine::{self, State}, environment_workspace as env, modal_editing::Key,
    modal_workspace::Event, pack};
fn lab()->pack::Scenario{pack::load(include_str!("../scenarios/editor-handover-lessons-v1.json")).unwrap().remove(0)}
fn state(s:&State)->&Session{let State::EditorHandover{simulation}=s else{panic!()};simulation}
fn complete(s:&pack::Scenario)->Session{let mut v=s.initial.clone();for e in &s.solution{engine::execute_for_goal(&mut v,&s.goal,e).unwrap();}state(&v).clone()}
fn decode(s:&Session)->Session{serde_json::from_slice(&serde_json::to_vec(s).unwrap()).unwrap()}
fn replay(s:&mut Session,d:&dlms_learning_lab::editor_handover::Definition,practice:bool){let r=env::run(s.task(d,practice),&s.draft(practice).source).unwrap();s.reverify_setup(d,practice,r).unwrap();}
#[test]
fn actual_replay_restores_only_prior_checked_work_without_changing_serialized_learning_data(){
    let lab=lab();let d=binding::source(&lab).unwrap();let before=complete(&lab);assert!(before.solved(d));let mut loaded=decode(&before);assert!(!loaded.solved(d));assert!(!loaded.draft(false).result.as_ref().unwrap().verified());
    replay(&mut loaded,d,false);assert!(loaded.solved(d));assert!(loaded.draft(false).result.as_ref().unwrap().verified());assert_eq!(serde_json::to_value(&loaded).unwrap(),serde_json::to_value(&before).unwrap());
}
#[test]
fn replay_cannot_invent_a_launch_or_check_or_fill_an_assignment(){
    let lab=lab();let d=binding::source(&lab).unwrap();let mut before=complete(&lab);before.files.assignment.checked=false;let mut loaded=decode(&before);replay(&mut loaded,d,false);assert!(loaded.opened(d,false));assert!(!loaded.files.assignment.checked);assert!(!loaded.solved(d));
    let mut before=d.new_session().unwrap();before.edit_setup(d,false,d.example_setup.clone()).unwrap();before.store_setup(d,false,env::run(&d.setup.assignment,&d.example_setup).unwrap()).unwrap();let file=before.files.assignment.clone();let mut loaded=decode(&before);replay(&mut loaded,d,false);assert!(loaded.assignment_launch.is_none());assert!(!loaded.solved(d));assert_eq!(loaded.files.assignment,file);
}
#[test]
fn broken_buffers_and_independent_practice_drafts_survive_reverification_without_credit(){
    let lab=lab();let d=binding::source(&lab).unwrap();let mut before=complete(&lab);before.edit_setup(d,true,"# Independent unfinished practice\nEDITOR='draft".into()).unwrap();
    before.apply_file(d,false,&Event::Key{key:Key::Character('i')}).unwrap();before.apply_file(d,false,&Event::Insert{text:"BROKEN".into()}).unwrap();let files=before.files.clone();let practice=before.setup.example.clone();let mut loaded=decode(&before);replay(&mut loaded,d,false);assert!(!loaded.solved(d));assert_eq!(loaded.files,files);assert_eq!(loaded.setup.example,practice);
}
#[test]
fn changed_observations_clear_prior_launch_and_check_instead_of_relabelling_old_results(){
    let lab=lab();let d=binding::source(&lab).unwrap();let mut loaded=decode(&complete(&lab));loaded.setup.assignment.result.as_mut().unwrap().results[0].parent.stderr="Different earlier output".into();d.context(&loaded).unwrap();let files=loaded.files.assignment.editors.clone();replay(&mut loaded,d,false);assert!(loaded.ready(d,false));assert!(loaded.assignment_launch.is_none());assert!(!loaded.files.assignment.checked);assert!(!loaded.solved(d));assert_eq!(loaded.files.assignment.editors,files);
}
#[test]
fn forged_fresh_records_and_live_or_never_run_results_cannot_use_replay_path(){
    let lab=lab();let d=binding::source(&lab).unwrap();let before=complete(&lab);let r=env::run(&d.setup.assignment,&before.setup.assignment.source).unwrap();let unverified:env::Record=serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();let mut loaded=decode(&before);let copy=loaded.clone();assert!(loaded.reverify_setup(d,false,unverified).is_err());assert_eq!(loaded,copy);
    let mut live=before;assert!(live.reverify_setup(d,false,r).is_err());let mut fresh=d.new_session().unwrap();let r=env::run(&d.setup.assignment,&d.example_setup).unwrap();assert!(fresh.reverify_setup(d,false,r).is_err());assert_eq!(fresh,d.new_session().unwrap());
}
