//! Bounded typing probes with independently documented virtual settings.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextSize {
    Standard,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    None,
    SlowKeys300Ms,
    BounceKeys300Ms,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub text: TextSize,
    pub high_contrast: bool,
    pub sticky_keys: bool,
    pub typing_filter: Filter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    LeftShift,
    A,
    B,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalTap {
    pub key: Key,
    pub pressed_at_ms: u32,
    pub released_at_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    PendingModifier,
    ReleasedModifier,
    Printed(char),
    RejectedTooShort,
    RejectedRepeated,
}

/// The exercise supplies ordered, nonoverlapping events, with one Shift use
/// and independent probes for each filter. It does not synthesize host keys.
pub fn preview(settings: Settings, events: &[PhysicalTap]) -> Option<Vec<Event>> {
    if events.len() > 20 {
        return None;
    }
    let mut last_release = None;
    let mut prior = [None::<u32>; 3];
    let mut latched = false;
    let mut result = Vec::with_capacity(events.len());
    for tap in events {
        if tap.pressed_at_ms >= tap.released_at_ms
            || tap.released_at_ms > 10000
            || last_release.is_some_and(|earlier| tap.pressed_at_ms < earlier)
        {
            return None;
        }
        last_release = Some(tap.released_at_ms);
        let index = tap.key as usize;
        if settings.typing_filter == Filter::SlowKeys300Ms
            && tap.released_at_ms - tap.pressed_at_ms < 300
        {
            result.push(Event::RejectedTooShort);
            continue;
        }
        if settings.typing_filter == Filter::BounceKeys300Ms
            && prior[index].is_some_and(|previous| tap.pressed_at_ms - previous < 300)
        {
            result.push(Event::RejectedRepeated);
            continue;
        }
        prior[index] = last_release;
        match tap.key {
            Key::LeftShift if settings.sticky_keys => {
                latched = true;
                result.push(Event::PendingModifier);
            }
            Key::LeftShift => result.push(Event::ReleasedModifier),
            Key::A | Key::B => {
                let lower = if tap.key == Key::A { 'a' } else { 'b' };
                result.push(Event::Printed(if latched {
                    lower.to_ascii_uppercase()
                } else {
                    lower
                }));
                latched = false;
            }
        }
    }
    Some(result)
}
