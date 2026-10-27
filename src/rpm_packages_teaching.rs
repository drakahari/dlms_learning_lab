//! Independent native RPM practice; actual closed commands, never a completion credential.
use crate::{
    rpm_packages::State,
    rpm_packages_terminal::{self as terminal, Outcome, Session},
    rpm_packages_workspace::{self as work, Definition},
    script_teaching::View,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
pub const MAX_LESSON: usize = 196_608;
pub const MAX_LEARNING: usize = 327_680;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "work::string::<_,128>")]
    pub title: String,
    #[serde(deserialize_with = "work::string::<_,2048>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleStep {
    #[serde(deserialize_with = "work::string::<_,2048>")]
    pub command: String,
    #[serde(deserialize_with = "work::string::<_,2048>")]
    pub explanation: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    #[serde(deserialize_with = "work::string::<_,2048>")]
    pub description: String,
    #[serde(deserialize_with = "work::list::<_,ExampleStep,16>")]
    pub commands: Vec<ExampleStep>,
    #[serde(deserialize_with = "work::list::<_,bool,24>")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Lesson {
    pub definition: Definition,
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub example_commands: Vec<ExampleStep>,
    pub try_change: Experiment,
    pub repair: Vec<ExampleStep>,
}
impl<'de> Deserialize<'de> for Lesson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            definition: Definition,
            #[serde(deserialize_with = "work::string::<_,2048>")]
            prerequisite_note: String,
            #[serde(deserialize_with = "work::list::<_,Concept,8>")]
            concepts: Vec<Concept>,
            #[serde(deserialize_with = "work::string::<_,4096>")]
            example_description: String,
            #[serde(deserialize_with = "work::list::<_,ExampleStep,16>")]
            example_commands: Vec<ExampleStep>,
            try_change: Experiment,
            #[serde(deserialize_with = "work::list::<_,ExampleStep,16>")]
            repair: Vec<ExampleStep>,
        }
        let r = Raw::deserialize(d)?;
        let l = Self {
            definition: r.definition,
            prerequisite_note: r.prerequisite_note,
            concepts: r.concepts,
            example_description: r.example_description,
            example_commands: r.example_commands,
            try_change: r.try_change,
            repair: r.repair,
        };
        if !l.bounded() {
            return Err(D::Error::custom("RPM Lesson bounds differ"));
        }
        Ok(l)
    }
}
fn prose(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && s.len() <= n && !s.contains('\0')
}
fn step(s: &ExampleStep) -> bool {
    prose(&s.explanation, 2048)
        && s.command.len() <= terminal::MAX_COMMAND
        && terminal::parse(&s.command).is_ok()
}
fn run(d: &Definition, s: &mut State, commands: &[ExampleStep]) -> Result<(), String> {
    for c in commands {
        match terminal::execute(d, s, &c.command) {
            Outcome::Utility { status: 0, .. } => {}
            Outcome::SimulatorUnsupported { reason } => return Err(work::notice(reason)),
            _ => return Err("Unexpected RPM success protocol".into()),
        }
    }
    Ok(())
}
pub fn separate(a: &State, b: &State) -> bool {
    let ac = a.catalog();
    let bc = b.catalog();
    fn capabilities(c: &crate::rpm_packages::Catalog) -> Vec<&str> {
        c.archives
            .values()
            .flat_map(|a| {
                std::iter::once(a.identity.name.as_str())
                    .chain(a.provides.keys().map(String::as_str))
                    .chain(a.requires.iter().map(|q| q.capability.as_str()))
            })
            .collect()
    }
    // Bounded names only, no payload snapshot. Complete capability namespaces too.
    let ca = capabilities(ac);
    let cb = capabilities(bc);
    ac.fixture != bc.fixture
        && ac.archives.keys().all(|id| !bc.archives.contains_key(id))
        && ac
            .allowed_paths
            .iter()
            .all(|p| !bc.allowed_paths.contains(p))
        && ca.iter().all(|n| !cb.contains(n))
}

impl Lesson {
    pub fn bounded(&self) -> bool {
        !self.concepts.is_empty()
            && self.concepts.len() <= 8
            && !self.example_commands.is_empty()
            && self.example_commands.len() <= 16
            && !self.try_change.commands.is_empty()
            && self.try_change.commands.len() <= 16
            && !self.repair.is_empty()
            && self.repair.len() <= 16
            && self.try_change.expected.len() == self.definition.requirements.len()
            && self.try_change.expected.iter().any(|x| !*x)
            && prose(&self.prerequisite_note, 2048)
            && prose(&self.example_description, 4096)
            && prose(&self.try_change.description, 2048)
            && self
                .concepts
                .iter()
                .all(|c| prose(&c.title, 128) && prose(&c.text, 2048))
            && self
                .example_commands
                .iter()
                .chain(&self.try_change.commands)
                .chain(&self.repair)
                .all(step)
            && self.definition.validate().is_ok()
            && work::within(self, MAX_LESSON)
    }
    pub fn separate_from(&self, a: &State) -> bool {
        self.bounded() && separate(&self.definition.initial, a)
    }
    pub fn separate_from_definition(&self, assignment: &Definition) -> bool {
        self.separate_from(&assignment.initial)
            && self.definition.archive_paths.keys().all(|p| {
                !assignment.archive_paths.contains_key(p)
                    && !assignment.initial.catalog().allowed_paths.contains(p)
            })
            && assignment
                .archive_paths
                .keys()
                .all(|p| !self.definition.initial.catalog().allowed_paths.contains(p))
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("RPM Lesson bounds/commands differ".into());
        }
        let mut s = self.definition.initial.clone();
        run(&self.definition, &mut s, &self.example_commands)?;
        if !self.definition.checks(&s).iter().all(|r| r.0) {
            return Err("RPM worked example does not satisfy its independent goal".into());
        }
        let before = s.clone();
        run(&self.definition, &mut s, &self.try_change.commands)?;
        if s == before
            || self
                .definition
                .checks(&s)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>()
                != self.try_change.expected
        {
            return Err(
                "RPM experiment must actually change State and match every declared row".into(),
            );
        }
        run(&self.definition, &mut s, &self.repair)?;
        if !self.definition.checks(&s).iter().all(|r| r.0) {
            return Err("RPM repair does not restore its outcomes".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub context: Definition,
    pub simulation: State,
    pub terminal: Session,
    pub selected_example: usize,
    pub checked: bool,
}
impl<'de> Deserialize<'de> for Learning {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u32,
            view: View,
            context: Definition,
            simulation: State,
            terminal: Session,
            selected_example: usize,
            checked: bool,
        }
        let r = Raw::deserialize(d)?;
        let l = Self {
            version: r.version,
            view: r.view,
            context: r.context,
            simulation: r.simulation,
            terminal: r.terminal,
            selected_example: r.selected_example,
            checked: r.checked,
        };
        if !l.valid() {
            return Err(D::Error::custom("RPM Learning admission differs"));
        }
        Ok(l)
    }
}
impl Learning {
    fn metadata_valid(&self) -> bool {
        self.version == 1
            && self.selected_example < 16
            && self.context.validate().is_ok()
            && self.context.same_assignment(&self.simulation)
            && self.terminal.valid_for(&self.simulation)
            && work::within(self, MAX_LEARNING)
    }
    pub fn valid(&self) -> bool {
        self.metadata_valid() && work::admitted(&self.simulation)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        l.bounded()
            && self.valid()
            && self.context == l.definition
            && self.selected_example < l.example_commands.len()
    }
    fn new_size(l: &Lesson) -> Option<usize> {
        #[derive(Serialize)]
        struct B<'a> {
            version: u32,
            view: View,
            context: &'a Definition,
            simulation: &'a State,
            terminal: Session,
            selected_example: usize,
            checked: bool,
        }
        work::size(
            &B {
                version: 1,
                view: View::Introduction,
                context: &l.definition,
                simulation: &l.definition.initial,
                terminal: Session::default(),
                selected_example: 0,
                checked: false,
            },
            MAX_LEARNING,
        )
    }
    pub fn new(l: &Lesson) -> Result<Self, String> {
        if !l.bounded() || Self::new_size(l).is_none() {
            return Err("RPM practice aggregate exceeds limit before cloning".into());
        }
        let t = Session::default();
        Ok(Self {
            version: 1,
            view: View::Introduction,
            context: l.definition.clone(),
            simulation: l.definition.initial.clone(),
            terminal: t,
            selected_example: 0,
            checked: false,
        })
    }
    pub fn insert(&mut self, l: &Lesson, command: &str) -> Result<(), String> {
        if command.len() > terminal::MAX_COMMAND || !self.valid_for(l) {
            return Err("Foreign/oversized RPM practice insertion".into());
        }
        self.terminal.insert(command)
    }
    pub fn run(&mut self, l: &Lesson) -> Result<Outcome, String> {
        if !self.valid_for(l) {
            return Err("Foreign RPM practice; retained unchanged".into());
        }
        self.terminal.run(&self.context, &mut self.simulation)
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !l.bounded()
            || !self.metadata_valid()
            || self.context != l.definition
            || self.selected_example >= l.example_commands.len()
        {
            return vec![(false, "RPM practice context differs".into())];
        }
        // The checks batch owns the only runtime validation for this practice grade.
        self.context.checks(&self.simulation)
    }
}
pub fn prepare(
    a: &mut crate::progress::Attempt,
    assignment: &Definition,
    l: &Lesson,
    new: bool,
) -> Result<(), String> {
    crate::workspace::reject_nfs_metadata(a)?;
    let crate::engine::State::RpmPackages { simulation } = &a.state else {
        return Err("RPM practice attached to another activity".into());
    };
    if a.workspace.as_ref().is_some_and(|w| {
        work::foreign(w)
            || !w.valid()
            || w.rpm_session
                .as_ref()
                .is_some_and(|t| !t.valid_for(simulation))
    }) {
        return Err("Foreign RPM workspace metadata; practice creation is atomic".into());
    }
    if assignment.validate().is_err()
        || !assignment.same_assignment(simulation)
        || !l.separate_from_definition(assignment)
    {
        return Err("RPM practice and assignment namespaces are not separate".into());
    }
    if let Some(old) = a.workspace.as_ref().and_then(|w| w.rpm_learning.as_ref()) {
        return if old.valid_for(l) {
            Ok(())
        } else {
            Err("Existing foreign RPM practice retained".into())
        };
    }
    let old = work::size(a, work::MAX_ATTEMPT).ok_or("RPM Attempt bound")?;
    let added = Learning::new_size(l).ok_or("RPM practice bound")?;
    if old
        .checked_add(added)
        .and_then(|n| n.checked_add(4096))
        .is_none_or(|n| n > work::MAX_ATTEMPT)
    {
        return Err("RPM combined Attempt admission before practice clone".into());
    }
    let mut learning = Learning::new(l)?;
    if !new {
        learning.view = View::Assignment;
    }
    crate::workspace::prepare(a);
    a.workspace
        .as_mut()
        .ok_or("Missing RPM workspace")?
        .rpm_learning = Some(learning);
    Ok(())
}
