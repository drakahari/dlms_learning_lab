//! Original finite2026 conversion foundation; no curriculum registration yet.
//! Offsets from IANA tzdb2026f facts. No upstream code, clock, file or process API.
use serde::{Deserialize, Serialize};
pub mod command;
pub mod format;
#[cfg(test)]
mod tests;
pub type Result<T> = std::result::Result<T, String>;
pub const FIRST: i64 = 1_767_225_600; //2026-01-01T00:00:00Z
pub const AFTER: i64 = 1_798_761_600; //2027-01-01T00:00:00Z
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Utc,
    Berlin,
    NewYork,
    Kolkata,
}
impl Zone {
    pub const ALL: [Self; 4] = [Self::Utc, Self::Berlin, Self::NewYork, Self::Kolkata];
    pub fn name(self) -> &'static str {
        match self {
            Self::Utc => "UTC",
            Self::Berlin => "Europe/Berlin",
            Self::NewYork => "America/New_York",
            Self::Kolkata => "Asia/Kolkata",
        }
    }
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
        "UTC"|"Etc/UTC"=>Ok(Self::Utc),"Europe/Berlin"=>Ok(Self::Berlin),
        "America/New_York"=>Ok(Self::NewYork),"Asia/Kolkata"=>Ok(Self::Kolkata),
        _=>Err("That time zone is outside this activity's supplied inventory. No host zone is read.".into()),
    }
    }
    pub fn at(self, epoch: i64) -> Result<Local> {
        require_epoch(epoch)?;
        // Explicit2026 UTC transition instants, not a universal future rule.
        let berlin_start = Civil::new(2026, 3, 29, 1, 0, 0)?.epoch()?;
        let berlin_end = Civil::new(2026, 10, 25, 1, 0, 0)?.epoch()?;
        let ny_start = Civil::new(2026, 3, 8, 7, 0, 0)?.epoch()?;
        let ny_end = Civil::new(2026, 11, 1, 6, 0, 0)?.epoch()?;
        let (offset, abbreviation) = match self {
            Self::Utc => (0, "UTC"),
            Self::Kolkata => (19_800, "IST"),
            Self::Berlin if (berlin_start..berlin_end).contains(&epoch) => (7200, "CEST"),
            Self::Berlin => (3600, "CET"),
            Self::NewYork if (ny_start..ny_end).contains(&epoch) => (-14_400, "EDT"),
            Self::NewYork => (-18_000, "EST"),
        };
        Ok(Local {
            civil: Civil::from_epoch(epoch + i64::from(offset))?,
            offset,
            abbreviation,
        })
    }
}
pub fn require_epoch(epoch: i64) -> Result<()> {
    if (FIRST..AFTER).contains(&epoch) {
        Ok(())
    } else {
        Err("This fixed activity supports UTC instants in2026 only; other years and leap seconds are outside its modeled data.".into())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Civil {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
fn leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}
fn month_days(y: i32, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28 + u8::from(leap(y)),
        _ => 0,
    }
}
impl Civil {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Result<Self> {
        if !(1970..=2100).contains(&year)
            || !(1..=12).contains(&month)
            || day == 0
            || day > month_days(year, month)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return Err("Use a real calendar date and ordinary hours, minutes and seconds.".into());
        }
        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }
    pub fn epoch(self) -> Result<i64> {
        Self::new(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        )?;
        let mut days = 0i64;
        for y in 1970..self.year {
            days += 365 + i64::from(leap(y));
        }
        for m in 1..self.month {
            days += i64::from(month_days(self.year, m));
        }
        days += i64::from(self.day) - 1;
        Ok(days * 86_400
            + i64::from(self.hour) * 3600
            + i64::from(self.minute) * 60
            + i64::from(self.second))
    }
    pub fn from_epoch(epoch: i64) -> Result<Self> {
        if !(0..=4_133_980_799).contains(&epoch) {
            return Err("Calendar conversion is outside the bounded1970–2100 helper.".into());
        }
        let mut days = epoch / 86_400;
        let seconds = epoch % 86_400;
        let mut y = 1970;
        while days >= 365 + i64::from(leap(y)) {
            days -= 365 + i64::from(leap(y));
            y += 1;
        }
        let mut m = 1;
        while days >= i64::from(month_days(y, m)) {
            days -= i64::from(month_days(y, m));
            m += 1;
        }
        Self::new(
            y,
            m,
            u8::try_from(days + 1).map_err(|_| "Calendar day overflow")?,
            u8::try_from(seconds / 3600).map_err(|_| "Hour overflow")?,
            u8::try_from(seconds % 3600 / 60).map_err(|_| "Minute overflow")?,
            u8::try_from(seconds % 60).map_err(|_| "Second overflow")?,
        )
    }
    pub fn iso(self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Local {
    pub civil: Civil,
    pub offset: i32,
    pub abbreviation: &'static str,
}
impl Local {
    pub fn numeric_offset(&self, colon: bool) -> String {
        let sign = if self.offset < 0 { '-' } else { '+' };
        let n = self.offset.unsigned_abs();
        if colon {
            format!("{sign}{:02}:{:02}", n / 3600, n % 3600 / 60)
        } else {
            format!("{sign}{:02}{:02}", n / 3600, n % 3600 / 60)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub tz: Option<String>,
}
impl Environment {
    pub fn effective(&self, configured: Zone) -> Result<Zone> {
        match self.tz.as_deref() {
            None => Ok(configured),
            Some("") => Ok(Zone::Utc),
            Some(s) => Zone::parse(s),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if let Some(s) = &self.tz {
            if s.len() > 64 {
                return Err("Time-zone override exceeds64bytes.".into());
            }
            if !s.is_empty() {
                Zone::parse(s)?;
            }
        }
        Ok(())
    }
}
