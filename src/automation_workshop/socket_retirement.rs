//! Supplied socket activation in a finite teaching inventory. Uses no host I/O.
use crate::unit_manager::{self as units, Action as ManagerAction, Runtime, UnitType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    ObsoleteService,
    ObsoleteSocket,
    RequiredService,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Start,
    Stop,
    Enable,
    Disable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SocketPolicy {
    pub enabled: bool,
    pub listening: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub services: units::State,
    pub activation: SocketPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub initial: Work,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub obsolete_service: String,
    pub obsolete_port: u16,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub activating_socket: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub required_service: String,
    pub required_port: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    AlreadyRunning,
    SocketActivated,
    NoListener,
    ServiceDidNotStart,
}

impl Request {
    fn answered(self) -> bool {
        matches!(self, Self::AlreadyRunning | Self::SocketActivated)
    }
}

fn listening(work: &Work, unit: &str, port: u16) -> Result<bool, String> {
    Ok(matches!(
        work.services.runtime().get(unit),
        Some(Runtime::Running { .. })
    ) && work
        .services
        .result(unit)
        .map_err(remap)?
        .is_some_and(|actual| actual.port == Some(port)))
}

fn remap(error: units::Error) -> String {
    error.message
}

impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.services.validate().map_err(remap)?;
        let definitions = self.initial.services.definitions().map_err(remap)?;
        let service = |name: &str| {
            definitions
                .get(name)
                .is_some_and(|def| matches!(&def.kind, UnitType::Service { .. }))
        };
        let socket_component = self.activating_socket.strip_suffix(".socket");
        let live = self.initial.activation;
        if !service(&self.obsolete_service)
            || !service(&self.required_service)
            || self.obsolete_service == self.required_service
            || socket_component.is_none_or(|text| {
                text.is_empty()
                    || text.len() > 56
                    || !text
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
            })
            || !live.enabled
            || !live.listening
            || self.obsolete_port == 0
            || self.required_port == 0
            || self.obsolete_port == self.required_port
            || !listening(&self.initial, &self.required_service, self.required_port)?
        {
            return Err("Supply a working, unnecessary socket activation path and a different required service.".into());
        }
        let mut trial = self.initial.clone();
        self.apply(&mut trial, Target::ObsoleteService, Operation::Stop)?;
        if self.request(&mut trial, Target::ObsoleteService)? != Request::SocketActivated {
            return Err(
                "The supplied unused service must be startable by its active socket.".into(),
            );
        }
        Ok(())
    }

    pub fn check_work(&self, state: &Work) -> Result<(), String> {
        let initial = &self.initial.services;
        if !state.services.same_context(initial)
            || state.services.documents() != initial.documents()
            || state.services.loaded_sources() != initial.loaded_sources()
            || state.services.configuration() != initial.configuration()
            || state.services.default_target() != initial.default_target()
        {
            return Err(
                "The running service and boot policy must belong to this supplied work order."
                    .into(),
            );
        }
        Ok(())
    }

    pub fn apply(
        &self,
        work: &mut Work,
        target: Target,
        operation: Operation,
    ) -> Result<(), String> {
        if target == Target::ObsoleteSocket {
            match operation {
                Operation::Start => work.activation.listening = true,
                Operation::Stop => work.activation.listening = false,
                Operation::Enable => work.activation.enabled = true,
                Operation::Disable => work.activation.enabled = false,
            }
            return Ok(());
        }
        let unit = if target == Target::ObsoleteService {
            self.obsolete_service.clone()
        } else {
            self.required_service.clone()
        };
        let action = match operation {
            Operation::Start => ManagerAction::Start { unit },
            Operation::Stop => ManagerAction::Stop { unit },
            Operation::Enable => ManagerAction::Enable { unit },
            Operation::Disable => ManagerAction::Disable { unit },
        };
        let mut next = work.services.clone();
        next.apply(&action).map_err(remap)?;
        if operation == Operation::Start {
            next.apply(&ManagerAction::AdvancePrograms {})
                .map_err(remap)?;
        }
        work.services = next;
        Ok(())
    }

    pub fn request(&self, work: &mut Work, target: Target) -> Result<Request, String> {
        let (unit, port) = match target {
            Target::ObsoleteService => (&self.obsolete_service, self.obsolete_port),
            Target::RequiredService => (&self.required_service, self.required_port),
            Target::ObsoleteSocket => {
                return Err("Select the supplied service endpoint to test a request.".into());
            }
        };
        if listening(work, unit, port)? {
            return Ok(Request::AlreadyRunning);
        }
        if target != Target::ObsoleteService || !work.activation.listening {
            return Ok(Request::NoListener);
        }
        self.apply(work, target, Operation::Start)?;
        Ok(if listening(work, unit, port)? {
            Request::SocketActivated
        } else {
            Request::ServiceDidNotStart
        })
    }

    pub fn next_session(&self, work: &Work) -> Result<Work, String> {
        let (mut services, _) = work.services.fresh_session().map_err(remap)?;
        services
            .apply(&ManagerAction::AdvancePrograms {})
            .map_err(remap)?;
        let enabled = work.activation.enabled;
        let default = services.default_target();
        let booted = services.runtime().get(default).is_some_and(Runtime::active);
        Ok(Work {
            services,
            activation: SocketPolicy {
                enabled,
                listening: enabled && booted,
            },
        })
    }

    pub fn checks(&self, work: &Work) -> Result<[bool; 6], String> {
        self.check_work(work)?;
        let mut present = work.clone();
        let mut boot = self.next_session(work)?;
        let obsolete_enrolled = work
            .services
            .boot_links()
            .values()
            .any(|units| units.contains(&self.obsolete_service));
        Ok([
            !work.activation.enabled,
            !work.activation.listening,
            !obsolete_enrolled,
            !self
                .request(&mut present, Target::ObsoleteService)?
                .answered(),
            !self.request(&mut boot, Target::ObsoleteService)?.answered(),
            self.request(&mut present, Target::RequiredService)?
                .answered()
                && self.request(&mut boot, Target::RequiredService)?.answered(),
        ])
    }
}
