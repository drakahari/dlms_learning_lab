//! Bounded client-name configuration practice; no host resolver or filesystem.
use crate::{client_names as model, resolver_search as search, script_teaching::View};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    net::Ipv4Addr,
};
pub const MAX_DEFINITION: usize = 524_288;
pub const MAX_SESSION: usize = 300_000;
pub const MAX_TICKS: usize = 64;
pub const PREFIX: &str = "NAME SOURCES\n";
pub type Files = BTreeMap<String, String>;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text_bound(s: &str) -> bool {
    s.len() <= model::MAX_CONFIG
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn document<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    struct Text;
    impl Visitor<'_> for Text {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most16KiB of configuration text")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > model::MAX_CONFIG {
                Err(E::custom("Name-source draft byte limit"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(Text)
}
fn files<'de, D: Deserializer<'de>>(d: D) -> Result<Files, D::Error> {
    struct Bundle;
    impl<'de> Visitor<'de> for Bundle {
        type Value = Files;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("one or two supplied bounded virtual configuration files")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Files, A::Error> {
            #[derive(Deserialize)]
            struct Name(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
            #[derive(Deserialize)]
            struct Body(#[serde(deserialize_with = "document")] String);
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("Virtual name-source file-count limit"))
                }
            }
            let mut out = Files::new();
            while out.len() < 2 {
                let Some(Name(name)) = a.next_key()? else {
                    return Ok(out);
                };
                if out.contains_key(&name) {
                    return Err(A::Error::custom("Duplicate virtual filename"));
                }
                let Body(body) = a.next_value()?;
                out.insert(name, body);
            }
            let _ = a.next_key::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_map(Bundle)
}
fn optional_files<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Files>, D::Error> {
    struct Maybe;
    impl<'de> Visitor<'de> for Maybe {
        type Value = Option<Files>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("optional checked virtual files")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            files(d).map(Some)
        }
    }
    d.deserialize_option(Maybe)
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N}bounded items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Vec<T>, A::Error> {
            let mut out = vec![];
            while out.len() < N {
                match a.next_element()? {
                    Some(x) => out.push(x),
                    None => return Ok(out),
                }
            }
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("Name-source exercise item limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn fits<T: Serialize>(value: &T, max: usize) -> bool {
    struct Count(usize, usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= self.1)
                .ok_or_else(|| std::io::Error::other("Name-source exercise byte limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0, max), value).is_ok()
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Sources,
    Search,
    SourcesWithSearch,
}
impl Mode {
    fn is_sources(&self) -> bool {
        *self == Self::Sources
    }
}
fn source_files(f: &Files) -> bool {
    f.len() == 2 && f.contains_key("hosts") && f.contains_key("nsswitch.conf")
}
fn search_files(f: &Files) -> bool {
    f.len() == 1 && f.contains_key("resolv.conf")
}
fn shape(f: &Files) -> bool {
    (source_files(f) || search_files(f)) && f.values().all(|s| text_bound(s))
}
fn hostname<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Host(#[serde(deserialize_with = "crate::cron::text::<_,253>")] String);
    Option::<Host>::deserialize(d).map(|h| h.map(|h| h.0))
}
fn parse(f: &Files) -> Result<(model::Switch, model::Hosts), String> {
    if !source_files(f) || !shape(f) {
        return Err("Keep the two supplied virtual files: hosts and nsswitch.conf".into());
    }
    Ok((
        model::Switch::parse(&f["nsswitch.conf"]).map_err(|e| e.to_string())?,
        model::Hosts::parse(&f["hosts"]).map_err(|e| e.to_string())?,
    ))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[serde(deserialize_with = "crate::cron::text::<_,254>")]
    pub query: String,
    pub address: Option<Ipv4Addr>,
    pub status: Option<model::Status>,
    pub via: Option<model::Service>,
}
impl Probe {
    fn validate(&self, mode: Mode) -> bool {
        model::name(if mode != Mode::Sources {
            self.query.strip_suffix('.').unwrap_or(&self.query)
        } else {
            &self.query
        }) && self.query == self.query.to_ascii_lowercase()
            && self
                .status
                .is_none_or(|status| self.address.is_some() == (status == model::Status::Success))
    }
    fn check(&self, r: &model::Lookup) -> (bool, String) {
        let via = r.steps.last().map(|s| s.service);
        let ok = r.address == self.address
            && self.status.is_none_or(|status| r.status == status)
            && self.via.is_none_or(|s| Some(s) == via);
        let expected = self.address.map(|ip| ip.to_string()).unwrap_or_else(|| {
            self.status
                .map(|status| format!("Unresolved ({})", status.label()))
                .unwrap_or_else(|| "Unresolved".into())
        });
        let actual = r
            .address
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| format!("Unresolved ({})", r.status.label()));
        let source = via.map(model::Service::label).unwrap_or("none");
        let why = if ok {
            "The observed answer and source meet this requirement."
        } else if r.steps.last().is_some_and(|s| {
            s.action == model::Action::Return && s.status != model::Status::Success
        }) {
            "This source returned without an answer and stopped the search. Check its bracket action before the next source."
        } else if r.address.is_some() {
            "A successful source normally stops the search. Check source order and the matching hosts name or alias."
        } else {
            "No consulted source supplied the required answer. Check the exact hosts name/alias and whether DNS was reached."
        };
        (
            ok,
            format!(
                "{}: expected {expected}{}; actual {actual} via {source}. {why}",
                self.query,
                self.via
                    .map(|s| format!(" via {}", s.label()))
                    .unwrap_or_default()
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub context: model::Context,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "hostname"
    )]
    pub hostname: Option<String>,
    #[serde(deserialize_with = "list::<_,Probe,12>")]
    pub probes: Vec<Probe>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(default, skip_serializing_if = "Mode::is_sources")]
    pub mode: Mode,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "hostname"
    )]
    pub resolver: Option<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1800>")]
    pub instructions: String,
    #[serde(deserialize_with = "files")]
    pub starter: Files,
    #[serde(deserialize_with = "list::<_,Case,4>")]
    pub cases: Vec<Case>,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !key(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1800
            || self
                .instructions
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            || !shape(&self.starter)
            || match self.mode {
                Mode::Sources | Mode::SourcesWithSearch => !source_files(&self.starter),
                Mode::Search => !search_files(&self.starter),
            }
            || !(2..=4).contains(&self.cases.len())
            || !fits(self, MAX_DEFINITION / 2)
        {
            return Err("Invalid bounded client-name task".into());
        }
        if (self.mode == Mode::SourcesWithSearch) != self.resolver.is_some() {
            return Err("Only combined source/search tasks supply a fixed resolver excerpt".into());
        }
        if self.mode != Mode::Search {
            parse(&self.starter)
                .map_err(|e| format!("Starter must use supported unfinished configuration: {e}"))?;
        }
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            c.context.validate().map_err(|e| e.to_string())?;
            match self.mode {
                Mode::Sources if c.hostname.is_some() => {
                    return Err(
                        "Local-name samples do not take a separate resolver hostname".into(),
                    );
                }
                Mode::Search => {
                    let host = c
                        .hostname
                        .as_deref()
                        .filter(|h| model::name(h))
                        .ok_or("Search samples need a bounded supplied hostname")?;
                    if !c.context.files_available
                        || !matches!(
                            c.context.dns.status,
                            model::Status::Success | model::Status::NotFound
                        )
                    {
                        return Err("Search samples support positive/missing DNS snapshots only; local-source availability is not varied here".into());
                    }
                    search::Settings::parse(&self.starter["resolv.conf"], host).map_err(|e| {
                        format!("Starter needs supported search configuration: {e}")
                    })?;
                }
                Mode::SourcesWithSearch => {
                    let host = c
                        .hostname
                        .as_deref()
                        .filter(|h| model::name(h))
                        .ok_or("Combined samples need a supported supplied hostname")?;
                    if !matches!(
                        c.context.dns.status,
                        model::Status::Success | model::Status::NotFound
                    ) {
                        return Err(
                            "Combined search supports positive/missing DNS snapshots only".into(),
                        );
                    }
                    let raw = self.resolver.as_deref().unwrap();
                    if raw.len() > 253 {
                        return Err("Supplied resolver excerpt exceeds253bytes".into());
                    }
                    search::Settings::parse(raw, host).map_err(|e| e.to_string())?;
                }
                _ => {}
            }
            if !key(&c.id)
                || !ids.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
                || c.probes.is_empty()
                || c.probes.len() > 12
            {
                return Err(
                    "Name-source samples need bounded unique identities and useful lookups".into(),
                );
            }
            let mut names = BTreeSet::new();
            for p in &c.probes {
                if !p.validate(self.mode) || !names.insert(&p.query) {
                    return Err("Each sample needs unique supported queries and consistent expected address/status".into());
                }
            }
        }
        Ok(())
    }
    pub fn run(&self, files: &Files) -> Result<Vec<CaseResult>, String> {
        let mut run = Run::new(self, files)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                return Ok(run.results);
            }
        }
        Err("Name-source checks exceeded their bounded phases".into())
    }
    /// Read-only utility execution against one supplied sample and current drafts.
    /// It neither runs grading nor creates tested-work credit.
    pub fn diagnose(
        &self,
        files: &Files,
        sample: usize,
        command: &str,
    ) -> Result<crate::client_diagnostic::Output, String> {
        self.validate()?;
        if self.mode != Mode::SourcesWithSearch {
            return Err("Diagnostic commands belong to the combined source/search activity".into());
        }
        let c = self
            .cases
            .get(sample)
            .ok_or("Choose a supplied DNS sample")?;
        let (switch, hosts) = parse(files)?;
        let settings = search::Settings::parse(
            self.resolver.as_deref().unwrap(),
            c.hostname.as_deref().unwrap(),
        )
        .map_err(|e| e.to_string())?;
        crate::client_diagnostic::getent(&switch, &hosts, &c.context, &settings, command)
            .map_err(|e| format!("{:?}: {e}", e.kind))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub label: String,
    pub diagnostic: Option<String>,
    pub checks: Vec<(bool, String)>,
    pub lookups: Vec<model::Lookup>,
    pub searches: Vec<SearchObservation>,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        self.diagnostic.is_none() && !self.checks.is_empty() && self.checks.iter().all(|r| r.0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchObservation {
    pub query: String,
    pub observed: search::Lookup,
}
pub struct Run {
    task: Task,
    files: Files,
    parsed: Option<Result<(model::Switch, model::Hosts), String>>,
    index: usize,
    probe: usize,
    pending: Option<CaseResult>,
    results: Vec<CaseResult>,
    cancelled: bool,
}
impl Run {
    pub fn new(task: &Task, files: &Files) -> Result<Self, String> {
        task.validate()?;
        if !shape(files) || !files.keys().eq(task.starter.keys()) {
            return Err("Retain this task’s supplied bounded virtual file set".into());
        }
        Ok(Self {
            task: task.clone(),
            files: files.clone(),
            parsed: (task.mode != Mode::Search).then(|| parse(files)),
            index: 0,
            probe: 0,
            pending: None,
            results: vec![],
            cancelled: false,
        })
    }
    pub fn done(&self) -> bool {
        !self.cancelled && self.index == self.task.cases.len()
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.pending = None;
        self.results.clear();
    }
    pub fn results(&self) -> Option<&[CaseResult]> {
        self.done().then_some(&self.results)
    }
    pub fn tick(&mut self) -> bool {
        if self.cancelled {
            return false;
        }
        if self.done() {
            return true;
        }
        let c = &self.task.cases[self.index];
        if self.pending.is_none() {
            self.pending = Some(CaseResult {
                id: c.id.clone(),
                label: c.label.clone(),
                diagnostic: None,
                checks: vec![],
                lookups: vec![],
                searches: vec![],
            });
            return false;
        }
        let r = self.pending.as_mut().unwrap();
        if let Some(Err(reason)) = &self.parsed {
            r.diagnostic = Some(reason.clone());
            r.checks.push((false, reason.clone()));
            self.probe = c.probes.len();
        } else if self.probe < c.probes.len() {
            let p = &c.probes[self.probe];
            let result = match self.task.mode {
                Mode::Sources => {
                    let (s, h) = self.parsed.as_ref().unwrap().as_ref().unwrap();
                    model::lookup(s, h, &c.context, &p.query)
                        .map(|observed| (observed, None))
                        .map_err(|e| e.to_string())
                }
                Mode::SourcesWithSearch => {
                    let (s, h) = self.parsed.as_ref().unwrap().as_ref().unwrap();
                    search::Settings::parse(
                        self.task.resolver.as_deref().unwrap(),
                        c.hostname.as_deref().unwrap(),
                    )
                    .and_then(|settings| {
                        crate::client_diagnostic::resolve(s, h, &c.context, &settings, &p.query)
                    })
                    .map(|r| {
                        let search = (!r.dns_attempts.is_empty()).then(|| SearchObservation {
                            query: p.query.clone(),
                            observed: search::Lookup {
                                address: r.dns_attempts.last().and_then(|a| a.address),
                                attempts: r.dns_attempts,
                            },
                        });
                        (
                            model::Lookup {
                                name: r.name,
                                address: r.address,
                                status: r.status,
                                steps: r.steps,
                            },
                            search,
                        )
                    })
                    .map_err(|e| e.to_string())
                }
                Mode::Search => search::Settings::parse(
                    &self.files["resolv.conf"],
                    c.hostname.as_deref().unwrap(),
                )
                .and_then(|settings| settings.lookup(&p.query, &c.context.dns))
                .map(|observed| {
                    let status = if observed.address.is_some() {
                        model::Status::Success
                    } else {
                        model::Status::NotFound
                    };
                    let lookup = model::Lookup {
                        name: p.query.clone(),
                        address: observed.address,
                        status,
                        steps: vec![model::Step {
                            service: model::Service::Dns,
                            status,
                            action: if status == model::Status::Success {
                                model::Action::Return
                            } else {
                                model::Action::Continue
                            },
                            answer: observed.address,
                        }],
                    };
                    (
                        lookup,
                        Some(SearchObservation {
                            query: p.query.clone(),
                            observed,
                        }),
                    )
                })
                .map_err(|e| e.to_string()),
            };
            match result {
                Ok((observed, search)) => {
                    let mut check = p.check(&observed);
                    if self.task.mode == Mode::Search && !check.0 {
                        check.1 = format!(
                            "{}: expected {}; actual {}. Search stops at its first answer. Compare the candidate names with the supplied DNS data; check suffix order, ndots and an absolute final dot.",
                            p.query,
                            p.address
                                .map(|a| a.to_string())
                                .unwrap_or_else(|| "Unresolved".into()),
                            observed
                                .address
                                .map(|a| a.to_string())
                                .unwrap_or_else(|| "Unresolved".into())
                        );
                    }
                    r.checks.push(check);
                    r.lookups.push(observed);
                    if let Some(search) = search {
                        r.searches.push(search);
                    }
                }
                Err(e) => {
                    r.diagnostic = Some(e.clone());
                    r.checks.push((false, e));
                }
            }
            self.probe += 1;
            return false;
        }
        self.results.push(self.pending.take().unwrap());
        self.index += 1;
        self.probe = 0;
        self.done()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "files")]
    pub example_worked: Files,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.mode != self.example.mode {
            return Err(
                "Worked and independent tasks must use the same configuration workspace".into(),
            );
        }
        if self.initial.id == self.example.id
            || !shape(&self.example_worked)
            || !fits(self, MAX_DEFINITION)
        {
            return Err("Separate bounded practice and assignment sources are required".into());
        }
        let queries = |t: &Task| {
            t.cases
                .iter()
                .flat_map(|c| c.probes.iter().map(|p| p.query.clone()))
                .collect::<BTreeSet<_>>()
        };
        if queries(&self.initial)
            .intersection(&queries(&self.example))
            .any(|q| q != "localhost")
        {
            return Err(
                "Worked queries must be separate from independent assignment queries".into(),
            );
        }
        for t in [&self.initial, &self.example] {
            if t.run(&t.starter)?.iter().all(CaseResult::passed) {
                return Err(
                    "Each name-source starter must retain meaningful unfinished work".into(),
                );
            }
        }
        if !self
            .example
            .run(&self.example_worked)?
            .iter()
            .all(CaseResult::passed)
        {
            return Err("Worked example does not reach its name-resolution requirements".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id
            || !s.assignment.matches(&self.initial)
            || !s.practice.matches(&self.example)
        {
            return Err("Name-source work belongs to another task; retained".into());
        }
        Ok(())
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok() && s.assignment.passed(&self.initial)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if s.assignment.tested.as_ref() != Some(&s.assignment.draft) {
            return vec![(false,"Not checked · run the current name-source configuration against the supplied samples.".into())];
        }
        match self.initial.run(&s.assignment.draft) {
            Err(e) => vec![(false, e)],
            Ok(rows) => rows
                .into_iter()
                .flat_map(|r| {
                    r.checks
                        .into_iter()
                        .map(move |(ok, t)| (ok, format!("{} · {t}", r.label)))
                })
                .collect(),
        }
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION {
            return Err("Name-source lesson exceeds512KiB".into());
        }
        let d: Self =
            serde_json::from_str(raw).map_err(|e| format!("Invalid name-source lesson: {e}"))?;
        d.validate()?;
        Ok(d)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "files")]
    pub draft: Files,
    #[serde(deserialize_with = "optional_files")]
    pub tested: Option<Files>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub selected: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<DiagnosticDraft>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticDraft {
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub input: String,
    pub sample: usize,
}
impl Scope {
    fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            tested: None,
            selected: t.starter.keys().next().unwrap().clone(),
            diagnostic: (t.mode == Mode::SourcesWithSearch).then(|| DiagnosticDraft {
                input: String::new(),
                sample: 0,
            }),
        }
    }
    fn matches(&self, t: &Task) -> bool {
        self.draft.keys().eq(t.starter.keys())
            && (self.diagnostic.is_some() == (t.mode == Mode::SourcesWithSearch))
            && self
                .diagnostic
                .as_ref()
                .is_none_or(|d| d.sample < t.cases.len())
    }
    pub fn select(&mut self, name: &str) -> Result<(), String> {
        if !self.draft.contains_key(name) {
            return Err("Choose a supplied virtual name-source file".into());
        }
        self.selected = name.into();
        Ok(())
    }
    pub fn edit(&mut self, name: &str, text: &str) -> Result<(), String> {
        if !text_bound(text) {
            return Err(
                "Use at most16KiB without control characters; previous draft retained".into(),
            );
        }
        let old = self
            .draft
            .get_mut(name)
            .ok_or("Only supplied virtual filenames can be edited")?;
        if old != text {
            *old = text.into();
            self.tested = None;
        }
        Ok(())
    }
    pub fn replace(&mut self, files: &Files) -> Result<(), String> {
        if !shape(files) || !files.keys().eq(self.draft.keys()) {
            return Err("Keep the supplied virtual filenames and bounded name-source text".into());
        }
        if files != &self.draft {
            self.draft = files.clone();
            self.tested = None;
        }
        Ok(())
    }
    pub fn begin(&mut self, t: &Task) -> Result<Run, String> {
        self.tested = None;
        Run::new(t, &self.draft)
    }
    pub fn finish(&mut self, t: &Task, run: &Run) -> Result<(), String> {
        if !run.done() || run.files != self.draft || run.task != *t {
            return Err(
                "Stopped, unfinished or changed name-source checks cannot award current credit"
                    .into(),
            );
        }
        self.tested = Some(self.draft.clone());
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<Vec<CaseResult>, String> {
        let mut run = self.begin(t)?;
        for _ in 0..MAX_TICKS {
            if run.tick() {
                self.finish(t, &run)?;
                return Ok(run.results);
            }
        }
        Err("Name-source checks reached their bounded phase limit".into())
    }
    pub fn passed(&self, t: &Task) -> bool {
        self.tested.as_ref() == Some(&self.draft)
            && t.run(&self.draft)
                .is_ok_and(|rows| !rows.is_empty() && rows.iter().all(CaseResult::passed))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.origin)
            || !fits(self, MAX_SESSION)
            || [&self.assignment, &self.practice].iter().any(|s| {
                !shape(&s.draft)
                    || s.diagnostic.as_ref().is_some_and(|d| {
                        d.input.len() > 512
                            || d.sample >= 4
                            || d.input.chars().any(|c| c.is_control() && c != '\t')
                    })
                    || !s.draft.contains_key(&s.selected)
                    || s.tested
                        .as_ref()
                        .is_some_and(|t| !shape(t) || !t.keys().eq(s.draft.keys()))
            })
        {
            return Err("Invalid version, selection or bounds in name-source progress".into());
        }
        Ok(())
    }
    pub fn from_json(text: &str, d: &Definition) -> Result<Self, String> {
        if text.len() > MAX_SESSION {
            return Err("Name-source progress exceeds its byte limit".into());
        }
        let s: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid name-source progress: {e}"))?;
        d.bound(&s)?;
        Ok(s)
    }
    /// Internal data-only reference adapter; not a shell command.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native name-source editor; this is internal reference data")?;
        if text.len() > MAX_SESSION {
            return Err("Reference bundle exceeds its byte limit".into());
        }
        #[derive(Deserialize)]
        struct Bundle(#[serde(deserialize_with = "files")] Files);
        let Bundle(files) = serde_json::from_str(text)
            .map_err(|e| format!("Invalid reference name-sources: {e}"))?;
        self.assignment.replace(&files)?;
        let rows = self.assignment.run(&d.initial)?;
        Ok(format!(
            "Simulated name lookups checked: {}/{} cases passed",
            rows.iter().filter(|r| r.passed()).count(),
            rows.len()
        ))
    }
}

pub const HELP: &str = "Edit only virtual hosts and the hosts database excerpt of nsswitch.conf, then Run lookups. Files and DNS sources, default return-on-success/continue-on-other-status and supported explicit actions are evaluated against finite supplied DNS snapshots. Trace actual source/status/action separately from task requirements. Only single IPv4 answers and unique bounded names/aliases are supported. No OS files, NSS module, DNS query, command or host resolver executes. Search/ndots, cache, IPv6/multiple answers, other modules/databases, negated actions, merge and SUCCESS=continue are outside this subset and refuse.";
pub const SEARCH_HELP: &str = "Edit only the search/domain/options-ndots excerpt of virtual resolv.conf, then Run lookups. The sample hostname supplies its default domain; DNS answers are finite responding/missing A snapshots, not real network queries. At most three suffixes and four candidates, ndots0–15, literal root suffix and absolute trailing-dot names. The first positive candidate stops the search. Nameserver routing, other options, unavailable/retry sequencing, cache, CNAME, IPv6, resolver flags, environment and real OS files/resolvers are outside this subset and refuse.";
pub fn help(mode: Mode) -> &'static str {
    match mode {
        Mode::Sources => HELP,
        Mode::Search => SEARCH_HELP,
        Mode::SourcesWithSearch => COMBINED_HELP,
    }
}
pub const COMBINED_HELP: &str = "Edit only the virtual hosts and hosts-only nsswitch.conf excerpts. The supplied resolver search settings stay fixed. getent ahostsv4 NAME reads the current drafts for one sample; Run lookups checks every task requirement across all samples. Files sees the original name once; only the DNS backend expands its search candidates. A successful single IPv4 answer has STREAM/DGRAM/RAW rows and status0; a missing key has no output and status2. Only one key, ordinary ASCII names or canonical dotted-decimal IPv4 are supported. Other getent databases/options, enumeration, legacy numeric forms, multiple addresses, IPv6, CNAME, retry/outage search sequencing, caches, systemd-resolved and OS/network access are outside this subset. Diagnostic traces are simulator observations, not additional real getent output.";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::ClientNames { simulation },
        crate::engine::Goal::ClientNames { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Name-source source/state mismatch".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Name-source source identity, task and supplied drafts must match".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::ClientNames { simulation } = &a.state else {
        return Err("Name-source work belongs to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible name-source attempt metadata; previous work retained".into());
    }
    d.bound(simulation)
}
