//! Two explicit, borrowed source policies; never derives authority from saved work.
use super::{
    Requirement, Tree, command_location_policy as modes, whereis_deployment as deployment,
};
#[derive(Clone, Copy)]
pub enum Reference<'a> {
    Modes(&'a modes::Definition),
    Deployment(&'a deployment::Definition),
}
impl<'a> Reference<'a> {
    pub fn initial(self) -> &'a Tree {
        match self {
            Self::Modes(d) => &d.initial,
            Self::Deployment(d) => &d.initial,
        }
    }
    pub fn valid_for(self, t: &Tree) -> bool {
        match self {
            Self::Modes(d) => d.valid_for(t),
            Self::Deployment(d) => d.valid_for(t),
        }
    }
    pub fn bound(self, t: &Tree) -> Result<(), String> {
        match self {
            Self::Modes(d) => modes::bound(&d.initial, t),
            Self::Deployment(d) => d.bound(t),
        }
    }
    pub fn execute(self, t: &mut Tree, input: &str) -> Result<String, String> {
        match self {
            Self::Modes(d) => t.execute_in_context(&d.initial, input),
            Self::Deployment(d) => d.execute(t, input),
        }
    }
    pub fn same_family(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Modes(_), Self::Modes(_)) | (Self::Deployment(_), Self::Deployment(_))
        )
    }
}
pub fn has_reference(rows: &[Requirement]) -> bool {
    rows.iter().any(|r| {
        matches!(
            r,
            Requirement::CommandLocation { .. } | Requirement::WhereisDeployment { .. }
        )
    })
}
pub fn unique(rows: &[Requirement]) -> Result<Option<Reference<'_>>, String> {
    if rows.len() > 20 {
        return Err("Too many filesystem objective rows".into());
    }
    let mut found = None;
    for row in rows {
        let r = match row {
            Requirement::CommandLocation { definition } => {
                definition.validate()?;
                Some(Reference::Modes(definition))
            }
            Requirement::WhereisDeployment { definition } => {
                definition.validate()?;
                Some(Reference::Deployment(definition))
            }
            _ => None,
        };
        if let Some(r) = r {
            if found.is_some() {
                return Err("Supply exactly one discovery source family, without duplicate or mixed references".into());
            }
            found = Some(r);
        }
    }
    Ok(found)
}
pub fn for_current<'a>(
    rows: &'a [Requirement],
    current: &Tree,
) -> Result<Option<Reference<'a>>, String> {
    match (unique(rows)?, current.command_location.as_ref()) {
        (None, None) => Ok(None),
        (Some(r), Some(_)) => {
            r.bound(current)?;
            Ok(Some(r))
        }
        _ => Err(
            "Discovery source and saved operational context do not match; previous work retained"
                .into(),
        ),
    }
}
