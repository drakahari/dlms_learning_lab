//! Source-bound locale/report and byte-conversion tasks. Worlds and grades are
//! derived, never trusted from progress. No catalog or native UI is implied.
use crate::{
    boot_entry::{Error, ErrorKind},
    localization::{
        self, Environment, Locale,
        command::{self, Output},
        virtual_files::{Files, Input},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_HISTORY: usize = 96;
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Bytes(
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u8,32768>")] pub Vec<u8>,
);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Locale,4>")]
    pub available: Vec<Locale>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Input,8>")]
    pub inputs: Vec<Input>,
}
impl Facts {
    pub fn validate(&self) -> Result<()> {
        localization::configuration("", &self.available)?;
        Files::new(self.inputs.clone())?;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub saved_locale: String,
    pub process: Environment,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    SaveLocale {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        text: String,
    },
    NewSession {},
    Report {
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        input: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Current,
    NewLogin,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Case {
    Report {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        id: String,
        scope: Scope,
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        input: String,
        expected: Bytes,
    },
    File {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        id: String,
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        path: String,
        expected: Bytes,
    },
}
impl Case {
    fn id(&self) -> &str {
        match self {
            Self::Report { id, .. } | Self::File { id, .. } => id,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: Facts,
    pub initial: Snapshot,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,32>")]
    pub reference: Vec<Action>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub correct: bool,
    pub feedback: String,
    pub expected: Option<Vec<u8>>,
    pub actual: Option<Vec<u8>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    task_id: String,
    facts: Facts,
    saved_locale: String,
    process: Environment,
    files: Files,
}
fn invalid(message: impl Into<String>) -> Error {
    Error {
        kind: ErrorKind::Invalid,
        message: message.into(),
    }
}
impl World {
    fn new(t: &Task) -> Result<Self> {
        Ok(Self {
            task_id: t.id.clone(),
            facts: t.facts.clone(),
            saved_locale: t.initial.saved_locale.clone(),
            process: t.initial.process.clone(),
            files: Files::new(t.facts.inputs.clone())?,
        })
    }
    pub fn saved_locale(&self) -> &str {
        &self.saved_locale
    }
    pub fn process(&self) -> &Environment {
        &self.process
    }
    pub fn files(&self) -> &Files {
        &self.files
    }
    pub fn apply(&mut self, action: &Action) -> std::result::Result<Output, Error> {
        match action {
            Action::Command { text } => command::run(
                &mut self.process,
                &mut self.files,
                &self.facts.available,
                text,
            ),
            Action::SaveLocale { text } => {
                if text.len() > localization::MAX_CONFIG || text.contains('\0') {
                    return Err(Error{kind:ErrorKind::Limit,message:"Keep locale settings within2,048bytes with no zero byte; nothing was saved.".into()});
                }
                self.saved_locale = text.clone();
                Ok(Output {
                    stdout:
                        b"Saved simulated /etc/locale.conf. The current session is unchanged.\n"
                            .to_vec(),
                    stderr: String::new(),
                    status: 0,
                })
            }
            Action::NewSession {} => {
                let next = localization::configuration(&self.saved_locale, &self.facts.available)
                    .map_err(invalid)?;
                self.process = next;
                Ok(Output {
                    stdout: b"Opened a simulated session from the saved locale settings.\n"
                        .to_vec(),
                    stderr: String::new(),
                    status: 0,
                })
            }
            Action::Report { input } => {
                let source = self.files.read(input).map_err(invalid)?;
                match localization::sample_report::render(source, &self.process) {
                    Ok(output) => Ok(output),
                    Err(reason) => Ok(Output {
                        stdout: vec![],
                        stderr: format!("The sample application cannot read this input: {reason}"),
                        status: 2,
                    }),
                }
            }
        }
    }
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Task {
    pub fn has_reports(&self) -> bool {
        self.cases.iter().any(|c| matches!(c, Case::Report { .. }))
    }
    pub fn validate(&self) -> Result<()> {
        if !identifier(&self.id)
            || !(3..=8).contains(&self.cases.len())
            || self.reference.is_empty()
            || self.reference.len() > 32
            || self.initial.saved_locale.len() > localization::MAX_CONFIG
            || self.initial.saved_locale.contains('\0')
        {
            return Err("Use a stable task ID, three to eight distinct outcome cases and a bounded reachable method.".into());
        }
        self.facts.validate()?;
        self.initial.process.validate(&self.facts.available)?;
        let mut ids = BTreeSet::new();
        let mut contexts = BTreeSet::new();
        let initial = World::new(self)?;
        for case in &self.cases {
            if !identifier(case.id()) || !ids.insert(case.id()) {
                return Err("Case identifiers must be distinct stable names.".into());
            }
            match case {
                Case::Report {
                    scope,
                    input,
                    expected,
                    ..
                } => {
                    if expected.0.len() > 32768
                        || !self.facts.inputs.iter().any(|f| &f.path == input)
                        || !contexts.insert(format!("report:{scope:?}:{input}"))
                    {
                        return Err("Report cases need distinct current/new-session inputs from the supplied documents.".into());
                    }
                    let mut utf8 = Environment::default();
                    utf8.set(localization::Variable::Lang, Some(Locale::Cutf8));
                    localization::sample_report::render(initial.files.read(input)?, &utf8)?;
                }
                Case::File { path, expected, .. } => {
                    if !path.starts_with("/work/")
                        || expected.0.len() > 32768
                        || !contexts.insert(format!("file:{path}"))
                    {
                        return Err("Output cases need distinct canonical /work paths and bounded expected bytes.".into());
                    }
                    let mut scratch = initial.files.clone();
                    scratch.write(path, vec![])?;
                }
            }
        }
        if self.checks(&initial).iter().all(|r| r.correct) {
            return Err("The independent starter must need meaningful repair.".into());
        }
        let solved = self.replay(&self.reference)?;
        if self.checks(&solved).iter().any(|r| !r.correct) {
            return Err("The taught method cannot reach every actual outcome.".into());
        }
        Ok(())
    }
    pub fn replay(&self, history: &[Action]) -> Result<World> {
        if history.len() > MAX_HISTORY {
            return Err("Locale/encoding history exceeds96actions.".into());
        }
        self.facts.validate()?;
        self.initial.process.validate(&self.facts.available)?;
        if self.initial.saved_locale.len() > localization::MAX_CONFIG
            || self.initial.saved_locale.contains('\0')
        {
            return Err("Initial saved locale settings exceed the source bounds.".into());
        }
        let mut world = World::new(self)?;
        for action in history {
            world.apply(action).map_err(|e| e.to_string())?;
        }
        Ok(world)
    }
    pub fn checks(&self, world: &World) -> Vec<Check> {
        if world.task_id != self.id || world.facts != self.facts {
            return vec![Check {
                label: "Current task source".into(),
                correct: false,
                feedback: "This work belongs to a different supplied activity.".into(),
                expected: None,
                actual: None,
            }];
        }
        self.cases.iter().map(|case|match case {
            Case::File{id,path,expected}=>{
                let actual=world.files.outputs().get(path).cloned();
                let correct=actual.as_ref()==Some(&expected.0);
                let feedback=if correct {"The saved output contains the required bytes.".into()}else if actual.is_none(){format!("No output was saved at {path}. Viewing converted text alone does not create an output file; use the tool's output option.")}else if expected.0.starts_with(actual.as_ref().unwrap())&&actual.as_ref().unwrap().len()<expected.0.len(){format!("{path} stops before all required characters. Check the last conversion's error and target encoding, then retry; a failed conversion can leave a partial file.")}else{format!("{path} contains different bytes or characters. Check the source encoding as well as the destination; a file can be valid UTF-8 yet contain the wrong text.")};
                Check{label:format!("Saved output · {id} · {path}"),correct,feedback,expected:Some(expected.0.clone()),actual}
            }
            Case::Report{id,scope,input,expected}=>{
                let environment=match scope {Scope::Current=>Ok(world.process.clone()),Scope::NewLogin=>localization::configuration(&world.saved_locale,&self.facts.available)};
                let label=format!("{} · {id}",match scope {Scope::Current=>"Current-session report",Scope::NewLogin=>"New-session report"});
                let actual=environment.and_then(|env|localization::sample_report::render(world.files.read(input)?,&env));
                match actual {
                    Err(reason)=>Check{label,correct:false,feedback:format!("This session cannot use its settings/input: {reason}"),expected:Some(expected.0.clone()),actual:None},
                    Ok(output)=>{
                        let correct=output.status==0&&output.stderr.is_empty()&&output.stdout==expected.0;
                        let feedback=if correct{"The actual report has the required messages, numbers and complete characters.".into()}else if output.status!=0{format!("The report stopped with status{}: {}",output.status,output.stderr)}else{"The report ran, but its heading, decimal separator or characters do not match the task. Compare the effective categories; saved defaults and the existing session can differ.".into()};
                        Check{label,correct,feedback,expected:Some(expected.0.clone()),actual:Some(output.stdout)}
                    }
                }
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests;

pub mod session;
pub use session::{Definition, Draft, Session};

pub mod bindings;
pub use bindings::{HELP, PREFIX, context, relevant, source};
