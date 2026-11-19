//! Offline tasks over supplied workstation profiles and input traces.
use super::{remote, typing_assist, xorg};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

impl Check {
    fn new(label: &str, passed: bool, detail: String) -> Self {
        Self {
            label: label.into(),
            passed,
            detail,
        }
    }
}

fn keyboard_cases<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Keyboard>, D::Error> {
    crate::cron_workspace::sequence::<_, Keyboard, 6>(d)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keyboard {
    pub device: xorg::Device,
    pub expected: xorg::Layout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub port: remote::Port,
    pub client: remote::Client,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteDraft {
    pub services: remote::Services,
    pub observer: Route,
    pub employee: Route,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "activity", rename_all = "snake_case", deny_unknown_fields)]
pub enum Task {
    Keyboard {
        #[serde(deserialize_with = "crate::cron::text::<_,{xorg::LIMIT}>")]
        starter: String,
        #[serde(deserialize_with = "keyboard_cases")]
        devices: Vec<Keyboard>,
    },
    Remote {
        starter: RemoteDraft,
        require_observer: bool,
        require_login: bool,
    },
    Typing {
        starter: typing_assist::Settings,
        visual_target: typing_assist::TextSize,
        requires_high_contrast: bool,
        #[serde(deserialize_with = "crate::cron::text::<_,16>")]
        expected_case_phrase: String,
        #[serde(deserialize_with = "crate::cron::text::<_,16>")]
        expected_filter_phrase: String,
        filter: Probe,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Probe {
    QuickRepeatedKey,
    AccidentalBriefKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "activity", rename_all = "snake_case", deny_unknown_fields)]
pub enum Workspace {
    Keyboard {
        #[serde(deserialize_with = "crate::cron::text::<_,{xorg::LIMIT}>")]
        draft: String,
        #[serde(deserialize_with = "crate::cron::text::<_,{xorg::LIMIT}>")]
        applied: String,
    },
    Remote {
        draft: RemoteDraft,
    },
    Typing {
        current: typing_assist::Settings,
    },
}

pub fn remote_trials(draft: RemoteDraft) -> [remote::Connection; 3] {
    use remote::Authorization::{NoGrant, RemoteLogin, SharedDesktop};
    let s = draft.services;
    [
        s.connect(draft.observer.client, draft.observer.port, SharedDesktop),
        s.connect(draft.employee.client, draft.employee.port, RemoteLogin),
        s.connect(remote::Client::RdpReady, draft.observer.port, NoGrant),
    ]
}

fn tap(key: typing_assist::Key, start: u32, length: u32) -> typing_assist::PhysicalTap {
    typing_assist::PhysicalTap {
        key,
        pressed_at_ms: start,
        released_at_ms: start + length,
    }
}

pub fn case_probe() -> [typing_assist::PhysicalTap; 3] {
    use typing_assist::Key::*;
    [tap(LeftShift, 0, 350), tap(A, 500, 420), tap(B, 1040, 320)]
}

pub fn filter_probe(probe: Probe) -> [typing_assist::PhysicalTap; 3] {
    use typing_assist::Key::*;
    match probe {
        Probe::QuickRepeatedKey => [tap(A, 0, 55), tap(A, 130, 70), tap(B, 240, 70)],
        Probe::AccidentalBriefKey => [tap(A, 0, 55), tap(B, 120, 350), tap(A, 600, 450)],
    }
}

pub fn sample(settings: typing_assist::Settings, taps: &[typing_assist::PhysicalTap]) -> String {
    use typing_assist::Event::Printed;
    typing_assist::preview(settings, taps)
        .expect("Short supplied ordered key traces")
        .into_iter()
        .filter_map(|event| {
            if let Printed(c) = event {
                Some(c)
            } else {
                None
            }
        })
        .collect()
}

impl Workspace {
    pub fn new(task: &Task) -> Self {
        match task {
            Task::Keyboard { starter, .. } => Self::Keyboard {
                draft: starter.clone(),
                applied: starter.clone(),
            },
            Task::Remote { starter, .. } => Self::Remote { draft: *starter },
            Task::Typing { starter, .. } => Self::Typing { current: *starter },
        }
    }

    pub fn validate(&self, task: &Task) -> Result<()> {
        match (self, task) {
            (Self::Keyboard { draft, applied }, Task::Keyboard { .. }) => {
                if draft.len() > xorg::LIMIT {
                    return Err("The saved virtual configuration draft exceeds 4 KiB.".into());
                }
                xorg::parse(applied)?;
                Ok(())
            }
            (Self::Remote { .. }, Task::Remote { .. })
            | (Self::Typing { .. }, Task::Typing { .. }) => Ok(()),
            _ => Err("This saved work is for a different workstation activity.".into()),
        }
    }

    pub fn checks(&self, task: &Task) -> Vec<Check> {
        if let Err(error) = self.validate(task) {
            return vec![Check::new("Saved workspace", false, error)];
        }
        match (self, task) {
            (Self::Keyboard { draft, applied }, Task::Keyboard { devices, .. }) => {
                let rules = xorg::parse(applied).expect("Validated virtual file");
                let mut results = vec![Check::new(
                    "Applied configuration",
                    draft == applied,
                    if draft == applied {
                        "The virtual session uses the current draft.".into()
                    } else {
                        "Apply the current fragment before checking the virtual session.".into()
                    },
                )];
                results.extend(devices.iter().map(|target| {
                    let actual = xorg::effective(&target.device, &rules);
                    let matched = if actual.matched.is_empty() {
                        "No InputClass matched. The default layout is active.".into()
                    } else {
                        format!(
                            "Matching rules in applied order: {}.",
                            actual.matched.join(" → ")
                        )
                    };
                    Check::new(
                        &target.device.product,
                        actual.layout == target.expected,
                        format!(
                            "Expected {}, observed {}. {matched}",
                            target.expected.name(),
                            actual.layout.name()
                        ),
                    )
                }));
                results
            }
            (
                Self::Remote { draft },
                Task::Remote {
                    require_observer,
                    require_login,
                    ..
                },
            ) => {
                use remote::Connection::*;
                let [view, login, _] = remote_trials(*draft);
                vec![
                    if *require_observer {
                        Check::new(
                            "View-only observer",
                            view == ObserveExisting,
                            view.detail().into(),
                        )
                    } else {
                        let off = draft.services.sharing == remote::Sharing::Off;
                        Check::new(
                            "Desktop Sharing stays off",
                            off,
                            if off {
                                "Desktop Sharing is off as required."
                            } else {
                                "Turn off Desktop Sharing for this profile."
                            }
                            .into(),
                        )
                    },
                    if *require_login {
                        Check::new(
                            "Employee login",
                            login == LoginToOwnSession,
                            login.detail().into(),
                        )
                    } else {
                        let off = !draft.services.remote_login;
                        Check::new("Remote Login stays off", off, if off {
                            "Remote Login is off as required."
                        } else {
                            "Turn off Remote Login. This example only needs to share the existing desktop."
                        }.into())
                    },
                ]
            }
            (
                Self::Typing { current },
                Task::Typing {
                    visual_target,
                    requires_high_contrast,
                    expected_case_phrase,
                    expected_filter_phrase,
                    filter,
                    ..
                },
            ) => {
                let lettering = sample(*current, &case_probe());
                let filtering = sample(*current, &filter_probe(*filter));
                let has_visual = current.text == *visual_target
                    && current.high_contrast == *requires_high_contrast;
                vec![
                    Check::new(
                        "Reading sample",
                        has_visual,
                        if has_visual {
                            "The reading sample uses the requested size and contrast.".into()
                        } else {
                            "Compare the sample with the requested text size and contrast.".into()
                        },
                    ),
                    Check::new(
                        "Modifier sequence",
                        lettering == *expected_case_phrase,
                        format!(
                            "Expected {expected_case_phrase:?}. The separate Shift and letter taps produced {lettering:?}."
                        ),
                    ),
                    Check::new(
                        "Typing sequence",
                        filtering == *expected_filter_phrase,
                        format!(
                            "Expected {expected_filter_phrase:?}. The measured taps produced {filtering:?}. Inspect the duration and repeat interval."
                        ),
                    ),
                ]
            }
            _ => unreachable!("Validated virtual activity shape"),
        }
    }
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Keyboard { starter, devices } => {
                xorg::parse(starter)?;
                let mut seen = std::collections::BTreeSet::new();
                if devices.len() < 2
                    || devices.len() > 6
                    || devices.iter().any(|device| {
                        let name = &device.device.product;
                        name.len() > 64
                            || name.trim().is_empty()
                            || name.chars().any(char::is_control)
                            || !seen.insert(name)
                    })
                {
                    return Err(
                        "Supply two to six virtual keyboards with distinct products.".into(),
                    );
                }
            }
            Self::Remote {
                require_observer,
                require_login,
                ..
            } => {
                if !require_observer && !require_login {
                    return Err("Supply at least one required virtual client connection.".into());
                }
            }
            Self::Typing {
                expected_case_phrase,
                expected_filter_phrase,
                ..
            } => {
                for phrase in [expected_case_phrase, expected_filter_phrase] {
                    if phrase.is_empty()
                        || phrase.len() > 16
                        || !phrase.bytes().all(|b| b.is_ascii_alphabetic())
                    {
                        return Err(
                            "Use short nonempty letter samples for the virtual key trials.".into(),
                        );
                    }
                }
            }
        }
        if Workspace::new(self).checks(self).iter().all(|c| c.passed) {
            return Err("Supply a workstation profile with at least one task to repair.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub workspace: Workspace,
    pub checked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    EditKeyboard {
        #[serde(deserialize_with = "crate::cron::text::<_,{xorg::LIMIT}>")]
        contents: String,
    },
    ApplyKeyboard,
    SetRemote {
        draft: RemoteDraft,
    },
    SetTyping {
        current: typing_assist::Settings,
    },
    Check,
    Reset,
}

impl Work {
    pub fn new(task: &Task) -> Self {
        Self {
            workspace: Workspace::new(task),
            checked: false,
        }
    }

    pub fn validate(&self, task: &Task) -> Result<()> {
        self.workspace.validate(task)
    }

    pub fn checks(&self, task: &Task) -> Vec<Check> {
        self.workspace.checks(task)
    }

    pub fn apply(&mut self, task: &Task, action: Action) -> Result<()> {
        match (&mut self.workspace, action) {
            (Workspace::Keyboard { draft, .. }, Action::EditKeyboard { contents }) => {
                if contents.len() > xorg::LIMIT {
                    return Err("Keep the keyboard configuration within 4 KiB.".into());
                }
                if *draft != contents {
                    *draft = contents;
                    self.checked = false;
                }
            }
            (Workspace::Keyboard { draft, applied }, Action::ApplyKeyboard) => {
                xorg::parse(draft)?;
                if *applied != *draft {
                    applied.clone_from(draft);
                    self.checked = false;
                }
            }
            (Workspace::Remote { draft }, Action::SetRemote { draft: next }) => {
                if *draft != next {
                    *draft = next;
                    self.checked = false;
                }
            }
            (Workspace::Typing { current }, Action::SetTyping { current: next }) => {
                if *current != next {
                    *current = next;
                    self.checked = false;
                }
            }
            (_, Action::Check) => self.checked = true,
            (_, Action::Reset) => *self = Self::new(task),
            _ => {
                return Err("This control is for a different virtual workstation activity.".into());
            }
        }
        Ok(())
    }
}
