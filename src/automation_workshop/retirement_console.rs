//! Bounded draft and output for the service and socket retirement console.
use super::socket_retirement::{Request, Target, Task, Work};
use serde::{Deserialize, Serialize};

const OUTPUT_LIMIT: usize = 768;
pub const INPUT_LIMIT: usize = 512;
const HISTORY_LIMIT: usize = 14;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,768>")]
    pub output: String,
    pub status: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Console {
    pub state: Work,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub draft: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Entry,14>")]
    pub history: Vec<Entry>,
}

fn valid_line(line: &str) -> bool {
    line.len() <= INPUT_LIMIT
        && !line
            .chars()
            .any(|scalar| scalar.is_control() && scalar != '\t')
}

impl Console {
    pub fn new(task: &Task) -> Self {
        Self {
            state: task.initial.clone(),
            draft: String::new(),
            history: Vec::new(),
        }
    }

    pub fn validate(&self, task: &Task) -> Result<(), String> {
        if !valid_line(&self.draft)
            || self.history.len() > HISTORY_LIMIT
            || self.history.iter().any(|line| {
                !valid_line(&line.input)
                    || line.output.len() > OUTPUT_LIMIT
                    || line.output.contains('\0')
            })
        {
            return Err("The terminal keeps fourteen recent, bounded entries.".into());
        }
        task.check_work(&self.state)
    }

    pub fn set_draft(&mut self, text: String) -> Result<(), String> {
        if !valid_line(&text) {
            return Err("Enter one line of up to 512 bytes in the practice terminal.".into());
        }
        self.draft = text;
        Ok(())
    }

    fn append(&mut self, entry: Entry) {
        if self.history.len() == HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.history.push(entry);
    }

    pub fn execute(&mut self, task: &Task, input: String) -> Result<(), String> {
        self.validate(task)?;
        if !valid_line(&input) {
            return Err("Enter one supported systemctl command.".into());
        }
        let (output, status) = match task.terminal(&mut self.state, &input) {
            Ok(result) => (result.text, result.status),
            Err(error) => (error, 1),
        };
        self.append(Entry {
            input,
            output,
            status,
        });
        self.draft.clear();
        self.validate(task)
    }

    pub fn observe_request(
        &mut self,
        task: &Task,
        required: bool,
        next_start: bool,
    ) -> Result<(), String> {
        let target = if required {
            Target::RequiredService
        } else {
            Target::ObsoleteService
        };
        let reply = if next_start {
            let mut started = task.next_session(&self.state)?;
            task.request(&mut started, target)?
        } else {
            task.request(&mut self.state, target)?
        };
        let unit = if required {
            &task.required_service
        } else {
            &task.obsolete_service
        };
        let observation = match reply {
            Request::AlreadyRunning => "The service answered the supplied request.",
            Request::SocketActivated => {
                "The listening socket started the service, which answered the request."
            }
            Request::NoListener => {
                "The supplied request received no response. No live service or activating socket handled it."
            }
            Request::ServiceDidNotStart => {
                "The activating socket was available, but the service failed to start and did not answer."
            }
        };
        let input = format!(
            "Test {unit}{}",
            if next_start {
                " after simulated startup"
            } else {
                " now"
            }
        );
        self.append(Entry {
            input,
            output: observation.into(),
            status: if matches!(reply, Request::NoListener | Request::ServiceDidNotStart) {
                1
            } else {
                0
            },
        });
        Ok(())
    }
}
