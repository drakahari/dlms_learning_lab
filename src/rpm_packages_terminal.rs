//! Closed literal RPM spellings over declared script-free noarch metadata; no host bridge.
use crate::{
    rpm_packages::{self as core, State},
    rpm_packages_workspace::{self as work, Definition},
    shell,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
pub const MAX_COMMAND: usize = 2048;
pub const MAX_OUTPUT: usize = 16_384;
pub const MAX_SESSION: usize = 98_304;
pub const FORMAT: &str = "%{NAME}\\t%{EPOCHNUM}:%{VERSION}-%{RELEASE}.%{ARCH}\\n";
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum Outcome {
    Utility {
        stdout: String,
        stderr: String,
        status: u8,
        summary: Option<String>,
    },
    SimulatorUnsupported {
        reason: String,
    },
}
impl<'de> Deserialize<'de> for Outcome {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", deny_unknown_fields)]
        enum Raw {
            Utility {
                #[serde(deserialize_with = "work::string::<_,MAX_OUTPUT>")]
                stdout: String,
                #[serde(deserialize_with = "work::string::<_,512>")]
                stderr: String,
                status: u8,
                #[serde(deserialize_with = "optional_notice")]
                summary: Option<String>,
            },
            SimulatorUnsupported {
                #[serde(deserialize_with = "work::string::<_,512>")]
                reason: String,
            },
        }
        let value = match Raw::deserialize(d)? {
            Raw::Utility {
                stdout,
                stderr,
                status,
                summary,
            } => Self::Utility {
                stdout,
                stderr,
                status,
                summary,
            },
            Raw::SimulatorUnsupported { reason } => Self::SimulatorUnsupported { reason },
        };
        if !value.valid() {
            return Err(D::Error::custom("Invalid RPM terminal outcome protocol"));
        }
        Ok(value)
    }
}
fn optional_notice<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<work::Text<512>>::deserialize(d)?.map(|v| v.0))
}
impl Outcome {
    pub fn valid(&self) -> bool {
        match self {
            Self::Utility {
                stdout,
                stderr,
                status,
                summary,
            } => {
                *status == 0
                    && stderr.is_empty()
                    && stdout.len() <= MAX_OUTPUT
                    && !stdout.contains('\0')
                    && summary
                        .as_ref()
                        .is_none_or(|s| s.len() <= 512 && !s.contains('\0'))
            }
            Self::SimulatorUnsupported { reason } => {
                !reason.is_empty() && reason.len() <= 512 && !reason.contains('\0')
            }
        }
    }
}
fn unsupported(s: impl AsRef<str>) -> Outcome {
    Outcome::SimulatorUnsupported {
        reason: work::notice(s),
    }
}
fn utility(stdout: String, summary: Option<String>) -> Outcome {
    if stdout.len() > MAX_OUTPUT {
        return unsupported("RPM output exceeds the admitted display bound");
    }
    Outcome::Utility {
        stdout,
        stderr: String::new(),
        status: 0,
        summary,
    }
}
#[derive(Clone, Debug)]
enum Query {
    Identity { format: bool },
    Files,
    Provides,
    Requires,
}
#[derive(Clone, Debug)]
enum Select {
    Installed(String),
    Archive(String),
    Owner { path: String, recorded: bool },
    Providers(String),
}
#[derive(Clone, Debug)]
enum Kind {
    Query(Select, Query),
    Install { paths: Vec<String>, upgrade: bool },
    Reinstall(String),
    Erase(Vec<String>),
}
#[derive(Clone, Debug)]
pub struct Command {
    kind: Kind,
}
fn scalar(s: &str) -> Result<String, String> {
    if !work::name(s) {
        return Err("Only exact bounded package/capability names are supported".into());
    }
    Ok(s.into())
}
fn path(s: &str) -> Result<String, String> {
    if !work::path(s) {
        return Err("Only canonical supplied absolute virtual paths are supported".into());
    }
    Ok(s.into())
}
pub fn parse(input: &str) -> Result<Command, String> {
    if input.len() > MAX_COMMAND {
        return Err("RPM command exceeds2048 bytes before trimming".into());
    }
    if input.chars().any(|c| c.is_control() && c != '\t') {
        return Err(
            "Enter one literal RPM command without physical newlines/control characters".into(),
        );
    }
    let w = shell::literal_command(input).map_err(work::notice)?;
    if w.len() > 32 || w.first().map(String::as_str) != Some("rpm") {
        return Err("Only the closed virtual rpm command grammar is supported".into());
    }
    let args: Vec<&str> = w[1..].iter().map(String::as_str).collect();
    let kind=match args.as_slice(){
 ["-q",n]=>Kind::Query(Select::Installed(scalar(n)?),Query::Identity{format:false}),
 ["-ql",n]=>Kind::Query(Select::Installed(scalar(n)?),Query::Files),
 ["-qp",p]=>Kind::Query(Select::Archive(path(p)?),Query::Identity{format:false}),
 ["-qpl",p]=>Kind::Query(Select::Archive(path(p)?),Query::Files),
 ["-qf",p]=>Kind::Query(Select::Owner{path:path(p)?,recorded:false},Query::Identity{format:false}),
 ["-q","--path",p]=>Kind::Query(Select::Owner{path:path(p)?,recorded:true},Query::Identity{format:false}),
 ["-q","--whatprovides",n]=>Kind::Query(Select::Providers(scalar(n)?),Query::Identity{format:false}),
 [mode,"--provides",n] if *mode=="-q"||*mode=="-qp"=>Kind::Query(if *mode=="-q"{Select::Installed(scalar(n)?)}else{Select::Archive(path(n)?)},Query::Provides),
 [mode,"--requires",n] if *mode=="-q"||*mode=="-qp"=>Kind::Query(if *mode=="-q"{Select::Installed(scalar(n)?)}else{Select::Archive(path(n)?)},Query::Requires),
 [mode,"--queryformat",f,n] if (*mode=="-q"||*mode=="-qp")&&*f==FORMAT=>Kind::Query(if *mode=="-q"{Select::Installed(scalar(n)?)}else{Select::Archive(path(n)?)},Query::Identity{format:true}),
 [mode,operands @ ..] if (*mode=="-i"||*mode=="-U")&&!operands.is_empty()&&operands.len()<=8=>{let paths=operands.iter().map(|s|path(s)).collect::<Result<Vec<_>,_>>()?;if paths.iter().enumerate().any(|(n,p)|paths[..n].contains(p)){return Err("Duplicate archive operand is unsupported".into());}Kind::Install{paths,upgrade:*mode=="-U"}},
 ["--reinstall",p]=>Kind::Reinstall(path(p)?),
 ["-e",names @ ..] if !names.is_empty()&&names.len()<=8=>{let names=names.iter().map(|s|scalar(s)).collect::<Result<Vec<_>,_>>()?;if names.iter().enumerate().any(|(n,p)|names[..n].contains(p)){return Err("Duplicate package operand is unsupported".into());}Kind::Erase(names)},
 _=>return Err("Unsupported RPM spelling/options; no -V, force, solver, macro, source, shell or host operation".into()),};
    Ok(Command { kind })
}
pub fn validate_guidance(input: &str) -> Result<(), String> {
    parse(input).map(|_| ())
}
fn installed<'a>(s: &'a State, n: &str) -> Option<&'a core::Archive> {
    s.runtime()
        .installed
        .iter()
        .find(|x| x.identity.name == n)
        .map(|x| &s.catalog().archives[&x.archive])
}
fn archive<'a>(d: &Definition, s: &'a State, p: &str) -> Result<&'a core::Archive, String> {
    let id = d
        .archive_paths
        .get(p)
        .ok_or("Archive path is not supplied by this exercise")?;
    if !s.runtime().available.contains(id) {
        return Err("Supplied archive is not locally present".into());
    }
    Ok(&s.catalog().archives[id])
}
fn parts(e: &core::Evr) -> (&str, &str, &str) {
    let (epoch, tail) = e.as_str().split_once(':').unwrap_or(("0", e.as_str()));
    let (v, r) = tail.rsplit_once('-').expect("validated complete EVR");
    (epoch, v, r)
}
fn nvra(a: &core::Archive) -> String {
    let (_, v, r) = parts(&a.evr);
    format!("{}-{v}-{r}.noarch", a.identity.name)
}
fn relation(r: core::Relation) -> &'static str {
    match r {
        core::Relation::Lt => "<",
        core::Relation::Le => "<=",
        core::Relation::Eq => "=",
        core::Relation::Ge => ">=",
        core::Relation::Gt => ">",
    }
}
fn query(d: &Definition, s: &State, select: &Select, q: &Query) -> Result<String, String> {
    let rows: Vec<&core::Archive> = match select {
        Select::Installed(n) => vec![
            installed(s, n)
                .ok_or("No installed package; exact RPM no-match output/status is not modeled")?,
        ],
        Select::Archive(p) => vec![archive(d, s, p)?],
        Select::Owner { path: p, recorded } => {
            if !s.catalog().allowed_paths.contains(p) {
                return Err("Path is outside the supplied file universe".into());
            }
            if !recorded && !s.runtime().files.contains_key(p) {
                return Err("-qf missing-file lookup is outside this contract; use --path for recorded ownership".into());
            }
            s.runtime()
                .installed
                .iter()
                .map(|x| &s.catalog().archives[&x.archive])
                .filter(|a| a.files.contains_key(p))
                .collect()
        }
        Select::Providers(n) => {
            let requirement = core::Requirement {
                capability: n.clone(),
                constraint: None,
            };
            s.installed_providers(&requirement)
                .map_err(|e| work::notice(e.message))?
        }
    };
    if rows.is_empty() {
        return Err("No matching declared installed record; exact RPM no-match output/status is not modeled".into());
    }
    let mut out = String::new();
    for a in rows {
        let lines: Vec<String> = match q {
            Query::Identity { format: false } => vec![nvra(a)],
            Query::Identity { format: true } => {
                let (e, v, r) = parts(&a.evr);
                let e = e.parse::<u32>().expect("validated numeric epoch");
                vec![format!("{}\t{e}:{v}-{r}.noarch", a.identity.name)]
            }
            Query::Files => {
                if a.files.is_empty() {
                    return Err("Empty-manifest -ql/-qpl output/status is not modeled".into());
                }
                a.files.keys().cloned().collect()
            }
            Query::Provides => {
                let mut m = a.provides.clone();
                m.insert(a.identity.name.clone(), Some(a.evr.clone()));
                m.into_iter()
                    .map(|(n, v)| v.map_or_else(|| n.clone(), |v| format!("{n} = {}", v.as_str())))
                    .collect()
            }
            Query::Requires => a
                .requires
                .iter()
                .map(|r| {
                    r.constraint.as_ref().map_or_else(
                        || r.capability.clone(),
                        |c| {
                            format!(
                                "{} {} {}",
                                r.capability,
                                relation(c.relation),
                                c.evr.as_str()
                            )
                        },
                    )
                })
                .collect(),
        };
        for line in lines {
            if out.len().saturating_add(line.len() + 1) > MAX_OUTPUT {
                return Err("RPM query output exceeds admitted bytes".into());
            }
            out.push_str(&line);
            out.push('\n');
        }
    }
    Ok(out)
}
fn action(d: &Definition, s: &State, k: &Kind) -> Result<core::Action, String> {
    match k {
        Kind::Install { paths, upgrade } => {
            let archives = paths
                .iter()
                .map(|p| {
                    archive(d, s, p)
                        .and_then(|_| d.archive_paths.get(p).cloned().ok_or("Unknown path".into()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(if *upgrade {
                core::Action::UpgradeSet { archives }
            } else {
                core::Action::InstallSet { archives }
            })
        }
        Kind::Reinstall(p) => {
            archive(d, s, p)?;
            Ok(core::Action::ReinstallExact {
                archive: d.archive_paths[p].clone(),
            })
        }
        Kind::Erase(names) => {
            let packages = names
                .iter()
                .map(|n| {
                    installed(s, n).map(|a| a.identity.clone()).ok_or_else(|| {
                        "Exact installed name is absent; RPM failure protocol is not modeled".into()
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(core::Action::EraseSet { packages })
        }
        _ => Err("A query is not a mutating action".into()),
    }
}
pub fn execute(d: &Definition, s: &mut State, input: &str) -> Outcome {
    let c = match parse(input) {
        Ok(c) => c,
        Err(e) => return unsupported(e),
    };
    if d.validate().is_err() || !d.same_assignment(s) || !work::admitted(s) {
        return unsupported("RPM bounded authored context differs; current state retained");
    }
    match &c.kind {
        Kind::Query(select, q) => match query(d, s, select, q) {
            Ok(out) => utility(out, None),
            Err(e) => unsupported(e),
        },
        _ => match action(d, s, &c.kind).and_then(|a| work::apply(d, s, &a)) {
            Ok(p) => utility(
                String::new(),
                Some(work::notice(format!(
                    "Simulation applied {} declared script-free package changes; no host RPM transaction",
                    p.changes.len()
                ))),
            ),
            Err(e) => unsupported(e),
        },
    }
}
pub fn output_text(o: Outcome) -> Result<String, String> {
    match o {
        Outcome::Utility {
            stdout, summary, ..
        } => Ok(summary.map_or(stdout.clone(), |s| {
            if stdout.is_empty() {
                s
            } else {
                format!("{stdout}\n{s}")
            }
        })),
        Outcome::SimulatorUnsupported { reason } => Err(format!("SimulatorUnsupported: {reason}")),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "work::string::<_,MAX_COMMAND>")]
    pub command: String,
    pub output: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Session {
    pub version: u32,
    pub command: String,
    pub entries: Vec<Entry>,
    pub selected_path: Option<String>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            version: 1,
            command: String::new(),
            entries: Vec::new(),
            selected_path: None,
        }
    }
}
impl<'de> Deserialize<'de> for Session {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u32,
            #[serde(deserialize_with = "work::string::<_,MAX_COMMAND>")]
            command: String,
            #[serde(deserialize_with = "work::list::<_,Entry,16>")]
            entries: Vec<Entry>,
            #[serde(default)]
            selected_path: Option<work::Text<160>>,
        }
        let r = Raw::deserialize(d)?;
        let s = Self {
            version: r.version,
            command: r.command,
            entries: r.entries,
            selected_path: r.selected_path.map(|s| s.0),
        };
        if !s.valid() {
            return Err(D::Error::custom("RPM Session bounds differ"));
        }
        Ok(s)
    }
}
impl Session {
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.command.len() <= MAX_COMMAND
            && !self.command.contains('\0')
            && self.entries.len() <= 16
            && self.entries.iter().all(|e| {
                e.command.len() <= MAX_COMMAND && !e.command.contains('\0') && e.output.valid()
            })
            && self.selected_path.as_ref().is_none_or(|p| work::path(p))
            && work::within(self, MAX_SESSION)
    }
    pub fn valid_for(&self, s: &State) -> bool {
        self.valid()
            && self
                .selected_path
                .as_ref()
                .is_none_or(|p| s.catalog().allowed_paths.contains(p))
    }
    fn fits(&self, command: &str, entries: &[&Entry]) -> bool {
        #[derive(Serialize)]
        struct B<'a> {
            version: u32,
            command: &'a str,
            entries: &'a [&'a Entry],
            selected_path: &'a Option<String>,
        }
        work::within(
            &B {
                version: self.version,
                command,
                entries,
                selected_path: &self.selected_path,
            },
            MAX_SESSION,
        )
    }
    pub fn insert(&mut self, text: &str) -> Result<(), String> {
        if text.len() > MAX_COMMAND || text.contains('\0') || !self.valid() {
            return Err("RPM insertion exceeds bounds; previous draft retained".into());
        }
        let entries = self.entries.iter().collect::<Vec<_>>();
        if !self.fits(text, &entries) {
            return Err("RPM Session aggregate limit before draft clone".into());
        }
        self.command = text.into();
        Ok(())
    }

    pub fn run(&mut self, d: &Definition, s: &mut State) -> Result<Outcome, String> {
        if self.command.len() > MAX_COMMAND {
            return Err(
                "SimulatorUnsupported: RPM command exceeds2048 bytes; it was not run".into(),
            );
        }
        if self.command.contains('\0') {
            return Err(
                "SimulatorUnsupported: NUL in command; it was not run or added to history".into(),
            );
        }
        if !self.valid_for(s) || !work::admitted(s) || !d.same_assignment(s) {
            return Err("Foreign/oversized RPM Session; previous work retained".into());
        }
        let mut state = s.clone();
        let output = execute(d, &mut state, &self.command);
        let entry = Entry {
            command: self.command.clone(),
            output: output.clone(),
        };
        let mut entries = self
            .entries
            .iter()
            .chain(std::iter::once(&entry))
            .collect::<Vec<_>>();
        while entries.len() > 16 || !self.fits(&self.command, &entries) {
            if entries.is_empty() {
                return Err("RPM output cannot fit Session".into());
            }
            entries.remove(0);
        }
        // Full retained history is copied only AFTER its actual escaped JSON fits.
        let next = Self {
            version: self.version,
            command: self.command.clone(),
            entries: entries.into_iter().cloned().collect(),
            selected_path: self.selected_path.clone(),
        };
        if !next.valid_for(&state) {
            return Err("RPM Session context changed".into());
        }
        *s = state;
        *self = next;
        Ok(output)
    }
}
