//! Source-bound editing and observed boot outcomes for a single temporary GRUB2 boot.
use crate::{
    boot_entry::{self, Inventory},
    grub_menu::{
        Menu,
        temporary::{self, BootComparison},
    },
};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;
type Checks = Vec<(bool, String)>;
pub const HELP: &str = "Select a supplied GRUB2 entry and edit one literal linux command. Boot once simulates the Linux handoff for the edit and an ordinary boot from the saved menu. No GRUB, services, real boot or host files are changed.";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub entry_id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub linux: String,
}
impl Input {
    fn shape(&self) -> Result<()> {
        if self.entry_id.is_empty()
            || self.entry_id.len() > 64
            || self.linux.len() > boot_entry::MAX_SOURCE
            || self.linux.contains('\0')
        {
            return Err(
                "Keep the draft within 4 KiB with no NUL and a valid supplied entry selection."
                    .into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub saved_menu: String,
    pub inventory: Inventory,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub starter_id: String,
    pub reference: Input,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub required_root: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub required_release: String,
}
impl Task {
    pub fn select(&self, id: &str) -> Result<Input> {
        let menu = Menu::parse(&self.saved_menu).map_err(|e| e.to_string())?;
        let item = menu
            .items
            .iter()
            .find(|item| item.id == id)
            .ok_or("Choose an entry from the supplied GRUB menu.")?;
        let linux = format!(
            "linux {} {}",
            item.entry.linux,
            item.entry.options.join(" ")
        );
        let input = Input {
            entry_id: item.id.clone(),
            linux,
        };
        input.shape()?;
        Ok(input)
    }

    pub fn report(&self, input: &Input) -> Result<BootComparison> {
        temporary::test(
            &self.inventory,
            &self.saved_menu,
            &input.entry_id,
            &input.linux,
        )
        .map_err(|e| format!("{:?}: {}", e.kind, e.message))
    }

    pub fn checks(&self, outcome: &BootComparison) -> Checks {
        let report = &outcome.edited;
        let handed_off = report.passed();
        let expected =
            |what: &str| format!("{what} is checked after the edited boot reaches Linux init.");
        vec![
            (
                handed_off,
                report.stages.last().map_or_else(
                    || "The edited boot did not start.".into(),
                    |stage| stage.detail.clone(),
                ),
            ),
            (
                report.release.as_deref() == Some(self.required_release.as_str()),
                format!(
                    "Use the supplied {} kernel for this recovery.",
                    self.required_release
                ),
            ),
            (
                report
                    .root
                    .as_ref()
                    .is_some_and(|uuid| uuid.eq_ignore_ascii_case(&self.required_root)),
                if handed_off {
                    format!("The recovery boot needs root UUID {}.", self.required_root)
                } else {
                    expected("The recovery root")
                },
            ),
            (
                handed_off && report.read_only,
                if handed_off {
                    "The root filesystem must start read-only.".into()
                } else {
                    expected("The initial root mode")
                },
            ),
            (
                report.target.as_deref() == Some("rescue.target"),
                if handed_off {
                    "The edited kernel command must request rescue.target.".into()
                } else {
                    expected("The startup target")
                },
            ),
            (
                outcome.ordinary.passed()
                    && outcome.ordinary.target.as_deref() == Some("multi-user.target"),
                "A later ordinary boot still needs to reach the saved multi-user startup.".into(),
            ),
        ]
    }

    pub fn starter(&self) -> Result<Input> {
        self.select(&self.starter_id)
    }

    pub fn validate(&self) -> Result<()> {
        if self.instructions.trim().is_empty() || self.instructions.len() > 2048 {
            return Err("Supply a clear temporary-boot task.".into());
        }
        self.inventory.validate().map_err(|e| e.to_string())?;
        self.reference.shape()?;
        let reference = self.report(&self.reference)?;
        if self.checks(&reference).iter().any(|r| !r.0) {
            return Err("The supplied reference must reach the required handoff and preserve ordinary startup.".into());
        }
        if self
            .checks(&self.report(&self.starter()?)?)
            .iter()
            .all(|r| r.0)
        {
            return Err("The supplied assignment starts already solved.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub input: Input,
    pub observed: Option<Input>,
}
impl Draft {
    pub fn new(task: &Task) -> Result<Self> {
        Ok(Self {
            input: task.starter()?,
            observed: None,
        })
    }

    pub fn shape(&self) -> Result<()> {
        self.input.shape()?;
        if self
            .observed
            .as_ref()
            .is_some_and(|seen| seen != &self.input)
        {
            return Err("Saved boot observation does not match its current edit.".into());
        }
        Ok(())
    }

    pub fn bound(&self, task: &Task) -> Result<()> {
        self.shape()?;
        task.select(&self.input.entry_id)?;
        Ok(())
    }

    pub fn edit(&mut self, input: Input) -> Result<()> {
        input.shape()?;
        if self.input != input {
            self.input = input;
            self.observed = None;
        }
        Ok(())
    }

    pub fn test(&mut self, task: &Task) -> Result<()> {
        self.bound(task)?;
        self.observed = Some(self.input.clone());
        Ok(())
    }

    pub fn report(&self, task: &Task) -> Option<Result<BootComparison>> {
        self.observed.as_ref().map(|input| task.report(input))
    }

    pub fn checks(&self, task: &Task) -> Checks {
        if let Err(e) = self.bound(task) {
            return vec![(false, e)];
        }
        match self.report(task) {
            None => vec![(false, "Use Boot once to test the current edit.".into())],
            Some(Err(e)) => vec![(false, e)],
            Some(Ok(outcome)) => task.checks(&outcome),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.id.is_empty()
            || self
                .initial
                .required_root
                .eq_ignore_ascii_case(&self.example.required_root)
            || self.initial.required_release == self.example.required_release
            || self.initial.saved_menu == self.example.saved_menu
        {
            return Err("Use separate supplied menus, recovery roots and kernel releases for practice and assignment.".into());
        }
        self.initial.validate()?;
        self.example.validate()
    }

    pub fn checks(&self, session: &Session) -> Checks {
        if let Err(error) = session.bound(self) {
            return vec![(false, error)];
        }
        session.assignment.checks(&self.initial)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    context: String,
    pub view: crate::script_teaching::View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(definition: &Definition) -> Result<Self> {
        Ok(Self {
            version: definition.version,
            context: definition.id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Draft::new(&definition.initial)?,
            practice: Draft::new(&definition.example)?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.context.is_empty() || self.context.len() > 64 {
            return Err("This saved GRUB edit uses incompatible lesson metadata.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }

    pub fn execute_reference(&mut self, definition: &Definition, command: &str) -> Result<String> {
        const PREFIX: &str = "TEMPORARY GRUB BOOT\n";
        self.bound(definition)?;
        let raw = command
            .strip_prefix(PREFIX)
            .filter(|text| text.len() <= 8192)
            .ok_or("Use the temporary GRUB editor for this exercise.")?;
        let input: Input = serde_json::from_str(raw)
            .map_err(|error| format!("Invalid temporary GRUB boot input: {error}"))?;
        let mut next = self.assignment.clone();
        next.edit(input)?;
        next.test(&definition.initial)?;
        let checks = next.checks(&definition.initial);
        let passed = checks.iter().filter(|row| row.0).count();
        self.assignment = next;
        Ok(format!(
            "{passed}/{} temporary boot checks matched.",
            checks.len()
        ))
    }

    pub fn bound(&self, definition: &Definition) -> Result<()> {
        self.validate()?;
        if self.version != definition.version || self.context != definition.id {
            return Err("This saved GRUB edit belongs to another lesson.".into());
        }
        self.assignment.bound(&definition.initial)?;
        self.practice.bound(&definition.example)
    }
}

fn admitted(scenario: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::GrubOnce { simulation },
        crate::engine::Goal::GrubOnce { definition },
    ) = (&scenario.initial, &scenario.goal)
    else {
        return Err("This temporary boot task and its supplied workspace do not match.".into());
    };
    if scenario.track != "LPIC-2"
        || definition.id != scenario.id
        || definition.initial.instructions != scenario.objective
        || **simulation != Session::new(definition)?
    {
        return Err(
            "This temporary boot task needs the supplied track, instructions and fresh workspace."
                .into(),
        );
    }
    Ok(definition)
}

pub fn source(scenario: &crate::pack::Scenario) -> Result<&Definition> {
    let definition = admitted(scenario)?;
    definition.validate()?;
    Ok(definition)
}

pub fn context(attempt: &crate::progress::Attempt, scenario: &crate::pack::Scenario) -> Result<()> {
    let definition = admitted(scenario)?;
    let crate::engine::State::GrubOnce { simulation } = &attempt.state else {
        return Err("This temporary GRUB edit belongs to another workspace.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("This saved GRUB edit has incompatible progress metadata.".into());
    }
    simulation.bound(definition)
}
