use super::*;
use super::debug_recovery::{paint,settle,tap,type_editor};
use dlms_learning_lab::{editor_handover as handover, editor_handover_source as binding,
    engine::State, environment_workspace as env, modal_editing::Key, modal_workspace::Event};
const ID:&str="lpic1-editor-handover";
fn opened()->(egui::Context,LabApp,usize){let ctx=egui::Context::default();let mut a=LabApp::from_context_with_catalog(&ctx,true,false,false,true);a.smoke=false;let i=a.scenarios.iter().position(|s|s.id==ID).unwrap();a.launch(i);(ctx,a,i)}
fn session(a:&LabApp)->&handover::Session{let State::EditorHandover{simulation}=&a.progress.attempts[ID].state else{panic!()};simulation}
fn session_mut(a:&mut LabApp)->&mut handover::Session{let State::EditorHandover{simulation}=&mut a.progress.attempts.get_mut(ID).unwrap().state else{panic!()};simulation}
fn complete(a:&mut LabApp,i:usize){let s=a.scenarios[i].clone();for event in &s.solution{engine::execute_for_goal(&mut a.progress.attempts.get_mut(ID).unwrap().state,&s.goal,event).unwrap();}assert!(progress::current_solved(&a.progress.attempts[ID],&s));}
fn reload(a:&mut LabApp){let raw=serde_json::to_vec(session(a)).unwrap();*session_mut(a)=serde_json::from_slice(&raw).unwrap();a.completion_index.clear();}
fn wait(ctx:&egui::Context,a:&mut LabApp){for _ in 0..2000{paint(ctx,a,vec![]);if a.editor_handover_run.is_none(){settle(ctx,a);return;}}panic!("Bounded automatic replay did not finish");}
fn click(ctx:&egui::Context,a:&mut LabApp,key:&str){let r=a.metrics.modal.controls.iter().find(|r|r.0==key).unwrap().1;tap(ctx,a,r);}
#[test]
fn saved_assignment_is_reverified_from_the_library_without_changing_learning_data(){
    let(ctx,mut a,i)=opened();complete(&mut a,i);let d=binding::source(&a.scenarios[i]).unwrap().clone();session_mut(&mut a).edit_setup(&d,true,"# Kept independent\nEDITOR='unfinished".into()).unwrap();reload(&mut a);
    let before=serde_json::to_value(&a.progress).unwrap();assert!(!progress::current_solved(&a.progress.attempts[ID],&a.scenarios[i]));a.selected=None;paint(&ctx,&mut a,vec![]);assert!(a.editor_handover_run.is_some());wait(&ctx,&mut a);
    assert!(progress::current_solved(&a.progress.attempts[ID],&a.scenarios[i]));assert!(session(&a).setup.assignment.result.as_ref().unwrap().verified());assert_eq!(serde_json::to_value(&a.progress).unwrap(),before);assert!(session(&a).setup.example.result.is_none());assert!(a.selected.is_none());
}
#[test]
fn selected_partial_buffer_is_not_repaired_or_counted_complete_by_automatic_replay(){
    let(ctx,mut a,i)=opened();complete(&mut a,i);let d=binding::source(&a.scenarios[i]).unwrap().clone();session_mut(&mut a).view=teaching::View::Assignment;
    for e in [Event::Key{key:Key::Character('i')},Event::Insert{text:"DRAFT".into()}]{session_mut(&mut a).apply_file(&d,false,&e).unwrap();}reload(&mut a);let files=session(&a).files.assignment.clone();wait(&ctx,&mut a);
    assert!(!progress::current_solved(&a.progress.attempts[ID],&a.scenarios[i]));assert!(!a.metrics.completion);assert_eq!(session(&a).files.assignment,files);assert!(session(&a).setup.assignment.result.as_ref().unwrap().verified());
}
#[test]
fn stop_suppresses_automatic_retry_and_manual_run_remains_available(){
    let(ctx,mut a,i)=opened();let d=binding::source(&a.scenarios[i]).unwrap().clone();let source="while :; do :; done\n";
    session_mut(&mut a).view=teaching::View::Assignment;session_mut(&mut a).edit_setup(&d,false,source.into()).unwrap();let record=env::run(&d.setup.assignment,source).unwrap();session_mut(&mut a).store_setup(&d,false,record).unwrap();reload(&mut a);
    paint(&ctx,&mut a,vec![]);assert!(a.editor_handover_run.is_some());click(&ctx,&mut a,"stop-setup");assert!(a.editor_handover_run.is_none());for _ in 0..30{paint(&ctx,&mut a,vec![]);assert!(a.editor_handover_run.is_none());}
    assert_eq!(session(&a).setup.assignment.source,source);assert!(!session(&a).setup.assignment.result.as_ref().unwrap().verified());
    type_editor(&ctx,&mut a,&d.example_setup);click(&ctx,&mut a,"run-setup");wait(&ctx,&mut a);assert!(session(&a).ready(&d,false));assert!(!session(&a).solved(&d));assert!(session(&a).assignment_launch.is_none());
}
#[test]
fn changing_source_cancels_an_inflight_replay_and_does_not_restore_stale_success(){
    let(ctx,mut a,i)=opened();complete(&mut a,i);reload(&mut a);paint(&ctx,&mut a,vec![]);assert!(a.editor_handover_run.is_some());
    let d=binding::source(&a.scenarios[i]).unwrap().clone();session_mut(&mut a).edit_setup(&d,false,"EDITOR=changed\n".into()).unwrap();paint(&ctx,&mut a,vec![]);assert!(a.editor_handover_run.is_none());assert!(session(&a).setup.assignment.result.is_none());assert!(!progress::current_solved(&a.progress.attempts[ID],&a.scenarios[i]));
}

#[test]
fn practice_completion_banner_is_independent_of_retained_assignment_credit_after_replay(){
    let(ctx,mut a,i)=opened();complete(&mut a,i);let d=binding::source(&a.scenarios[i]).unwrap().clone();
    session_mut(&mut a).view=teaching::View::Example;
    session_mut(&mut a).edit_setup(&d,true,"# Independent practice\nEDITOR='unfinished".into()).unwrap();
    reload(&mut a);wait(&ctx,&mut a);
    assert!(progress::current_solved(&a.progress.attempts[ID],&a.scenarios[i]));
    assert!(!a.metrics.completion,"Practice is unfinished even when assignment credit is valid");
    click(&ctx,&mut a,"assignment");assert!(a.metrics.completion);
    click(&ctx,&mut a,"example");assert!(!a.metrics.completion);
    assert!(progress::current_solved(&a.progress.attempts[ID],&a.scenarios[i]));
}
