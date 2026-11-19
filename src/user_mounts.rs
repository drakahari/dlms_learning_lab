//! Opt-in caller permissions over virtual removable storage. No host operations.
//! Flags describe the supplied policy; no binary/device/setuid execution is modeled.
use crate::{removable_mount as removable, storage};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor},
};
use std::{collections::BTreeMap, fmt};

pub const MAX_STATE_BYTES: usize = 131_072;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    Root,
    First,
    Second,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    RootOnly,
    MounterOnly,
    AnyUser,
}
/// In this explicitly supplied base configuration all three restrictions start
/// off. `defaults` is a no-op, never a claim about every kernel/filesystem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flags {
    pub noexec: bool,
    pub nosuid: bool,
    pub nodev: bool,
}
impl Flags {
    fn restricted() -> Self {
        Self {
            noexec: true,
            nosuid: true,
            nodev: true,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    UnknownSource,
    Unavailable,
    TypeMismatch,
    Occupied,
    NotMounted,
    PermissionDenied,
    Mismatch,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: Option<usize>,
    #[serde(deserialize_with = "text::<_,512>")]
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "Line {line}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, line: Option<usize>, message: &str) -> Error {
    Error {
        kind,
        line,
        message: message.into(),
    }
}
impl From<storage::Error> for Error {
    fn from(e: storage::Error) -> Self {
        let kind = match e.kind {
            storage::ErrorKind::Invalid => ErrorKind::Invalid,
            storage::ErrorKind::Unsupported => ErrorKind::Unsupported,
            storage::ErrorKind::Limit => ErrorKind::Limit,
            storage::ErrorKind::UnknownSource => ErrorKind::UnknownSource,
            storage::ErrorKind::TypeMismatch => ErrorKind::TypeMismatch,
            storage::ErrorKind::Occupied => ErrorKind::Occupied,
            storage::ErrorKind::NotMounted => ErrorKind::NotMounted,
        };
        Self {
            kind,
            line: e.line,
            message: e.message,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub entry: removable::Entry,
    pub permission: Permission,
    pub flags: Flags,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    raw: String,
    core: removable::Policy,
    rules: Vec<Rule>,
}
impl Policy {
    pub fn parse(raw: &str) -> Result<Self> {
        if raw.len() > storage::MAX_CONFIG_BYTES || raw.lines().count() > 128 {
            return Err(error(
                ErrorKind::Limit,
                None,
                "Use at most 16 KiB and 128 lines of mount policy",
            ));
        }
        if raw
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return Err(error(
                ErrorKind::Invalid,
                None,
                "Mount policy contains an unsupported control character",
            ));
        }
        let mut normalized = String::with_capacity(raw.len());
        let mut attributes = Vec::new();
        for (index, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                normalized.push_str(line);
                normalized.push('\n');
                continue;
            }
            let fields: Vec<_> = line.split_ascii_whitespace().take(7).collect();
            if fields.len() != 6 {
                return Err(error(
                    ErrorKind::Invalid,
                    Some(index + 1),
                    "Use six fields: source, mountpoint, filesystem, options, dump and check order",
                ));
            }
            if attributes.len() == storage::MAX_VOLUMES {
                return Err(error(
                    ErrorKind::Limit,
                    Some(index + 1),
                    "Too many virtual mount entries",
                ));
            }
            let mut permission = Permission::RootOnly;
            let mut permission_seen = false;
            let mut flags = Flags::default();
            let mut core_options = Vec::new();
            for (n, option) in fields[3].split(',').enumerate() {
                if n == 16 {
                    return Err(error(
                        ErrorKind::Limit,
                        Some(index + 1),
                        "Use at most 16 mount options per entry",
                    ));
                }
                match option {
                    "user" | "users" | "nouser" => {
                        if permission_seen {
                            return Err(error(
                                ErrorKind::Unsupported,
                                Some(index + 1),
                                "Mixed or repeated user/users/nouser options are outside this subset; use at most one permission option",
                            ));
                        }
                        permission_seen = true;
                        permission = match option {
                            "user" => Permission::MounterOnly,
                            "users" => Permission::AnyUser,
                            _ => Permission::RootOnly,
                        };
                        if option != "nouser" {
                            flags = Flags::restricted();
                        }
                    }
                    "exec" => flags.noexec = false,
                    "noexec" => flags.noexec = true,
                    "suid" => flags.nosuid = false,
                    "nosuid" => flags.nosuid = true,
                    "dev" => flags.nodev = false,
                    "nodev" => flags.nodev = true,
                    "defaults" | "ro" | "rw" | "auto" | "noauto" | "nofail" => {
                        core_options.push(option)
                    }
                    _ => {
                        return Err(error(
                            ErrorKind::Unsupported,
                            Some(index + 1),
                            "Unsupported mount option: this slice uses defaults, ro/rw, auto/noauto, nofail, one bare user/users/nouser and exec/noexec, suid/nosuid, dev/nodev",
                        ));
                    }
                }
            }
            let options = if core_options.is_empty() {
                "defaults".into()
            } else {
                core_options.join(",")
            };
            normalized.push_str(&format!(
                "{} {} {} {} {} {}\n",
                fields[0], fields[1], fields[2], options, fields[4], fields[5]
            ));
            attributes.push((permission, flags));
        }
        if normalized.len() == storage::MAX_CONFIG_BYTES + 1 && normalized.ends_with('\n') {
            normalized.pop();
        }
        if normalized.len() > storage::MAX_CONFIG_BYTES {
            return Err(error(
                ErrorKind::Limit,
                None,
                "Normalized mount policy exceeds 16 KiB; no field was truncated",
            ));
        }
        let core = removable::Policy::parse(&normalized)?;
        if core.entries().len() != attributes.len() {
            return Err(error(
                ErrorKind::Mismatch,
                None,
                "Mount rule count differs from its parsed policy",
            ));
        }
        let rules = core
            .entries()
            .iter()
            .cloned()
            .zip(attributes)
            .map(|(entry, (permission, flags))| Rule {
                entry,
                permission,
                flags,
            })
            .collect();
        Ok(Self {
            raw: raw.into(),
            core,
            rules,
        })
    }
    pub fn configuration(&self) -> &str {
        &self.raw
    }
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
    pub fn rule(&self, target: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.entry.mount.target == target)
    }
    pub fn core_policy(&self) -> &removable::Policy {
        &self.core
    }
    pub fn startup(&self, environment: &removable::Environment) -> Result<Startup> {
        let projection = self.core.startup(environment)?;
        let mut ledger = BTreeMap::new();
        for (target, mount) in projection.state.live() {
            let rule = self.rule(target).ok_or_else(|| {
                error(
                    ErrorKind::Mismatch,
                    None,
                    "A startup mount has no policy entry",
                )
            })?;
            ledger.insert(
                target.clone(),
                MountedBy {
                    volume: mount.volume.clone(),
                    mounter: Some(Actor::Root),
                    flags: Some(rule.flags),
                },
            );
        }
        let live = Live {
            version: 1,
            configuration: self.raw.clone(),
            state: projection.state,
            ledger,
        };
        live.validate_for(environment)?;
        Ok(Startup {
            live,
            rows: projection.rows,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Startup {
    pub live: Live,
    pub rows: Vec<removable::Row>,
}
impl Startup {
    pub fn required_attempts_satisfied(&self) -> bool {
        self.rows
            .iter()
            .filter(|r| r.dependency == removable::Dependency::Required)
            .all(|r| r.outcome == removable::Outcome::Mounted)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MountedBy {
    #[serde(deserialize_with = "text::<_,48>")]
    pub volume: String,
    /// None is unknown history, never inferred root or an ordinary caller.
    pub mounter: Option<Actor>,
    pub flags: Option<Flags>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "LiveData")]
pub struct Live {
    version: u8,
    configuration: String,
    state: storage::State,
    ledger: BTreeMap<String, MountedBy>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveData {
    version: u8,
    #[serde(deserialize_with = "text::<_,16384>")]
    configuration: String,
    #[serde(deserialize_with = "storage_state")]
    state: storage::State,
    #[serde(deserialize_with = "mount_map::<_,MountedBy,256>")]
    ledger: BTreeMap<String, MountedBy>,
}
impl TryFrom<LiveData> for Live {
    type Error = Error;
    fn try_from(data: LiveData) -> Result<Self> {
        let value = Self {
            version: data.version,
            configuration: data.configuration,
            state: data.state,
            ledger: data.ledger,
        };
        value.validate()?;
        Ok(value)
    }
}
impl Live {
    /// Existing mounts are imported without a claimed mounter or applied flags.
    /// New authored fixtures should prefer Policy::startup or actual mount_as.
    pub fn new(
        environment: &removable::Environment,
        policy: &Policy,
        mounts: BTreeMap<String, storage::Mount>,
    ) -> Result<Self> {
        environment.validate()?;
        let state = storage::State::new(
            environment.inventory.clone(),
            policy.core.core_configuration().into(),
            mounts,
        )?;
        let ledger = state
            .live()
            .iter()
            .map(|(target, mount)| {
                (
                    target.clone(),
                    MountedBy {
                        volume: mount.volume.clone(),
                        mounter: None,
                        flags: None,
                    },
                )
            })
            .collect();
        let live = Self {
            version: 1,
            configuration: policy.raw.clone(),
            state,
            ledger,
        };
        live.validate_for(environment)?;
        Ok(live)
    }
    pub fn state(&self) -> &storage::State {
        &self.state
    }
    pub fn ledger(&self) -> &BTreeMap<String, MountedBy> {
        &self.ledger
    }
    /// Saved original user policy, distinct from the normalized core configuration.
    pub fn configuration(&self) -> &str {
        &self.configuration
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.ledger.len() > storage::MAX_VOLUMES {
            return Err(error(
                ErrorKind::Invalid,
                None,
                "Unsupported user-mount state version or ledger count",
            ));
        }
        self.state.validate()?;
        let policy = Policy::parse(&self.configuration)?;
        if self.state.configuration() != policy.core.core_configuration()
            || self.ledger.keys().ne(self.state.live().keys())
            || self.ledger.iter().any(|(target, entry)| {
                target.len() > 256
                    || entry.volume.len() > 48
                    || self
                        .state
                        .live()
                        .get(target)
                        .is_none_or(|m| m.volume != entry.volume)
                    || entry.mounter.is_some() != entry.flags.is_some()
            })
        {
            return Err(error(
                ErrorKind::Mismatch,
                None,
                "Saved policy, actual mounts and mounter ledger disagree",
            ));
        }
        Ok(())
    }
    pub fn validate_for(&self, environment: &removable::Environment) -> Result<()> {
        self.validate()?;
        environment.validate()?;
        if self.state.inventory() != &environment.inventory {
            return Err(error(
                ErrorKind::Mismatch,
                None,
                "Mount operation belongs to a different supplied inventory or device-alias context",
            ));
        }
        if self
            .state
            .live()
            .values()
            .any(|m| !environment.present.contains(&m.volume))
        {
            return Err(error(
                ErrorKind::Unsupported,
                None,
                "Hot removal of an already mounted medium is outside this settled-state activity",
            ));
        }
        Ok(())
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_STATE_BYTES {
            return Err(error(
                ErrorKind::Limit,
                None,
                "User-mount snapshot exceeds 128 KiB",
            ));
        }
        serde_json::from_str(raw).map_err(|_| {
            error(
                ErrorKind::Invalid,
                None,
                "Invalid bounded user-mount snapshot; original work retained",
            )
        })
    }
    /// Apply saved text only. Actual mounts, their original mounters and applied
    /// flags stay exact, even when the new saved options disagree with them.
    pub fn save_policy(&mut self, policy: &Policy) -> Result<()> {
        self.validate()?;
        let mut next = self.clone();
        next.state
            .save_configuration(policy.core.core_configuration().into())?;
        next.configuration = policy.raw.clone();
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn mount_as(
        &mut self,
        environment: &removable::Environment,
        actor: Actor,
        target: &str,
    ) -> Result<storage::MountOutcome> {
        self.validate_for(environment)?;
        self.scope(target)?;
        let policy = Policy::parse(&self.configuration)?;
        let rule = policy.rule(target).ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                None,
                "No saved entry names that mountpoint",
            )
        })?;
        if actor != Actor::Root && rule.permission == Permission::RootOnly {
            return Err(error(
                ErrorKind::PermissionDenied,
                None,
                "This entry does not permit ordinary callers to mount it",
            ));
        }
        let id = environment.inventory.resolve(&rule.entry.mount.source)?;
        if !environment.present.contains(id) {
            return Err(error(
                ErrorKind::Unavailable,
                None,
                "The configured medium is absent; nofail does not make a manual mount succeed",
            ));
        }
        let mut next = self.clone();
        let result = policy
            .core
            .manual_mount(environment, &mut next.state, target)?;
        // An already-mounted request MUST NOT transfer ownership or rewrite
        // applied flags, including after a saved permission/flag edit.
        if result == storage::MountOutcome::AlreadyMounted {
            return Ok(result);
        }
        next.ledger.insert(
            target.into(),
            MountedBy {
                volume: id.into(),
                mounter: Some(actor),
                flags: Some(rule.flags),
            },
        );
        next.validate_for(environment)?;
        *self = next;
        Ok(result)
    }
    pub fn unmount_as(
        &mut self,
        environment: &removable::Environment,
        actor: Actor,
        target: &str,
    ) -> Result<()> {
        self.validate_for(environment)?;
        self.scope(target)?;
        let actual = self.state.live().get(target).ok_or_else(|| {
            error(
                ErrorKind::NotMounted,
                None,
                "No live volume is mounted at that location",
            )
        })?;
        if actor != Actor::Root {
            let policy = Policy::parse(&self.configuration)?;
            let rule = policy.rule(target).ok_or_else(|| {
                error(
                    ErrorKind::Mismatch,
                    None,
                    "Ordinary unmount requires a matching current saved entry",
                )
            })?;
            if environment.inventory.resolve(&rule.entry.mount.source)? != actual.volume.as_str() {
                return Err(error(
                    ErrorKind::Mismatch,
                    None,
                    "The current saved source and the actual mounted volume differ",
                ));
            }
            let mounter = self.ledger[target].mounter;
            if rule.permission != Permission::AnyUser
                && !(rule.permission == Permission::MounterOnly && mounter == Some(actor))
            {
                return Err(error(
                    ErrorKind::PermissionDenied,
                    None,
                    if mounter.is_none() {
                        "The mounting caller is unknown; no ordinary mounter-only authority can be inferred"
                    } else {
                        "Current policy does not let this caller unmount that volume; user requires its actual mounter, while users permits a handover"
                    },
                ));
            }
        }
        let mut next = self.clone();
        next.state.umount(target)?;
        next.ledger.remove(target);
        next.validate_for(environment)?;
        *self = next;
        Ok(())
    }
    fn scope(&self, target: &str) -> Result<()> {
        if !self.state.inventory().mountpoints().contains(target) {
            return Err(error(
                ErrorKind::Invalid,
                None,
                "Choose an exact supplied mountpoint; arbitrary paths, device operands and path traversal are outside this activity",
            ));
        }
        Ok(())
    }
}

// Bound strings/maps while decoding, including the nested legacy core wire
// shape. These are validation adapters, not filesystem or permission engines.
struct Text<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Bounded<const N: usize>;
        impl<const N: usize> Visitor<'_> for Bounded<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {N} bytes without NUL")
            }
            fn visit_str<E: serde::de::Error>(
                self,
                s: &str,
            ) -> std::result::Result<Self::Value, E> {
                if s.len() > N || s.contains('\0') {
                    Err(E::custom("User-mount text limit"))
                } else {
                    Ok(Text(s.into()))
                }
            }
        }
        d.deserialize_str(Bounded::<N>)
    }
}
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> std::result::Result<String, D::Error> {
    Ok(Text::<N>::deserialize(d)?.0)
}
fn mount_map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const K: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct Map<V, const K: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const K: usize> Visitor<'de> for Map<V, K> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "at most {} unique bounded mount keys",
                storage::MAX_VOLUMES
            )
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            if a.size_hint().is_some_and(|n| n > storage::MAX_VOLUMES) {
                return Err(A::Error::custom("User-mount map limit"));
            }
            let mut out = BTreeMap::new();
            while let Some(Text(key)) = a.next_key::<Text<K>>()? {
                if out.len() == storage::MAX_VOLUMES || out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate or excessive user-mount key"));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Map::<V, K>(std::marker::PhantomData))
}
fn storage_state<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<storage::State, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Mount {
        #[serde(deserialize_with = "text::<_,48>")]
        volume: String,
        read_only: bool,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Saved {
        inventory: storage::Inventory,
        #[serde(deserialize_with = "text::<_,16384>")]
        configuration: String,
        #[serde(deserialize_with = "mount_map::<_,Mount,256>")]
        live: BTreeMap<String, Mount>,
    }
    let s = Saved::deserialize(d)?;
    storage::State::new(
        s.inventory,
        s.configuration,
        s.live
            .into_iter()
            .map(|(k, m)| {
                (
                    k,
                    storage::Mount {
                        volume: m.volume,
                        read_only: m.read_only,
                    },
                )
            })
            .collect(),
    )
    .map_err(D::Error::custom)
}
