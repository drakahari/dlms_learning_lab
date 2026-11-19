//! Typed, bounded parsing for the documented Sieve filtering subset.
use super::sieve_lex::{Fault, Kind, Located, Token, fail, lex};
use std::collections::BTreeSet;
pub const MAX_DEPTH: usize = 16;
pub const MAX_LIST: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Octet,
    AsciiCasemap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Match {
    Is,
    Contains,
    Matches,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressPart {
    All,
    Local,
    Domain,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Test {
    Envelope {
        comparison: Comparison,
        mode: Match,
        part: AddressPart,
        names: Vec<String>,
        keys: Vec<String>,
    },
    Address {
        comparison: Comparison,
        mode: Match,
        part: AddressPart,
        names: Vec<String>,
        keys: Vec<String>,
    },
    Constant(bool),
    Not(Box<Test>),
    All(Vec<Test>),
    Any(Vec<Test>),
    Exists(Vec<String>),
    Header {
        comparison: Comparison,
        mode: Match,
        names: Vec<String>,
        keys: Vec<String>,
    },
    Size {
        over: bool,
        bytes: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement {
    Keep,
    Discard,
    Stop,
    FileInto(String),
    Redirect(String),
    Reject(String),
    Vacation(Box<crate::sieve_vacation_policy::Request>),
    If {
        branches: Vec<(Test, Vec<Statement>)>,
        otherwise: Vec<Statement>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub(crate) statements: Vec<Statement>,
}
pub fn parse(source: &str) -> Result<Program, Fault> {
    parse_profile(source, false, false, false)
}
/// Address-enabled profile; callers must disclose its bounded mailbox syntax.
pub fn parse_address(source: &str) -> Result<Program, Fault> {
    parse_profile(source, true, false, false)
}
pub fn parse_delivery(source: &str) -> Result<Program, Fault> {
    parse_profile(source, true, true, false)
}
pub fn parse_vacation(source: &str) -> Result<Program, Fault> {
    parse_profile(source, true, true, true)
}
fn parse_profile(
    source: &str,
    address_tests: bool,
    delivery_actions: bool,
    vacation_actions: bool,
) -> Result<Program, Fault> {
    let mut p = Parser {
        address_tests,
        delivery_actions,
        vacation_actions,
        tokens: lex(source)?,
        at: 0,
        end: source.len(),
        required: BTreeSet::new(),
    };
    while p.name_is("require") {
        p.at += 1;
        for capability in p.strings()? {
            if capability != "fileinto"
                && !(p.delivery_actions && matches!(capability.as_str(), "envelope" | "reject"))
                && !(p.vacation_actions && capability == "vacation")
            {
                return p.error(Kind::Unsupported,if p.vacation_actions {"This profile supports fileinto, envelope, reject and plain vacation. Other required extensions are unsupported."}else if p.delivery_actions {"Supported declared extensions here are fileinto, envelope and reject; other capabilities prevent running."}else{"Only the fileinto extension is supported here. Other required capabilities prevent this script from running."});
            }
            p.required.insert(capability);
        }
        p.want(Token::Semi, "End require with a semicolon.")?;
    }
    let statements = p.sequence(0, false)?;
    Ok(Program { statements })
}
struct Parser {
    address_tests: bool,
    delivery_actions: bool,
    vacation_actions: bool,
    tokens: Vec<Located>,
    at: usize,
    end: usize,
    required: BTreeSet<String>,
}
impl Parser {
    fn offset(&self) -> usize {
        self.tokens.get(self.at).map_or(self.end, |t| t.offset)
    }
    fn error<T>(&self, kind: Kind, message: &str) -> Result<T, Fault> {
        fail(kind, self.offset(), message)
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at).map(|t| &t.token)
    }
    fn name_is(&self, name: &str) -> bool {
        matches!(self.peek(),Some(Token::Name(n))if n==name)
    }
    fn take(&mut self, token: &Token) -> bool {
        if self.peek() == Some(token) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn want(&mut self, token: Token, message: &str) -> Result<(), Fault> {
        if self.take(&token) {
            Ok(())
        } else {
            self.error(Kind::Syntax, message)
        }
    }
    fn text(&mut self) -> Result<String, Fault> {
        if let Some(Token::Text(s)) = self.peek() {
            let s = s.clone();
            self.at += 1;
            Ok(s)
        } else {
            self.error(Kind::Syntax, "Use a double-quoted string.")
        }
    }
    fn strings(&mut self) -> Result<Vec<String>, Fault> {
        if !self.take(&Token::ListOpen) {
            return self.text().map(|s| vec![s]);
        }
        let mut values = vec![self.text()?];
        while self.take(&Token::Comma) {
            if values.len() == MAX_LIST {
                return self.error(Kind::Limit, "A string list may contain at most32 entries.");
            }
            values.push(self.text()?);
        }
        self.want(Token::ListClose, "Close the string list with ].")?;
        Ok(values)
    }
    fn sequence(&mut self, depth: usize, in_block: bool) -> Result<Vec<Statement>, Fault> {
        if depth > MAX_DEPTH {
            return self.error(Kind::Limit, "The block nesting limit is16.");
        }
        let mut statements = Vec::new();
        while self.peek().is_some() {
            if self.peek() == Some(&Token::Close) {
                if in_block {
                    self.at += 1;
                    return Ok(statements);
                }
                return self.error(Kind::Syntax, "This closing brace has no opening block.");
            }
            if self.name_is("if") {
                self.at += 1;
                let test = self.test(0)?;
                self.want(Token::Open, "Open the rule body with {.")?;
                let mut branches = vec![(test, self.sequence(depth + 1, true)?)];
                while self.name_is("elsif") {
                    self.at += 1;
                    let test = self.test(0)?;
                    self.want(Token::Open, "Open the elsif body with {.")?;
                    branches.push((test, self.sequence(depth + 1, true)?));
                }
                let otherwise = if self.name_is("else") {
                    self.at += 1;
                    self.want(Token::Open, "Use else { ... }, not else if.")?;
                    self.sequence(depth + 1, true)?
                } else {
                    Vec::new()
                };
                statements.push(Statement::If {
                    branches,
                    otherwise,
                });
                continue;
            }
            let Some(Token::Name(name)) = self.peek() else {
                return self.error(Kind::Syntax, "Start with a Sieve action or if test.");
            };
            let name = name.clone();
            self.at += 1;
            let action=match name.as_str(){
                "keep"=>Statement::Keep,"discard"=>Statement::Discard,"stop"=>Statement::Stop,
                "fileinto"=>{
                    if !self.required.contains("fileinto"){return self.error(Kind::Syntax,"Declare require \"fileinto\"; at the start before using fileinto.");}
                    if matches!(self.peek(),Some(Token::Tag(_))){return self.error(Kind::Unsupported,"fileinto options such as :copy are outside this subset.");}
                    Statement::FileInto(self.text()?)
                },
                "redirect" if self.delivery_actions=>{
                    if matches!(self.peek(),Some(Token::Tag(_))){return self.error(Kind::Unsupported,"Redirect options such as :copy are outside this delivery subset.");}
                    Statement::Redirect(self.text()?)
                },
                "reject" if self.delivery_actions=>{
                    if !self.required.contains("reject"){return self.error(Kind::Syntax,"Declare require \"reject\"; before using reject.");}
                    if matches!(self.peek(),Some(Token::Tag(_))){return self.error(Kind::Unsupported,"Reject options are outside this delivery subset.");}
                    Statement::Reject(self.text()?)
                },
                "vacation" if self.vacation_actions=>{self.vacation()?},
                "require"=>return self.error(Kind::Syntax,"Put require declarations at the start, before any rule or action."),
                "elsif"|"else"=>return self.error(Kind::Syntax,"An elsif or else must immediately follow its if rule."),
                _=>return self.error(Kind::Unsupported,if self.vacation_actions {"This command is outside the plain vacation profile. Supported actions are keep, fileinto, redirect, reject, vacation, discard and stop; declare required extensions first."}else if self.delivery_actions {"This command is outside the delivery subset. Available actions: keep, fileinto, redirect, reject, discard and stop; envelope and reject require their declarations."}else{"This command is outside the supported Sieve subset. Available actions: keep, fileinto, discard and stop."}),
            };
            if matches!(self.peek(), Some(Token::Tag(_))) {
                return self.error(
                    Kind::Unsupported,
                    "Options for this action are outside the supported subset.",
                );
            }
            self.want(Token::Semi, "End this action with a semicolon.")?;
            statements.push(action);
        }
        if in_block {
            return self.error(Kind::Syntax, "Close this rule body with }.");
        }
        Ok(statements)
    }
    fn vacation(&mut self) -> Result<Statement, Fault> {
        if !self.required.contains("vacation") {
            return self.error(
                Kind::Syntax,
                "Declare require \"vacation\"; before using vacation.",
            );
        }
        let mut request = crate::sieve_vacation_policy::Request::default();
        let mut seen = BTreeSet::new();
        while let Some(Token::Tag(tag)) = self.peek() {
            let tag = tag.clone();
            self.at += 1;
            if !seen.insert(tag.clone()) {
                return self.error(Kind::Syntax, "Specify each vacation option only once.");
            }
            match tag.as_str() {
                "days"=>{let Some(Token::Number(n))=self.peek()else{return self.error(Kind::Syntax,"Use a nonnegative number of days; values are clamped to this site's1..365day range.");};request.days=Some(*n);self.at+=1;},
                "subject"=>request.subject=Some(self.text()?),
                "from"=>request.from=Some(self.text()?),
                "addresses"=>request.addresses=self.strings()?,
                "handle"=>request.handle=Some(self.text()?),
                _=>return self.error(Kind::Unsupported,"Supported vacation options are :days, :subject, :from, :addresses and :handle. MIME, seconds and variables are outside this profile."),
            }
        }
        request.reason = self.text()?;
        Ok(Statement::Vacation(Box::new(request)))
    }
    fn test(&mut self, depth: usize) -> Result<Test, Fault> {
        if depth > MAX_DEPTH {
            return self.error(Kind::Limit, "The test nesting limit is16.");
        }
        let Some(Token::Name(name)) = self.peek() else {
            return self.error(Kind::Syntax, "Supply a test after if, elsif or not.");
        };
        let name = name.clone();
        self.at += 1;
        match name.as_str(){
            "true"=>Ok(Test::Constant(true)),"false"=>Ok(Test::Constant(false)),
            "not"=>Ok(Test::Not(Box::new(self.test(depth+1)?))),
            "allof"|"anyof"=>{
                self.want(Token::TestOpen,"Put combined tests inside parentheses, separated by commas.")?;
                let mut tests=vec![self.test(depth+1)?];
                while self.take(&Token::Comma){if tests.len()==MAX_LIST{return self.error(Kind::Limit,"A combined test may contain at most32 entries.");}tests.push(self.test(depth+1)?);}
                self.want(Token::TestClose,"Close the test list with ).")?;
                Ok(if name=="allof"{Test::All(tests)}else{Test::Any(tests)})
            },
            "exists"=>self.strings().map(Test::Exists),
            "size"=>{
                let over=match self.peek(){Some(Token::Tag(t))if t=="over"=>true,Some(Token::Tag(t))if t=="under"=>false,_=>return self.error(Kind::Syntax,"Use size :over NUMBER or size :under NUMBER.")};self.at+=1;
                let Some(Token::Number(bytes))=self.peek()else{return self.error(Kind::Syntax,"Give the size as a number, optionally followed by K, M or G.");};
                let bytes=*bytes;self.at+=1;Ok(Test::Size{over,bytes})
            },
            "address"|"envelope" if self.address_tests && (name=="address" || self.delivery_actions) => {
                if name=="envelope" && !self.required.contains("envelope") { return self.error(Kind::Syntax,"Declare require \"envelope\"; before using envelope tests."); }

                let mut mode=None; let mut comparison=None; let mut part=None;
                while let Some(Token::Tag(tag))=self.peek() {
                    let tag=tag.clone(); self.at+=1;
                    match tag.as_str() {
                        "is"|"contains"|"matches" => {
                            if mode.is_some() { return self.error(Kind::Syntax,"Choose one match type for each address test."); }
                            mode=Some(match tag.as_str(){"is"=>Match::Is,"contains"=>Match::Contains,_=>Match::Matches});
                        },
                        "all"|"localpart"|"domain" => {
                            if part.is_some() { return self.error(Kind::Syntax,"Choose one address part: :all, :localpart or :domain."); }
                            part=Some(match tag.as_str(){"localpart"=>AddressPart::Local,"domain"=>AddressPart::Domain,_=>AddressPart::All});
                        },
                        "comparator" => {
                            if comparison.is_some() { return self.error(Kind::Syntax,"Choose one comparator for each address test."); }
                            comparison=Some(match self.text()?.as_str(){"i;octet"=>Comparison::Octet,"i;ascii-casemap"=>Comparison::AsciiCasemap,_=>return self.error(Kind::Unsupported,"Supported comparators are i;octet and i;ascii-casemap.")});
                        },
                        _=>return self.error(Kind::Unsupported,"Unsupported address option. Use :all, :localpart, :domain, :is, :contains, :matches or :comparator."),
                    }
                }
                let names=self.strings()?; let keys=self.strings()?;
                if name=="envelope" {
                    if names.iter().any(|field| !field.eq_ignore_ascii_case("from") && !field.eq_ignore_ascii_case("to")) { return self.error(Kind::Unsupported,"Supported envelope parts are from and to; visible headers are separate inputs."); }
                    return Ok(Test::Envelope { comparison:comparison.unwrap_or(Comparison::AsciiCasemap),mode:mode.unwrap_or(Match::Is),part:part.unwrap_or(AddressPart::All),names,keys });
                }
                Ok(Test::Address { comparison:comparison.unwrap_or(Comparison::AsciiCasemap),mode:mode.unwrap_or(Match::Is),part:part.unwrap_or(AddressPart::All),names,keys })
            },
            "header"=>{
                let mut mode=None;let mut comparison=None;
                while let Some(Token::Tag(tag))=self.peek(){
                    let tag=tag.clone();self.at+=1;
                    match tag.as_str(){
                        "is"|"contains"|"matches"=>{
                            if mode.is_some(){return self.error(Kind::Syntax,"Choose one match type for each header test.");}
                            mode=Some(match tag.as_str(){"is"=>Match::Is,"contains"=>Match::Contains,_=>Match::Matches});
                        },
                        "comparator"=>{
                            if comparison.is_some(){return self.error(Kind::Syntax,"Choose one comparator for each header test.");}
                            comparison=Some(match self.text()?.as_str(){"i;octet"=>Comparison::Octet,"i;ascii-casemap"=>Comparison::AsciiCasemap,_=>return self.error(Kind::Unsupported,"Supported comparators are i;octet and i;ascii-casemap.")});
                        },
                        _=>return self.error(Kind::Unsupported,"Unsupported header option. Use :is, :contains, :matches or :comparator."),
                    }
                }
                let names=self.strings()?;let keys=self.strings()?;
                Ok(Test::Header{comparison:comparison.unwrap_or(Comparison::AsciiCasemap),mode:mode.unwrap_or(Match::Is),names,keys})
            },
            _=>self.error(Kind::Unsupported,"This test is outside the supported subset. Use header, exists, size, true, false, not, allof or anyof."),
        }
    }
}
