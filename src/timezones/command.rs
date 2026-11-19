//! Parsed finite commands over an immutable virtual UTC clock.
use super::{Environment, Zone, format, require_epoch};
use crate::boot_entry::{Error, ErrorKind};
mod literal;
type Result<T> = std::result::Result<T, Error>;
fn fail(kind: ErrorKind, text: impl Into<String>) -> Error {
    Error {
        kind,
        message: text.into(),
    }
}
fn unsupported(text: impl Into<String>) -> Error {
    fail(ErrorKind::Unsupported, text)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: u8,
}
fn output(text: String) -> Output {
    Output {
        stdout: text,
        stderr: String::new(),
        status: 0,
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    configured: String,
    environment: Environment,
    clock: i64,
}
impl World {
    pub fn new(configured: String, environment: Environment, clock: i64) -> super::Result<Self> {
        Zone::parse(&configured)?;
        environment.validate()?;
        require_epoch(clock)?;
        Ok(Self {
            configured,
            environment,
            clock,
        })
    }
    pub fn configured_name(&self) -> &str {
        &self.configured
    }
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
    pub fn clock(&self) -> i64 {
        self.clock
    }
    pub fn fresh_environment(&self) -> Environment {
        Environment { tz: None }
    }
    pub fn at(&self, epoch: i64, utc: bool, template: &str) -> super::Result<String> {
        let configured = Zone::parse(&self.configured)?;
        let zone = if utc {
            Zone::Utc
        } else {
            self.environment.effective(configured)?
        };
        format::render(zone, epoch, template)
    }
    pub fn command(&mut self, raw: &str) -> Result<Output> {
        let words = literal::words(raw)?;
        let args = words.iter().map(String::as_str).collect::<Vec<_>>();
        match args.as_slice() {
            ["date", rest @ ..] => self.date(rest),
            ["export", assignment] => {
                let Some(("TZ", value)) = assignment.split_once('=') else {
                    return Err(unsupported(
                        "Only literal export TZ=value is supported in this activity.",
                    ));
                };
                let env = Environment {
                    tz: Some(value.into()),
                };
                env.validate().map_err(unsupported)?;
                self.environment = env;
                Ok(output(String::new()))
            }
            ["unset", "TZ"] => {
                self.environment.tz = None;
                Ok(output(String::new()))
            }
            ["printenv", "TZ"] => Ok(match &self.environment.tz {
                Some(value) => output(format!("{value}\n")),
                None => Output {
                    stdout: String::new(),
                    stderr: String::new(),
                    status: 1,
                },
            }),
            ["readlink", "/etc/localtime"] => Ok(output(format!(
                "../usr/share/zoneinfo/{}\n",
                self.configured
            ))),
            ["timedatectl", "list-timezones"] => {
                let mut names = Zone::ALL.map(Zone::name).to_vec();
                names.push("Etc/UTC");
                names.sort();
                Ok(output(format!("{}\n", names.join("\n"))))
            }
            ["timedatectl", "set-timezone", name] => {
                Zone::parse(name).map_err(unsupported)?;
                self.configured = (*name).into();
                Ok(output(String::new()))
            }
            ["timedatectl", "show", rest @ ..] => self.show(rest),
            _ => Err(unsupported(
                "This terminal supports the supplied date, timedatectl timezone, TZ export/unset, printenv TZ and readlink forms only. No host command runs.",
            )),
        }
    }
    fn show(&self, args: &[&str]) -> Result<Output> {
        let mut property = None;
        let mut value = false;
        let mut i = 0;
        while i < args.len() {
            let next = match args[i] {
                "--value" if !value => {
                    value = true;
                    i += 1;
                    continue;
                }
                "-p" | "--property" => {
                    i += 1;
                    *args.get(i).ok_or_else(|| {
                        fail(
                            ErrorKind::Syntax,
                            "Name the Timezone property after this flag.",
                        )
                    })?
                }
                word if word.starts_with("--property=") => &word[11..],
                word if word.starts_with("-p") && word.len() > 2 => &word[2..],
                _ => {
                    return Err(unsupported(
                        "This activity supports timedatectl show -p Timezone [--value], not other properties or flags.",
                    ));
                }
            };
            if property.replace(next).is_some() || next != "Timezone" {
                return Err(unsupported(
                    "Request the Timezone property once; other properties are not modeled.",
                ));
            }
            i += 1;
        }
        if property.is_none() {
            return Err(unsupported(
                "Request -p Timezone explicitly; a full daemon status is outside this activity.",
            ));
        }
        Ok(output(if value {
            format!("{}\n", self.configured)
        } else {
            format!("Timezone={}\n", self.configured)
        }))
    }
    fn date(&self, args: &[&str]) -> Result<Output> {
        let mut utc = false;
        let mut epoch = None;
        let mut template = None;
        let mut i = 0;
        let mut end = false;
        while i < args.len() {
            let arg = args[i];
            if !end && arg == "--" {
                end = true;
                i += 1;
                continue;
            }
            if !end && matches!(arg, "-u" | "--utc" | "--universal") {
                if utc {
                    return Err(unsupported("Use the UTC flag once in this finite subset."));
                }
                utc = true;
                i += 1;
                continue;
            }
            if let Some(format) = arg.strip_prefix('+') {
                if template.replace(format).is_some() {
                    return Err(fail(ErrorKind::Syntax, "Supply one +FORMAT argument."));
                }
                i += 1;
                continue;
            }
            let stamp = if !end && matches!(arg, "-d" | "--date") {
                i += 1;
                *args.get(i).ok_or_else(|| {
                    fail(
                        ErrorKind::Syntax,
                        "Give an @SECONDS timestamp after --date.",
                    )
                })?
            } else if !end && arg.starts_with("--date=") {
                &arg[7..]
            } else if !end && arg.starts_with("-d") && arg.len() > 2 {
                &arg[2..]
            } else {
                return Err(unsupported(
                    "Supported date forms use -u, --date=@SECONDS and a literal +FORMAT. Clock setting, arbitrary date strings and other flags are not modeled.",
                ));
            };
            if epoch.is_some() {
                return Err(unsupported("Use one explicit timestamp."));
            }
            let digits = stamp.strip_prefix('@').ok_or_else(|| {
                unsupported(
                    "This activity accepts only @SECONDS input, not general date-string parsing.",
                )
            })?;
            if digits.is_empty() || digits.len() > 12 || !digits.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(fail(
                    ErrorKind::Syntax,
                    "Use @ followed by ordinary whole Unix seconds for a supplied2026instant.",
                ));
            }
            let number = digits
                .parse::<i64>()
                .map_err(|_| fail(ErrorKind::Syntax, "Timestamp exceeds supported bounds."))?;
            require_epoch(number).map_err(unsupported)?;
            epoch = Some(number);
            i += 1;
        }
        let text = self
            .at(
                epoch.unwrap_or(self.clock),
                utc,
                template.unwrap_or(format::DEFAULT),
            )
            .map_err(unsupported)?;
        Ok(output(format!("{text}\n")))
    }
}
