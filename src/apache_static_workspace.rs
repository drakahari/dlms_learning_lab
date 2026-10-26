//! Bounded app context, editors and current/fresh static HTTP predicates.
use crate::apache_static::{
    self as web, Action, EntryKind, Inventory, Method, Outcome, Request, State,
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, SeqAccess, Visitor},
};
use std::{fmt, io::Write, marker::PhantomData};
pub const MAX_STATE: usize = 524_288;
pub const MAX_GOAL: usize = 786_432;
pub const MAX_DRAFT: usize = 196_608;
pub const MAX_REQUIREMENTS: usize = 32;
pub const PREFIX: &str = "apache-static ";
pub const HELP: &str = "Edit only supplied virtual Apache-style configuration/files; Save explicitly, then use typed Start/Reload/Stop and virtual requests. Check current serving and the saved next-start result separately. No Apache/Linux command, socket, host filesystem, TLS, proxy or full server lifecycle is executed.";
pub fn notice(s: &str) -> String {
    let mut n = s.len().min(512);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    s[..n].to_owned()
}
pub(crate) fn within<T: Serialize + ?Sized>(v: &T, cap: usize) -> bool {
    struct C {
        n: usize,
        cap: usize,
    }
    impl Write for C {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("app bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(C { n: 0, cap }, v).is_ok()
}
pub(crate) struct Text<const N: usize>(pub String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct S<const N: usize>;
        impl<const N: usize> Visitor<'_> for S<N> {
            type Value = Text<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {N} bytes")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if s.len() > N {
                    return Err(E::custom("Apache app text limit"));
                }
                Ok(Text(s.to_owned()))
            }
        }
        d.deserialize_str(S::<N>)
    }
}
pub(crate) fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    Ok(Text::<N>::deserialize(d)?.0)
}
pub(crate) fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct L<T, const N: usize>(PhantomData<T>);
    struct Next<T> {
        allowed: bool,
        t: PhantomData<T>,
    }
    impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Next<T> {
        type Value = T;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("Apache app collection limit"));
            }
            T::deserialize(d)
        }
    }
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for L<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(x) = a.next_element_seed(Next::<T> {
                allowed: v.len() < N,
                t: PhantomData,
            })? {
                v.push(x)
            }
            Ok(v)
        }
    }
    d.deserialize_seq(L::<T, N>(PhantomData))
}
pub(crate) fn option_text<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<Option<String>, D::Error> {
    Ok(Option::<Text<N>>::deserialize(d)?.map(|s| s.0))
}
fn canonical(p: &str) -> bool {
    p == "/"
        || (p.starts_with('/')
            && p.len() <= 160
            && !p.ends_with('/')
            && p[1..].split('/').count() <= 8
            && p[1..].split('/').all(|p| {
                !p.is_empty()
                    && p != "."
                    && p != ".."
                    && p.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
            }))
}
pub fn admitted(s: &State) -> bool {
    s.validate().is_ok() && within(s, MAX_STATE)
}
pub fn same_context(initial: &State, current: &State) -> bool {
    admitted(initial) && admitted(current) && initial.inventory == current.inventory
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseCause {
    File,
    DirectorySlash,
    InvalidHost,
    AccessDenied,
    MissingResource,
    NoIndex,
}
impl ResponseCause {
    fn matches(self, c: web::Cause) -> bool {
        matches!(
            (self, c),
            (Self::File, web::Cause::File)
                | (Self::DirectorySlash, web::Cause::DirectorySlash)
                | (Self::InvalidHost, web::Cause::InvalidHost)
                | (Self::AccessDenied, web::Cause::AccessDenied)
                | (Self::MissingResource, web::Cause::MissingResource)
                | (Self::NoIndex, web::Cause::NoIndex)
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpExpectation {
    pub status: u16,
    #[serde(default, deserialize_with = "option_text::<_,253>")]
    pub vhost: Option<String>,
    #[serde(default, deserialize_with = "option_text::<_,160>")]
    pub resource: Option<String>,
    #[serde(deserialize_with = "text::<_,4096>")]
    pub body: String,
    pub representation_bytes: Option<usize>,
    #[serde(default, deserialize_with = "option_text::<_,256>")]
    pub redirect_path: Option<String>,
    pub cause: Option<ResponseCause>,
}
impl HttpExpectation {
    fn valid(&self, r: &Request) -> bool {
        if self.body.len() > web::MAX_FILE
            || self.vhost.as_ref().is_some_and(|s| {
                s.is_empty() || s.len() > 253 || !s.is_ascii() || s.chars().any(char::is_whitespace)
            })
            || self.resource.as_ref().is_some_and(|p| !canonical(p))
            || self.redirect_path.as_ref().is_some_and(|p| {
                p.len() > 256
                    || !p.starts_with('/')
                    || !p.is_ascii()
                    || p.chars().any(char::is_control)
            })
        {
            return false;
        }
        match self.status {
            200 => {
                self.vhost.is_some()
                    && self.resource.is_some()
                    && self.redirect_path.is_none()
                    && match r.method {
                        Method::Get => self.representation_bytes == Some(self.body.len()),
                        Method::Head => {
                            self.body.is_empty()
                                && self
                                    .representation_bytes
                                    .is_some_and(|n| n <= web::MAX_FILE)
                        }
                    }
            }
            301 => {
                self.vhost.is_some()
                    && self.resource.is_some()
                    && self.redirect_path.is_some()
                    && self.body.is_empty()
                    && self.representation_bytes.is_none()
            }
            400 => {
                self.vhost.is_none()
                    && self.resource.is_none()
                    && self.body.is_empty()
                    && self.representation_bytes.is_none()
                    && self.redirect_path.is_none()
            }
            403 | 404 => {
                self.vhost.is_some()
                    && self.resource.is_some()
                    && self.body.is_empty()
                    && self.representation_bytes.is_none()
                    && self.redirect_path.is_none()
            }
            _ => false,
        }
    }
    fn matches(&self, o: &Outcome) -> bool {
        let Outcome::Http(r) = o else { return false };
        self.status == r.status.code()
            && match (&self.vhost, &r.vhost) {
                (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
                (None, None) => true,
                _ => false,
            }
            && self.resource == r.resource
            && self.body == r.body
            && self.representation_bytes == r.representation_bytes
            && self.redirect_path == r.redirect_path
            && self.cause.is_none_or(|c| c.matches(r.cause))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    CurrentHttp {
        request: Request,
        expected: HttpExpectation,
    },
    NextStartHttp {
        request: Request,
        expected: HttpExpectation,
    },
    ActiveMatchesSaved {},
    FilePreserved {
        #[serde(deserialize_with = "text::<_,160>")]
        path: String,
    },
    FileMatches {
        #[serde(deserialize_with = "text::<_,160>")]
        path: String,
        #[serde(deserialize_with = "text::<_,4096>")]
        contents: String,
    },
    SavedTextPreserved {},
}
pub fn requirements<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Requirement>, D::Error> {
    list::<D, Requirement, MAX_REQUIREMENTS>(d)
}
#[derive(Clone, Debug, Serialize)]
pub struct Definition {
    pub initial: State,
    pub requirements: Vec<Requirement>,
}
impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct R {
            initial: State,
            #[serde(deserialize_with = "requirements")]
            requirements: Vec<Requirement>,
        }
        let r =
            R::deserialize(d).map_err(|_| D::Error::custom("Invalid bounded Apache definition"))?;
        let v = Self {
            initial: r.initial,
            requirements: r.requirements,
        };
        v.validate().map_err(D::Error::custom)?;
        Ok(v)
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.requirements.is_empty()
            || self.requirements.len() > MAX_REQUIREMENTS
            || !admitted(&self.initial)
        {
            return Err("Apache definition initial/count admission".into());
        }
        let file = |p: &str| {
            p.len() <= 160 && self.initial.inventory.entries.get(p) == Some(&EntryKind::File)
        };
        let mut current = false;
        let mut fresh = false;
        for (i, r) in self.requirements.iter().enumerate() {
            let ok = match r {
                Requirement::CurrentHttp { request, expected }
                | Requirement::NextStartHttp { request, expected } => {
                    let current_rule = matches!(r, Requirement::CurrentHttp { .. });
                    if expected.status == 200 {
                        if current_rule {
                            current = true
                        } else {
                            fresh = true
                        }
                    }
                    request.shape().is_ok()
                        && request.server == self.initial.inventory.server
                        && expected.valid(request)
                }
                Requirement::FilePreserved { path } => file(path),
                Requirement::FileMatches { path, contents } => {
                    file(path) && contents.len() <= web::MAX_FILE
                }
                _ => true,
            };
            if !ok {
                return Err("Invalid bounded Apache request/outcome or supplied file".into());
            }
            if self.requirements[..i].contains(r) {
                return Err("Duplicate Apache outcome".into());
            }
        }
        if !current || !fresh || !within(self, MAX_GOAL) {
            return Err(
                "Apache goals need current and next-start positive200 serving within goal bounds"
                    .into(),
            );
        }
        Ok(())
    }
    pub fn same_assignment(&self, current: &State) -> bool {
        self.validate().is_ok() && same_context(&self.initial, current)
    }
    pub fn checks(&self, current: &State) -> Vec<(bool, String)> {
        if !self.same_assignment(current) {
            return vec![(
                false,
                "Apache current state/goal differs from its admitted inventory".into(),
            )];
        }
        let mut fresh = current.clone();
        let fresh_error = fresh
            .apply(&Action::Stop {})
            .and_then(|_| fresh.apply(&Action::Start {}))
            .err()
            .map(|e| notice(&e.message));
        let fresh_ok = fresh_error.is_none() && admitted(&fresh);
        let current_view = web::RequestView::new(current);
        let fresh_view = if fresh_ok {
            Some(web::RequestView::new(&fresh))
        } else {
            None
        };
        self.requirements.iter().enumerate().map(|(i,r)| {
            let (p,m) = match r {
                Requirement::CurrentHttp {request,expected} => probe(current_view.as_ref(),request,expected,&format!("Current request {}",i+1)),
                Requirement::NextStartHttp {request,expected} => if let Some(view) = &fresh_view {
                    probe(view.as_ref(),request,expected,&format!("Next-start test {}",i+1))
                } else { (false,format!("Next-start test {}: saved configuration could not start: {}",i+1,fresh_error.as_deref().unwrap_or("copy exceeds app admission"))) },
                Requirement::ActiveMatchesSaved {} => { let pass=current.active.as_ref().is_some_and(|a|a.source==current.saved); (pass,if pass {"Active configuration matches the saved document".into()}else{"Active configuration differs from saved text; inspect Save and Reload separately".into()}) },
                Requirement::FilePreserved {path} => { let pass=current.files.get(path)==self.initial.files.get(path); (pass,format!("Supplied file {path}: {}",if pass {"preserved"}else{"changed; restore the requested content"})) },
                Requirement::FileMatches {path,contents} => { let pass=current.files.get(path)==Some(contents); (pass,format!("Required current contents at {path}: {}",if pass {"match"}else{"differ"})) },
                Requirement::SavedTextPreserved {} => { let pass=current.saved==self.initial.saved; (pass,if pass {"Explicitly preserved saved text matches".into()}else{"Explicitly preserved saved text changed".into()}) },
            }; (p,notice(&m))
        }).collect()
    }
}
fn probe(
    view: Result<&web::RequestView<'_>, &web::Fault>,
    request: &Request,
    expected: &HttpExpectation,
    label: &str,
) -> (bool, String) {
    match view.map_err(Clone::clone).and_then(|v| v.query(request)) {
        Err(e) => (
            false,
            format!("{label}: virtual request rejected: {}", e.message),
        ),
        Ok(Outcome::NoListener) => (
            false,
            format!(
                "{label}: no active virtual listener for this server/port; expected HTTP{}",
                expected.status
            ),
        ),
        Ok(outcome) => {
            let pass = expected.matches(&outcome);
            let Outcome::Http(response) = outcome else {
                unreachable!()
            };
            (
                pass,
                if pass {
                    format!(
                        "{label}: HTTP{} and declared response fields match",
                        response.status.code()
                    )
                } else {
                    format!(
                        "{label}: HTTP{} from {} at {}; expected HTTP{} with the declared content/response values. Inspect active roots, access rules and current files",
                        response.status.code(),
                        response.vhost.as_deref().unwrap_or("no selected site"),
                        response.resource.as_deref().unwrap_or("no mapped resource"),
                        expected.status
                    )
                },
            )
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub version: u8,
    pub context: Inventory,
    pub config_text: String,
    pub selected_file: Option<String>,
    pub file_text: String,
    pub request_port: String,
    pub request_method: Method,
    pub request_hosts: Vec<String>,
    pub request_path: String,
    pub notice: Option<String>,
}
impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct R {
            version: u8,
            context: Inventory,
            #[serde(deserialize_with = "text::<_,16384>")]
            config_text: String,
            #[serde(default, deserialize_with = "option_text::<_,160>")]
            selected_file: Option<String>,
            #[serde(deserialize_with = "text::<_,4096>")]
            file_text: String,
            #[serde(deserialize_with = "text::<_,8>")]
            request_port: String,
            request_method: Method,
            #[serde(deserialize_with = "hosts")]
            request_hosts: Vec<String>,
            #[serde(deserialize_with = "text::<_,256>")]
            request_path: String,
            #[serde(default, deserialize_with = "option_text::<_,512>")]
            notice: Option<String>,
        }
        let r = R::deserialize(d).map_err(|_| D::Error::custom("Invalid bounded Apache draft"))?;
        let v = Self {
            version: r.version,
            context: r.context,
            config_text: r.config_text,
            selected_file: r.selected_file,
            file_text: r.file_text,
            request_port: r.request_port,
            request_method: r.request_method,
            request_hosts: r.request_hosts,
            request_path: r.request_path,
            notice: r.notice,
        };
        if !v.valid() {
            return Err(D::Error::custom("Invalid bounded Apache draft"));
        }
        Ok(v)
    }
}
fn hosts<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(list::<D, Text<260>, 2>(d)?
        .into_iter()
        .map(|s| s.0)
        .collect())
}
impl Draft {
    pub fn new(s: &State) -> Result<Self, String> {
        if !admitted(s) {
            return Err("Apache editor initial admission".into());
        }
        let d = Self {
            version: 1,
            context: s.inventory.clone(),
            config_text: s.saved.clone(),
            selected_file: None,
            file_text: String::new(),
            request_port: "8080".into(),
            request_method: Method::Get,
            request_hosts: vec![],
            request_path: "/".into(),
            notice: None,
        };
        if !d.valid() {
            return Err("Apache draft bound".into());
        }
        Ok(d)
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.context.validate().is_ok()
            && self.config_text.len() <= web::MAX_CONFIG
            && self.file_text.len() <= web::MAX_FILE
            && self.request_port.len() <= 8
            && self.request_hosts.len() <= 2
            && self.request_hosts.iter().all(|h| h.len() <= 260)
            && self.request_path.len() <= 256
            && self.notice.as_ref().is_none_or(|s| s.len() <= 512)
            && match &self.selected_file {
                Some(p) => p.len() <= 160 && self.context.entries.get(p) == Some(&EntryKind::File),
                None => self.file_text.is_empty(),
            }
            && within(self, MAX_DRAFT)
    }
    pub fn valid_for(&self, s: &State) -> bool {
        self.valid() && admitted(s) && self.context == s.inventory
    }
    /// Request fields and selection are observations, never pending configuration.
    pub fn matches(&self, s: &State) -> bool {
        self.valid_for(s)
            && self.config_text == s.saved
            && self
                .selected_file
                .as_ref()
                .is_none_or(|p| s.files.get(p) == Some(&self.file_text))
    }
    pub fn select_file(&mut self, s: &State, path: &str) -> Result<(), String> {
        if !self.valid_for(s)
            || path.len() > 160
            || s.inventory.entries.get(path) != Some(&EntryKind::File)
        {
            return Err("Choose one supplied Apache file".into());
        }
        if self.selected_file.as_deref() == Some(path) {
            return Ok(());
        }
        if self
            .selected_file
            .as_ref()
            .is_some_and(|p| p != path && s.files.get(p) != Some(&self.file_text))
        {
            return Err("Save or restore the pending file text before changing selection".into());
        }
        self.selected_file = Some(path.into());
        self.file_text = s.files[path].clone();
        Ok(())
    }
    pub fn query(&self, s: &State) -> Result<Outcome, String> {
        if !self.valid_for(s) {
            return Err("Apache request editor context differs".into());
        }
        if self.request_port.is_empty() || !self.request_port.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Request port needs a decimal number1..65535".into());
        }
        let port = self
            .request_port
            .parse::<u16>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| "Request port needs a decimal number1..65535".to_owned())?;
        let r = Request {
            server: s.inventory.server.clone(),
            port,
            method: self.request_method,
            hosts: self.request_hosts.clone(),
            path: self.request_path.clone(),
        };
        web::request(s, &r).map_err(|e| notice(&e.message))
    }
    pub fn save_config(&mut self, initial: &State, s: &mut State) -> Result<(), String> {
        if !self.valid_for(s) {
            return Err("Apache draft differs from context".into());
        }
        let a = Action::SaveConfig {
            text: self.config_text.clone(),
        };
        apply(initial, s, &a)?;
        self.notice = Some("Configuration saved; active serving has not been reloaded".into());
        Ok(())
    }
    pub fn save_file(&mut self, initial: &State, s: &mut State) -> Result<(), String> {
        if !self.valid_for(s) {
            return Err("Apache draft differs from context".into());
        }
        let path = self
            .selected_file
            .as_ref()
            .ok_or_else(|| "Select a supplied file".to_owned())?;
        let a = Action::WriteFile {
            path: path.clone(),
            text: self.file_text.clone(),
        };
        apply(initial, s, &a)?;
        self.notice =
            Some("Virtual file saved; later requests read current content without reload".into());
        Ok(())
    }
}
pub fn apply(initial: &State, current: &mut State, action: &Action) -> Result<bool, String> {
    action.validate().map_err(|e| notice(&e.message))?;
    if !same_context(initial, current) {
        return Err("Apache fixed inventory or admission differs; original retained".into());
    }
    let mut candidate = current.clone();
    let changed = candidate.apply(action).map_err(|e| notice(&e.message))?;
    if !admitted(&candidate) {
        return Err("Proposed Apache state exceeds app admission; original retained".into());
    }
    *current = candidate;
    Ok(changed)
}
pub fn execute(initial: &State, current: &mut State, input: &str) -> Result<String, String> {
    if input.len() > PREFIX.len() + web::MAX_ACTION_JSON {
        return Err("Typed Apache action exceeds raw bound".into());
    }
    let raw = input.strip_prefix(PREFIX).ok_or_else(|| HELP.to_owned())?;
    let action = Action::decode(raw.as_bytes()).map_err(|e| notice(&e.message))?;
    apply(initial, current, &action)?;
    Ok("Virtual Apache action applied; inspect current serving and saved next-start tests separately".into())
}
