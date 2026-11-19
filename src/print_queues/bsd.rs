//! Finite CUPS-compatible BSD command names, without an LPD transport or daemon.
use super::command::{literal_path, syntax, unsupported};
use super::*;
fn copies(raw: &str) -> Result<u16> {
    if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(syntax("Use a positive numeric copy count after -#."));
    }
    raw.parse()
        .map_err(|_| syntax("This simulator supports 1–20 copies."))
}
pub(super) fn lpr(world: &mut World, facts: &Facts, args: &[String]) -> Result<String> {
    let mut destination = None;
    let mut count = None;
    let mut held = false;
    let mut file = None;
    let mut i = 0;
    while i < args.len() {
        let word = &args[i];
        if word == "-P" || word == "-#" {
            i += 1;
            let value = args
                .get(i)
                .ok_or_else(|| syntax("This option needs a value."))?;
            if word == "-P" {
                if destination.replace(value.clone()).is_some() {
                    return Err(syntax("Use one destination."));
                }
            } else {
                if count.is_some() {
                    return Err(syntax("Use one copy count."));
                }
                count = Some(copies(value)?);
            }
        } else if let Some(value) = word.strip_prefix("-P").filter(|s| !s.is_empty()) {
            if destination.replace(value.to_string()).is_some() {
                return Err(syntax("Use one destination."));
            }
        } else if let Some(value) = word.strip_prefix("-#").filter(|s| !s.is_empty()) {
            if count.is_some() {
                return Err(syntax("Use one copy count."));
            }
            count = Some(copies(value)?);
        } else if word == "-q" {
            if held {
                return Err(syntax("Use -q once to hold this request."));
            }
            held = true;
        } else if word.starts_with('-') || file.replace(word.clone()).is_some() {
            return Err(unsupported());
        }
        i += 1;
    }
    let file = file.ok_or_else(|| {
        syntax("Name one supplied document; stdin printing is outside this subset.")
    })?;
    world.submit(
        facts,
        destination.as_deref(),
        &literal_path(&file)?,
        count.unwrap_or(1),
        held,
    )?;
    Ok(String::new()) // lpr is silent on ordinary success; inspect its actual request.
}
fn rank(number: usize) -> String {
    let suffix = if (11..=13).contains(&(number % 100)) {
        "th"
    } else {
        match number % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{number}{suffix}")
}
pub(super) fn lpq(world: &World, facts: &Facts, args: &[String]) -> Result<String> {
    let mut selected = None;
    let mut long = false;
    let mut i = 0;
    while i < args.len() {
        let word = &args[i];
        if word == "-l" {
            if long {
                return Err(syntax("Use -l once."));
            }
            long = true;
        } else if word == "-P" {
            i += 1;
            let value = args
                .get(i)
                .ok_or_else(|| syntax("-P needs a supplied printer name."))?;
            if selected.replace(value.clone()).is_some() {
                return Err(syntax("Use one destination."));
            }
        } else if let Some(value) = word.strip_prefix("-P").filter(|s| !s.is_empty()) {
            if selected.replace(value.to_string()).is_some() {
                return Err(syntax("Use one destination."));
            }
        } else {
            return Err(unsupported());
        }
        i += 1;
    }
    let destination = selected
        .or_else(|| world.local_default.as_ref().map(|p| p.0.clone()))
        .ok_or_else(|| {
            invalid("No default destination. Choose -P PRINTER or set the local default.")
        })?;
    let printer = world.printer(&destination)?;
    let mut text = format!(
        "{} is {}\n",
        printer.name.0,
        if printer.enabled {
            "ready"
        } else {
            "not ready"
        }
    );
    let mut jobs = world
        .jobs
        .iter()
        .filter(|j| j.printer.0.eq_ignore_ascii_case(&destination) && j.state.active())
        .collect::<Vec<_>>();
    jobs.sort_by_key(|j| j.id);
    if jobs.is_empty() {
        text.push_str("no entries\n");
        return Ok(text);
    }
    if !long {
        text.push_str("Rank    Owner        Job    File(s)                  Total Size\n");
    }
    for (index, job) in jobs.iter().enumerate() {
        let document = facts.document(&job.document)?;
        let file = job.document.rsplit('/').next().unwrap();
        let bytes = document.text.len().div_ceil(1024) * 1024;
        if long {
            let description = if job.copies == 1 {
                file.to_string()
            } else {
                format!("{} copies of {file}", job.copies)
            };
            text.push_str(&format!(
                "\n{}: {} [job {} simulated]\n    {description} {bytes} bytes\n",
                facts.operator,
                rank(index + 1),
                job.id
            ));
        } else {
            text.push_str(&format!(
                "{:<7} {:<12} {:<6} {:<24} {bytes} bytes\n",
                rank(index + 1),
                facts.operator,
                job.id,
                file
            ));
        }
    }
    Ok(text)
}
pub(super) fn lprm(world: &mut World, args: &[String]) -> Result<String> {
    if args.len() != 1 || args[0].is_empty() || !args[0].bytes().all(|b| b.is_ascii_digit()) {
        return Err(unsupported());
    }
    world.cancel(&args[0])?;
    Ok(String::new())
}
