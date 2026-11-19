//! Complete management documents with a disclosed ordered training-service method list.
use crate::{
    aaa_methods as aaa, acl_editor as editor, local_access_document as local, management_path,
    management_policy,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub incoming: management_policy::Policy,
    pub local: crate::local_access::Policy,
    pub authentication: aaa::Policy,
}
fn kw(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}
fn error(category: editor::Category, line: usize, message: &str) -> editor::Diagnostic {
    editor::Diagnostic {
        category,
        line: (line != 0).then_some(line),
        message: message.into(),
    }
}
fn methods(
    tokens: &[&str],
    services: &aaa::Services,
    line: usize,
) -> Result<aaa::Policy, editor::Diagnostic> {
    let mut at = 4;
    let mut methods = Vec::new();
    if tokens.len() == at {
        return Err(error(
            editor::Category::Syntax,
            line,
            "Add an explicit group NAME or local-case method after default.",
        ));
    }
    while at < tokens.len() {
        if kw(tokens[at], "local-case") {
            methods.push(aaa::Method::LocalCase);
            at += 1;
        } else if kw(tokens[at], "group") {
            let Some(name) = tokens.get(at + 1) else {
                return Err(error(
                    editor::Category::Syntax,
                    line,
                    "Write the supplied training service name after group.",
                ));
            };
            methods.push(aaa::Method::TrainingService {
                name: (*name).into(),
            });
            at += 2;
        } else {
            return Err(error(
                editor::Category::Unsupported,
                line,
                "This subset supports supplied group NAME and local-case methods. Other methods, bypass, protocol configuration and inferred defaults are not implemented.",
            ));
        }
        if methods.len() > aaa::MAX_METHODS {
            return Err(error(
                editor::Category::Limit,
                line,
                "Use at most four explicit methods.",
            ));
        }
    }
    let p = aaa::Policy {
        version: 1,
        methods,
    };
    p.validate(services)
        .map_err(|e| error(editor::Category::Unsupported, line, &e))?;
    Ok(p)
}
pub fn parse(
    network: &management_path::Network,
    supplied: &management_policy::Policy,
    services: &aaa::Services,
    text: &str,
) -> Result<Settings, editor::Diagnostic> {
    services
        .validate()
        .map_err(|e| error(editor::Category::Syntax, 0, &e))?;
    if text.len() > editor::MAX_TEXT {
        return Err(error(
            editor::Category::Limit,
            0,
            "The complete authentication document supports at most 64 KiB.",
        ));
    }
    let mut selected = None;
    let mut normalized = Vec::new();
    for (n, raw) in text.split_terminator('\n').enumerate() {
        let line = n + 1;
        if line > editor::MAX_LINES || raw.len() > editor::MAX_LINE {
            return Err(error(
                editor::Category::Limit,
                line,
                "Use at most 1024 lines and 512 bytes per line.",
            ));
        }
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if raw.chars().any(|c| c.is_control() && c != '\t') {
            return Err(error(
                editor::Category::Unsupported,
                line,
                "Use ordinary text, tabs and line endings; other control characters are unsupported.",
            ));
        }
        let trim = raw.trim();
        let tokens = trim.split_whitespace().collect::<Vec<_>>();
        if tokens.len() > editor::MAX_TOKENS {
            return Err(error(
                editor::Category::Limit,
                line,
                "A configuration line supports at most sixteen words.",
            ));
        }
        if !trim.starts_with('!')
            && tokens.len() >= 4
            && tokens[..4]
                .iter()
                .zip(["aaa", "authentication", "login", "default"])
                .all(|(a, b)| kw(a, b))
        {
            if selected.is_some() {
                return Err(error(
                    editor::Category::Unsupported,
                    line,
                    "State the default authentication list once in this complete document; incremental overrides are not modeled.",
                ));
            }
            selected = Some(methods(&tokens, services, line)?);
            // Keep AAA scope, required settings and source diagnostic lines under
            // the unchanged local document parser. This does not select its local
            // method for evaluation; the explicit parsed list above owns that.
            normalized.push("aaa authentication login default local-case");
        } else {
            normalized.push(raw);
        }
    }
    let settings = local::parse(network, supplied, &normalized.join("\n"))?;
    let authentication = selected.ok_or_else(|| {
        error(
            editor::Category::Syntax,
            0,
            "Supply one explicit default login authentication list.",
        )
    })?;
    Ok(Settings {
        incoming: settings.incoming,
        local: settings.local,
        authentication,
    })
}
