//! Finite literal GRUB2 menu subset. No GRUB, shell, firmware or host I/O.
use crate::boot_entry::{self as boot, Entry, Error, ErrorKind, Options, Report};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, Error>;
pub const HELP: &str = "Literal flat GRUB2 menu subset: one set default=... and up to four menuentry blocks, each with a literal title and --id, one linux command and an optional following initrd command. Select an existing entry by zero-based position, ID or title. Single/double literal quotes, comments and backslash-newline joins are supported; expansion, other commands, submenus, semicolon-separated commands, repeated image commands, firmware, installation and generation are outside this subset. Paths refer only to supplied virtual $BOOT. Linux root=UUID= identifies the system filesystem; it is different from GRUB's file-reading root, which is supplied here. This is a hand-authored teaching menu, not advice to edit a distribution-generated grub.cfg. No command is executed on your computer.";
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Word(String),
    Open,
    Close,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub entry: Entry,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    pub default: String,
    pub items: Vec<Item>,
}
fn word(current: &mut String, started: &mut bool, line: &mut Vec<Token>) {
    if *started {
        line.push(Token::Word(std::mem::take(current)));
        *started = false;
    }
}
fn lines(source: &str) -> Result<Vec<Vec<Token>>> {
    if source.len() > boot::MAX_SOURCE || source.lines().count() > boot::MAX_LINES {
        return Err(error(
            ErrorKind::Limit,
            "Keep the supplied GRUB menu within48lines and4KiB.",
        ));
    }
    if source
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
    {
        return Err(error(
            ErrorKind::Syntax,
            "Use UTF-8 text with LF newlines; no control characters.",
        ));
    }
    let mut result = Vec::new();
    let mut line = Vec::new();
    let mut current = String::new();
    let mut started = false;
    let mut quote = None;
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' && quote.is_some() {
            return Err(error(
                ErrorKind::Unsupported,
                "Multiline quoted values are outside the literal menu subset; use a backslash-newline join outside single quotes.",
            ));
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else if q == '"' && c == '\\' {
                let n = chars.next().ok_or_else(|| {
                    error(
                        ErrorKind::Syntax,
                        "A trailing backslash needs a following character.",
                    )
                })?;
                match n {
                    '\n' => {}
                    '$' | '"' | '\\' => current.push(n),
                    '`' => {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Backtick expressions are outside this literal subset.",
                        ));
                    }
                    _ => {
                        current.push('\\');
                        current.push(n);
                    }
                }
            } else if q == '"' && matches!(c, '$' | '`') {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Variable/expression expansion is outside this literal menu subset; use a literal selection or path.",
                ));
            } else {
                current.push(c);
            }
            continue;
        }
        match c {
            '\'' | '"' => {
                quote = Some(c);
                started = true;
            }
            '\\' => {
                let n = chars.next().ok_or_else(|| {
                    error(
                        ErrorKind::Syntax,
                        "A trailing backslash needs a following character.",
                    )
                })?;
                if n != '\n' {
                    current.push(n);
                    started = true;
                }
            }
            '#' if !started => {
                for n in chars.by_ref() {
                    if n == '\n' {
                        break;
                    }
                    if n == '\\' {
                        return Err(error(
                            ErrorKind::Unsupported,
                            "Backslashes inside comments are outside this subset; put comments on ordinary lines.",
                        ));
                    }
                }
                if !line.is_empty() {
                    result.push(std::mem::take(&mut line));
                }
            }
            '\n' => {
                word(&mut current, &mut started, &mut line);
                if !line.is_empty() {
                    result.push(std::mem::take(&mut line));
                }
            }
            ' ' | '\t' => word(&mut current, &mut started, &mut line),
            '{' | '}' => {
                if started {
                    return Err(error(
                        ErrorKind::Unsupported,
                        "Separate braces from literal values with whitespace in this subset.",
                    ));
                }
                line.push(if c == '{' { Token::Open } else { Token::Close });
            }
            '$' | '`' | ';' | '&' | '|' | '<' | '>' | '(' | ')' => {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Expansions, device expressions and compound commands are outside the literal menu subset; use one supported command per line.",
                ));
            }
            _ => {
                current.push(c);
                started = true;
            }
        }
    }
    if quote.is_some() {
        return Err(error(
            ErrorKind::Syntax,
            "Close the quoted value before the end of the line.",
        ));
    }
    word(&mut current, &mut started, &mut line);
    if !line.is_empty() {
        result.push(line);
    }
    Ok(result)
}
fn string(t: &Token) -> Result<&str> {
    if let Token::Word(s) = t {
        Ok(s)
    } else {
        Err(error(
            ErrorKind::Syntax,
            "A brace appears where a literal value is required.",
        ))
    }
}
fn literal(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
fn identifier(s: &str) -> bool {
    literal(s, 64)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        && !s.bytes().all(|b| b.is_ascii_digit())
}
pub mod temporary;

fn load_linux(entry: &mut Entry, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        return Err(error(
            ErrorKind::Syntax,
            "linux needs a supplied kernel path followed by its arguments.",
        ));
    }
    if !boot::path(args[0]) {
        return Err(error(
            ErrorKind::Unsupported,
            "Use a normalized literal kernel path under the supplied virtual $BOOT; device expressions and traversal are outside this subset.",
        ));
    }
    if args.len() > 17 {
        return Err(error(ErrorKind::Limit, "Use at most 16 kernel arguments."));
    }
    if args[1..].iter().any(|s| s.chars().any(char::is_whitespace)) {
        return Err(error(
            ErrorKind::Unsupported,
            "Quoted kernel arguments containing spaces are outside this subset.",
        ));
    }
    entry.linux = args[0].into();
    entry.options = args[1..].iter().map(|s| (*s).into()).collect();
    Ok(())
}

impl Menu {
    pub fn parse(source: &str) -> Result<Self> {
        let mut default = None;
        let mut items = Vec::new();
        let mut active: Option<Item> = None;
        let mut initrd_seen = false;
        let mut ids = BTreeSet::new();
        let mut titles = BTreeSet::new();
        for line in lines(source)? {
            if let Some(item) = &mut active {
                if line == [Token::Close] {
                    if item.entry.linux.is_empty() {
                        return Err(error(
                            ErrorKind::Syntax,
                            format!(
                                "Entry {:?} needs one linux command before it can be tested.",
                                item.title
                            ),
                        ));
                    }
                    Options::parse(&item.entry.options)?;
                    items.push(active.take().unwrap());
                    continue;
                }
                let command = string(&line[0])?;
                let args = line[1..].iter().map(string).collect::<Result<Vec<_>>>()?;
                match command {
                    "linux" => {
                        if !item.entry.linux.is_empty() {
                            return Err(error(
                                ErrorKind::Unsupported,
                                "Repeated linux commands are outside this subset; keep one image choice in each entry.",
                            ));
                        }
                        load_linux(&mut item.entry, &args)?;
                    }
                    "initrd" => {
                        if item.entry.linux.is_empty() {
                            return Err(error(
                                ErrorKind::Syntax,
                                "Put linux before initrd: the initramfs belongs to the kernel that was loaded first.",
                            ));
                        }
                        if initrd_seen {
                            return Err(error(
                                ErrorKind::Unsupported,
                                "Repeated initrd commands are outside this subset; list the ordered images on one initrd command.",
                            ));
                        }
                        if args.is_empty() {
                            return Err(error(
                                ErrorKind::Syntax,
                                "initrd needs at least one supplied image path.",
                            ));
                        }
                        if args.len() > 4 {
                            return Err(error(
                                ErrorKind::Limit,
                                "Use at most4ordered initramfs paths.",
                            ));
                        }
                        if args.iter().any(|s| !boot::path(s)) {
                            return Err(error(
                                ErrorKind::Unsupported,
                                "Use normalized literal initramfs paths from the supplied virtual $BOOT.",
                            ));
                        }
                        item.entry.initrd = args.iter().map(|s| (*s).into()).collect();
                        initrd_seen = true;
                    }
                    _ => {
                        return Err(error(
                            ErrorKind::Unsupported,
                            format!(
                                "Command {command:?} is outside this literal menu subset; nothing was executed."
                            ),
                        ));
                    }
                }
            } else {
                let command = string(&line[0])?;
                match command {
                    "set" => {
                        if line.len() != 2 {
                            return Err(error(
                                ErrorKind::Syntax,
                                "Write set default= followed by an existing position, ID or quoted title.",
                            ));
                        }
                        let value=string(&line[1])?.strip_prefix("default=").ok_or_else(||error(ErrorKind::Unsupported,"Only the literal default setting is supported; GRUB's file-reading root is supplied, not edited here."))?;
                        if default.is_some() {
                            return Err(error(
                                ErrorKind::Unsupported,
                                "Repeated default assignments are outside this subset; keep one explicit set default=... .",
                            ));
                        }
                        if !literal(value, 80) {
                            return Err(error(
                                ErrorKind::Syntax,
                                "default needs a nonempty literal selection up to80bytes.",
                            ));
                        }
                        default = Some(value.into());
                    }
                    "menuentry" => {
                        if items.len() >= 4 {
                            return Err(error(
                                ErrorKind::Limit,
                                "This lab supports at most4flat menu entries.",
                            ));
                        }
                        if line.last() != Some(&Token::Open) {
                            return Err(error(
                                ErrorKind::Syntax,
                                "Open menuentry with { on its title/ID line; close it with } on its own line.",
                            ));
                        }
                        let words = line[1..line.len() - 1]
                            .iter()
                            .map(string)
                            .collect::<Result<Vec<_>>>()?;
                        if words.len() < 2 {
                            return Err(error(
                                ErrorKind::Syntax,
                                "menuentry needs a title and a stable --id.",
                            ));
                        }
                        let title = words[0];
                        let id=if words.len()==2 {words[1].strip_prefix("--id=")} else if words.len()==3&&words[1]=="--id" {Some(words[2])}else{None}.ok_or_else(||error(ErrorKind::Unsupported,"Use a literal title and --id=identifier; other menuentry options are outside this subset."))?;
                        if !literal(title, 80)
                            || title.bytes().all(|b| b.is_ascii_digit())
                            || !identifier(id)
                        {
                            return Err(error(
                                ErrorKind::Unsupported,
                                "Use a nonempty title and identifier, not all digits; numeric values are reserved for zero-based positions in this subset.",
                            ));
                        }
                        if !ids.insert(id.to_owned()) || !titles.insert(title.to_owned()) {
                            return Err(error(
                                ErrorKind::Unsupported,
                                "Duplicate IDs or titles are ambiguous in this subset; keep both unique.",
                            ));
                        }
                        active = Some(Item {
                            id: id.into(),
                            title: title.into(),
                            entry: Entry {
                                title: Some(title.into()),
                                version: None,
                                linux: String::new(),
                                initrd: Vec::new(),
                                options: Vec::new(),
                            },
                        });
                        initrd_seen = false;
                    }
                    _ => {
                        return Err(error(
                            ErrorKind::Unsupported,
                            format!(
                                "Command {command:?} is outside this literal menu subset; nothing was executed."
                            ),
                        ));
                    }
                }
            }
        }
        if active.is_some() {
            return Err(error(
                ErrorKind::Syntax,
                "Close the final menuentry with } on its own line.",
            ));
        }
        if items.is_empty() {
            return Err(error(
                ErrorKind::Syntax,
                "Add at least one supplied menuentry.",
            ));
        }
        if ids.iter().any(|s| titles.contains(s)) {
            return Err(error(
                ErrorKind::Unsupported,
                "An ID also names an entry title; use unambiguous IDs and titles in this subset.",
            ));
        }
        let menu=Self{default:default.ok_or_else(||error(ErrorKind::Unsupported,"This lab requires an explicit literal set default=...; unset/saved defaults are outside the subset."))?,items};
        menu.selected()?;
        Ok(menu)
    }
    pub fn selected(&self) -> Result<&Item> {
        let item = if self.default.bytes().all(|b| b.is_ascii_digit()) {
            self.default
                .parse::<usize>()
                .ok()
                .and_then(|n| self.items.get(n))
        } else {
            self.items
                .iter()
                .find(|e| e.id == self.default || e.title == self.default)
        };
        item.ok_or_else(||error(ErrorKind::Unsupported,"The default names no supplied entry. Automatic missing-default fallback behavior is outside this subset; select an existing zero-based position, ID or title."))
    }
    pub fn changed_fields(&self, original: &Self, id: &str) -> Vec<&'static str> {
        let Some(a) = self.items.iter().find(|e| e.id == id) else {
            return vec!["entry is missing or its ID was changed"];
        };
        let Some(b) = original.items.iter().find(|e| e.id == id) else {
            return vec!["entry is not in the supplied menu"];
        };
        let mut changes = Vec::new();
        if a.title != b.title {
            changes.push("title changed");
        }
        if a.entry.linux != b.entry.linux {
            changes.push("kernel path changed");
        }
        if a.entry.initrd != b.entry.initrd {
            changes.push("early image paths/order changed");
        }
        match (
            Options::parse(&a.entry.options),
            Options::parse(&b.entry.options),
        ) {
            (Ok(x), Ok(y)) => {
                if x.root != y.root {
                    changes.push("system-root UUID changed");
                }
                if x.read_only != y.read_only {
                    changes.push("initial read mode changed");
                }
                if x.target != y.target {
                    changes.push("startup target changed");
                }
            }
            _ => changes.push("kernel arguments are not supported/valid"),
        }
        if a.entry.options.iter().any(|x| x == "quiet")
            != b.entry.options.iter().any(|x| x == "quiet")
        {
            changes.push("quiet setting changed");
        }
        changes
    }
    pub fn equivalent_item(&self, original: &Self, id: &str) -> bool {
        self.changed_fields(original, id).is_empty()
    }
}
pub fn test(inventory: &boot::Inventory, source: &str) -> Result<Report> {
    let menu = Menu::parse(source)?;
    let selected = menu.selected()?;
    // This is a pure typed image/root trace, never GRUB execution.
    let mut report = boot::trace_entry(inventory, &selected.entry)?;
    report.stages.insert(
        0,
        boot::Stage {
            title: "Menu selection",
            passed: true,
            detail: format!(
                "Default {:?} selects {:?} (ID {}).",
                menu.default, selected.title, selected.id
            ),
        },
    );
    Ok(report)
}
