use dlms_learning_lab::desktop::{
    remote::{self, Authorization, Client, Connection, Port, Services, Sharing},
    session::Definition,
    typing_assist::{self, Event, Filter, Key, PhysicalTap, Settings, TextSize},
    workspace::{Action, Task, Work, Workspace},
    xorg::{self, Layout},
};

fn fixture(id: &str) -> (Definition, serde_json::Value) {
    let all: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/desktop_profile_tasks_v1.json")).unwrap();
    let row = &all[id];
    (
        serde_json::from_value(row["definition"].clone()).unwrap(),
        row["answers"].clone(),
    )
}

#[test]
fn later_matching_inputclass_changes_only_selected_devices() {
    let (d, answers) = fixture("lpic1-xorg-keyboard-scope");
    let task = &d.assignment;
    let Task::Keyboard { starter, devices } = task else {
        panic!()
    };
    let initial = xorg::parse(starter).unwrap();
    assert_eq!(
        xorg::effective(&devices[0].device, &initial).layout,
        Layout::Us
    );
    assert_eq!(
        xorg::effective(&devices[1].device, &initial).layout,
        Layout::Fr
    );
    let mut w = Work::new(task);
    let answer = answers["assignment"].as_str().unwrap();
    let alternate = answer.replace("Language-desk keyboard FR", "Language-desk");
    w.apply(
        task,
        Action::EditKeyboard {
            contents: alternate.clone(),
        },
    )
    .unwrap();
    assert!(!w.checks(task)[0].passed);
    w.apply(task, Action::ApplyKeyboard).unwrap();
    assert!(w.checks(task).iter().all(|check| check.passed));
    w.apply(task, Action::Check).unwrap();
    let broad = alternate.replace(
        "MatchProduct \"Language-desk\"",
        "MatchProduct \"Workshop\"",
    );
    w.apply(task, Action::EditKeyboard { contents: broad })
        .unwrap();
    assert!(!w.checked);
    w.apply(task, Action::ApplyKeyboard).unwrap();
    let outcomes = w.checks(task);
    assert!(outcomes[1].passed);
    assert!(!outcomes[2].passed && !outcomes[3].passed);
}

#[test]
fn malformed_fragments_never_replace_previously_applied_layouts() {
    let (d, _) = fixture("lpic1-xorg-keyboard-scope");
    let task = &d.example;
    let mut w = Work::new(task);
    let original = w.workspace.clone();
    for text in [
        "Section \"InputClass\"\nIdentifier \"No closing section\"",
        "Include \"/etc/X11/xorg.conf\"",
        "Section \"InputClass\"\nIdentifier \"Unknown layout\"\nOption \"XkbLayout\" \"us,de\"\nEndSection",
        "Section \"Device\"\nIdentifier \"Not input\"\nEndSection",
    ] {
        w.apply(
            task,
            Action::EditKeyboard {
                contents: text.into(),
            },
        )
        .unwrap();
        assert!(w.apply(task, Action::ApplyKeyboard).is_err());
        let Workspace::Keyboard { applied, .. } = &w.workspace else {
            panic!()
        };
        let Workspace::Keyboard {
            applied: before, ..
        } = &original
        else {
            panic!()
        };
        assert_eq!(applied, before);
    }
    assert!(xorg::parse(&"#".repeat(4097)).is_err());
    assert!(xorg::parse("Section \"InputClass\"\nIdentifier \"one\"\nOption \"XkbLayout\" \"de\"\nEndSection # accepted comment").is_ok());
}

#[test]
fn running_both_gnome_services_changes_sharing_port_and_keeps_roles_distinct() {
    let mut settings = Services {
        sharing: Sharing::ViewOnly,
        remote_login: false,
    };
    assert_eq!(settings.sharing_port(), Some(Port::Standard));
    assert_eq!(
        settings.connect(
            Client::RdpReady,
            Port::Standard,
            Authorization::SharedDesktop
        ),
        Connection::ObserveExisting
    );
    assert_eq!(
        settings.connect(
            Client::OtherProtocolOnly,
            Port::Standard,
            Authorization::SharedDesktop
        ),
        Connection::WrongProtocol
    );
    settings.remote_login = true;
    assert_eq!(settings.sharing_port(), Some(Port::Secondary));
    assert_eq!(
        settings.connect(
            Client::RdpReady,
            Port::Standard,
            Authorization::SharedDesktop
        ),
        Connection::WrongAccess
    );
    assert_eq!(
        settings.connect(Client::RdpReady, Port::Standard, Authorization::RemoteLogin),
        Connection::LoginToOwnSession
    );
    assert_eq!(
        settings.connect(Client::RdpReady, Port::Secondary, Authorization::NoGrant),
        Connection::WrongAccess
    );
    settings.sharing = Sharing::AllowControl;
    assert_eq!(
        settings.connect(
            Client::RdpReady,
            Port::Secondary,
            Authorization::SharedDesktop
        ),
        Connection::ControlExisting
    );
}

#[test]
fn typing_filters_differentiate_duration_and_repeats_at_the_boundary() {
    assert_eq!(remote::Port::Standard.number(), 3389);
    let mut current = Settings {
        text: TextSize::Standard,
        high_contrast: false,
        sticky_keys: false,
        typing_filter: Filter::SlowKeys300Ms,
    };
    let tap = |key, start, end| PhysicalTap {
        key,
        pressed_at_ms: start,
        released_at_ms: end,
    };
    assert_eq!(
        typing_assist::preview(current, &[tap(Key::A, 0, 299), tap(Key::B, 320, 620)]).unwrap(),
        [Event::RejectedTooShort, Event::Printed('b')],
    );
    current.typing_filter = Filter::BounceKeys300Ms;
    assert_eq!(
        typing_assist::preview(
            current,
            &[
                tap(Key::A, 0, 50),
                tap(Key::A, 349, 350),
                tap(Key::A, 350, 400),
                tap(Key::B, 420, 470)
            ]
        )
        .unwrap(),
        [
            Event::Printed('a'),
            Event::RejectedRepeated,
            Event::Printed('a'),
            Event::Printed('b')
        ],
    );
    assert!(typing_assist::preview(current, &[tap(Key::A, 5, 10), tap(Key::A, 9, 12)]).is_none());
    assert!(typing_assist::preview(current, &[tap(Key::A, 5, 4)]).is_none());
}

#[test]
fn remote_practice_explains_a_deliberately_disabled_login() {
    let (definition, _) = fixture("lpic1-remote-desktop-arrangement");
    let task = &definition.example;
    let Task::Remote { starter, .. } = task else {
        panic!("Expected a supplied GNOME profile");
    };
    let mut draft = *starter;
    let mut work = Work::new(task);
    let active = &work.checks(task)[1];
    assert_eq!(active.label, "Remote Login stays off");
    assert!(!active.passed);
    assert!(active.detail.contains("Turn off Remote Login"));
    draft.services.remote_login = false;
    work.apply(task, Action::SetRemote { draft }).unwrap();
    let disabled = &work.checks(task)[1];
    assert!(disabled.passed);
    assert_eq!(disabled.detail, "Remote Login is off as required.");
}

#[test]
fn all_source_owned_examples_pass_with_their_own_workspace_and_no_cross_credit() {
    for id in [
        "lpic1-xorg-keyboard-scope",
        "lpic1-remote-desktop-arrangement",
        "lpic1-accessible-input-profile",
    ] {
        let (d, answers) = fixture(id);
        d.validate().unwrap();
        let mut session = dlms_learning_lab::desktop::session::Session::new(&d).unwrap();
        let example = &mut session.example.work;
        match &d.example {
            Task::Keyboard { .. } => {
                example
                    .apply(
                        &d.example,
                        Action::EditKeyboard {
                            contents: answers["example"].as_str().unwrap().into(),
                        },
                    )
                    .unwrap();
                example.apply(&d.example, Action::ApplyKeyboard).unwrap();
            }
            Task::Remote { .. } => example
                .apply(
                    &d.example,
                    Action::SetRemote {
                        draft: serde_json::from_value(answers["example"].clone()).unwrap(),
                    },
                )
                .unwrap(),
            Task::Typing { .. } => example
                .apply(
                    &d.example,
                    Action::SetTyping {
                        current: serde_json::from_value(answers["example"].clone()).unwrap(),
                    },
                )
                .unwrap(),
        }
        example.apply(&d.example, Action::Check).unwrap();
        assert!(example.checks(&d.example).iter().all(|c| c.passed), "{id}");
        assert!(!session.assignment.work.checked);
        assert!(
            session
                .assignment
                .work
                .checks(&d.assignment)
                .iter()
                .any(|c| !c.passed)
        );
    }
}
