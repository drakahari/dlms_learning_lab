//! Closed literal Debian spellings over fictional script-free data; no host utility.
use crate::{
    debian_packages::{self as core, Action, State, Status, Version},
    debian_packages_workspace::{self as work, Definition},
    shell,
};
use std::cmp::Ordering;
pub const MAX_COMMAND_BYTES: usize = 2048;
pub const MAX_OUTPUT_BYTES: usize = 32768;
pub const HELP: &str = "Virtual Debian package console: dpkg-query -W [-f='FORMAT'] [NAME]; dpkg-query -S ABS_PATH (installed recorded owner); apt-cache --important depends NAME; apt-get --error-on=any update; apt-get -y install NAME[=VERSION]...; apt-get -y --reinstall install NAME=VERSION (same installed/cached artifact; all installed Depends healthy); dpkg -r|--remove NAME; dpkg -P|--purge NAME; dpkg --compare-versions V1 lt|le|eq|ne|ge|gt V2. Literal arguments only. Formatter fields: Package, binary:Package, Version, Architecture, db:Status-Status. Script-free final-state simulation, not dpkg/APT transaction or rollback execution, archives, downloads, privilege or security checks. No dpkg -i/unpack/configure/verify/audit, APT removal/resolver/pinning or RPM aliases. Mutation output is explicitly a model-derived simulation summary. help/show are simulator helpers; reading help or inspecting data does not complete the assignment.";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Syntax,
    Unsupported,
    Context,
    Limit,
    Model,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub status: u8,
    pub message: String,
}
type Result<T> = std::result::Result<T, Diagnostic>;
fn error(kind: DiagnosticKind, status: u8, text: impl AsRef<str>) -> Diagnostic {
    Diagnostic {
        kind,
        status,
        message: work::notice(text),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Dpkg,
    Query,
    Apt,
}
impl Family {
    fn failure(self) -> u8 {
        if self == Self::Apt { 100 } else { 2 }
    }
}
#[derive(Clone, Debug)]
enum Piece {
    Text(String),
    Field(Field),
}
#[derive(Clone, Copy, Debug)]
enum Field {
    Package,
    Version,
    Architecture,
    Status,
}
#[derive(Clone, Copy, Debug)]
enum Relation {
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
}
#[derive(Clone, Debug)]
enum Kind {
    Show {
        package: Option<String>,
        format: Vec<Piece>,
    },
    Owner(String),
    Depends(String),
    Update,
    Install(Vec<core::Request>),
    Reinstall {
        package: String,
        version: Version,
    },
    Remove {
        package: String,
        purge: bool,
    },
    Compare {
        left: Version,
        relation: Relation,
        right: Version,
    },
    Help,
}
/// Only parse constructs a Command; it has no decoder or public mutable fields.
#[derive(Clone, Debug)]
pub struct Command {
    family: Family,
    kind: Kind,
}
/// Admit only the disclosed parser subset in optional authored guidance.
/// A simulator helper must be labeled even when its name was quoted.
pub fn validate_guidance(input: &str, simulator_only: bool) -> Result<()> {
    let command = parse(input)?;
    if matches!(command.kind, Kind::Help) && !simulator_only {
        return Err(error(
            DiagnosticKind::Unsupported,
            2,
            "Label package-console help as a simulator-only helper",
        ));
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: u8,
    pub changed: bool,
}
fn package(s: &str) -> bool {
    s.len() >= 2
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"+.-".contains(&b))
}
fn exact_name(s: &str, family: Family) -> Result<String> {
    if !package(s) {
        return Err(error(
            DiagnosticKind::Unsupported,
            family.failure(),
            "Use one exact admitted package name; patterns, architecture/release selectors and paths are unsupported",
        ));
    }
    if family == Family::Apt && s.ends_with(['+', '-']) {
        return Err(error(
            DiagnosticKind::Unsupported,
            100,
            "APT trailing + or - selectors are unsupported, including quoted selectors",
        ));
    }
    Ok(s.into())
}
fn exact_path(s: &str) -> bool {
    s.len() <= 160
        && s.len() > 1
        && s.starts_with('/')
        && !s.chars().any(char::is_control)
        && !s.contains(['*', '?', '[', ']', '\\'])
        && s[1..]
            .split('/')
            .all(|p| !p.is_empty() && !matches!(p, "." | ".."))
}
fn version(s: &str, family: Family) -> Result<Version> {
    Version::parse(s).map_err(|e| error(DiagnosticKind::Unsupported, family.failure(), e.message))
}
fn request(s: &str) -> Result<core::Request> {
    // A trailing byte inside an exact VERSION is not a package-name selector.
    let (name, value) = match s.split_once('=') {
        Some((n, v)) => (n, Some(v)),
        None => (s, None),
    };
    Ok(core::Request {
        name: exact_name(name, Family::Apt)?,
        version: value.map(|v| version(v, Family::Apt)).transpose()?,
    })
}
fn format(source: &str) -> Result<Vec<Piece>> {
    if source.len() > 256 {
        return Err(error(
            DiagnosticKind::Limit,
            2,
            "Query format exceeds256 UTF8 bytes",
        ));
    }
    let mut chars = source.chars().peekable();
    let mut out = Vec::new();
    let mut text = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => text.push(match chars.next() {
                Some('n') => '\n',
                Some('t') => '\t',
                Some('\\') => '\\',
                _ => {
                    return Err(error(
                        DiagnosticKind::Unsupported,
                        2,
                        "Only \\n, \\t and \\\\ format escapes are supported",
                    ));
                }
            }),
            '$' => {
                if chars.next() != Some('{') {
                    return Err(error(
                        DiagnosticKind::Unsupported,
                        2,
                        "Only the documented literal ${FIELD} query format is supported",
                    ));
                }
                let mut name = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    name.push(c);
                }
                if !closed {
                    return Err(error(
                        DiagnosticKind::Syntax,
                        2,
                        "Close the query format field",
                    ));
                }
                let field = match name.as_str() {
                    "Package" | "binary:Package" => Field::Package,
                    "Version" => Field::Version,
                    "Architecture" => Field::Architecture,
                    "db:Status-Status" => Field::Status,
                    _ => {
                        return Err(error(
                            DiagnosticKind::Unsupported,
                            2,
                            "Unsupported query format field; selection/error flags and other database fields are not modeled",
                        ));
                    }
                };
                if !text.is_empty() {
                    out.push(Piece::Text(std::mem::take(&mut text)));
                }
                out.push(Piece::Field(field));
            }
            c if c.is_control() => {
                return Err(error(
                    DiagnosticKind::Unsupported,
                    2,
                    "Use escaped tabs/newlines in the query format",
                ));
            }
            _ => text.push(c),
        }
    }
    if !text.is_empty() {
        out.push(Piece::Text(text));
    }
    Ok(out)
}
fn query(words: &[String], alias: bool) -> Result<Kind> {
    let mut action = None;
    let mut pattern = None;
    let mut fmt = None;
    let mut i = 0;
    while i < words.len() {
        let s = words[i].as_str();
        match s {
            "-W" | "--show" | "-S" | "--search" => {
                if action.is_some() {
                    return Err(error(DiagnosticKind::Syntax, 2, "Choose one query action"));
                }
                action = Some(s == "-S" || s == "--search");
            }
            "-f" | "--showformat" => {
                i += 1;
                let value = words.get(i).ok_or_else(|| {
                    error(DiagnosticKind::Syntax, 2, "Query format needs a value")
                })?;
                if fmt.replace(value.as_str()).is_some() {
                    return Err(error(DiagnosticKind::Syntax, 2, "Repeated query format"));
                }
            }
            s if s.starts_with("-f=") || s.starts_with("--showformat=") => {
                let value = s.split_once('=').unwrap().1;
                if fmt.replace(value).is_some() {
                    return Err(error(DiagnosticKind::Syntax, 2, "Repeated query format"));
                }
            }
            s if s.starts_with('-') => {
                return Err(error(
                    DiagnosticKind::Unsupported,
                    2,
                    "Unsupported query option; see the closed package Help",
                ));
            }
            _ => {
                if pattern.replace(s).is_some() {
                    return Err(error(
                        DiagnosticKind::Unsupported,
                        2,
                        "This query accepts at most one literal name or path",
                    ));
                }
            }
        }
        i += 1;
    }
    match action {
        Some(true) if fmt.is_none() => {
            let path = pattern.ok_or_else(|| {
                error(DiagnosticKind::Syntax, 2, "Search needs one absolute path")
            })?;
            if !exact_path(path) {
                return Err(error(
                    DiagnosticKind::Unsupported,
                    2,
                    "Search supports one canonical absolute regular-file path, without patterns",
                ));
            }
            Ok(Kind::Owner(path.into()))
        }
        Some(false) if !alias => Ok(Kind::Show {
            package: pattern.map(|p| exact_name(p, Family::Query)).transpose()?,
            format: format(fmt.unwrap_or("${binary:Package}\\t${Version}\\n"))?,
        }),
        _ => Err(error(
            DiagnosticKind::Unsupported,
            2,
            "Use dpkg-query -W with an optional format, or dpkg[-query] -S PATH",
        )),
    }
}
pub fn parse(input: &str) -> Result<Command> {
    if input.len() > MAX_COMMAND_BYTES {
        return Err(error(
            DiagnosticKind::Limit,
            2,
            "Package command exceeds2048 UTF8 bytes",
        ));
    }
    if input.chars().any(|c| c.is_control() && c != '\t') {
        return Err(error(
            DiagnosticKind::Unsupported,
            2,
            "Enter one literal command without NUL, physical newlines or control characters",
        ));
    }
    let words =
        shell::literal_command(input).map_err(|e| error(DiagnosticKind::Unsupported, 2, e))?;
    if words.is_empty() || words.len() > 64 {
        return Err(error(
            DiagnosticKind::Limit,
            2,
            "Enter one supported package command, at most64 words",
        ));
    }
    let family = match words[0].as_str() {
        "apt-get" | "apt-cache" => Family::Apt,
        "dpkg-query" => Family::Query,
        _ => Family::Dpkg,
    };
    let rest = &words[1..];
    let kind = match words[0].as_str() {
        "help" if rest.is_empty() => Kind::Help,
        "dpkg-query" => query(rest, false)?,
        "dpkg" => match rest {
            [action, name] if matches!(action.as_str(), "-r" | "--remove" | "-P" | "--purge") => {
                Kind::Remove {
                    package: exact_name(name, Family::Dpkg)?,
                    purge: matches!(action.as_str(), "-P" | "--purge"),
                }
            }
            [action, left, op, right] if action == "--compare-versions" => Kind::Compare {
                left: version(left, Family::Dpkg)?,
                relation: match op.as_str() {
                    "lt" => Relation::Lt,
                    "le" => Relation::Le,
                    "eq" => Relation::Eq,
                    "ne" => Relation::Ne,
                    "ge" => Relation::Ge,
                    "gt" => Relation::Gt,
                    _ => {
                        return Err(error(
                            DiagnosticKind::Unsupported,
                            2,
                            "Unsupported version comparison operator",
                        ));
                    }
                },
                right: version(right, Family::Dpkg)?,
            },
            _ => query(rest, true)?,
        },
        "apt-cache" => {
            let names: Vec<_> = rest
                .iter()
                .filter(|s| s.as_str() != "--important")
                .collect();
            if rest.iter().filter(|s| s.as_str() == "--important").count() != 1
                || names.len() != 2
                || names[0] != "depends"
            {
                return Err(error(
                    DiagnosticKind::Unsupported,
                    100,
                    "Only apt-cache --important depends NAME is modeled; policy/source/priority tables are not",
                ));
            }
            Kind::Depends(exact_name(names[1], Family::Apt)?)
        }
        "apt-get" => {
            let (mut yes, mut reinstall, mut strict) = (false, false, false);
            let mut operands = Vec::new();
            for s in rest {
                let slot = match s.as_str() {
                    "-y" | "--yes" | "--assume-yes" => Some(&mut yes),
                    "--reinstall" => Some(&mut reinstall),
                    "--error-on=any" => Some(&mut strict),
                    _ => None,
                };
                if let Some(slot) = slot {
                    if *slot {
                        return Err(error(DiagnosticKind::Syntax, 100, "Repeated APT option"));
                    }
                    *slot = true;
                } else if s.starts_with('-') {
                    return Err(error(
                        DiagnosticKind::Unsupported,
                        100,
                        "Unsupported APT option; no force, target release, architecture or configuration overrides",
                    ));
                } else {
                    operands.push(s);
                }
            }
            match operands.as_slice() {
                [action] if action.as_str() == "update" && strict && !yes && !reinstall => {
                    Kind::Update
                }
                [action, packages @ ..] if action.as_str() == "install" && yes && !strict => {
                    if packages.is_empty() || packages.len() > 8 {
                        return Err(error(
                            DiagnosticKind::Limit,
                            100,
                            "Install needs1..8 explicit requests",
                        ));
                    }
                    let requests = packages
                        .iter()
                        .map(|s| request(s))
                        .collect::<Result<Vec<_>>>()?;
                    if reinstall {
                        if requests.len() != 1 {
                            return Err(error(
                                DiagnosticKind::Unsupported,
                                100,
                                "Reinstall supports one exact current package only",
                            ));
                        }
                        let mut it = requests.into_iter();
                        let r = it.next().unwrap();
                        Kind::Reinstall {
                            package: r.name,
                            version: r.version.ok_or_else(|| {
                                error(
                                    DiagnosticKind::Unsupported,
                                    100,
                                    "Reinstall requires NAME=exact-current-version",
                                )
                            })?,
                        }
                    } else {
                        Kind::Install(requests)
                    }
                }
                _ => {
                    return Err(error(
                        DiagnosticKind::Unsupported,
                        100,
                        "Use strict update or noninteractive explicit install; other APT operations/prompts are not modeled",
                    ));
                }
            }
        }
        _ => {
            return Err(error(
                DiagnosticKind::Unsupported,
                2,
                "Unsupported virtual package command; no host program was run. See Help",
            ));
        }
    };
    Ok(Command { family, kind })
}
#[derive(Default)]
struct Text {
    value: String,
}
impl Text {
    fn push(&mut self, text: &str, family: Family) -> Result<()> {
        if self
            .value
            .len()
            .checked_add(text.len())
            .is_none_or(|n| n > MAX_OUTPUT_BYTES)
        {
            return Err(error(
                DiagnosticKind::Limit,
                family.failure(),
                "Package output exceeds32KiB; no result or mutation published",
            ));
        }
        self.value.push_str(text);
        Ok(())
    }
}
fn done(
    stdout: String,
    stderr: String,
    status: u8,
    changed: bool,
    family: Family,
) -> Result<Output> {
    if stdout
        .len()
        .checked_add(stderr.len())
        .is_none_or(|n| n > MAX_OUTPUT_BYTES)
    {
        return Err(error(
            DiagnosticKind::Limit,
            family.failure(),
            "Package combined output exceeds32KiB",
        ));
    }
    Ok(Output {
        stdout,
        stderr,
        status,
        changed,
    })
}
fn model(e: core::Error, family: Family) -> Diagnostic {
    error(
        match e.code {
            core::ErrorCode::Unsupported | core::ErrorCode::Conffile => DiagnosticKind::Unsupported,
            core::ErrorCode::Limit => DiagnosticKind::Limit,
            _ => DiagnosticKind::Model,
        },
        family.failure(),
        e.message,
    )
}

fn arch(a: core::Architecture) -> &'static str {
    match a {
        core::Architecture::Amd64 => "amd64",
        core::Architecture::Arm64 => "arm64",
        core::Architecture::All => "all",
    }
}
fn compatible(s: &State, package: &str) -> Result<()> {
    let c = s
        .cached_candidate(package)
        .map_err(|e| model(e, Family::Apt))?
        .ok_or_else(|| {
            error(
                DiagnosticKind::Model,
                100,
                "Package is absent from the current cached metadata",
            )
        })?;
    if c.architecture != core::Architecture::All && c.architecture != s.catalog().architecture {
        return Err(error(
            DiagnosticKind::Unsupported,
            100,
            "Foreign-architecture APT candidate selection is outside this closed single-architecture query",
        ));
    }
    if s.runtime()
        .records
        .get(package)
        .filter(|r| r.status == Status::Installed)
        .is_some_and(|r| {
            s.catalog().archives[&r.archive].version.compare(c.version) == Ordering::Greater
        })
    {
        return Err(error(
            DiagnosticKind::Unsupported,
            100,
            "Installed-newer/installed-only APT candidate fallback needs a separately modeled policy view",
        ));
    }
    Ok(())
}
fn mutate(
    definition: &Definition,
    state: &mut State,
    action: &Action,
    family: Family,
) -> Result<Output> {
    // Plan only sizes a derived response. Actual apply recomputes from current data;
    // no saved plan/inspection token grants publication or completion.
    let plan = state.plan(action).map_err(|e| model(e, family))?;
    let summary = format!(
        "DLMS simulation: {} package record(s), {} virtual file(s) changed; cached metadata {}; {} supplied artifact(s) made available. No host transaction.\n",
        plan.packages.len(),
        plan.changed_files.len(),
        plan.cached_after.as_deref().unwrap_or("absent"),
        plan.acquired_archives.len()
    );
    let output = done(summary, String::new(), 0, false, family)?;
    let report = work::apply(definition, state, action)
        .map_err(|e| error(DiagnosticKind::Model, family.failure(), e))?;
    let changed = !report.packages.is_empty()
        || !report.changed_files.is_empty()
        || report.cached_before != report.cached_after
        || !report.acquired_archives.is_empty();
    Ok(Output { changed, ..output })
}
/// Fresh goal/context admission for queries as well as mutations; never host execution.
pub fn execute(definition: &Definition, state: &mut State, input: &str) -> Result<Output> {
    if input.len() > MAX_COMMAND_BYTES {
        return Err(error(
            DiagnosticKind::Limit,
            2,
            "Package command exceeds2048 UTF8 bytes",
        ));
    }
    if !definition.same_assignment(state) {
        return Err(error(
            DiagnosticKind::Context,
            2,
            "Foreign/oversized logical package assignment; original retained",
        ));
    }
    let command = parse(input)?;
    let family = command.family;
    match command.kind {
        Kind::Help => done(HELP.into(), String::new(), 0, false, family),
        Kind::Show { package, format } => {
            let mut out = Text::default();
            let mut found = false;
            for (name, record) in &state.runtime().records {
                if package.as_ref().is_some_and(|p| p != name) {
                    continue;
                }
                found = true;
                let a = &state.catalog().archives[&record.archive];
                for p in &format {
                    let text = match p {
                        Piece::Text(text) => text.as_str(),
                        Piece::Field(Field::Package) => name,
                        Piece::Field(Field::Version) => a.version.as_str(),
                        Piece::Field(Field::Architecture) => arch(a.architecture),
                        Piece::Field(Field::Status) => {
                            if record.status == Status::Installed {
                                "installed"
                            } else {
                                "config-files"
                            }
                        }
                    };
                    out.push(text, family)?;
                }
            }
            if !found {
                return done(
                    String::new(),
                    "dpkg-query: no matching package record in the simulated database\n".into(),
                    1,
                    false,
                    family,
                );
            }
            done(out.value, String::new(), 0, false, family)
        }
        Kind::Owner(path) => {
            let owner = state.recorded_owner(&path).map_err(|e| model(e, family))?;
            match owner {
                Some(o) if o.status == Status::Installed => done(
                    format!("{}: {path}\n", o.package),
                    String::new(),
                    0,
                    false,
                    family,
                ),
                Some(_) => Err(error(
                    DiagnosticKind::Unsupported,
                    2,
                    "Retained configuration-only file-list search is not yet verified; query the explicit record status instead",
                )),
                None => done(
                    String::new(),
                    "dpkg-query: no recorded owner in the simulated database\n".into(),
                    1,
                    false,
                    family,
                ),
            }
        }
        Kind::Depends(package) => {
            compatible(state, &package)?;
            let c = state
                .cached_candidate(&package)
                .map_err(|e| model(e, family))?
                .unwrap();
            let mut out = Text::default();
            out.push(&format!("{package}\n"), family)?;
            for d in c.depends {
                if state
                    .cached_candidate(&d.name)
                    .map_err(|e| model(e, family))?
                    .is_none()
                {
                    return Err(error(
                        DiagnosticKind::Unsupported,
                        100,
                        "Missing/provider dependency display is outside the admitted metadata query",
                    ));
                }
                out.push(&format!("  Depends: {}\n", d.name), family)?;
            }
            done(out.value, String::new(), 0, false, family)
        }
        Kind::Update => mutate(definition, state, &Action::RefreshIndex {}, family),
        Kind::Install(packages) => {
            for p in &packages {
                compatible(state, &p.name)?;
                if let Some(v) = &p.version {
                    let c = state
                        .cached_candidate(&p.name)
                        .map_err(|e| model(e, family))?
                        .unwrap();
                    if v.as_str() != c.version.as_str() {
                        return Err(error(
                            DiagnosticKind::Unsupported,
                            100,
                            "Only the exact exposed version text is supported; no alternate-version lookup",
                        ));
                    }
                }
            }
            mutate(definition, state, &Action::Install { packages }, family)
        }
        Kind::Reinstall { package, version } => {
            let r = state
                .runtime()
                .records
                .get(&package)
                .filter(|r| r.status == Status::Installed)
                .ok_or_else(|| {
                    error(
                        DiagnosticKind::Model,
                        100,
                        "Reinstall needs a currently installed package",
                    )
                })?;
            let a = &state.catalog().archives[&r.archive];
            let c = state
                .cached_candidate(&package)
                .map_err(|e| model(e, family))?
                .ok_or_else(|| {
                    error(
                        DiagnosticKind::Unsupported,
                        100,
                        "Reinstall requires the exact installed artifact as the cached candidate",
                    )
                })?;
            if c.archive != r.archive || a.version.as_str() != version.as_str() {
                return Err(error(
                    DiagnosticKind::Unsupported,
                    100,
                    "Installed/candidate/version differ; unrestricted APT reinstall/upgrade is not modeled",
                ));
            }
            // Core Reinstall does not perform APT's global broken-dependency repair.
            if !state
                .dependency_issues()
                .map_err(|e| model(e, family))?
                .is_empty()
            {
                return Err(error(
                    DiagnosticKind::Unsupported,
                    100,
                    "Reinstall requires healthy Depends for ALL currently installed records; global broken-dependency repair is unsupported",
                ));
            }
            mutate(
                definition,
                state,
                &Action::Reinstall { name: package },
                family,
            )
        }
        Kind::Remove { package, purge } => {
            if !state.runtime().records.contains_key(&package) {
                return done(
                    format!(
                        "DLMS simulation: no record for {package}; no modeled files or records changed.\n"
                    ),
                    String::new(),
                    0,
                    false,
                    family,
                );
            }
            let action = if purge {
                Action::Purge { name: package }
            } else {
                Action::Remove { name: package }
            };
            mutate(definition, state, &action, family)
        }
        Kind::Compare {
            left,
            relation,
            right,
        } => {
            let order = left.compare(&right);
            let yes = match relation {
                Relation::Lt => order.is_lt(),
                Relation::Le => order.is_le(),
                Relation::Eq => order.is_eq(),
                Relation::Ne => !order.is_eq(),
                Relation::Ge => order.is_ge(),
                Relation::Gt => order.is_gt(),
            };
            done(String::new(), String::new(), u8::from(!yes), false, family)
        }
    }
}
/// Existing terminal bridge keeps statuses explicit without treating text as credit.
pub fn output_text(result: Result<Output>) -> std::result::Result<String, String> {
    match result {
        Ok(o) if o.status == 0 => Ok(format!("{}{}", o.stdout, o.stderr)),
        Ok(o) => Err(format!(
            "{}{}Simulator note: command exit status {}.",
            o.stdout, o.stderr, o.status
        )),
        Err(e) => Err(format!(
            "{:?}: {} (command exit status {}).",
            e.kind, e.message, e.status
        )),
    }
}
