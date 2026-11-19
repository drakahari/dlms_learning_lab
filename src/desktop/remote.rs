//! Supplied GNOME RDP desktop sharing and remote login profiles, no transport.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sharing {
    Off,
    ViewOnly,
    AllowControl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Services {
    pub sharing: Sharing,
    pub remote_login: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Port {
    Standard,
    Secondary,
}

impl Port {
    pub const fn number(self) -> u16 {
        match self {
            Self::Standard => 3389,
            Self::Secondary => 3390,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Client {
    RdpReady,
    OtherProtocolOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authorization {
    SharedDesktop,
    RemoteLogin,
    NoGrant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    WrongProtocol,
    NoService,
    WrongAccess,
    ObserveExisting,
    ControlExisting,
    LoginToOwnSession,
}

impl Connection {
    pub const fn detail(self) -> &'static str {
        match self {
            Self::WrongProtocol => "This supplied client cannot start an RDP connection.",
            Self::NoService => "No service is enabled on that port in the virtual profile.",
            Self::WrongAccess => {
                "The service answered. This client role does not have its login grant."
            }
            Self::ObserveExisting => "Connected to the existing desktop with viewing only.",
            Self::ControlExisting => "Connected to the existing desktop with input control.",
            Self::LoginToOwnSession => {
                "Authenticated to the remote login screen for a separate session."
            }
        }
    }
}

impl Services {
    pub const fn sharing_port(self) -> Option<Port> {
        match (self.sharing, self.remote_login) {
            (Sharing::Off, _) => None,
            (_, true) => Some(Port::Secondary),
            (_, false) => Some(Port::Standard),
        }
    }

    pub const fn connect(
        self,
        client: Client,
        port: Port,
        authorization: Authorization,
    ) -> Connection {
        if matches!(client, Client::OtherProtocolOnly) {
            return Connection::WrongProtocol;
        }
        if matches!(port, Port::Standard) && self.remote_login {
            return if matches!(authorization, Authorization::RemoteLogin) {
                Connection::LoginToOwnSession
            } else {
                Connection::WrongAccess
            };
        }
        match (self.sharing_port(), port) {
            (Some(Port::Standard), Port::Standard) | (Some(Port::Secondary), Port::Secondary) => {
                match (self.sharing, authorization) {
                    (Sharing::ViewOnly, Authorization::SharedDesktop) => {
                        Connection::ObserveExisting
                    }
                    (Sharing::AllowControl, Authorization::SharedDesktop) => {
                        Connection::ControlExisting
                    }
                    _ => Connection::WrongAccess,
                }
            }
            _ => Connection::NoService,
        }
    }
}
