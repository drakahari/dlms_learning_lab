//! Literal command adapter over the finite in-memory print-queue world.
//! Never interprets shell programs or opens a host path/socket/process.
use super::*;

pub const HELP: &str = "Print-queue subset: lpstat [-l] [-W completed|not-completed] [-t|-d|-p [PRINTER]|-a [PRINTER]|-o [PRINTER]]; lp [-d PRINTER] [-n COPIES] [-H hold|immediate] [--] FILE; lp -i REQUEST -H hold|resume; lpr [-P PRINTER] [-# COPIES] [-q] FILE; lpq [-l] [-P PRINTER]; lprm NUMERIC_REQUEST; cancel REQUEST; cupsenable PRINTER; cupsdisable [-r REASON] PRINTER; cupsaccept PRINTER; cupsreject [-r REASON] PRINTER; lpoptions -d PRINTER; ls [/work]; cat FILE. Literal quotes/escapes and separate option words work; lpr/lpq also accept -PNAME and lpr -#N. A hash inside an ordinary word is literal; shell comments are unsupported. Use -W before -o; one destination/file/request per command. All supplied jobs share an authorized fictional operator. Local default only; no LPDEST/PRINTER environment, server defaults, instances, classes, device/driver configuration, media/options, lprm implicit/all-job forms, lpq repeat intervals, LPD transport, real authentication, network or host I/O. At most64 historical jobs,20 copies per request and32KiB output. The finite scheduler completes pending jobs when enabled/submitted/released; individual holds are separate. Times/devices are explicitly simulated; spool size rounds supplied unfiltered text to KiB, not actual filter output. No shell expansion/operators or arbitrary files.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub status: u8,
}
pub(super) fn unsupported() -> Error {
    Error{kind:ErrorKind::Unsupported,message:"Command/options are outside the disclosed print-queue subset. No host command or printer was used.".into()}
}
pub(super) fn syntax(message: &str) -> Error {
    Error {
        kind: ErrorKind::Syntax,
        message: message.into(),
    }
}
pub(super) fn literal_path(raw: &str) -> Result<String> {
    let path = if raw.starts_with('/') {
        raw.to_string()
    } else {
        format!("/work/{raw}")
    };
    if !file_name(&path) {
        return Err(syntax(
            "Use a supplied virtual document name or its full path; no traversal or expansion.",
        ));
    }
    Ok(path)
}
fn listing(
    w: &World,
    f: &Facts,
    kind: &str,
    selected: Option<&str>,
    completed: bool,
    long: bool,
) -> Result<String> {
    if let Some(name) = selected {
        w.printer(name)?;
    }
    let include = |name: &str| selected.is_none_or(|s| s.eq_ignore_ascii_case(name));
    let mut text = String::new();
    match kind {
        "default" => text.push_str(
            &w.local_default
                .as_ref()
                .map(|n| format!("system default destination: {}\n", n.0))
                .unwrap_or("no system default destination\n".into()),
        ),
        "printers" => {
            for p in w.printers.iter().filter(|p| include(&p.name.0)) {
                text.push_str(&format!(
                    "printer {} {} since simulated snapshot\n",
                    p.name.0,
                    if p.enabled {
                        "is idle. enabled"
                    } else {
                        "disabled"
                    }
                ));
                if !p.enabled && !p.stopped_reason.is_empty() {
                    text.push_str(&format!("\t{}\n", p.stopped_reason));
                }
            }
        }
        "accepting" => {
            for p in w.printers.iter().filter(|p| include(&p.name.0)) {
                text.push_str(&format!(
                    "{} {} requests since simulated snapshot\n",
                    p.name.0,
                    if p.accepting {
                        "accepting"
                    } else {
                        "not accepting"
                    }
                ));
                if !p.accepting && !p.rejection_reason.is_empty() {
                    text.push_str(&format!("\t{}\n", p.rejection_reason));
                }
            }
        }
        "jobs" => {
            for j in w
                .jobs
                .iter()
                .filter(|j| include(&j.printer.0) && j.state.active() != completed)
            {
                let d = f.document(&j.document)?;
                text.push_str(&format!(
                    "{}-{} {} {} simulated submission\n",
                    j.printer.0,
                    j.id,
                    f.operator,
                    d.text.len().div_ceil(1024) * 1024
                ));
                if long {
                    text.push_str(&format!(
                        "\tStatus: {}\n\tqueued for {}\n",
                        match j.state {
                            JobState::Pending => "pending",
                            JobState::Held => "held",
                            JobState::Completed => "completed",
                            JobState::Canceled => "canceled",
                        },
                        j.printer.0
                    ));
                }
            }
        }
        "all" => {
            text.push_str("scheduler is running (simulated)\n");
            text.push_str(&listing(w, f, "default", None, false, false)?);
            for p in &w.printers {
                text.push_str(&format!("device for {}: simulated:offline\n", p.name.0));
            }
            for kind in ["accepting", "printers", "jobs"] {
                text.push_str(&listing(w, f, kind, None, false, long)?);
            }
        }
        _ => return Err(unsupported()),
    }
    Ok(text)
}
fn lp(w: &mut World, f: &Facts, args: &[String]) -> Result<String> {
    let mut printer = None;
    let mut copies = None;
    let mut handling = None;
    let mut request = None;
    let mut file = None;
    let mut i = 0;
    let mut end = false;
    while i < args.len() {
        let arg = &args[i];
        if !end && arg == "--" {
            end = true;
            i += 1;
            continue;
        }
        if !end && ["-d", "-n", "-H", "-i"].contains(&arg.as_str()) {
            i += 1;
            let value = args
                .get(i)
                .ok_or_else(|| syntax("This option needs a value."))?;
            match arg.as_str() {
                "-d" => {
                    if printer.replace(value.clone()).is_some() {
                        return Err(syntax("Use one destination."));
                    }
                }
                "-n" => {
                    if copies.is_some()
                        || value.is_empty()
                        || !value.bytes().all(|b| b.is_ascii_digit())
                    {
                        return Err(syntax("Use one positive numeric copy count."));
                    }
                    copies = Some(
                        value
                            .parse::<u16>()
                            .map_err(|_| syntax("This simulator supports 1–20 copies."))?,
                    );
                }
                "-H" => {
                    if handling.replace(value.clone()).is_some() {
                        return Err(syntax("Use one hold/resume choice."));
                    }
                }
                "-i" => {
                    if request.replace(value.clone()).is_some() {
                        return Err(syntax("Use one request ID."));
                    }
                }
                _ => unreachable!(),
            }
        } else if (!end && arg.starts_with('-')) || file.replace(arg.clone()).is_some() {
            return Err(unsupported());
        }
        i += 1;
    }
    if let Some(job) = request {
        if printer.is_some() || copies.is_some() || file.is_some() {
            return Err(unsupported());
        }
        match handling.as_deref() {
            Some("hold") => w.hold(&job, true)?,
            Some("resume") => w.hold(&job, false)?,
            _ => return Err(unsupported()),
        }
        Ok(String::new())
    } else {
        let held = match handling.as_deref() {
            None | Some("immediate") => false,
            Some("hold") => true,
            _ => return Err(unsupported()),
        };
        let path = literal_path(file.as_ref().ok_or_else(|| {
            syntax("Name one supplied virtual document; stdin printing is outside this subset.")
        })?)?;
        let id = w.submit(f, printer.as_deref(), &path, copies.unwrap_or(1), held)?;
        let name = &w.request(&id.to_string())?.printer.0;
        Ok(format!("request id is {name}-{id} (1 file(s))\n"))
    }
}
fn status(w: &World, f: &Facts, args: &[String]) -> Result<String> {
    let mut long = false;
    let mut completed = false;
    let mut i = 0;
    if args.first().is_some_and(|s| s == "-l") {
        long = true;
        i += 1;
    }
    if args.get(i).is_some_and(|s| s == "-W") {
        let value = args
            .get(i + 1)
            .ok_or_else(|| syntax("-W needs completed or not-completed before -o."))?;
        completed = match value.as_str() {
            "completed" => true,
            "not-completed" => false,
            _ => return Err(unsupported()),
        };
        i += 2;
    }
    let kind = match args.get(i).map(String::as_str) {
        None => "jobs",
        Some("-t") => "all",
        Some("-d") => "default",
        Some("-p") => "printers",
        Some("-a") => "accepting",
        Some("-o") => "jobs",
        _ => return Err(unsupported()),
    };
    if i < args.len() {
        i += 1;
    }
    let selected = args.get(i).map(String::as_str);
    if selected.is_some() {
        i += 1;
    }
    if i != args.len()
        || (selected.is_some() && matches!(kind, "all" | "default"))
        || (completed && kind != "jobs")
    {
        return Err(unsupported());
    }
    listing(w, f, kind, selected, completed, long)
}
/// Transactional simulator property, not a promise about real printing tools.
pub fn run(w: &mut World, f: &Facts, raw: &str) -> Result<Output> {
    w.validate(f)?;
    let words = super::literal::words(raw).map_err(|mut e| {
        e.message = e
            .message
            .replace("install command", "print command")
            .replace("literal install subset", "literal print subset");
        e
    })?;
    let (cmd, args) = words
        .split_first()
        .ok_or_else(|| syntax("Type one command from Supported commands."))?;
    let mut next = w.clone();
    let stdout = match cmd.as_str() {
        "lpstat" => status(&next, f, args)?,
        "lp" => lp(&mut next, f, args)?,
        "lpr" => super::bsd::lpr(&mut next, f, args)?,
        "lpq" => super::bsd::lpq(&next, f, args)?,
        "lprm" => super::bsd::lprm(&mut next, args)?,
        "cancel" if args.len() == 1 => {
            next.cancel(&args[0])?;
            String::new()
        }
        "cupsenable" | "cupsdisable" | "cupsaccept" | "cupsreject" => {
            let (reason, name) = match args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                [name] => ("Reason Unknown".to_string(), name.to_string()),
                ["-r", reason, name] if matches!(cmd.as_str(), "cupsdisable" | "cupsreject") => {
                    (reason.to_string(), name.to_string())
                }
                _ => return Err(unsupported()),
            };
            match cmd.as_str() {
                "cupsenable" => next.set_enabled(&name, true, "")?,
                "cupsdisable" => next.set_enabled(&name, false, &reason)?,
                "cupsaccept" => next.set_accepting(&name, true, "")?,
                "cupsreject" => next.set_accepting(&name, false, &reason)?,
                _ => unreachable!(),
            }
            String::new()
        }
        "lpoptions" if args.len() == 2 && args[0] == "-d" => {
            next.set_default(&args[1])?;
            "copies=1\n".into()
        }
        "cat" if args.len() == 1 => f.document(&literal_path(&args[0])?)?.text.clone(),
        "ls" if args.is_empty()
            || args.len() == 1 && [".", "/work"].contains(&args[0].as_str()) =>
        {
            f.documents
                .iter()
                .filter_map(|d| d.path.strip_prefix("/work/"))
                .map(|s| format!("{s}\n"))
                .collect()
        }
        _ => return Err(unsupported()),
    };
    if stdout.len() > 32768 {
        return Err(Error {
            kind: ErrorKind::Limit,
            message: "Simulated output exceeds 32 KiB.".into(),
        });
    }
    next.validate(f)?;
    *w = next;
    Ok(Output { stdout, status: 0 })
}
