//! Literal locale/codec tools over typed environments and virtual byte files.
use super::{
    Category, Environment, Locale, Variable,
    encoding::{Encoding, FailureKind},
    virtual_files::Files,
};
use crate::boot_entry::{Error, ErrorKind};
mod literal;
type Result<T> = std::result::Result<T, Error>;
pub const HELP: &str = "Simulated locale/encoding terminal: locale (normalized six-category listing), locale -a, locale [ -k ] charmap|decimal_point; export NAME=VALUE... and unset NAME... for LANG, LC_ALL and the six supported LC_ categories only; iconv -f FROM -t TO [-o OUTPUT] INPUT using ASCII, ISO-8859-1 or UTF-8; ls [/work] and cat FILE. Short attached or separate iconv values and --from-code/--to-code/--output equivalents are supported; duplicate flags, stdin, multiple inputs, transliteration/discard flags, pipes, redirection, scripts, expansions and all other commands refuse explicitly. Provided files are read-only; outputs are bounded /work files. Missing -f/-t is unsupported here rather than guessing a real iconv default. No host utility, gconv module, file, process or environment is accessed.";
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: Vec<u8>,
    pub stderr: String,
    pub status: u8,
}
impl Output {
    fn text(value: String) -> Self {
        Self {
            stdout: value.into_bytes(),
            stderr: String::new(),
            status: 0,
        }
    }
}
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn unsupported(message: impl Into<String>) -> Error {
    error(ErrorKind::Unsupported, message)
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Query {
    Listing,
    Available,
    Charmap(bool),
    Decimal(bool),
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Command {
    Query(Query),
    Export(Vec<(Variable, Option<Locale>)>),
    Unset(Vec<Variable>),
    Iconv {
        from: Encoding,
        to: Encoding,
        input: String,
        output: Option<String>,
    },
    List,
    Cat(String),
}
fn binding(raw: &str) -> Result<(Variable, Option<Locale>)> {
    let (name, value) = raw.split_once('=').ok_or_else(|| {
        unsupported("Use export NAME=value; bare-name export is outside this simulated subset.")
    })?;
    let variable = Variable::parse(name).map_err(unsupported)?;
    let value = if value.is_empty() {
        None
    } else {
        Some(Locale::parse(value).map_err(unsupported)?)
    };
    Ok((variable, value))
}
fn option(raw: &str) -> Option<(&str, Option<&str>)> {
    for (short, long) in [
        ("-f", "--from-code"),
        ("-t", "--to-code"),
        ("-o", "--output"),
    ] {
        if raw == short || raw == long {
            return Some((short, None));
        }
        if let Some(value) = raw.strip_prefix(short).filter(|v| !v.is_empty()) {
            return Some((short, Some(value)));
        }
        if let Some(value) = raw
            .strip_prefix(long)
            .and_then(|tail| tail.strip_prefix('='))
        {
            return Some((short, Some(value)));
        }
    }
    None
}
fn parse(raw: &str) -> Result<Command> {
    let words = literal::words(raw)?;
    let Some((name, args)) = words.split_first() else {
        return Err(error(ErrorKind::Syntax, "Type one supported command."));
    };
    match name.as_str() {
        "locale" => Ok(Command::Query(
            match args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice()
            {
                [] => Query::Listing,
                ["-a"] => Query::Available,
                ["charmap"] => Query::Charmap(false),
                ["decimal_point"] => Query::Decimal(false),
                ["-k", "charmap"] => Query::Charmap(true),
                ["-k", "decimal_point"] => Query::Decimal(true),
                _ => {
                    return Err(unsupported(
                        "Supported inspection: locale, locale -a, and locale [-k] charmap or decimal_point. Other categories/flags are outside this subset.",
                    ));
                }
            },
        )),
        "export" => {
            if args.is_empty() || args.len() > 8 {
                return Err(unsupported(
                    "Use export with one to eight explicit supported locale assignments.",
                ));
            }
            Ok(Command::Export(
                args.iter().map(|a| binding(a)).collect::<Result<_>>()?,
            ))
        }
        "unset" => {
            if args.is_empty() || args.len() > 8 {
                return Err(unsupported(
                    "Use unset with one to eight supported locale variable names.",
                ));
            }
            Ok(Command::Unset(
                args.iter()
                    .map(|a| Variable::parse(a).map_err(unsupported))
                    .collect::<Result<_>>()?,
            ))
        }
        "ls" if args.is_empty() || args == ["/work"] => Ok(Command::List),
        "cat" if args.len() == 1 && !args[0].starts_with('-') => Ok(Command::Cat(args[0].clone())),
        "iconv" => {
            let (mut from, mut to, mut output, mut input) = (None, None, None, None);
            let mut n = 0;
            while n < args.len() {
                if let Some((flag, attached)) = option(&args[n]) {
                    let value = if let Some(value) = attached {
                        value
                    } else {
                        n += 1;
                        args.get(n).map(String::as_str).ok_or_else(|| {
                            error(ErrorKind::Syntax, "Give the iconv option its value.")
                        })?
                    };
                    if value.is_empty() {
                        return Err(error(
                            ErrorKind::Syntax,
                            "Give the iconv option a nonempty value.",
                        ));
                    }
                    match flag {
                        "-f" => {
                            if from.is_some() {
                                return Err(unsupported(
                                    "Duplicate -f is outside this subset; keep one source encoding.",
                                ));
                            }
                            from = Some(Encoding::parse(value).map_err(unsupported)?);
                        }
                        "-t" => {
                            if to.is_some() {
                                return Err(unsupported(
                                    "Duplicate -t is outside this subset; keep one destination encoding.",
                                ));
                            }
                            to = Some(Encoding::parse(value).map_err(unsupported)?);
                        }
                        "-o" => {
                            if output.is_some() {
                                return Err(unsupported(
                                    "Duplicate -o is outside this subset; keep one output path.",
                                ));
                            }
                            output = Some(value.to_string());
                        }
                        _ => unreachable!(),
                    }
                } else {
                    if args[n].starts_with('-') {
                        return Err(unsupported(
                            "This iconv subset supports explicit -f/-t/-o only; stdin, discard, transliteration and other flags are not simulated.",
                        ));
                    }
                    if input.replace(args[n].clone()).is_some() {
                        return Err(unsupported(
                            "Use exactly one supplied input file per iconv command.",
                        ));
                    }
                }
                n += 1;
            }
            Ok(Command::Iconv{from:from.ok_or_else(||unsupported("Specify -f FROM in this subset. Real iconv can otherwise use the current character encoding."))?,to:to.ok_or_else(||unsupported("Specify -t TO in this subset. Real iconv can otherwise use the current character encoding."))?,input:input.ok_or_else(||unsupported("Supply one input file; reading standard input is not simulated."))?,output})
        }
        _ => Err(unsupported(format!(
            "Command or form {name:?} is outside this locale/encoding terminal. Open Supported commands; nothing ran on the host."
        ))),
    }
}
fn query(query: Query, env: &Environment, available: &[Locale]) -> String {
    match query {
        Query::Charmap(keyword) => {
            let value = env.effective(Category::Ctype).charmap();
            if keyword {
                format!("charmap=\"{value}\"\n")
            } else {
                format!("{value}\n")
            }
        }
        Query::Decimal(keyword) => {
            let value = env.effective(Category::Numeric).decimal();
            if keyword {
                format!("decimal_point=\"{value}\"\n")
            } else {
                format!("{value}\n")
            }
        }
        Query::Available => {
            let mut names = available
                .iter()
                .map(|l| match l {
                    Locale::C => "C",
                    Locale::Cutf8 => "C.utf8",
                    Locale::EnUsUtf8 => "en_US.utf8",
                    Locale::DeDeUtf8 => "de_DE.utf8",
                })
                .collect::<Vec<_>>();
            names.push("POSIX");
            names.sort();
            names.iter().map(|name| format!("{name}\n")).collect()
        }
        Query::Listing => {
            let mut value = format!(
                "LANG={}\n",
                env.get(Variable::Lang).map_or("", Locale::name)
            );
            for category in Category::ALL {
                value.push_str(&format!(
                    "{}=\"{}\"\n",
                    category.name(),
                    env.effective(category).name()
                ));
            }
            value.push_str(&format!(
                "LC_ALL={}\n",
                env.get(Variable::All).map_or("", Locale::name)
            ));
            value
        }
    }
}
/// Parse fully before execution. A conversion status1 can still change a file;
/// unsupported/syntax/resource errors are distinct from an executed conversion.
pub fn run(
    env: &mut Environment,
    files: &mut Files,
    available: &[Locale],
    raw: &str,
) -> Result<Output> {
    let command = parse(raw)?;
    super::configuration("", available).map_err(|e| error(ErrorKind::Invalid, e))?;
    env.validate(available)
        .map_err(|e| error(ErrorKind::Invalid, e))?;
    files.validate().map_err(|e| error(ErrorKind::Invalid, e))?;
    match command {
        Command::Query(q) => Ok(Output::text(query(q, env, available))),
        Command::Export(bindings) => {
            let mut next = env.clone();
            for (name, value) in bindings {
                next.set(name, value);
            }
            next.validate(available).map_err(unsupported)?;
            *env = next;
            Ok(Output::text(String::new()))
        }
        Command::Unset(names) => {
            for name in names {
                env.unset(name);
            }
            Ok(Output::text(String::new()))
        }
        Command::List => {
            let mut names = files
                .inputs()
                .iter()
                .map(|f| f.path.trim_start_matches("/work/"))
                .chain(
                    files
                        .outputs()
                        .keys()
                        .map(|p| p.trim_start_matches("/work/")),
                )
                .collect::<Vec<_>>();
            names.sort();
            Ok(Output::text(
                names.iter().map(|name| format!("{name}\n")).collect(),
            ))
        }
        Command::Cat(path) => Ok(Output {
            stdout: files
                .read(&path)
                .map_err(|e| error(ErrorKind::Invalid, e))?
                .to_vec(),
            stderr: String::new(),
            status: 0,
        }),
        Command::Iconv {
            from,
            to,
            input,
            output,
        } => {
            let converted = files
                .convert(&input, output.as_deref(), from, to)
                .map_err(|e| error(ErrorKind::Invalid, e))?;
            let stderr = converted.failure.as_ref().map_or(String::new(), |f| {
                format!(
                    "iconv: {} at input byte {}\n",
                    match f.kind {
                        FailureKind::InvalidInput => "invalid input sequence",
                        FailureKind::IncompleteInput => "incomplete input sequence",
                        FailureKind::Unrepresentable =>
                            "character cannot be represented in the target encoding",
                    },
                    f.position
                )
            });
            Ok(Output {
                status: if converted.failure.is_some() { 1 } else { 0 },
                stdout: if output.is_some() {
                    vec![]
                } else {
                    converted.bytes
                },
                stderr,
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::super::virtual_files::Input;
    use super::*;
    fn files() -> Files {
        Files::new(vec![Input {
            path: "/work/feed.txt".into(),
            label: "Latin-1 feed".into(),
            bytes: b"Caf\xe9\n".to_vec(),
        }])
        .unwrap()
    }
    const AVAILABLE: [Locale; 4] = [Locale::C, Locale::Cutf8, Locale::EnUsUtf8, Locale::DeDeUtf8];
    #[test]
    fn exports_unsets_and_actual_queries_follow_current_environment() {
        let (mut env, mut f) = (Environment::default(), files());
        run(
            &mut env,
            &mut f,
            &AVAILABLE,
            "export LANG=de_DE.UTF-8 LC_CTYPE='C.UTF-8' LC_NUMERIC=C",
        )
        .unwrap();
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "locale charmap")
                .unwrap()
                .stdout,
            b"UTF-8\n"
        );
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "locale -k decimal_point")
                .unwrap()
                .stdout,
            b"decimal_point=\".\"\n"
        );
        run(&mut env, &mut f, &AVAILABLE, "export LC_ALL=C").unwrap();
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "locale charmap")
                .unwrap()
                .stdout,
            b"ANSI_X3.4-1968\n"
        );
        run(&mut env, &mut f, &AVAILABLE, "unset LC_ALL LC_NUMERIC").unwrap();
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "locale decimal_point")
                .unwrap()
                .stdout,
            b",\n"
        );
    }
    #[test]
    fn equivalent_flag_forms_and_orders_create_identical_real_outputs() {
        for command in [
            "iconv -f ISO-8859-1 -t UTF-8 -o fixed.txt feed.txt",
            "iconv feed.txt -tUTF8 -fLATIN1 -ofixed.txt",
            "iconv --from-code=LATIN1 --to-code UTF-8 --output=fixed.txt feed.txt",
        ] {
            let (mut env, mut f) = (Environment::default(), files());
            let result = run(&mut env, &mut f, &AVAILABLE, command).unwrap();
            assert_eq!(result.status, 0);
            assert!(result.stdout.is_empty() && result.stderr.is_empty());
            assert_eq!(f.read("fixed.txt").unwrap(), "Café\n".as_bytes());
        }
    }
    #[test]
    fn nonzero_conversion_changes_file_to_partial_prefix_and_retry_repairs_it() {
        let (mut env, mut f) = (Environment::default(), files());
        let result = run(
            &mut env,
            &mut f,
            &AVAILABLE,
            "iconv -f LATIN1 -t ASCII -o fixed.txt feed.txt",
        )
        .unwrap();
        assert_eq!(result.status, 1);
        assert!(result.stderr.contains("input byte 3"));
        assert_eq!(f.read("fixed.txt").unwrap(), b"Caf");
        run(
            &mut env,
            &mut f,
            &AVAILABLE,
            "iconv -f LATIN1 -t UTF8 -o fixed.txt feed.txt",
        )
        .unwrap();
        assert_eq!(f.read("fixed.txt").unwrap(), "Café\n".as_bytes());
    }
    #[test]
    fn malformed_unsupported_expansion_host_paths_and_unknown_settings_never_execute() {
        for command in [
            "bash anything",
            "sh -c id",
            "locale | cat",
            "locale;id",
            "cat /etc/passwd",
            "cat ../private",
            "export PATH=/usr/bin",
            "export GCONV_PATH=/tmp",
            "export LANG=$(id)",
            "export LANG=C LC_CTYPE=unknown",
            "iconv -f LATIN1 -t UTF8 --output /etc/private feed.txt",
            "iconv -f LATIN1 -t UTF8 -o fixed.txt feed.txt extra.txt",
            "iconv -f LATIN1 -t UTF8//IGNORE feed.txt",
            "iconv -f LATIN1 feed.txt",
            "iconv -t UTF8 feed.txt",
            "iconv -f LATIN1 -f UTF8 -t UTF8 feed.txt",
            "iconv -f LATIN1 -t UTF8 -o feed.txt feed.txt",
            "cat 'unfinished",
        ] {
            let (mut env, mut f) = (Environment::default(), files());
            let before = (env.clone(), f.clone());
            assert!(
                run(&mut env, &mut f, &AVAILABLE, command).is_err(),
                "{command}"
            );
            assert_eq!((env, f), before, "{command}");
        }
    }
    #[test]
    fn stdout_preview_is_not_saved_work_and_syntax_limits_are_explicit() {
        let (mut env, mut f) = (Environment::default(), files());
        let before = f.clone();
        assert_eq!(
            run(
                &mut env,
                &mut f,
                &AVAILABLE,
                "iconv -f LATIN1 -t UTF8 feed.txt"
            )
            .unwrap()
            .stdout,
            "Café\n".as_bytes()
        );
        assert_eq!(f, before);
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "ls /work")
                .unwrap()
                .stdout,
            b"feed.txt\n"
        );
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "cat feed.txt")
                .unwrap()
                .stdout,
            b"Caf\xe9\n"
        );
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, &"x".repeat(1025))
                .unwrap_err()
                .kind,
            ErrorKind::Limit
        );
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "cat 'unfinished")
                .unwrap_err()
                .kind,
            ErrorKind::Syntax
        );
        assert_eq!(
            run(&mut env, &mut f, &AVAILABLE, "sh anything")
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }
}
