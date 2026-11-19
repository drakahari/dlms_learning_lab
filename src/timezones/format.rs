//! Bounded C-locale date formatting, not a full strftime.
use super::{Result, Zone};
pub const DEFAULT: &str = "%a %b %e %H:%M:%S %Z %Y";
pub fn render(zone: Zone, epoch: i64, format: &str) -> Result<String> {
    if format.len() > 256 || format.contains('\0') {
        return Err("Keep the date format within256bytes without zero bytes.".into());
    }
    let local = zone.at(epoch)?;
    let c = local.civil;
    let mut out = String::new();
    let mut chars = format.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        let spec = chars.next().ok_or(
            "The final percent sign needs a supported date field, or %% for a literal percent.",
        )?;
        let value = match spec {
            '%' => "%".into(),
            'n' => "\n".into(),
            't' => "\t".into(),
            'Y' => format!("{:04}", c.year),
            'm' => format!("{:02}", c.month),
            'd' => format!("{:02}", c.day),
            'e' => format!("{:2}", c.day),
            'H' => format!("{:02}", c.hour),
            'M' => format!("{:02}", c.minute),
            'S' => format!("{:02}", c.second),
            'F' => format!("{:04}-{:02}-{:02}", c.year, c.month, c.day),
            'T' => format!("{:02}:{:02}:{:02}", c.hour, c.minute, c.second),
            'R' => format!("{:02}:{:02}", c.hour, c.minute),
            'z' => local.numeric_offset(false),
            'Z' => local.abbreviation.into(),
            's' => epoch.to_string(),
            ':' => {
                if chars.next() != Some('z') {
                    return Err("Only %:z is supported for a colon-separated offset; other flags/modifiers are outside this subset.".into());
                }
                local.numeric_offset(true)
            }
            'a' => {
                let day = c.epoch()?.div_euclid(86_400);
                let index =
                    usize::try_from((day + 4).rem_euclid(7)).map_err(|_| "Weekday bounds")?;
                ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][index].into()
            }
            'b' => [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ][usize::from(c.month - 1)]
            .into(),
            _ => {
                return Err(format!(
                    "Date field %{spec} is outside this activity's supported subset; it has not been approximated."
                ));
            }
        };
        out.push_str(&value);
        if out.len() > 2048 {
            return Err("The formatted date exceeds this activity's output bound.".into());
        }
    }
    Ok(out)
}
