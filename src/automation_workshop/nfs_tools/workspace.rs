//! Opt-in command practice built on the same supplied NFS state as the file editor.
use super::operator;
use crate::{automation_workshop::workspace::Outcome, nfs, nfs_workspace};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;
const TERMINAL_LIMIT: usize = 6;
pub const INPUT_LIMIT: usize = 160;
const OUTPUT_LIMIT: usize = 4096;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub definition: Box<nfs_workspace::Definition>,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub server: String,
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub withdrawn_root: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,RequirementText,36>")]
    pub labels: Vec<RequirementText>,
    pub clients: [ClientTarget; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementText {
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub title: String,
    #[serde(deserialize_with = "crate::cron::text::<_,220>")]
    pub retry: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientTarget {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub client: String,
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub mountpoint: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub profile: String,
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub relative: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Command,
    Client,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub origin: Origin,
    #[serde(deserialize_with = "crate::cron::text::<_,INPUT_LIMIT>")]
    pub request: String,
    #[serde(deserialize_with = "crate::cron::text::<_,OUTPUT_LIMIT>")]
    pub reply: String,
    pub success: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub state: nfs::State,
    #[serde(deserialize_with = "crate::cron::text::<_,INPUT_LIMIT>")]
    pub draft: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Entry,TERMINAL_LIMIT>")]
    pub history: Vec<Entry>,
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        self.definition.validate()?;
        let inventory = self.definition.initial.inventory();
        if inventory.servers.len() != 1
            || inventory.servers[0].id != self.server
            || self.clients[0].client == self.clients[1].client
        {
            return Err(
                "Supply one named virtual NFS server and two distinct test clients.".into(),
            );
        }
        let root = &self.withdrawn_root;
        let retired = &self.clients[0];
        if !inventory
            .roots
            .iter()
            .any(|r| &r.path == root && r.server == self.server)
            || !self.definition.initial.runtime().mounts.iter().any(|m| {
                m.client == retired.client
                    && m.specification.mountpoint == retired.mountpoint
                    && &m.specification.root == root
                    && m.specification.server == self.server
            })
        {
            return Err(
                "The supplied retired client must begin with the obsolete share mounted.".into(),
            );
        }
        if self.labels.len() + 1 != self.definition.row_count()
            || self
                .labels
                .iter()
                .any(|row| row.title.trim().is_empty() || row.retry.trim().is_empty())
        {
            return Err(
                "Supply one meaningful label and retry hint for each independent check.".into(),
            );
        }
        for test in &self.clients {
            let client = inventory
                .clients
                .iter()
                .find(|c| c.id == test.client)
                .ok_or("Unknown source-held client.")?;
            if !client.mountpoints.contains(&test.mountpoint)
                || !client.profiles.contains(&test.profile)
            {
                return Err("Use a supplied mount target and file-identity case.".into());
            }
            let mut scratch = self.definition.initial.clone();
            scratch
                .apply(&nfs::Action::ReadExisting {
                    client: test.client.clone(),
                    mountpoint: test.mountpoint.clone(),
                    profile: test.profile.clone(),
                    relative: test.relative.clone(),
                })
                .map_err(|error| error.message)?;
        }
        Ok(())
    }

    pub fn start(&self) -> Work {
        Work {
            state: self.definition.initial.clone(),
            draft: String::new(),
            history: Vec::new(),
        }
    }
}

impl Work {
    pub fn validate(&self, task: &Task) -> Result<()> {
        if !task.definition.same_assignment(&self.state)
            || self.draft.len() > INPUT_LIMIT
            || self.draft.chars().any(char::is_control)
            || self.history.len() > TERMINAL_LIMIT
            || self.history.iter().any(|row| {
                row.request.len() > INPUT_LIMIT
                    || row.reply.len() > OUTPUT_LIMIT
                    || row.request.chars().any(char::is_control)
                    || row.reply.contains('\0')
            })
        {
            return Err(
                "Keep the terminal and virtual files within this work order's bounds.".into(),
            );
        }
        self.state.validate().map_err(|e| e.message)
    }

    pub fn edit_exports(&mut self, task: &Task, text: String) -> Result<()> {
        self.validate(task)?;
        self.state
            .apply(&nfs::Action::SetExportsDraft {
                server: task.server.clone(),
                text,
            })
            .map_err(|e| e.message)?;
        Ok(())
    }

    pub fn save_exports(&mut self, task: &Task) -> Result<()> {
        self.validate(task)?;
        self.state
            .apply(&nfs::Action::SaveExportsDraft {
                server: task.server.clone(),
            })
            .map_err(|e| e.message)?;
        Ok(())
    }

    fn record(&mut self, origin: Origin, request: String, reply: Result<String>) {
        if self.history.len() == TERMINAL_LIMIT {
            self.history.remove(0);
        }
        let (success, mut reply) = match reply {
            Ok(text) => (true, text),
            Err(text) => (false, text),
        };
        if reply.len() > OUTPUT_LIMIT {
            reply.truncate(reply.floor_char_boundary(OUTPUT_LIMIT - 48));
            reply.push_str("\n[Virtual terminal display limit]\n");
        }
        self.history.push(Entry {
            origin,
            request,
            reply,
            success,
        });
    }

    pub fn command(&mut self, task: &Task, text: String) -> Result<()> {
        self.validate(task)?;
        if text.is_empty() || text.len() > INPUT_LIMIT || text.chars().any(char::is_control) {
            return Err("Enter one supported command on a single line.".into());
        }
        let result = operator::execute(&mut self.state, &task.server, &text);
        self.record(Origin::Command, text, result);
        self.draft.clear();
        Ok(())
    }

    pub fn mount(&mut self, task: &Task, client: usize) -> Result<()> {
        self.validate(task)?;
        let target = task
            .clients
            .get(client)
            .ok_or("Choose a supplied test client.")?;
        let result = match self.state.apply(&nfs::Action::MountSaved {
            client: target.client.clone(),
            mountpoint: target.mountpoint.clone(),
        }) {
            Ok(nfs::Decision::Mounted { .. }) => Ok(format!(
                "{} now has a current mount at {}.",
                target.client, target.mountpoint
            )),
            Ok(nfs::Decision::Denied {
                stage: nfs::DenialStage::ExportAdmission,
                ..
            }) => {
                Err("This client is outside the applied export's admission rule.".into())
            }
            Err(error) => Err(error.message),
            _ => Err("The supplied mount did not complete.".into()),
        };
        self.record(
            Origin::Client,
            format!("Test saved mount · {}", target.client),
            result,
        );
        Ok(())
    }

    pub fn unmount_retired(&mut self, task: &Task) -> Result<()> {
        self.validate(task)?;
        let old = &task.clients[0];
        let result = self
            .state
            .apply(&nfs::Action::Unmount {
                client: old.client.clone(),
                mountpoint: old.mountpoint.clone(),
            })
            .map(|_| format!("Removed the current mount at {}.", old.mountpoint))
            .map_err(|error| error.message);
        self.record(
            Origin::Client,
            format!("Unmount retired share · {}", old.client),
            result,
        );
        Ok(())
    }

    pub fn outcomes(&self, task: &Task) -> Result<Vec<Outcome>> {
        self.validate(task)?;
        let rows = task.definition.checks(&self.state);
        if rows.len() != task.definition.row_count() || !rows[0].0 {
            return Err(rows.first().map_or_else(
                || "Could not evaluate the supplied work order.".into(),
                |r| r.1.clone(),
            ));
        }
        let layers = &self.state.runtime().servers[0];
        let withdrawn = layers
            .applied
            .iter()
            .all(|export| export.root != task.withdrawn_root);
        let mut outcomes=vec![Outcome{
            name:"Withdraw only the retired share".into(), passed:withdrawn,
            detail:if withdrawn {"The applied server list excludes the retired path."} else {"Remove the retired export from the saved file, then re-export it. Keep the other exports available."}.into(),
        }];
        let expected = task.definition.initial.runtime().servers[0]
            .applied
            .iter()
            .filter(|export| export.root != task.withdrawn_root)
            .collect::<Vec<_>>();
        let others = layers
            .applied
            .iter()
            .filter(|export| export.root != task.withdrawn_root)
            .collect::<Vec<_>>();
        let unchanged = expected.len() == others.len()
            && expected.iter().all(|want| {
                others.iter().any(|got| {
                    got.root == want.root
                        && got.clients.len() == want.clients.len()
                        && want.clients.iter().all(|rule| got.clients.contains(rule))
                })
            });
        outcomes.push(Outcome {
            name: "Preserve the remaining export rules".into(), passed: unchanged,
            detail: if unchanged { "The remaining exports admit the same clients with the same options." } else {
                "Keep the approved reference and archive rules unchanged while removing the obsolete rule."
            }.into(),
        });
        outcomes.extend(rows.into_iter().skip(1).zip(&task.labels).map(
            |((passed, actual), label)| Outcome {
                name: label.title.clone(),
                passed,
                detail: if passed { actual } else { label.retry.clone() },
            },
        ));
        Ok(outcomes)
    }

    pub fn read(&mut self, task: &Task, client: usize) -> Result<()> {
        self.validate(task)?;
        let target = task
            .clients
            .get(client)
            .ok_or("Choose a supplied test client.")?;
        let result = match self.state.apply(&nfs::Action::ReadExisting {
            client: target.client.clone(),
            mountpoint: target.mountpoint.clone(),
            profile: target.profile.clone(),
            relative: target.relative.clone(),
        }) {
            Ok(nfs::Decision::AllowedRead { text, .. }) => Ok(text),
            Ok(nfs::Decision::Denied { stage, .. }) => Err(match stage {
                nfs::DenialStage::MountAbsent => {
                    "Mount the supplied share on this client before reading."
                }
                nfs::DenialStage::DirectorySearch => {
                    "The current file identity cannot search a directory in the mounted path."
                }
                nfs::DenialStage::FilePermission => {
                    "The current file identity cannot read this file."
                }
                _ => "The supplied request was denied by the current mount or export policy.",
            }
            .into()),
            Err(error) => Err(error.message),
            _ => Err("The supplied read did not complete.".into()),
        };
        self.record(
            Origin::Client,
            format!("Read sample file · {}", target.client),
            result,
        );
        Ok(())
    }
}
