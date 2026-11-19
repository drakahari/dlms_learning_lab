//! Optional bounded allow-transfer fragment for the authenticated transfer exercise.
use super::{Parser, Token, tokens};
use crate::{
    automation_workshop::authenticated_transfer::{Policy, Task},
    dns,
};
use std::net::Ipv4Addr;

pub fn parse_transfer_fragment(text: &str, task: &Task) -> Result<Policy, String> {
    let mut parser = Parser {
        tokens: tokens(text)?,
        at: 0,
    };
    parser.expect(Token::Word("allow-transfer"))?;
    parser.expect(Token::Open)?;
    let policy=match parser.take()? {
        Token::Word("none")=>Policy::Deny,
        Token::Word("key")=>{
            let name=match parser.take()? {
                Token::Word(name)|Token::Quoted(name)=>name,
                _=>return Err("Name one of this lab's provisioned keys after key.".into()),
            };
            let canonical=dns::canonical_name(name).map_err(|error|error.message)?;
            let reference=task.approved.iter().find(|key|key.name==canonical)
                .ok_or("That key name has no provisioned primary reference in this exercise.")?;
            Policy::VerifiedKey{reference:reference.clone()}
        }
        Token::Word(address) if address.parse::<Ipv4Addr>().ok()==Some(task.secondary_address)=>Policy::SourceAddress,
        _=>return Err("Use one supplied key, none, or the supplied secondary address in this limited allow-transfer fragment.".into()),
    };
    parser.expect(Token::Semi)?;
    parser.expect(Token::Close)?;
    parser.expect(Token::Semi)?;
    if parser.peek().is_some() {
        return Err(
            "Only one allow-transfer statement is supported in this virtual zone editor.".into(),
        );
    }
    Ok(policy)
}
