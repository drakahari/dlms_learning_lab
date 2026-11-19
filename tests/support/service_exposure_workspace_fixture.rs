//! Fixed replay/source fixture, not a registered learner lesson.
use crate::task_fixture;
use dlms_learning_lab::service_exposure_workspace::{Action, Definition, Event};
pub fn action(event: Event) -> Action {
    Action::Event { event }
}
pub fn repair() -> Vec<Action> {
    [22, 9000]
        .into_iter()
        .map(|port| {
            action(Event::Admin {
                input: format!("iptables -A INPUT -s 192.0.2.0/24 -p tcp --dport {port} -j DROP"),
            })
        })
        .chain([action(Event::Check)])
        .collect()
}
pub fn definition() -> Definition {
    fn change(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::String(s) if s == "203.0.113.10" => *s = "203.0.113.42".into(),
            serde_json::Value::Array(a) => {
                for x in a {
                    change(x)
                }
            }
            serde_json::Value::Object(o) => {
                for x in o.values_mut() {
                    change(x)
                }
            }
            _ => {}
        }
    }
    let assignment = task_fixture::fixture();
    let mut value = serde_json::to_value(&assignment).unwrap();
    change(&mut value);
    let example = serde_json::from_value(value).unwrap();
    let d = Definition {
        version: 1,
        source_id: "exposure-fixture".into(),
        assignment,
        example,
        worked: repair(),
    };
    d.validate().unwrap();
    d
}
