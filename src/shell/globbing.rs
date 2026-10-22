//! Pure C-locale matching over one flat virtual filename; never host paths.
#[derive(Debug)]
enum Atom {
    Byte(u8),
    Any,
    Star,
}
pub(super) struct Pattern {
    atoms: Vec<Atom>,
    pub absolute: bool,
}
impl Pattern {
    pub fn parse(text: &str, unquoted: &[bool]) -> Result<Option<Self>, String> {
        let bytes = text.as_bytes();
        debug_assert_eq!(bytes.len(), unquoted.len());
        let active = |i: usize| unquoted[i];
        let mut bracket = false;
        for (i, &byte) in bytes.iter().enumerate() {
            if active(i) && byte == b'[' {
                bracket = true;
            }
            if active(i) && byte == b']' && bracket {
                return Err("Unsupported bracket filename pattern; this simulator supports unquoted * and ? only. Quote literal brackets.".into());
            }
        }
        if !(0..bytes.len()).any(|i| active(i) && matches!(bytes[i], b'*' | b'?')) {
            return Ok(None);
        }
        if bytes.len() > 256 {
            return Err("Filename pattern limit exceeded (256 bytes)".into());
        }
        if (0..bytes.len()).any(|i| active(i) && bytes[i] == b'\\') {
            return Err("Unsupported unquoted backslash in a filename pattern; quote the value or use a simple * or ? pattern".into());
        }
        let absolute = text.starts_with("/work/");
        let offset = if absolute { 6 } else { 0 };
        if bytes[offset..].contains(&b'/') || bytes[offset..].is_empty() {
            return Err("Filename patterns support flat virtual /work names only; directory patterns and other locations are unsupported".into());
        }
        let mut atoms = Vec::new();
        for (i, &byte) in bytes.iter().enumerate().skip(offset) {
            let atom = match (byte, active(i)) {
                (b'*', true) => {
                    if matches!(atoms.last(), Some(Atom::Star)) {
                        return Err("Unsupported ** filename pattern; use one * in the flat virtual workspace".into());
                    }
                    Atom::Star
                }
                (b'?', true) => Atom::Any,
                (byte, _) => Atom::Byte(byte),
            };
            atoms.push(atom);
        }
        Ok(Some(Self { atoms, absolute }))
    }
    pub fn matches(
        &self,
        name: &str,
        mut charge: impl FnMut(usize) -> Result<(), String>,
    ) -> Result<bool, String> {
        let bytes = name.as_bytes();
        if bytes.first() == Some(&b'.') && !matches!(self.atoms.first(), Some(Atom::Byte(b'.'))) {
            return Ok(false);
        }
        let (mut p, mut n) = (0, 0);
        let mut star = None;
        let mut retry = 0;
        while n < bytes.len() {
            charge(1)?;
            match self.atoms.get(p) {
                Some(Atom::Byte(b)) if *b == bytes[n] => {
                    p += 1;
                    n += 1;
                }
                Some(Atom::Any) => {
                    p += 1;
                    n += 1;
                }
                Some(Atom::Star) => {
                    star = Some(p);
                    p += 1;
                    retry = n;
                }
                _ => {
                    if let Some(s) = star {
                        retry += 1;
                        n = retry;
                        p = s + 1;
                    } else {
                        return Ok(false);
                    }
                }
            }
        }
        while matches!(self.atoms.get(p), Some(Atom::Star)) {
            charge(1)?;
            p += 1;
        }
        Ok(p == self.atoms.len())
    }
}
