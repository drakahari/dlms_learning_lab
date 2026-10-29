//! Streaming visitors for the six new tagged one-off enums only.
//! A tag may appear anywhere. Before it arrives, each recognized union field
//! uses its largest documented variant bound; afterward incompatible keys are
//! rejected before their values. Unknown/duplicate keys always fail immediately.
//! Legacy outer State/Goal Content buffering is a separate raw-admitted boundary.
use super::*;
use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
struct Text<const N: usize>;
impl<'de, const N: usize> DeserializeSeed<'de> for Text<N> {
    type Value = String;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
        crate::cron::text::<D, N>(d)
    }
}
struct Environment;
impl<'de> DeserializeSeed<'de> for Environment {
    type Value = BTreeMap<String, String>;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        environment_map(d)
    }
}
struct Selectors;
impl<'de> DeserializeSeed<'de> for Selectors {
    type Value = Vec<Selector>;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        bounded::sequence::<D, Selector, 16>(d)
    }
}

impl<'de> Deserialize<'de> for Timing {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Tagged;
        impl<'de> Visitor<'de> for Tagged {
            type Value = Timing;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded tagged Timing")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut tag: Option<String> = None;
                let mut submitted: Option<Moment> = None;
                let mut due: Option<Moment> = None;
                let mut minutes: Option<u32> = None;
                while let Some(key) = map.next_key_seed(Text::<32>)? {
                    match key.as_str() {
                        "kind" => {
                            if tag.is_some() {
                                return Err(A::Error::custom("Duplicate one-off tag before value"));
                            }
                            let value = map.next_value_seed(Text::<32>)?;
                            match value.as_str() {
                                "Absolute" => {
                                    if minutes.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "AfterSubmission" => {
                                    if submitted.is_some() || due.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                _ => return Err(A::Error::custom("Unknown one-off tag")),
                            }
                            tag = Some(value);
                        }
                        "submitted" => {
                            if submitted.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Absolute"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            submitted = Some(map.next_value()?);
                        }
                        "due" => {
                            if due.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Absolute"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            due = Some(map.next_value()?);
                        }
                        "minutes" => {
                            if minutes.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "AfterSubmission"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            minutes = Some(map.next_value()?);
                        }
                        _ => return Err(A::Error::custom("Unknown one-off field before value")),
                    }
                }
                match tag
                    .as_deref()
                    .ok_or_else(|| A::Error::missing_field("kind"))?
                {
                    "Absolute" => Ok(Timing::Absolute {
                        submitted: submitted.ok_or_else(|| A::Error::missing_field("submitted"))?,
                        due: due.ok_or_else(|| A::Error::missing_field("due"))?,
                    }),
                    "AfterSubmission" => Ok(Timing::AfterSubmission {
                        minutes: minutes.ok_or_else(|| A::Error::missing_field("minutes"))?,
                    }),
                    _ => Err(A::Error::custom("Unknown one-off tag")),
                }
            }
        }
        d.deserialize_map(Tagged)
    }
}
impl<'de> Deserialize<'de> for ProbeWindow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Tagged;
        impl<'de> Visitor<'de> for Tagged {
            type Value = ProbeWindow;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded tagged ProbeWindow")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut tag: Option<String> = None;
                let mut start: Option<Moment> = None;
                let mut end: Option<Moment> = None;
                let mut start_minutes: Option<u32> = None;
                let mut end_minutes: Option<u32> = None;
                while let Some(key) = map.next_key_seed(Text::<32>)? {
                    match key.as_str() {
                        "kind" => {
                            if tag.is_some() {
                                return Err(A::Error::custom("Duplicate one-off tag before value"));
                            }
                            let value = map.next_value_seed(Text::<32>)?;
                            match value.as_str() {
                                "Absolute" => {
                                    if start_minutes.is_some() || end_minutes.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "AfterSubmission" => {
                                    if start.is_some() || end.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                _ => return Err(A::Error::custom("Unknown one-off tag")),
                            }
                            tag = Some(value);
                        }
                        "start" => {
                            if start.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Absolute"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            start = Some(map.next_value()?);
                        }
                        "end" => {
                            if end.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Absolute"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            end = Some(map.next_value()?);
                        }
                        "start_minutes" => {
                            if start_minutes.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "AfterSubmission"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            start_minutes = Some(map.next_value()?);
                        }
                        "end_minutes" => {
                            if end_minutes.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "AfterSubmission"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            end_minutes = Some(map.next_value()?);
                        }
                        _ => return Err(A::Error::custom("Unknown one-off field before value")),
                    }
                }
                match tag
                    .as_deref()
                    .ok_or_else(|| A::Error::missing_field("kind"))?
                {
                    "Absolute" => Ok(ProbeWindow::Absolute {
                        start: start.ok_or_else(|| A::Error::missing_field("start"))?,
                        end: end.ok_or_else(|| A::Error::missing_field("end"))?,
                    }),
                    "AfterSubmission" => Ok(ProbeWindow::AfterSubmission {
                        start_minutes: start_minutes
                            .ok_or_else(|| A::Error::missing_field("start_minutes"))?,
                        end_minutes: end_minutes
                            .ok_or_else(|| A::Error::missing_field("end_minutes"))?,
                    }),
                    _ => Err(A::Error::custom("Unknown one-off tag")),
                }
            }
        }
        d.deserialize_map(Tagged)
    }
}
impl<'de> Deserialize<'de> for ExpectedWhen {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Tagged;
        impl<'de> Visitor<'de> for Tagged {
            type Value = ExpectedWhen;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded tagged ExpectedWhen")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut tag: Option<String> = None;
                let mut at: Option<Moment> = None;
                while let Some(key) = map.next_key_seed(Text::<32>)? {
                    match key.as_str() {
                        "kind" => {
                            if tag.is_some() {
                                return Err(A::Error::custom("Duplicate one-off tag before value"));
                            }
                            let value = map.next_value_seed(Text::<32>)?;
                            match value.as_str() {
                                "Absolute" => {}
                                "Dispatch" => {
                                    if at.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                _ => return Err(A::Error::custom("Unknown one-off tag")),
                            }
                            tag = Some(value);
                        }
                        "at" => {
                            if at.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Absolute"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            at = Some(map.next_value()?);
                        }
                        _ => return Err(A::Error::custom("Unknown one-off field before value")),
                    }
                }
                match tag
                    .as_deref()
                    .ok_or_else(|| A::Error::missing_field("kind"))?
                {
                    "Absolute" => Ok(ExpectedWhen::Absolute {
                        at: at.ok_or_else(|| A::Error::missing_field("at"))?,
                    }),
                    "Dispatch" => Ok(ExpectedWhen::Dispatch),
                    _ => Err(A::Error::custom("Unknown one-off tag")),
                }
            }
        }
        d.deserialize_map(Tagged)
    }
}
impl<'de> Deserialize<'de> for Requirement {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Tagged;
        impl<'de> Visitor<'de> for Tagged {
            type Value = Requirement;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded tagged Requirement")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut tag: Option<String> = None;
                let mut source: Option<CaptureSource> = None;
                let mut jobs: Option<Vec<Selector>> = None;
                let mut path: Option<String> = None;
                let mut text: Option<String> = None;
                let mut at: Option<Moment> = None;
                let mut job: Option<Selector> = None;
                let mut values: Option<BTreeMap<String, String>> = None;
                let mut start: Option<Moment> = None;
                let mut end: Option<Moment> = None;
                while let Some(key) = map.next_key_seed(Text::<32>)? {
                    match key.as_str() {
                        "kind" => {
                            if tag.is_some() {
                                return Err(A::Error::custom("Duplicate one-off tag before value"));
                            }
                            let value = map.next_value_seed(Text::<32>)?;
                            match value.as_str() {
                                "QueueExactly" => {
                                    if path.is_some()
                                        || text.is_some()
                                        || at.is_some()
                                        || job.is_some()
                                        || values.is_some()
                                        || start.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "CurrentFile" => {
                                    if source.is_some()
                                        || jobs.is_some()
                                        || at.is_some()
                                        || job.is_some()
                                        || values.is_some()
                                        || start.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Clock" => {
                                    if source.is_some()
                                        || jobs.is_some()
                                        || path.is_some()
                                        || text.is_some()
                                        || job.is_some()
                                        || values.is_some()
                                        || start.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "CapturedEnvironment" => {
                                    if jobs.is_some()
                                        || path.is_some()
                                        || text.is_some()
                                        || at.is_some()
                                        || start.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "RanWindow" => {
                                    if source.is_some()
                                        || jobs.is_some()
                                        || path.is_some()
                                        || text.is_some()
                                        || at.is_some()
                                        || job.is_some()
                                        || values.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Dispatched" => {
                                    if source.is_some()
                                        || jobs.is_some()
                                        || path.is_some()
                                        || text.is_some()
                                        || at.is_some()
                                        || values.is_some()
                                        || start.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                _ => return Err(A::Error::custom("Unknown one-off tag")),
                            }
                            tag = Some(value);
                        }
                        "source" => {
                            if source.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| {
                                !matches!(kind, "QueueExactly" | "CapturedEnvironment")
                            }) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            source = Some(map.next_value()?);
                        }
                        "jobs" => {
                            if jobs.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "QueueExactly"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            jobs = Some(map.next_value_seed(Selectors)?);
                        }
                        "path" => {
                            if path.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "CurrentFile"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            path = Some(map.next_value_seed(Text::<106>)?);
                        }
                        "text" => {
                            if text.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "CurrentFile"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            text = Some(map.next_value_seed(Text::<98304>)?);
                        }
                        "at" => {
                            if at.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| !matches!(kind, "Clock")) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            at = Some(map.next_value()?);
                        }
                        "job" => {
                            if job.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| {
                                !matches!(kind, "CapturedEnvironment" | "Dispatched")
                            }) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            job = Some(map.next_value()?);
                        }
                        "values" => {
                            if values.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "CapturedEnvironment"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            values = Some(map.next_value_seed(Environment)?);
                        }
                        "start" => {
                            if start.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "RanWindow"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            start = Some(map.next_value()?);
                        }
                        "end" => {
                            if end.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "RanWindow"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            end = Some(map.next_value()?);
                        }
                        _ => return Err(A::Error::custom("Unknown one-off field before value")),
                    }
                }
                match tag
                    .as_deref()
                    .ok_or_else(|| A::Error::missing_field("kind"))?
                {
                    "QueueExactly" => Ok(Requirement::QueueExactly {
                        source: source.ok_or_else(|| A::Error::missing_field("source"))?,
                        jobs: jobs.ok_or_else(|| A::Error::missing_field("jobs"))?,
                    }),
                    "CurrentFile" => Ok(Requirement::CurrentFile {
                        path: path.ok_or_else(|| A::Error::missing_field("path"))?,
                        text: text.ok_or_else(|| A::Error::missing_field("text"))?,
                    }),
                    "Clock" => Ok(Requirement::Clock {
                        at: at.ok_or_else(|| A::Error::missing_field("at"))?,
                    }),
                    "CapturedEnvironment" => Ok(Requirement::CapturedEnvironment {
                        source: source.ok_or_else(|| A::Error::missing_field("source"))?,
                        job: job.ok_or_else(|| A::Error::missing_field("job"))?,
                        values: values.ok_or_else(|| A::Error::missing_field("values"))?,
                    }),
                    "RanWindow" => Ok(Requirement::RanWindow {
                        start: start.ok_or_else(|| A::Error::missing_field("start"))?,
                        end: end.ok_or_else(|| A::Error::missing_field("end"))?,
                    }),
                    "Dispatched" => Ok(Requirement::Dispatched {
                        job: job.ok_or_else(|| A::Error::missing_field("job"))?,
                    }),
                    _ => Err(A::Error::custom("Unknown one-off tag")),
                }
            }
        }
        d.deserialize_map(Tagged)
    }
}
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Tagged;
        impl<'de> Visitor<'de> for Tagged {
            type Value = Action;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded tagged Action")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut tag: Option<String> = None;
                let mut text: Option<String> = None;
                let mut path: Option<String> = None;
                let mut actor: Option<String> = None;
                let mut values: Option<BTreeMap<String, String>> = None;
                let mut end: Option<Moment> = None;
                while let Some(key) = map.next_key_seed(Text::<32>)? {
                    match key.as_str() {
                        "kind" => {
                            if tag.is_some() {
                                return Err(A::Error::custom("Duplicate one-off tag before value"));
                            }
                            let value = map.next_value_seed(Text::<32>)?;
                            match value.as_str() {
                                "CommandDraft" => {
                                    if path.is_some()
                                        || actor.is_some()
                                        || values.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "WindowDraft" => {
                                    if path.is_some()
                                        || actor.is_some()
                                        || values.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Edit" => {
                                    if actor.is_some() || values.is_some() || end.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Save" => {
                                    if text.is_some()
                                        || actor.is_some()
                                        || values.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Select" => {
                                    if text.is_some()
                                        || actor.is_some()
                                        || values.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Actor" => {
                                    if text.is_some()
                                        || path.is_some()
                                        || values.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Environment" => {
                                    if text.is_some()
                                        || path.is_some()
                                        || actor.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Command" => {
                                    if path.is_some()
                                        || actor.is_some()
                                        || values.is_some()
                                        || end.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Run" => {
                                    if text.is_some()
                                        || path.is_some()
                                        || actor.is_some()
                                        || values.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                _ => return Err(A::Error::custom("Unknown one-off tag")),
                            }
                            let limit = match value.as_str() {
                                "WindowDraft" => Some(32),
                                "Command" | "CommandDraft" => Some(2048),
                                _ => None,
                            };
                            if limit.is_some_and(|limit| {
                                text.as_ref().is_some_and(|text| text.len() > limit)
                            }) {
                                return Err(A::Error::custom("One-off variant text limit"));
                            }
                            tag = Some(value);
                        }
                        "text" => {
                            if text.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| {
                                !matches!(kind, "CommandDraft" | "WindowDraft" | "Edit" | "Command")
                            }) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            text = Some(match tag.as_deref() {
                                Some("WindowDraft") => map.next_value_seed(Text::<32>)?,
                                Some("Command" | "CommandDraft") => {
                                    map.next_value_seed(Text::<2048>)?
                                }
                                _ => map.next_value_seed(Text::<98304>)?,
                            });
                        }
                        "path" => {
                            if path.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Edit" | "Save" | "Select"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            path = Some(map.next_value_seed(Text::<106>)?);
                        }
                        "actor" => {
                            if actor.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| !matches!(kind, "Actor")) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            actor = Some(map.next_value_seed(Text::<32>)?);
                        }
                        "values" => {
                            if values.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Environment"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            values = Some(map.next_value_seed(Environment)?);
                        }
                        "end" => {
                            if end.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| !matches!(kind, "Run")) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            end = Some(map.next_value()?);
                        }
                        _ => return Err(A::Error::custom("Unknown one-off field before value")),
                    }
                }
                match tag
                    .as_deref()
                    .ok_or_else(|| A::Error::missing_field("kind"))?
                {
                    "CommandDraft" => {
                        let text = text.ok_or_else(|| A::Error::missing_field("text"))?;
                        if text.len() > 2048 {
                            return Err(A::Error::custom("One-off variant text limit"));
                        }
                        Ok(Action::CommandDraft { text })
                    }
                    "WindowDraft" => {
                        let text = text.ok_or_else(|| A::Error::missing_field("text"))?;
                        if text.len() > 32 {
                            return Err(A::Error::custom("One-off variant text limit"));
                        }
                        Ok(Action::WindowDraft { text })
                    }
                    "Edit" => Ok(Action::Edit {
                        path: path.ok_or_else(|| A::Error::missing_field("path"))?,
                        text: text.ok_or_else(|| A::Error::missing_field("text"))?,
                    }),
                    "Save" => Ok(Action::Save {
                        path: path.ok_or_else(|| A::Error::missing_field("path"))?,
                    }),
                    "Select" => Ok(Action::Select {
                        path: path.ok_or_else(|| A::Error::missing_field("path"))?,
                    }),
                    "Actor" => Ok(Action::Actor {
                        actor: actor.ok_or_else(|| A::Error::missing_field("actor"))?,
                    }),
                    "Environment" => Ok(Action::Environment {
                        values: values.ok_or_else(|| A::Error::missing_field("values"))?,
                    }),
                    "Command" => {
                        let text = text.ok_or_else(|| A::Error::missing_field("text"))?;
                        if text.len() > 2048 {
                            return Err(A::Error::custom("One-off variant text limit"));
                        }
                        Ok(Action::Command { text })
                    }
                    "Run" => Ok(Action::Run {
                        end: end.ok_or_else(|| A::Error::missing_field("end"))?,
                    }),
                    _ => Err(A::Error::custom("Unknown one-off tag")),
                }
            }
        }
        d.deserialize_map(Tagged)
    }
}
impl<'de> Deserialize<'de> for ReferenceAction {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Tagged;
        impl<'de> Visitor<'de> for Tagged {
            type Value = ReferenceAction;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded tagged ReferenceAction")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut tag: Option<String> = None;
                let mut path: Option<String> = None;
                let mut text: Option<String> = None;
                let mut actor: Option<String> = None;
                let mut environment: Option<BTreeMap<String, String>> = None;
                let mut end: Option<Moment> = None;
                while let Some(key) = map.next_key_seed(Text::<32>)? {
                    match key.as_str() {
                        "kind" => {
                            if tag.is_some() {
                                return Err(A::Error::custom("Duplicate one-off tag before value"));
                            }
                            let value = map.next_value_seed(Text::<32>)?;
                            match value.as_str() {
                                "Write" => {
                                    if actor.is_some() || environment.is_some() || end.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Command" => {
                                    if path.is_some() || end.is_some() {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                "Run" => {
                                    if path.is_some()
                                        || text.is_some()
                                        || actor.is_some()
                                        || environment.is_some()
                                    {
                                        return Err(A::Error::custom(
                                            "Incompatible one-off field before next value",
                                        ));
                                    }
                                }
                                _ => return Err(A::Error::custom("Unknown one-off tag")),
                            }
                            let limit = (value == "Command").then_some(2048);
                            if limit.is_some_and(|limit| {
                                text.as_ref().is_some_and(|text| text.len() > limit)
                            }) {
                                return Err(A::Error::custom("One-off variant text limit"));
                            }
                            tag = Some(value);
                        }
                        "path" => {
                            if path.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| !matches!(kind, "Write")) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            path = Some(map.next_value_seed(Text::<106>)?);
                        }
                        "text" => {
                            if text.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Write" | "Command"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            text = Some(if tag.as_deref() == Some("Command") {
                                map.next_value_seed(Text::<2048>)?
                            } else {
                                map.next_value_seed(Text::<98304>)?
                            });
                        }
                        "actor" => {
                            if actor.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Command"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            actor = Some(map.next_value_seed(Text::<32>)?);
                        }
                        "environment" => {
                            if environment.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag
                                .as_deref()
                                .is_some_and(|kind| !matches!(kind, "Command"))
                            {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            environment = Some(map.next_value_seed(Environment)?);
                        }
                        "end" => {
                            if end.is_some() {
                                return Err(A::Error::custom(
                                    "Duplicate one-off field before value",
                                ));
                            }
                            if tag.as_deref().is_some_and(|kind| !matches!(kind, "Run")) {
                                return Err(A::Error::custom(
                                    "Incompatible one-off field before value",
                                ));
                            }
                            end = Some(map.next_value()?);
                        }
                        _ => return Err(A::Error::custom("Unknown one-off field before value")),
                    }
                }
                match tag
                    .as_deref()
                    .ok_or_else(|| A::Error::missing_field("kind"))?
                {
                    "Write" => Ok(ReferenceAction::Write {
                        path: path.ok_or_else(|| A::Error::missing_field("path"))?,
                        text: text.ok_or_else(|| A::Error::missing_field("text"))?,
                    }),
                    "Command" => {
                        let text = text.ok_or_else(|| A::Error::missing_field("text"))?;
                        if text.len() > 2048 {
                            return Err(A::Error::custom("One-off variant text limit"));
                        }
                        Ok(ReferenceAction::Command {
                            actor: actor.ok_or_else(|| A::Error::missing_field("actor"))?,
                            environment: environment
                                .ok_or_else(|| A::Error::missing_field("environment"))?,
                            text,
                        })
                    }
                    "Run" => Ok(ReferenceAction::Run {
                        end: end.ok_or_else(|| A::Error::missing_field("end"))?,
                    }),
                    _ => Err(A::Error::custom("Unknown one-off tag")),
                }
            }
        }
        d.deserialize_map(Tagged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn failure(raw: &str, needle: &str) {
        let e = serde_json::from_str::<ReferenceAction>(raw)
            .err()
            .expect("Must refuse reference input")
            .to_string();
        assert!(e.contains(needle), "{e}");
    }
    #[test]
    fn reference_command_invalid_or_excess_environment_key_refuses_poison_before_its_value() {
        failure(
            r#"{"kind":"Command","actor":"alex","text":"atq","environment":{"OPTERR": POISON_MUST_NOT_BE_READ"#,
            "Invalid/duplicate one-off key before value",
        );
        failure(
            r#"{"environment":{"PATH": POISON_MUST_NOT_BE_READ"#,
            "Invalid/duplicate one-off key before value",
        );
        let mut entries = Vec::new();
        for n in 0..16 {
            entries.push(format!("\"V{n}\":\"literal\""));
        }
        let raw = format!(
            "{{\"kind\":\"Command\",\"environment\":{{{},\"EXCESS\":POISON_MUST_NOT_BE_READ",
            entries.join(",")
        );
        failure(&raw, "One-off map count limit before excess key/value");
    }
    #[test]
    fn reference_tag_anywhere_and_variant_bounds_are_preserved() {
        for raw in [
            r#"{"kind":"Command","actor":"alex","environment":{},"text":"atq"}"#,
            r#"{"text":"atq","environment":{},"actor":"alex","kind":"Command"}"#,
        ] {
            let action: ReferenceAction = serde_json::from_str(raw).unwrap();
            let ReferenceAction::Command {
                actor,
                environment,
                text,
            } = action
            else {
                panic!("Command shape")
            };
            assert_eq!(actor, "alex");
            assert!(environment.is_empty());
            assert_eq!(text, "atq");
        }
        failure(
            r#"{"kind":"Run","text": POISON_MUST_NOT_BE_READ"#,
            "Incompatible one-off field before value",
        );
        failure(
            r#"{"text":"admitted union text","kind":"Run","later": POISON_MUST_NOT_BE_READ"#,
            "Incompatible one-off field before next value",
        );
        for tag_first in [true, false] {
            let text = "x".repeat(2049);
            let raw = if tag_first {
                format!(
                    "{{\"kind\":\"Command\",\"actor\":\"alex\",\"environment\":{{}},\"text\":\"{text}\"}}"
                )
            } else {
                format!(
                    "{{\"text\":\"{text}\",\"actor\":\"alex\",\"environment\":{{}},\"kind\":\"Command\"}}"
                )
            };
            assert!(serde_json::from_str::<ReferenceAction>(&raw).is_err());
        }
    }
}

#[cfg(test)]
mod canonical_tests {
    use super::*;
    fn round<T: serde::Serialize + for<'de> Deserialize<'de>>(value: serde_json::Value) {
        let ordinary: T = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(&ordinary).unwrap(), value);
        let mut fields = value
            .as_object()
            .unwrap()
            .iter()
            .filter(|(k, _)| k.as_str() != "kind")
            .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap(), v))
            .collect::<Vec<_>>();
        fields.push(format!("\"kind\":{}", value["kind"]));
        let late: T = serde_json::from_str(&format!("{{{}}}", fields.join(","))).unwrap();
        assert_eq!(serde_json::to_value(&late).unwrap(), value);
    }
    #[test]
    fn every_new_variant_keeps_its_canonical_fields_with_tag_first_or_last() {
        use serde_json::json;
        let time = json!({"year":2026,"month":1,"day":6,"hour":10,"minute":0});
        let job = json!({"owner":"alex","script_path":"/work/job.sh","timing":{"kind":"AfterSubmission","minutes":5}});
        for v in [
            json!({"kind":"Absolute","submitted":time,"due":time}),
            json!({"kind":"AfterSubmission","minutes":5}),
        ] {
            round::<Timing>(v);
        }
        for v in [
            json!({"kind":"Absolute","start":time,"end":time}),
            json!({"kind":"AfterSubmission","start_minutes":0,"end_minutes":6}),
        ] {
            round::<ProbeWindow>(v);
        }
        for v in [
            json!({"kind":"Absolute","at":time}),
            json!({"kind":"Dispatch"}),
        ] {
            round::<ExpectedWhen>(v);
        }
        for v in [
            json!({"kind":"QueueExactly","source":"Queued","jobs":[job]}),
            json!({"kind":"CurrentFile","path":"/work/out.txt","text":"out\n"}),
            json!({"kind":"Clock","at":time}),
            json!({"kind":"CapturedEnvironment","source":"LastRun","job":job,"values":{"MESSAGE":"literal"}}),
            json!({"kind":"RanWindow","start":time,"end":time}),
            json!({"kind":"Dispatched","job":job}),
        ] {
            round::<Requirement>(v);
        }
        for v in [
            json!({"kind":"CommandDraft","text":"atq"}),
            json!({"kind":"WindowDraft","text":"2026-01-06 10:06"}),
            json!({"kind":"Edit","path":"/work/job.sh","text":"echo ready\n"}),
            json!({"kind":"Save","path":"/work/job.sh"}),
            json!({"kind":"Select","path":"/work/job.sh"}),
            json!({"kind":"Actor","actor":"alex"}),
            json!({"kind":"Environment","values":{"MESSAGE":"literal"}}),
            json!({"kind":"Command","text":"atq"}),
            json!({"kind":"Run","end":time}),
        ] {
            round::<Action>(v);
        }
        for v in [
            json!({"kind":"Write","path":"/work/job.sh","text":"echo ready\n"}),
            json!({"kind":"Command","actor":"alex","environment":{"MESSAGE":"literal"},"text":"atq"}),
            json!({"kind":"Run","end":time}),
        ] {
            round::<ReferenceAction>(v);
        }
    }
}
