//! Source-bound transfer plans and independent practice. Retained operations,
//! not stored packet/file reports, produce each deterministic case's state.
use crate::{
    acl_editor, acl_routing as route, acl_workspace, network_transfer as core,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
pub const MAX_OPERATIONS: usize = 16;
pub const MAX_SESSION: usize = 2 * 1024 * 1024;
pub const MAX_ATTEMPT: usize = MAX_SESSION / 2 - 256;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileRef {
    pub host: String,
    pub path: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Copied {
        destination: FileRef,
        source: FileRef,
    },
    Unchanged {
        file: FileRef,
    },
    Ready {
        request: core::Request,
    },
    Blocked {
        label: String,
        flow: route::Flow,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,100>")]
    pub name: String,
    pub context: core::Context,
    pub initial: core::World,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,16>")]
    pub requirements: Vec<Requirement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub request: core::Request,
    pub documents: acl_editor::Drafts,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,4>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,12>")]
    pub editable_routers: Vec<String>,
    pub starter: core::Request,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Last {
    pub before: usize,
    pub operation: Draft,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub draft: Draft,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Draft,16>")]
    pub history: Vec<Draft>,
    pub last: Option<Last>,
    pub checked: bool,
}
fn initial_documents(c: &core::Context) -> Result<acl_editor::Drafts, String> {
    Ok(acl_workspace::Draft::new(&c.network)?.documents)
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id) || self.cases.is_empty() || self.cases.len() > 4 {
            return Err("Supply one to four named transfer cases.".into());
        }
        let first = &self.cases[0];
        let docs = initial_documents(&first.context)?;
        if self.editable_routers.len() > 12
            || self
                .editable_routers
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.editable_routers.len()
            || self
                .editable_routers
                .iter()
                .any(|id| !docs.routers.contains_key(id))
        {
            return Err(
                "Editable transfer policies must be unique supplied transit routers.".into(),
            );
        }
        let mut names = std::collections::BTreeSet::new();
        for case in &self.cases {
            case.initial
                .validate(&case.context)
                .map_err(|e| e.to_string())?;
            if case.name.is_empty()
                || case.name.len() > 100
                || !names.insert(&case.name)
                || case.context.client != first.context.client
                || case.context.network != first.context.network
                || case.context.servers.keys().ne(first.context.servers.keys())
                || case.requirements.len() > 16
                || !case
                    .requirements
                    .iter()
                    .any(|r| matches!(r, Requirement::Copied { .. }))
            {
                return Err("Each named case needs a positive file-copy goal and the same source-held endpoint/policy layout.".into());
            }
            for req in &case.requirements {
                let file = |r: &FileRef, exists: bool| -> Result<(), String> {
                    if !case.initial.files.contains_key(&r.host)
                        || !core::path(&r.path)
                        || exists && !case.initial.files[&r.host].contains_key(&r.path)
                    {
                        return Err(
                            "Requirement references an unknown initial virtual file or endpoint."
                                .into(),
                        );
                    }
                    Ok(())
                };
                match req {
                    Requirement::Copied {
                        source,
                        destination,
                    } => {
                        file(source, true)?;
                        file(destination, false)?;
                        if source == destination {
                            return Err("A copy must have a distinct destination.".into());
                        }
                    }
                    Requirement::Unchanged { file: r } => file(r, true)?,
                    Requirement::Ready { request } => {
                        request.validate(&case.context).map_err(|e| e.to_string())?
                    }
                    Requirement::Blocked { label, flow } => {
                        if label.is_empty() || label.len() > 160 {
                            return Err("Supply a short protected-flow label.".into());
                        }
                        flow.validate().map_err(|e| e.to_string())?;
                        if !matches!(
                            case.context
                                .network
                                .inspect(flow)
                                .map_err(|e| e.to_string())?
                                .baseline,
                            route::Baseline::Deliverable(_)
                        ) {
                            return Err(
                                "Protected traffic must be routable without its ACL.".into()
                            );
                        }
                    }
                }
            }
        }
        self.bound_draft(&self.new_draft()?)
    }
    pub fn new_draft(&self) -> Result<Draft, String> {
        let first = self.cases.first().ok_or("Missing initial transfer case")?;
        Ok(Draft {
            request: self.starter.clone(),
            documents: initial_documents(&first.context)?,
        })
    }
    pub fn bound_draft(&self, d: &Draft) -> Result<(), String> {
        let c = &self
            .cases
            .first()
            .ok_or("Missing transfer context")?
            .context;
        let initial = initial_documents(c)?;
        d.documents
            .validate(&c.network)
            .map_err(|e| e.to_string())?;
        if d.documents.routers.keys().ne(initial.routers.keys())
            || d.documents.routers.iter().any(|(id, text)| {
                !self.editable_routers.contains(id) && initial.routers.get(id) != Some(text)
            })
        {
            return Err(
                "Keep every supplied router and its protected policy document intact.".into(),
            );
        }
        if !c.servers.contains_key(&d.request.server)
            || d.request.local_path.len() > 120
            || d.request.remote_path.len() > 120
            || d.request
                .login
                .as_ref()
                .is_some_and(|s| s.len() > 32 || s.contains('\0'))
        {
            return Err("Transfer draft exceeds its source-held endpoint or field limits.".into());
        }
        // Incomplete/unsupported input and unfinished ACL syntax remain drafts;
        // only Transfer/Check attempts parse them. No invalid draft is executed.
        Ok(())
    }
    fn network(&self, case: &Case, d: &Draft) -> Result<route::Network, String> {
        self.bound_draft(d)?;
        acl_workspace::apply_documents(&case.context.network, &d.documents)
    }
    fn perform(&self, case: &Case, d: &Draft, w: &mut core::World) -> Result<core::Report, String> {
        let network = self.network(case, d)?;
        core::transfer(&case.context, &network, w, &d.request).map_err(|e| e.to_string())
    }
    pub fn replay(&self, case_index: usize, history: &[Draft]) -> Result<core::World, String> {
        if history.len() > MAX_OPERATIONS {
            return Err("Transfer history exceeds 16 distinct successful transfers.".into());
        }
        let case = self
            .cases
            .get(case_index)
            .ok_or("Unknown supplied transfer case")?;
        case.initial
            .validate(&case.context)
            .map_err(|e| e.to_string())?;
        let mut w = case.initial.clone();
        for op in history {
            self.bound_draft(op)?;
            // A plan can work in the first sample but fail in another. Keep that
            // sample's prior bytes and continue later repairs; grade final state.
            let result = self.perform(case, op, &mut w);
            if case_index == 0 && !result.as_ref().is_ok_and(|r| r.success) {
                return Err("Saved copy history is not reachable from this supplied source; original data retained.".into());
            }
        }
        Ok(w)
    }
    pub fn case_checks(&self, index: usize, a: &Attempt) -> Vec<(bool, String)> {
        let result = (|| -> Result<Vec<(bool, String)>, String> {
            self.validate()?;
            let case = self.cases.get(index).ok_or("Unknown transfer case")?;
            let w = self.replay(index, &a.history)?;
            let network = self.network(case, &a.draft)?;
            Ok(case.requirements.iter().map(|req|match req {
                Requirement::Copied{destination,source}=>{
                    let expected=&case.initial.files[&source.host][&source.path];
                    let actual=w.files[&destination.host].get(&destination.path);
                    (actual==Some(expected),format!("{}/{} must contain the complete original {}/{} bytes. {}",destination.host,destination.path,source.host,source.path,match actual{None=>"The destination file has not been created.",Some(v)if v!=expected=>"The destination exists but contains a different file or incomplete content.",_=>"The copied bytes match, including the final line ending."}))
                },
                Requirement::Unchanged{file}=>{
                    let same=w.files[&file.host].get(&file.path)==case.initial.files[&file.host].get(&file.path);
                    (same,format!("Preserve {}/{}: {}",file.host,file.path,if same{"original bytes retained."}else{"a transfer overwrote this protected file. Reset this attempt to restore its original source files, then copy into the requested new name."}))
                },
                Requirement::Ready{request}=>{
                    let mut probe=w.clone();let result=core::transfer(&case.context,&network,&mut probe,request);
                    let pass=result.as_ref().is_ok_and(|r|r.success);
                    (pass,if pass{format!("The current {} transfer path is ready for {}.",request.protocol.label(),request.server)}else{format!("Keep the transfer path ready: {}",match result{Ok(r)=>r.detail,Err(e)=>e.to_string()})})
                },
                Requirement::Blocked{label,flow}=>{
                    let observed=network.inspect(flow);
                    let pass=observed.is_ok_and(|i|matches!(i.baseline,route::Baseline::Deliverable(_))&&matches!(i.actual,route::Outcome::Denied{..}));
                    (pass,format!("{label}: {}",if pass{"the current ACL still blocks this routable flow."}else{"traffic is no longer blocked by the current ACL; keep the protection while repairing transfers."}))
                }
            }).collect())
        })();
        result.unwrap_or_else(|e| vec![(false, e)])
    }
}
impl Attempt {
    pub fn new(t: &Task) -> Result<Self, String> {
        Ok(Self {
            draft: t.new_draft()?,
            history: vec![],
            last: None,
            checked: false,
        })
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        if self.history.len() > MAX_OPERATIONS
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_ATTEMPT
        {
            return Err(
                "Saved transfer work exceeds its operation or byte budget; original data retained."
                    .into(),
            );
        }
        t.bound_draft(&self.draft)?;
        if self.history.len() > MAX_OPERATIONS {
            return Err("Saved transfer plan exceeds its operation bound.".into());
        }
        t.replay(0, &self.history)?;
        if let Some(last) = &self.last {
            t.bound_draft(&last.operation)?;
            if last.before > self.history.len()
                || self.history.len().saturating_sub(last.before) > 1
            {
                return Err("Saved transfer observation has an invalid plan position.".into());
            }
            let mut w = t.replay(0, &self.history[..last.before])?;
            let result = t.perform(&t.cases[0], &last.operation, &mut w);
            if last.before < self.history.len() {
                if self.history.get(last.before) != Some(&last.operation)
                    || !result.is_ok_and(|r| r.success)
                {
                    return Err("Saved transfer receipt does not match its actual source-reachable operation.".into());
                }
            } else if result.is_ok_and(|r| r.success)
                && self.history.last() != Some(&last.operation)
                && last.before < MAX_OPERATIONS
            {
                return Err(
                    "A successful state-changing transfer is missing from the saved plan.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn edit(&mut self, t: &Task, draft: Draft) -> Result<(), String> {
        t.bound_draft(&draft)?;
        if self.draft != draft {
            let mut next = self.clone();
            next.draft = draft;
            next.checked = false;
            next.last = None;
            if serde_json::to_vec(&next).map_err(|e| e.to_string())?.len() > MAX_ATTEMPT {
                return Err("This draft would exceed the attempt's saved-byte budget. Existing draft and file history remain unchanged.".into());
            }
            *self = next;
        }
        Ok(())
    }
    pub fn transfer(&mut self, t: &Task) -> Result<core::Report, String> {
        self.bound(t)?;
        self.checked = false;
        let before = self.history.len();
        let operation = self.draft.clone();
        let mut world = t.replay(0, &self.history)?;
        let result = t.perform(&t.cases[0], &operation, &mut world);
        let mut next = self.clone();
        next.last = Some(Last {
            before,
            operation: operation.clone(),
        });
        let at_limit = result.as_ref().is_ok_and(|r| r.success)
            && next.history.last() != Some(&operation)
            && next.history.len() == MAX_OPERATIONS;
        if result.as_ref().is_ok_and(|r| r.success)
            && next.history.last() != Some(&operation)
            && !at_limit
        {
            next.history.push(operation);
        }
        if serde_json::to_vec(&next).map_err(|e| e.to_string())?.len() > MAX_ATTEMPT {
            self.last = None;
            return Err("The saved-byte budget was reached. Your draft and prior file history remain; this transfer was not recorded.".into());
        }
        *self = next;
        if at_limit {
            return Err("This attempt retains at most 16 successful transfer operations. Your files and draft remain; reset only this example/assignment to start a new plan.".into());
        }
        result
    }
    pub fn last_report(&self, t: &Task) -> Option<Result<core::Report, String>> {
        let last = self.last.as_ref()?;
        Some((|| {
            let mut world = t.replay(
                0,
                self.history
                    .get(..last.before)
                    .ok_or("Invalid retained transfer position")?,
            )?;
            let r = t.perform(&t.cases[0], &last.operation, &mut world)?;
            if r.success
                && last.before == MAX_OPERATIONS
                && self.history.last() != Some(&last.operation)
            {
                return Err("The 16-operation retention limit was reached; no additional file operation was recorded.".into());
            }
            Ok(r)
        })())
    }
    pub fn check(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        self.checked = true;
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(t) {
            return vec![(false, e)];
        }
        let mut rows=vec![(self.checked,if self.checked{"Current transferred files and policy were checked."}else{"Select Check work to evaluate every supplied case; a transfer result alone is not completion."}.into())];
        for (i, c) in t.cases.iter().enumerate() {
            rows.extend(
                t.case_checks(i, self)
                    .into_iter()
                    .map(|(pass, why)| (pass, format!("{} — {why}", c.name))),
            );
        }
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub version: u32,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Draft,16>")]
    pub worked_example: Vec<Draft>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub version: u32,
    pub view: View,
    pub assignment: Attempt,
    pub practice: Attempt,
}
fn identifier(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !identifier(&self.id)
            || self.assignment.cases.len() < 2
            || self.worked_example.is_empty()
            || self.worked_example.len() > MAX_OPERATIONS
        {
            return Err("Supply a versioned transfer lesson, multiple assignment cases and a worked practice plan.".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.cases[0].context.client == self.example.cases[0].context.client {
            return Err(
                "Transfer practice must have a distinct client and source-held files.".into(),
            );
        }
        for task in [&self.assignment, &self.example] {
            let mut a = Attempt::new(task)?;
            a.checked = true;
            if a.checks(task).iter().all(|r| r.0) {
                return Err("The initial transfer assignment/example is already complete; retain meaningful independent work.".into());
            }
        }
        let mut example = Attempt::new(&self.example)?;
        for draft in &self.worked_example {
            example.edit(&self.example, draft.clone())?;
            let r = example.transfer(&self.example)?;
            if !r.success {
                return Err(format!("Worked transfer failed: {}", r.detail));
            }
        }
        example.check(&self.example)?;
        if !example.checks(&self.example).iter().all(|r| r.0) {
            return Err("The worked practice transfers do not meet their complete file/policy requirements.".into());
        }
        Ok(())
    }
    pub fn new_session(&self) -> Result<Session, String> {
        Ok(Session {
            id: self.id.clone(),
            version: self.version,
            view: View::Introduction,
            assignment: Attempt::new(&self.assignment)?,
            practice: Attempt::new(&self.example)?,
        })
    }
    pub fn context(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        if s.id != self.id
            || s.version != self.version
            || serde_json::to_vec(s).map_err(|e| e.to_string())?.len() > MAX_SESSION
        {
            return Err(
                "Transfer source/version or saved-size limit differs; original progress retained."
                    .into(),
            );
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !identifier(&self.id)
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_SESSION
        {
            return Err(
                "Unknown transfer source/version or saved work exceeds its size limit.".into(),
            );
        }
        for a in [&self.assignment, &self.practice] {
            if a.history.len() > MAX_OPERATIONS
                || a.last.as_ref().is_some_and(|last| {
                    last.before > a.history.len() || a.history.len() - last.before > 1
                })
            {
                return Err("Saved transfer plan or observation exceeds its bound.".into());
            }
            for draft in std::iter::once(&a.draft)
                .chain(a.history.iter())
                .chain(a.last.iter().map(|l| &l.operation))
            {
                if draft.request.server.len() > 32
                    || draft.request.local_path.len() > 120
                    || draft.request.remote_path.len() > 120
                    || draft.request.login.as_ref().is_some_and(|s| s.len() > 32)
                    || draft.documents.schema_version != 1
                    || draft.documents.routers.len() > 12
                    || draft
                        .documents
                        .routers
                        .values()
                        .any(|text| text.len() > acl_editor::MAX_TEXT)
                    || draft
                        .documents
                        .routers
                        .values()
                        .map(String::len)
                        .sum::<usize>()
                        > acl_editor::MAX_DRAFT_TOTAL
                {
                    return Err(
                        "Saved transfer fields or policy documents exceed their bounds.".into(),
                    );
                }
            }
        }
        Ok(())
    }
    pub fn task<'a>(&self, d: &'a Definition, practice: bool) -> &'a Task {
        if practice { &d.example } else { &d.assignment }
    }
    pub fn attempt(&self, practice: bool) -> &Attempt {
        if practice {
            &self.practice
        } else {
            &self.assignment
        }
    }
    pub fn attempt_mut(&mut self, practice: bool) -> &mut Attempt {
        if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        }
    }
    pub fn checks(&self, d: &Definition, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = d.context(self) {
            return vec![(false, e)];
        }
        self.attempt(practice).checks(self.task(d, practice))
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.context(self)?;
        *self.attempt_mut(practice) = Attempt::new(self.task(d, practice))?;
        Ok(())
    }
}
