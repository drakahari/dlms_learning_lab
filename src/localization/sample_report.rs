//! Original demonstration application: UTF-8 item;cents input, localized report.
//! Not a Linux utility, gettext implementation or general CSV parser.
use super::{
    Category, Environment, Locale,
    command::Output,
    encoding::{self, Encoding},
};
pub fn render(input: &[u8], env: &Environment) -> Result<Output, String> {
    if input.len() > encoding::MAX_INPUT {
        return Err("Sample input exceeds16KiB.".into());
    }
    let text=std::str::from_utf8(input).map_err(|_|"The sample application expects a UTF-8 input document; changing the locale does not convert file bytes.")?;
    let mut lines = text.lines();
    if lines.next() != Some("item;cents") {
        return Err("The sample input starts with item;cents, followed by unquoted item;integer-cents rows.".into());
    }
    let mut records = vec![];
    for line in lines {
        if records.len() == 16 {
            return Err("At most16 sample rows are supported.".into());
        }
        let (item,cents)=line.split_once(';').ok_or("Each sample row needs one item and an integer number of cents, separated by a semicolon.")?;
        if item.is_empty()
            || item.len() > 96
            || item.chars().any(char::is_control)
            || item.contains('"')
        {
            return Err("Use a nonempty, unquoted item label within96UTF-8 bytes.".into());
        }
        if cents.is_empty()
            || cents.starts_with('+')
            || cents
                .bytes()
                .enumerate()
                .any(|(n, b)| !b.is_ascii_digit() && !(n == 0 && b == b'-'))
        {
            return Err("Amounts are signed integer cents, such as1250or-75; no locale-dependent decimal input is parsed.".into());
        }
        let amount = cents
            .parse::<i64>()
            .map_err(|_| "The sample amount exceeds the supported integer range.")?;
        records.push((item, super::report_number(amount, env)?));
    }
    if records.is_empty() {
        return Err("Supply at least one item row.".into());
    }
    let mut text = if env.effective(Category::Messages) == Locale::DeDeUtf8 {
        "Bestand\nArtikel;Betrag\n".to_string()
    } else {
        "Stock\nItem;Amount\n".to_string()
    };
    for (item, amount) in records {
        text.push_str(item);
        text.push(';');
        text.push_str(&amount);
        text.push('\n');
    }
    let target = if env.effective(Category::Ctype) == Locale::C {
        Encoding::Ascii
    } else {
        Encoding::Utf8
    };
    let converted = encoding::convert(text.as_bytes(), Encoding::Utf8, target)?;
    let stderr=converted.failure.as_ref().map_or(String::new(),|f|format!("The sample report cannot encode a character at output byte {} using the current character setting. Inspect LC_CTYPE and any overriding LC_ALL; the input document itself is unchanged.",f.position));
    Ok(Output {
        stdout: converted.bytes,
        status: if converted.failure.is_some() { 1 } else { 0 },
        stderr,
    })
}
#[cfg(test)]
mod tests {
    use super::super::{Variable, configuration};
    use super::*;
    const AVAILABLE: [Locale; 4] = [Locale::C, Locale::Cutf8, Locale::EnUsUtf8, Locale::DeDeUtf8];
    const SAMPLE: &[u8] = "item;cents\nCafé;1250\nSpare;-75\nEmpty;0\n".as_bytes();
    #[test]
    fn localized_messages_numeric_override_and_utf8_names_produce_actual_rows() {
        let env = configuration("LANG=de_DE.UTF-8\nLC_NUMERIC=C\n", &AVAILABLE).unwrap();
        let r = render(SAMPLE, &env).unwrap();
        assert_eq!(r.status, 0);
        assert!(r.stderr.is_empty());
        assert_eq!(
            r.stdout,
            "Bestand\nArtikel;Betrag\nCafé;12.50\nSpare;-0.75\nEmpty;0.00\n".as_bytes()
        );
    }
    #[test]
    fn equivalent_effective_category_settings_produce_the_same_application_behavior() {
        let a = configuration("LANG=de_DE.UTF-8\nLC_NUMERIC=C\n", &AVAILABLE).unwrap();
        let b = configuration(
            "LANG=C.UTF-8\nLC_MESSAGES=de_DE.UTF-8\nLC_NUMERIC=en_US.UTF-8\n",
            &AVAILABLE,
        )
        .unwrap();
        assert_eq!(render(SAMPLE, &a).unwrap(), render(SAMPLE, &b).unwrap());
    }
    #[test]
    fn blanket_all_override_can_break_encoding_even_if_numbers_use_a_dot() {
        let mut env = configuration("LANG=de_DE.UTF-8\nLC_NUMERIC=C\n", &AVAILABLE).unwrap();
        env.set(Variable::All, Some(Locale::C));
        let r = render(SAMPLE, &env).unwrap();
        assert_eq!(r.status, 1);
        assert_eq!(r.stdout, b"Stock\nItem;Amount\nCaf");
        assert!(r.stderr.contains("LC_CTYPE") && r.stderr.contains("LC_ALL"));
        env.unset(Variable::All);
        assert_eq!(render(SAMPLE, &env).unwrap().status, 0);
    }
    #[test]
    fn numeric_policy_changes_actual_values_for_distinct_inputs_without_rewriting_names() {
        let env = configuration("LANG=de_DE.UTF-8", &AVAILABLE).unwrap();
        let a = render(b"item;cents\nSmall;1\n", &env).unwrap();
        let b = render(b"item;cents\nLarge;1012500\n", &env).unwrap();
        assert_eq!(a.stdout, b"Bestand\nArtikel;Betrag\nSmall;0,01\n");
        assert_eq!(b.stdout, b"Bestand\nArtikel;Betrag\nLarge;10125,00\n");
        assert_ne!(a.stdout, b.stdout);
    }
    #[test]
    fn malformed_unknown_and_oversized_sample_data_refuse_without_guessing_a_codec() {
        let env = Environment::default();
        for source in [
            b"item;cents\nCaf\xe9;10\n".as_slice(),
            b"name;cents\nx;10",
            b"item;cents\n",
            b"item;cents\nx;1.5",
            b"item;cents\nx;1,5",
            b"item;cents\nx;99999999999999999999999",
            b"item;cents\nx;+5",
            b"item;cents\n\"quoted\";10",
            b"item;cents\nx;5;6",
        ] {
            assert!(render(source, &env).is_err(), "{source:?}");
        }
        assert!(render(&vec![b'A'; encoding::MAX_INPUT + 1], &env).is_err());
        assert!(
            render(
                format!("item;cents\n{}", "x;1\n".repeat(17)).as_bytes(),
                &env
            )
            .is_err()
        );
    }
}
