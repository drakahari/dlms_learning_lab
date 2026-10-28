//! Literal terminal adapter over virtual archives. Never opens a host path.
use crate::file_backup::{self, Archive, Snapshot, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MAX_STORE: usize = 2 * 1024 * 1024;
const MAX_TREE_TEXT: usize = 32_768;

/// Fixed supplied role, not the host user. Backup restoration models privileged
/// metadata preservation; ordinary-user tar/umask ownership rules are excluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Root,
}
pub const OPERATOR: Operator = Operator::Root;

pub const HELP: &str = "Simulated backup administrator: root (no host privileges). Virtual backup terminal: pwd; cd PATH; ls [-a] [PATH]; stat -c FORMAT [--] FILE (only %a, %A, %U, %G, %n and %%); cat [--] FILE; cp [--] ARCHIVE_OR_SNAPSHOT DESTINATION; rm [--] FILE; mkdir [--] PATH; touch [--] FILE; chmod OCTAL PATH; tar -c/-x/-t -f ARCHIVE (-v for create/extract only) [-C DIRECTORY] [-g SNAPSHOT | -G] [SOURCE]. Long create/extract/list/verbose/file/directory/listed-incremental/incremental options are supported. One selected source directory; extract/list support literal stored file members; selective directory recursion is unsupported. No compression, links, devices, shell operators or host execution. See optional reference for exact limits.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StoredSnapshot {
    #[serde(deserialize_with = "absolute_path")]
    pub source: String,
    pub snapshot: Snapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Store {
    /// Fixed disjoint virtual roots, e.g. /srv/team and /restore/team.
    #[serde(deserialize_with = "maps::<_,Tree,6>")]
    pub trees: BTreeMap<String, Tree>,
    /// One flat namespace for opaque virtual archive and snapshot objects.
    #[serde(deserialize_with = "absolute_path")]
    pub storage: String,
    #[serde(deserialize_with = "maps::<_,Archive,8>")]
    pub archives: BTreeMap<String, Archive>,
    #[serde(deserialize_with = "maps::<_,StoredSnapshot,8>")]
    pub snapshots: BTreeMap<String, StoredSnapshot>,
    #[serde(deserialize_with = "absolute_path")]
    pub cwd: String,
}

fn absolute_path<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::Visitor;
    struct Path;
    impl Visitor<'_> for Path {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("one canonical virtual absolute path of at most1024bytes")
        }
        fn visit_str<E: serde::de::Error>(self, p: &str) -> Result<String, E> {
            if !canonical(p) {
                return Err(E::custom("Invalid virtual absolute path"));
            }
            Ok(p.into())
        }
    }
    d.deserialize_str(Path)
}
fn maps<'de, D, V, const N: usize>(d: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
    struct Key<'a, V, const N: usize>(&'a BTreeMap<String, V>);
    impl<V, const N: usize> Visitor<'_> for Key<'_, V, N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a unique canonical virtual path")
        }
        fn visit_str<E: serde::de::Error>(self, p: &str) -> Result<String, E> {
            if !canonical(p) || self.0.contains_key(p) {
                return Err(E::custom("Invalid or duplicate virtual artifact/root path"));
            }
            Ok(p.into())
        }
    }
    impl<'de, V, const N: usize> DeserializeSeed<'de> for Key<'_, V, N> {
        type Value = String;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
            if self.0.len() >= N {
                return Err(D::Error::custom(
                    "Virtual root/artifact map capacity exceeded",
                ));
            }
            d.deserialize_str(self)
        }
    }
    struct Entries<V, const N: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most{N}unique virtual entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(key) = map.next_key_seed(Key::<V, N>(&out))? {
                out.insert(key, map.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries::<V, N>(std::marker::PhantomData))
}

fn canonical(p: &str) -> bool {
    p == "/"
        || (p.starts_with('/')
            && p.len() <= 1024
            && p[1..].split('/').count() <= 64
            && p[1..].split('/').all(|n| {
                !n.is_empty()
                    && n.len() <= 255
                    && n != "."
                    && n != ".."
                    && !n.chars().any(char::is_control)
            }))
}
fn beneath(p: &str, root: &str) -> bool {
    (root == "/" && p.starts_with('/'))
        || p == root
        || p.strip_prefix(root)
            .is_some_and(|tail| tail.starts_with('/'))
}
fn parent(p: &str) -> &str {
    p.rsplit_once('/')
        .map_or("/", |(s, _)| if s.is_empty() { "/" } else { s })
}
fn basename(p: &str) -> &str {
    p.rsplit('/').next().unwrap()
}
fn join(directory: &str, name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("A virtual path cannot be blank".into());
    }
    let result = if name.starts_with('/') {
        name.into()
    } else {
        format!("{}/{name}", directory.trim_end_matches('/'))
    };
    if !canonical(&result) {
        return Err("Use a literal virtual path without empty, dot or parent components".into());
    }
    Ok(result)
}
fn permission_text(node: &crate::engine::Node) -> String {
    let mut chars = crate::engine::mode_string(node).chars().collect::<Vec<_>>();
    for (bit, index, executed, plain, capital) in [
        (0o4000, 3, 0o100, 's', 'S'),
        (0o2000, 6, 0o010, 's', 'S'),
        (0o1000, 9, 0o001, 't', 'T'),
    ] {
        if node.mode & bit != 0 {
            chars[index] = if node.mode & executed != 0 {
                plain
            } else {
                capital
            };
        }
    }
    chars.into_iter().collect()
}
struct Count(usize);
impl std::io::Write for Count {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(data.len())
            .filter(|n| *n <= MAX_STORE)
            .ok_or_else(|| std::io::Error::other("Virtual backup store exceeds2MiB"))?;
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn tree_size(t: &Tree) -> bool {
    t.valid()
        && t.entries.len() <= 64
        && t.entries
            .values()
            .filter_map(|e| e.text.as_ref())
            .map(String::len)
            .sum::<usize>()
            <= MAX_TREE_TEXT
}

impl Store {
    fn artifact(&self, path: &str) -> bool {
        canonical(path) && parent(path) == self.storage
    }
    fn directory(&self, path: &str) -> bool {
        path == "/"
            || path == self.storage
            || beneath(&self.storage, path)
            || self.trees.iter().any(|(root, tree)| {
                beneath(root, path)
                    || self.locate(path).is_some_and(|(r, p)| {
                        r == root && tree.entries.get(p).is_some_and(|e| e.node.directory)
                    })
            })
    }
    fn locate<'a>(&'a self, path: &'a str) -> Option<(&'a String, &'a str)> {
        self.trees.keys().find_map(|root| {
            if path == root {
                Some((root, ""))
            } else {
                path.strip_prefix(root)
                    .and_then(|tail| tail.strip_prefix('/'))
                    .map(|p| (root, p))
            }
        })
    }
    pub fn valid(&self) -> bool {
        if self.trees.is_empty()
            || self.trees.len() > 6
            || self.archives.len() > 8
            || self.snapshots.len() > 8
            || !canonical(&self.storage)
            || self.storage == "/"
            || !canonical(&self.cwd)
            || self.trees.iter().any(|(p, t)| {
                !canonical(p)
                    || p == "/"
                    || beneath(p, &self.storage)
                    || beneath(&self.storage, p)
                    || !tree_size(t)
            })
            || self
                .trees
                .keys()
                .any(|p| self.trees.keys().any(|q| p != q && beneath(p, q)))
            || self.archives.iter().any(|(p, a)| {
                !self.artifact(p)
                    || self.snapshots.contains_key(p)
                    || !a.valid()
                    || a.entries.len() > 64
                    || a.entries
                        .values()
                        .filter_map(|e| e.text.as_ref())
                        .map(String::len)
                        .sum::<usize>()
                        > MAX_TREE_TEXT
            })
            || self.snapshots.iter().any(|(p, s)| {
                !self.artifact(p)
                    || !s.snapshot.valid()
                    || s.snapshot.inventory.len() > 64
                    || !self.trees.contains_key(&s.source)
                    || basename(&s.source) != s.snapshot.root
            })
            || !self.directory(&self.cwd)
        {
            return false;
        }
        serde_json::to_writer(Count(0), self).is_ok()
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_STORE {
            return Err("Virtual backup workspace exceeds2MiB".into());
        }
        let value: Self = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        if !value.valid() {
            return Err(
                "Invalid virtual backup roots, artifacts, capacity or current directory".into(),
            );
        }
        Ok(value)
    }
    pub fn execute(&mut self, input: &str) -> Result<String, String> {
        if !self.valid() {
            return Err("Invalid virtual backup workspace; no changes applied".into());
        }
        let words = crate::shell::literal_command(input)?;
        if words.len() > 32 {
            return Err("Use at most32 literal command words".into());
        }
        let mut candidate = self.clone();
        let output = candidate.command(&words)?;
        if output.len() > 32768 || !candidate.valid() {
            return Err("Virtual backup capacity exceeded; previous state retained".into());
        }
        *self = candidate;
        Ok(output)
    }
    fn path(&self, operand: &str) -> Result<String, String> {
        join(&self.cwd, operand)
    }
    fn command(&mut self, words: &[String]) -> Result<String, String> {
        let Some(command) = words.first().map(String::as_str) else {
            return Ok(String::new());
        };
        let mut args = &words[1..];
        match command {
            "help" if args.is_empty() => Ok(HELP.into()),
            "pwd" if args.is_empty() => Ok(format!("{}\n", self.cwd)),
            "cd" if args.len() == 1 => {
                let path = self.path(&args[0])?;
                if !self.directory(&path) {
                    return Err("No such virtual directory".into());
                }
                self.cwd = path;
                Ok(String::new())
            }
            "tar" => self.tar(args),
            "ls" => {
                let all = args.first().is_some_and(|a| a == "-a");
                if all {
                    args = &args[1..];
                }
                if args.first().is_some_and(|a| a == "--") {
                    args = &args[1..];
                }
                if args.len() > 1 || args.first().is_some_and(|a| a.starts_with('-')) {
                    return Err("Supported ls subset: ls [-a] [literal directory]".into());
                }
                let p = if let Some(a) = args.first() {
                    self.path(a)?
                } else {
                    self.cwd.clone()
                };
                self.list(&p, all)
            }
            "stat" => {
                if !(args.len() == 3 || (args.len() == 4 && args[2] == "--"))
                    || args[0] != "-c"
                    || args[1].len() > 256
                {
                    return Err("Supported stat subset: stat -c FORMAT [--] FILE; format supports %a, %A, %U, %G, %n and %% only".into());
                }
                let operand = args.last().unwrap();
                if operand.starts_with('-') && args.len() != 4 {
                    return Err("Use -- before a literal filename beginning with -".into());
                }
                let path = self.path(operand)?;
                let (root,relative)=self.locate(&path).ok_or("Metadata is available for supplied virtual tree entries only; archive/snapshot bytes and ancestor metadata are not modeled")?;
                let entry = self.trees[root]
                    .entries
                    .get(relative)
                    .ok_or("No such virtual tree entry")?;
                let mut output = String::new();
                let mut chars = args[1].chars();
                while let Some(c) = chars.next() {
                    if c != '%' {
                        output.push(c);
                        continue;
                    }
                    match chars.next() {
                        Some('%')=>output.push('%'),
                        Some('a')=>output.push_str(&format!("{:o}",entry.node.mode)),
                        Some('A')=>output.push_str(&permission_text(&entry.node)),
                        Some('U')=>output.push_str(&entry.node.owner),
                        Some('G')=>output.push_str(&entry.node.group),
                        Some('n')=>output.push_str(operand),
                        _=>return Err("Unsupported stat format: use %a, %A, %U, %G, %n or %% without width/escape modifiers".into()),
                    }
                }
                output.push('\n');
                Ok(output)
            }
            "cat" | "cp" | "rm" | "mkdir" | "touch" => {
                let ended = args.first().is_some_and(|a| a == "--");
                if ended {
                    args = &args[1..];
                }
                if (!ended && args.iter().any(|a| a.starts_with('-')))
                    || args.len() != if command == "cp" { 2 } else { 1 }
                {
                    return Err(format!(
                        "Unsupported {command} arguments; use help for this virtual backup subset"
                    ));
                }
                let p = self.path(&args[0])?;
                if command == "cat" && args[0] == "-" {
                    return Err("A single - means standard input, even after --. This one-command backup terminal has no interactive input stream; use a supplied virtual filename".into());
                }
                if command == "cp" {
                    let dest = self.path(&args[1])?;
                    if p == dest {
                        return Err("Source and destination name the same virtual file".into());
                    }
                    if !self.artifact(&dest) {
                        return Err("Archive/snapshot copies must stay in the supplied virtual backup folder".into());
                    }
                    if self.snapshots.contains_key(&p) {
                        if self.archives.contains_key(&dest) {
                            return Err("Destination is an archive, not a snapshot".into());
                        }
                        self.snapshots.insert(dest, self.snapshots[&p].clone());
                    } else if self.archives.contains_key(&p) {
                        if self.snapshots.contains_key(&dest) {
                            return Err("Destination is a snapshot, not an archive".into());
                        }
                        self.archives.insert(dest, self.archives[&p].clone());
                    } else {
                        return Err(
                            "This backup subset copies opaque archive/snapshot artifacts only"
                                .into(),
                        );
                    }
                    return Ok(String::new());
                }
                if command == "rm" && self.artifact(&p) {
                    if self.archives.remove(&p).is_none() && self.snapshots.remove(&p).is_none() {
                        return Err("No such virtual backup file".into());
                    }
                    return Ok(String::new());
                }
                if self.artifact(&p) {
                    return Err("Backup artifacts are opaque: use tar -t for an archive; cp preserves a snapshot".into());
                }
                let (root, rel) = self
                    .locate(&p)
                    .ok_or("Path is outside the supplied virtual trees")?;
                let (root, rel) = (root.clone(), rel.to_owned());
                let tree = self.trees.get_mut(&root).unwrap();
                match command {
                    "cat" => Ok(tree
                        .entries
                        .get(&rel)
                        .and_then(|e| e.text.clone())
                        .ok_or("No such virtual text file")?),
                    "rm" => {
                        if tree.entries.get(&rel).is_some_and(|e| e.node.directory) {
                            return Err(
                                "Removing directories with rm is outside this subset".into()
                            );
                        }
                        tree.remove(&rel)?;
                        Ok(String::new())
                    }
                    "touch" if tree.entries.get(&rel).is_some_and(|e| !e.node.directory) => {
                        let text = tree.entries[&rel].text.clone().unwrap();
                        tree.write(&rel, &text)?;
                        Ok(String::new())
                    }
                    "touch" if tree.entries.get(&rel).is_some_and(|e| e.node.directory) => {
                        Err("Touching directories is outside this virtual backup subset".into())
                    }
                    "touch" | "mkdir" => {
                        let parent = rel.rsplit_once('/').map_or("", |(p, _)| p);
                        let directory = tree
                            .entries
                            .get(parent)
                            .ok_or("Virtual parent folder does not exist")?;
                        let inherited =
                            directory.node.directory && directory.node.mode & 0o2000 != 0;
                        let mut node = directory.node.clone();
                        node.owner = "root".into();
                        node.group = if inherited {
                            directory.node.group.clone()
                        } else {
                            "root".into()
                        };
                        node.directory = command == "mkdir";
                        node.mode = if node.directory { 0o755 } else { 0o644 };
                        if node.directory && inherited {
                            node.mode |= 0o2000;
                        }
                        tree.add(&rel, node, if command == "mkdir" { None } else { Some("") })?;
                        Ok(String::new())
                    }
                    _ => Err("Unsupported virtual backup command".into()),
                }
            }
            "chmod" if args.len() == 2 => {
                let mode = &args[0];
                if mode.is_empty()
                    || mode.len() > 5
                    || !mode.bytes().all(|b| matches!(b, b'0'..=b'7'))
                {
                    return Err(
                        "Use one to five octal digits with a value no greater than7777".into(),
                    );
                }
                let path = self.path(&args[1])?;
                let (root, rel) = self.locate(&path).ok_or("No such virtual tree")?;
                let (root, rel) = (root.clone(), rel.to_owned());
                let tree = self.trees.get_mut(&root).unwrap();
                let requested = u16::from_str_radix(mode, 8).unwrap();
                if requested > 0o7777 {
                    return Err("Permission mode exceeds7777".into());
                }
                let entry = tree.entries.get(&rel).ok_or("Entry does not exist")?;
                // GNU numeric modes of at most four digits preserve a
                // directory's unmentioned set-ID bits, but not its sticky bit.
                let effective = requested
                    | if entry.node.directory && mode.len() <= 4 {
                        entry.node.mode & 0o6000
                    } else {
                        0
                    };
                tree.chmod(&rel, effective)?;
                Ok(String::new())
            }
            _ => Err(format!(
                "Unsupported {command} command in this backup workspace; help lists the supported subset. Nothing runs on your computer."
            )),
        }
    }
    fn list(&self, path: &str, all: bool) -> Result<String, String> {
        if !self.directory(path) {
            return Err("Choose an existing virtual directory".into());
        }
        let mut names = BTreeSet::new();
        if all {
            names.extend([".".to_owned(), "..".to_owned()]);
        }
        if path == self.storage {
            names.extend(
                self.archives
                    .keys()
                    .chain(self.snapshots.keys())
                    .map(|p| basename(p).to_owned()),
            );
        } else if let Some((root, rel)) = self.locate(path) {
            names.extend(
                self.trees[root]
                    .entries
                    .keys()
                    .filter(|p| !p.is_empty() && p.rsplit_once('/').map_or("", |(s, _)| s) == rel)
                    .map(|p| basename(p).to_owned()),
            );
        } else {
            for root in self.trees.keys().chain(std::iter::once(&self.storage)) {
                if beneath(root, path) && root != path {
                    let tail = root
                        .strip_prefix(path.trim_end_matches('/'))
                        .unwrap()
                        .trim_start_matches('/');
                    names.insert(tail.split('/').next().unwrap().to_owned());
                }
            }
        }
        let names = names
            .into_iter()
            .filter(|n| all || !n.starts_with('.'))
            .collect::<Vec<_>>();
        Ok(if names.is_empty() {
            String::new()
        } else {
            format!("{}\n", names.join("\n"))
        })
    }
    fn tar(&mut self, args: &[String]) -> Result<String, String> {
        let options = Tar::parse(args)?;
        let raw_archive = options
            .file
            .as_deref()
            .ok_or("tar requires -f ARCHIVE in this subset")?;
        if raw_archive == "-" {
            return Err("tar archive - denotes an input/output byte stream. This backup workspace supports named virtual archives only; no archive stream or host command runs".into());
        }
        if raw_archive
            .find(':')
            .is_some_and(|colon| raw_archive.find('/').is_none_or(|slash| colon < slash))
        {
            return Err("HOST:FILE is remote tar archive syntax and is unsupported offline. Choose a named virtual archive; an explicit absolute virtual path is local".into());
        }
        let archive = self.path(raw_archive)?;
        if !self.artifact(&archive) {
            return Err("Choose an archive in the supplied virtual backup folder".into());
        }
        let directory = if let Some(d) = &options.directory {
            self.path(d)?
        } else {
            self.cwd.clone()
        };
        if !self.directory(&directory) {
            return Err("tar directory does not exist in this virtual workspace".into());
        }
        let op = options
            .operation
            .ok_or("Choose tar -c (create), -x (extract) or -t (list)")?;
        if op == 't' && options.verbose {
            return Err("Verbose tar metadata listings are unsupported here. Use tar -t for stored member names; no simulated timestamps are presented as physical archive dates".into());
        }
        match op {
            'c' => {
                if options.members.len() != 1 || options.incremental {
                    return Err("Create supports one supplied directory and optional -g SNAPSHOT; old -G creation is unsupported".into());
                }
                let root_name = &options.members[0];
                if root_name.contains('/')
                    || root_name.is_empty()
                    || matches!(root_name.as_str(), "." | "..")
                {
                    return Err("Use -C PARENT and one immediate source directory name".into());
                }
                let source = join(&directory, root_name)?;
                let tree = self
                    .trees
                    .get(&source)
                    .ok_or("Source must be one complete supplied virtual tree")?;
                let snapshot = options
                    .snapshot
                    .as_ref()
                    .map(|p| self.path(p))
                    .transpose()?;
                if snapshot.as_ref().is_some_and(|p| {
                    !self.artifact(p) || self.archives.contains_key(p) || *p == archive
                }) || self.snapshots.contains_key(&archive)
                {
                    return Err("Archive and snapshot need distinct virtual backup files".into());
                }
                let prior_file = snapshot.as_ref().and_then(|p| self.snapshots.get(p));
                if prior_file.is_some_and(|s| s.source != source) {
                    return Err("This snapshot belongs to a different supplied source tree; inode-based source relocation is unsupported".into());
                }
                let previous = prior_file.map(|s| &s.snapshot);
                let (value, next) =
                    file_backup::create(tree, root_name, previous, snapshot.is_some())?;
                let output = if options.verbose {
                    members(&value)
                } else {
                    String::new()
                };
                self.archives.insert(archive, value);
                if let Some(p) = snapshot {
                    self.snapshots.insert(
                        p,
                        StoredSnapshot {
                            source,
                            snapshot: next,
                        },
                    );
                }
                Ok(output)
            }
            'x' | 't' => {
                let value = self.archives.get(&archive).ok_or_else(|| {
                    format!("No such virtual archive; inspect ls {}", self.storage)
                })?;
                let selected = options.members.iter().map(|member| {
                    let path = member.strip_prefix(&value.root).and_then(|tail| tail.strip_prefix('/'))
                        .ok_or("Use complete stored member names beginning with the archive root; inspect tar -t")?;
                    let entry = value.entries.get(path).ok_or("File member is not stored in this archive; inspect earlier backup levels")?;
                    if entry.node.directory {return Err("Selective directory recursion is unsupported; choose literal stored file members");}
                    Ok(path)
                }).collect::<Result<Vec<_>,_>>()?;
                if selected.iter().collect::<BTreeSet<_>>().len() != selected.len() {
                    return Err(
                        "Repeated member selectors are unsupported; list each literal file once"
                            .into(),
                    );
                }
                if op == 't' {
                    return Ok(if selected.is_empty() {
                        members(value)
                    } else {
                        selected
                            .iter()
                            .map(|p| format!("{}/{p}\n", value.root))
                            .collect()
                    });
                }
                if let Some(snapshot) = &options.snapshot {
                    self.path(snapshot)?;
                }
                let destination = join(&directory, &value.root)?;
                let target = self
                    .trees
                    .get_mut(&destination)
                    .ok_or("Use -C with the parent of a supplied restore tree")?;
                if selected.is_empty() {
                    value.extract(target, options.incremental || options.snapshot.is_some())?;
                } else {
                    value.extract_files(target, &selected)?;
                }
                Ok(if options.verbose && !selected.is_empty() {
                    selected
                        .iter()
                        .map(|p| format!("{}/{p}\n", value.root))
                        .collect()
                } else if options.verbose {
                    members(value)
                } else {
                    String::new()
                })
            }
            _ => unreachable!(),
        }
    }
}
fn members(archive: &Archive) -> String {
    archive
        .entries
        .iter()
        .map(|(p, e)| {
            format!(
                "{}{}{}{}\n",
                archive.root,
                if p.is_empty() { "" } else { "/" },
                p,
                if e.node.directory { "/" } else { "" }
            )
        })
        .collect()
}

#[derive(Default)]
struct Tar {
    operation: Option<char>,
    file: Option<String>,
    directory: Option<String>,
    snapshot: Option<String>,
    incremental: bool,
    verbose: bool,
    members: Vec<String>,
}
impl Tar {
    fn operation(&mut self, op: char) -> Result<(), String> {
        if self.operation.replace(op).is_some() {
            return Err("Choose only one tar operation".into());
        }
        Ok(())
    }
    fn value(&mut self, flag: char, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err("A tar option value cannot be empty".into());
        }
        if flag == 'C' && !self.members.is_empty() {
            return Err("Place -C/--directory before source or selected member names; multiple positional directory contexts are unsupported".into());
        }
        let slot = match flag {
            'f' => &mut self.file,
            'C' => &mut self.directory,
            'g' => &mut self.snapshot,
            _ => unreachable!(),
        };
        if slot.replace(value.into()).is_some() {
            return Err("Repeated file/directory/snapshot options are outside this subset".into());
        }
        Ok(())
    }
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut result = Self::default();
        let mut i = 0;
        let mut ended = false;
        while i < args.len() {
            let arg = &args[i];
            if !ended && arg == "--" {
                ended = true;
            } else if !ended && arg.starts_with("--") {
                let (key, value) = arg
                    .split_once('=')
                    .map_or((arg.as_str(), None), |(k, v)| (k, Some(v)));
                match key {
                    "--create" | "--extract" | "--list" if value.is_none() => {
                        result.operation(match key {
                            "--create" => 'c',
                            "--extract" => 'x',
                            _ => 't',
                        })?
                    }
                    "--verbose" if value.is_none() => result.verbose = true,
                    "--same-permissions" | "--preserve-permissions" if value.is_none() => {}
                    "--incremental" if value.is_none() => result.incremental = true,
                    "--file" | "--directory" | "--listed-incremental" => {
                        let value = if let Some(v) = value {
                            v
                        } else {
                            i += 1;
                            args.get(i).ok_or("Missing tar option value")?
                        };
                        result.value(
                            match key {
                                "--file" => 'f',
                                "--directory" => 'C',
                                _ => 'g',
                            },
                            value,
                        )?;
                    }
                    _ => {
                        return Err(format!(
                            "Unsupported tar option {key}; no archive was changed"
                        ));
                    }
                }
            } else if !ended && arg.starts_with('-') && arg != "-" {
                for (pos, c) in arg[1..].char_indices() {
                    match c {
                        'c' | 'x' | 't' => result.operation(c)?,
                        'v' => result.verbose = true,
                        'p' => {} // Already the fixed simulated superuser's default.
                        'G' => result.incremental = true,
                        'f' | 'C' | 'g' => {
                            let tail = &arg[pos + 2..];
                            let value = if tail.is_empty() {
                                i += 1;
                                args.get(i).ok_or("Missing tar option value")?
                            } else {
                                tail
                            };
                            result.value(c, value)?;
                            break;
                        }
                        _ => {
                            return Err(format!(
                                "Unsupported tar option -{c}; no archive was changed"
                            ));
                        }
                    }
                }
            } else {
                result.members.push(arg.clone());
            }
            i += 1;
        }
        Ok(result)
    }
}
