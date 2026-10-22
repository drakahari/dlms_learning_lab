//! Declarative beginner teaching and isolated example drafts. Uses only the existing VM.
use crate::{progress::Attempt, shell};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    pub title: String,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example: String,
    pub inputs: String,
    pub try_change: String,
    pub fixture: shell::Case,
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        if self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.concepts.iter().any(|c| {
                c.title.is_empty()
                    || c.title.len() > 100
                    || c.text.is_empty()
                    || c.text.len() > 1200
            })
            || [&self.prerequisite_note, &self.inputs, &self.try_change]
                .iter()
                .any(|s| s.is_empty() || s.len() > 1200)
            || self.example.len() > shell::MAX_SOURCE
            || !self.fixture.valid()
        {
            return Err("Invalid script teaching content".into());
        }
        if !self
            .fixture
            .accepts(&shell::run(&self.example, &self.fixture))
        {
            return Err("Teaching example does not produce its declared behavior".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Introduction,
    Example,
    Assignment,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub example_source: String,
    #[serde(default, skip_serializing_if = "crate::shell::legacy_revision")]
    pub example_revision: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example_result: Option<shell::Outcome>,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            example_source: lesson.example.clone(),
            example_revision: shell::EVALUATOR_REVISION,
            example_result: None,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && shell::Record {
                engine_revision: self.example_revision,
                source: self.example_source.clone(),
                results: self.example_result.clone().into_iter().collect(),
            }
            .valid()
    }
}
pub fn prepare(a: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    let w = a.workspace.as_mut().expect("workspace prepared");
    if w.script_learning.is_none() {
        let mut learning = Learning::new(lesson);
        if !new_attempt {
            learning.view = View::Assignment;
        }
        w.script_learning = Some(learning);
    }
}
/// Feedback classification also understands old saved error strings without rewriting them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    Syntax,
    Unsupported,
    Limit,
    Stopped,
    Execution,
}
pub fn diagnostic(message: &str) -> Diagnostic {
    let m = message.to_ascii_lowercase();
    if m.starts_with("syntax error:") {
        Diagnostic::Syntax
    } else if m.contains("unsupported") || m.contains(" are supported") || m.contains(" supports ")
    {
        Diagnostic::Unsupported
    } else if m.contains("limit") || m.contains("exceeds") {
        Diagnostic::Limit
    } else if m.starts_with("stopped") {
        Diagnostic::Stopped
    } else if m.starts_with("expected")
        || m.starts_with("close ")
        || m.starts_with("unfinished")
        || m.contains("needs a ")
    {
        Diagnostic::Syntax
    } else {
        Diagnostic::Execution
    }
}
pub fn diagnostic_title(message: &str) -> &'static str {
    match diagnostic(message) {
        Diagnostic::Syntax => "Syntax error — the script cannot be read yet",
        Diagnostic::Unsupported => {
            "Unsupported syntax or command — outside this simulator’s subset"
        }
        Diagnostic::Limit => "Execution limit — check repeated work or growing output",
        Diagnostic::Stopped => "Stopped — no result was graded",
        Diagnostic::Execution => "Command error — check the values used by this command",
    }
}
pub fn diagnostic_advice(message: &str) -> &'static str {
    let m = message.to_ascii_lowercase();
    if m.contains("expansion work limit") {
        "Replacing variables repeatedly can require much more work than the script’s length suggests, even when nothing is printed. Use smaller values or avoid repeating a large value, then retry."
    } else if m.contains("expansion memory limit") {
        "Joining large variable values builds one large piece of text. Process smaller pieces separately instead of joining many copies, then retry."
    } else if m.contains("division by zero") {
        "A division needs a nonzero divisor. Check or reject zero input before calculating, then retry."
    } else if m.contains("invalid integer") {
        "Use an integer value. A leading 0 means octal (digits 0–7); floating-point values and expressions stored in named variables are outside this subset."
    } else if m.contains("quoted string") {
        "A quote was opened but not closed. Add the matching single or double quote."
    } else if m.contains("expected fi") {
        "An if choice ends with fi. Check that every if has its matching ending on a separate line or after a semicolon."
    } else if m.contains("expected then") {
        "Put then after the condition, separated by a new line or semicolon, before the instructions to run."
    } else if m.contains("expected done") {
        "A loop ends with done. Check that its do instructions have a matching ending."
    } else if m.contains("redirection needs") {
        "After >, >> or <, give a virtual file path, such as \"$2\"."
    } else {
        match diagnostic(message) {
            Diagnostic::Unsupported => {
                "This simulator implements a limited shell. Check Supported shell subset in optional help; no command was sent to your computer."
            }
            Diagnostic::Syntax => {
                "Check matching quotes, spaces around test brackets and the ending of each choice or loop."
            }
            Diagnostic::Limit => {
                "The simulator stopped safely. Check whether a loop can finish and whether it prints too much."
            }
            Diagnostic::Stopped => "Run again when ready; interrupted runs do not earn completion.",
            Diagnostic::Execution => {
                "Check the command’s input values and virtual file paths, then retry."
            }
        }
    }
}
pub fn output_advice(expected: &str, actual: &str) -> &'static str {
    if actual.trim_end_matches('\n') == expected.trim_end_matches('\n') {
        "The text matches, but the final line break differs. In printf, \\n starts a new line."
    } else if actual.split_whitespace().eq(expected.split_whitespace()) {
        "The words match, but spacing or line breaks differ. Quote values and check the fixed spaces and \\n in the format."
    } else {
        "Check the labels, values and their order. Use %s for a value and put double quotes around its variable name so spaces stay together."
    }
}
