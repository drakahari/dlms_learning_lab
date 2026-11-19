use dlms_learning_lab::{course, engine, pack, progress::{self, Attempt, Progress}, sieve_source as binding, sieve_workspace::{Action, Definition, Session}};
const ID: &str = "lpic2-procmail-sieve-migration";
fn source() -> pack::Scenario { pack::load_catalog(&[include_str!("../scenarios/sieve-filter-lessons-v1.json"),include_str!("../scenarios/procmail-migration-lessons-v1.json")]).unwrap().pop().unwrap() }
fn run(s: &mut Session,d:&Definition,code:&str,practice:bool){
 s.action(d,&Action::Edit{script:code.into()},practice).unwrap();
 s.action(d,&Action::Run,practice).unwrap();
 for _ in 0..12{s.action(d,&Action::Tick,practice).unwrap();}
}
#[test]
fn exact477_scenarios_and189_units_precede_one_distinct_migration_task(){
 let before=pack::accepted477().unwrap();let now=pack::accepted478().unwrap();
 assert_eq!((before.len(),now.len()),(477,478));assert_eq!(serde_json::to_value(&now[..477]).unwrap(),serde_json::to_value(&before).unwrap());assert_eq!(now[477].id,ID);
 let old=course::accepted477(&before).unwrap();let units=course::accepted478(&now).unwrap();assert_eq!((old.len(),units.len()),(189,190));
 assert_eq!(serde_json::to_value(units.iter().filter(|u|u.id!="linux-procmail-migration").collect::<Vec<_>>()).unwrap(),serde_json::to_value(old).unwrap());
 let at=units.iter().position(|u|u.id=="linux-procmail-migration").unwrap();assert_eq!(units[at].labs,[ID]);assert_eq!(units[at].prerequisites,["linux-sieve-filters"]);
 for(track,n)in[("CCNA",113),("LPIC-1",154),("LPIC-2",131),("Bash/Shell",80)]{assert_eq!(now.iter().filter(|s|s.track==track).count(),n);}
}
#[test]
fn taught_method_and_separate_example_work_without_assignment_credit(){
 let source=source();let d=binding::source(&source).unwrap();let mut s=Session::new(d).unwrap();let original=s.assignment.clone();
 run(&mut s,d,&d.example.reference,true);assert!(s.practice.solved(&d.example));assert_eq!(s.assignment,original);assert!(!d.solved(&s));
 let mut state=engine::State::SieveFiltering{simulation:Box::new(s)};
 for action in &source.solution{engine::execute_for_goal(&mut state,&source.goal,action).unwrap();}
 assert!(engine::solved(&state,&source.goal),"{:?}",engine::checks(&state,&source.goal));
}
#[test]
fn equivalent_supported_branches_pass_and_plausible_errors_do_not(){
 let source=source();let d=binding::source(&source).unwrap();let mut s=Session::new(d).unwrap();
 run(&mut s,d,&d.assignment.starter,false);assert!(d.assignment.delivery_hint(&d.assignment.cases[0],&s.assignment.results[0]).unwrap().contains("No required filing happened"));
 let alternate=r#"require "fileinto";if header :is "X-Priority" "urgent" {if header :is "X-Team" "operations"{fileinto "Archive";}fileinto "Urgent";}else{if header :is "X-Team" "operations"{fileinto "Archive";}keep;}"#;
 run(&mut s,d,alternate,false);assert!(d.solved(&s),"{:?}",d.checks(&s));
 let wrong=[d.assignment.starter.clone(),d.assignment.reference.replace("\nkeep;\n","\n"),d.assignment.reference.replacen("fileinto \"Archive\";","fileinto \"Archive\"; stop;",1),d.assignment.reference.replace("    stop;\n", ""),d.assignment.reference.replace("header :is","header :contains")];
 for bad in wrong{run(&mut s,d,&bad,false);assert!(!d.solved(&s),"wrong filter passed: {bad}");assert!(s.assignment.results.iter().any(|r|r.passed));assert!(d.checks(&s).iter().any(|c|!c.passed&&c.detail.contains("Expected:")));}
 run(&mut s,d,&d.assignment.reference,false);assert!(d.solved(&s));s.action(d,&Action::Edit{script:"keep;".into()},false).unwrap();assert!(!d.solved(&s));assert!(s.assignment.results.is_empty());
}
#[test]
fn guided_missing_keep_experiment_teaches_the_actual_implicit_keep_difference(){
 let source=source();let d=binding::source(&source).unwrap();let mut s=Session::new(d).unwrap();let original=s.assignment.clone();
 let bad=d.example.reference.replace("\nkeep;\n", "\n");run(&mut s,d,&bad,true);assert!(!s.practice.solved(&d.example));
 assert!(s.practice.results[0].passed);assert!(!s.practice.results[1].passed);assert_eq!(s.practice.results[1].actual.iter().map(String::as_str).collect::<Vec<_>>(),["Sketches"]);assert_eq!(s.assignment,original);
 assert!(d.example.delivery_hint(&d.example.cases[1],&s.practice.results[1]).unwrap().contains("cancels implicit keep"));
 run(&mut s,d,&d.example.reference,true);assert!(s.practice.solved(&d.example));assert_eq!(s.assignment,original);
}
#[test]
fn legacy_context_is_inert_bounded_and_absent_from_old_serialized_tasks(){
 let source=source();let d=binding::source(&source).unwrap();let mut t=d.assignment.clone();
 let p=t.parse(&t.reference).unwrap();let before=t.evaluate(&t.cases[0],&p);
 t.legacy_recipe=Some("| NEVER EXECUTED: /host/path $(command)".into());t.validate().unwrap();assert_eq!(t.evaluate(&t.cases[0],&p),before);
 assert!(t.parse(t.legacy_recipe.as_ref().unwrap()).is_err());
 for bad in [" ".into(),"x".repeat(4097),"\0".into(),"\u{1b}".into()]{t.legacy_recipe=Some(bad);assert!(t.validate().unwrap_err().contains("Legacy recipe"));}
 let old=pack::load_catalog(&[include_str!("../scenarios/sieve-filter-lessons-v1.json")]).unwrap();
 for source in old{let d=binding::source(&source).unwrap();assert!(d.assignment.legacy_recipe.is_none());assert!(serde_json::to_value(&d.assignment).unwrap().get("legacy_recipe").is_none());}
}
#[test]
fn ordinary_progress_preserves_distinct_partial_work_stop_retry_and_reset(){
 let source=source();let d=binding::source(&source).unwrap();let mut s=Session::new(d).unwrap();
 run(&mut s,d,&d.example.reference,true);run(&mut s,d,&d.assignment.reference.replace("\nkeep;\n", "\n"),false);s.assignment.selected=1;assert!(!d.solved(&s));
 let before=s.clone();let mut a=Attempt::new(&source,1);a.state=engine::State::SieveFiltering{simulation:Box::new(s)};
 let mut p=Progress{schema_version:1,clock:2,..Default::default()};p.favorites.insert(source.id.clone());p.attempts.insert(source.id.clone(),a);
 let dir=tempfile::tempdir().unwrap();let path=dir.path().join("progress-v1.json");progress::save_with_catalog(&path,&p,std::slice::from_ref(&source)).unwrap();let restored=progress::load(&path,std::slice::from_ref(&source)).unwrap();
 let a=&restored.attempts[&source.id];binding::context(a,&source).unwrap();assert!(!progress::current_solved(a,&source));assert!(restored.favorites.contains(&source.id));
 let engine::State::SieveFiltering{simulation}= &a.state else{panic!()};assert_eq!(**simulation,before);
 let mut s=(**simulation).clone();s.action(d,&Action::Run,false).unwrap();s.action(d,&Action::Tick,false).unwrap();s.action(d,&Action::Stop,false).unwrap();assert!(s.assignment.stopped);assert_eq!(s.assignment.results.len(),1);
 run(&mut s,d,&d.assignment.reference,false);assert!(d.solved(&s));let practice=s.practice.clone();s.assignment=dlms_learning_lab::sieve_workspace::Scope::new(&d.assignment);assert!(!d.solved(&s));assert_eq!(s.practice,practice);
}
