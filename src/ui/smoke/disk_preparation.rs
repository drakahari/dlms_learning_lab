//! Fixed-source conditional native route: real editors/actions, never installed results.
use super::*;
use dlms_learning_lab::{
    partition_fs as core, partition_fs_workspace as work, script_teaching::View,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};
pub(in crate::ui) const FIRST: usize = 90000;
const RESERVE: usize = 4000;
const IDS: [&str; 4] = [
    "lpic1-disk-safe-gap",
    "lpic1-disk-hints-versus-content",
    "lpic1-disk-eligible-media",
    "lpic1-disk-delivery",
];
#[derive(Clone, Debug)]
enum Target {
    Search,
    Start(usize),
    Field(&'static str),
    Control(&'static str),
    Choice(String),
    Back,
    Reset,
    Cancel,
    Confirm,
    Mode(bool),
    Theme,
    Scale,
    HeaderChoice(String),
    Goal,
    Feedback,
    Notice,
    Text(String),
    ReferenceFirst,
    History(usize),
}
#[derive(Clone, Copy, Debug)]
enum Scope {
    Assignment,
    Example,
    Progress,
    Others,
}
#[derive(Clone, Debug)]
enum Step {
    Init,
    Click(Target),
    Replace(String),
    FieldValue(&'static str, String),
    Draft(work::Operation, bool),
    Key(egui::Key, egui::Modifiers),
    Focus(&'static str),
    Remember(String, Scope),
    Same(String, Scope),
    GuidanceDelta(String, usize),
    RememberWarm(String),
    ResumeExact(String, bool),
    Task(usize),
    Selected(String),
    Answers(bool),
    Rows(Vec<bool>, bool),
    Completion(bool),
    Notice(bool),
    Fresh,
    PracticeFresh,
    Save,
    InvalidPacket,
    Window(bool),
    Setting(f32, String),
    Geometry(usize),
    Shot(String, Target),
    Done,
}
fn sources() -> &'static Vec<pack::Scenario> {
    static S: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/disk-preparation-v1.json"
        ))
        .unwrap()
        .scenarios
    })
}
fn review() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/disk-preparation-review.json"
        ))
        .unwrap()
    })
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(||{let p=
vec![
    Step::Init,
    Step::Window(false),
    Step::Shot("disk-fixture-library".into(), Target::Search),
    Step::Click(Target::Search),
    Step::Replace("Prepare an export area beside retained evidence".into()),
    Step::Click(Target::Start(0)),
    Step::Task(0),
    Step::Shot("lpic1-disk-safe-gap-introduction".into(), Target::Text("No disk-command knowledge is required. Familiarity with file paths is useful; read these concepts and try the independent practice before the full assignment. The introduction is skippable, and prerequisite suggestions never lock access.".into())),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,true], false),
    Step::Shot("lpic1-disk-safe-gap-whole-assignment".into(), Target::Goal),
    Step::Remember("help-before".into(), Scope::Progress),
    Step::Click(Target::Control("help")),
    Step::Answers(false),
    Step::Click(Target::Control("show-hint")),
    Step::GuidanceDelta("help-before".into(), 1),
    Step::Shot("lpic1-disk-safe-gap-requested-hint".into(), Target::Text("The archive ends at 18431. Pick an aligned start outside it; do not replace its GPT.".into())),
    Step::Answers(false),
    Step::Click(Target::Control("worked")),
    Step::Answers(true),
    Step::Shot("lpic1-disk-safe-gap-requested-worked".into(), Target::ReferenceFirst),
    Step::Click(Target::Control("references")),
    Step::Shot("lpic1-disk-safe-gap-requested-reference".into(), Target::Text("HELP".into())),
    Step::GuidanceDelta("help-before".into(), 1),
    Step::Click(Target::Control("references")),
    Step::Click(Target::Control("worked")),
    Step::Click(Target::Control("help")),
    Step::Remember("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("practice")),
    Step::Shot("lpic1-disk-safe-gap-separate-practice".into(), Target::Goal),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vde".into(), name:"sample".into(), start:12288, end:18431, type_hint:core::TypeHint::LinuxData }, true),
    Step::Shot("lpic1-disk-safe-gap-filled-not-applied".into(), Target::Field("name")),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::Format { partition:"/dev/vde2".into(), format:work::FormatChoice::Ext4, label:"sample".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true], true),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vde".into())),
    Step::Selected("/dev/vde".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("22527".into()),
    Step::FieldValue("end", "22527".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vde".into(), name:"extra".into(), start:20480, end:22527, type_hint:core::TypeHint::LinuxData }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,false], true),
    Step::Shot("lpic1-disk-safe-gap-practice-experiment".into(), Target::Feedback),
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-safe-gap-practice-inspection".into(), Target::Text("/dev/vde1 · /keep.txt: Aspen practice retained archive\n".into())),
    Step::Click(Target::Control("inspect")),
    Step::Click(Target::Control("reset-practice")),
    Step::PracticeFresh,
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vde".into(), name:"sample".into(), start:12288, end:18431, type_hint:core::TypeHint::LinuxData }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::Format { partition:"/dev/vde2".into(), format:work::FormatChoice::Ext4, label:"sample".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true], true),
    Step::Same("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("assignment")),
    Step::Completion(false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vda".into() }, false),
    Step::Remember("atomic-before-action".into(), Scope::Progress),
    Step::Click(Target::Control("create-gpt")),
    Step::Same("atomic-before-action".into(), Scope::Progress),
    Step::Notice(true),
    Step::Shot("lpic1-disk-safe-gap-atomic-refusal".into(), Target::Notice),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("staging".into()),
    Step::FieldValue("name", "staging".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("36863".into()),
    Step::FieldValue("end", "36863".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"staging".into(), start:20480, end:36863, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("wrong".into()),
    Step::FieldValue("label", "wrong".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Ext4, label:"wrong".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,true], false),
    Step::Shot("lpic1-disk-safe-gap-wrong-applied-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("staging".into()),
    Step::FieldValue("name", "staging".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("36863".into()),
    Step::FieldValue("end", "36863".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"staging".into(), start:20480, end:36863, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,true], false),
    Step::Shot("lpic1-disk-safe-gap-partial-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Field("name")),
    Step::Remember("reset-before-undo".into(), Scope::Progress),
    Step::Key(egui::Key::Z, egui::Modifiers { ctrl:true, command:true, ..Default::default() }),
    Step::FieldValue("name", "".into()),
    Step::Same("reset-before-undo".into(), Scope::Progress),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("staging".into()),
    Step::FieldValue("name", "staging".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("36863".into()),
    Step::FieldValue("end", "36863".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"staging".into(), start:20480, end:36863, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("export".into()),
    Step::FieldValue("label", "export".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Ext4, label:"export".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-safe-gap-reference-complete".into(), Target::Feedback),
    Step::Save,
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-safe-gap-assignment-inspection".into(), Target::Text("/dev/vda1 · /keep.txt: Birch evidence retained archive\n".into())),
    Step::Click(Target::Control("inspect")),
    Step::RememberWarm("before-next".into()),
    Step::Click(Target::Control("next")),
    Step::Task(1),
    Step::Fresh,
    Step::Completion(false),
    Step::Click(Target::Back),
    Step::Click(Target::Search),
    Step::Replace("Prepare an export area beside retained evidence".into()),
    Step::Click(Target::Start(0)),
    Step::Task(0),
    Step::ResumeExact("before-next".into(), true),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("staging".into()),
    Step::FieldValue("name", "staging".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("45055".into()),
    Step::FieldValue("end", "45055".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"staging".into(), start:20480, end:45055, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("export".into()),
    Step::FieldValue("label", "export".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000ca".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000ca".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Ext4, label:"export".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000ca".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-safe-gap-alternate-1".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("staging".into()),
    Step::FieldValue("name", "staging".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("40960".into()),
    Step::FieldValue("start", "40960".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("61439".into()),
    Step::FieldValue("end", "61439".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"staging".into(), start:40960, end:61439, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("export".into()),
    Step::FieldValue("label", "export".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000d4".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000d4".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Ext4, label:"export".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000d4".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-safe-gap-alternate-2".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("staging".into()),
    Step::FieldValue("name", "staging".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("36863".into()),
    Step::FieldValue("end", "36863".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"staging".into(), start:20480, end:36863, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("export".into()),
    Step::FieldValue("label", "export".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Ext4, label:"export".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true], false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("65536".into()),
    Step::FieldValue("start", "65536".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("67583".into()),
    Step::FieldValue("end", "67583".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"extra".into(), start:65536, end:67583, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,false], false),
    Step::Completion(false),
    Step::Shot("lpic1-disk-safe-gap-success-invalidated".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("staging".into()),
    Step::FieldValue("name", "staging".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("36863".into()),
    Step::FieldValue("end", "36863".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"staging".into(), start:20480, end:36863, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("export".into()),
    Step::FieldValue("label", "export".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Ext4, label:"export".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true], false),
    Step::Click(Target::Field("name")),
    Step::Replace("unfinished".into()),
    Step::FieldValue("name", "unfinished".into()),
    Step::RememberWarm("warm-return".into()),
    Step::Save,
    Step::Click(Target::Back),
    Step::Click(Target::Start(0)),
    Step::Task(0),
    Step::ResumeExact("warm-return".into(), false),
    Step::Remember("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Reset),
    Step::Click(Target::Cancel),
    Step::Same("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Field("name")),
    Step::Remember("bad-packet".into(), Scope::Progress),
    Step::InvalidPacket,
    Step::Same("bad-packet".into(), Scope::Progress),
    Step::Notice(true),
    Step::Click(Target::Field("name")),
    Step::Replace("resumed".into()),
    Step::FieldValue("name", "resumed".into()),
    Step::Notice(false),
    Step::Key(egui::Key::Tab, Default::default()),
    Step::Focus("start"),
    Step::Key(egui::Key::Tab, egui::Modifiers { shift:true,..Default::default() }),
    Step::Focus("name"),
    Step::Click(Target::Mode(true)),
    Step::Shot("lpic1-disk-safe-gap-challenge".into(), Target::Field("name")),
    Step::Click(Target::Mode(false)),
    Step::Click(Target::Control("assignment")),
    Step::Save,
    Step::Click(Target::Back),
    Step::Click(Target::Search),
    Step::Replace("Finish a correctly allocated handoff".into()),
    Step::Click(Target::Start(1)),
    Step::Task(1),
    Step::Shot("lpic1-disk-hints-versus-content-introduction".into(), Target::Text("No disk-command knowledge is required. Familiarity with file paths is useful; read these concepts and try the independent practice before the full assignment. The introduction is skippable, and prerequisite suggestions never lock access.".into())),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,false,true], false),
    Step::Shot("lpic1-disk-hints-versus-content-whole-assignment".into(), Target::Goal),
    Step::Remember("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("practice")),
    Step::Shot("lpic1-disk-hints-versus-content-separate-practice".into(), Target::Goal),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::Format { partition:"/dev/vde1".into(), format:work::FormatChoice::Ext4, label:"draft".into(), uuid:None }, true),
    Step::Shot("lpic1-disk-hints-versus-content-filled-not-applied".into(), Target::Field("label")),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::Format { partition:"/dev/vde2".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], true),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vde".into())),
    Step::Selected("/dev/vde".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("32768".into()),
    Step::FieldValue("start", "32768".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("34815".into()),
    Step::FieldValue("end", "34815".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vde".into(), name:"extra".into(), start:32768, end:34815, type_hint:core::TypeHint::LinuxData }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,false], true),
    Step::Shot("lpic1-disk-hints-versus-content-practice-experiment".into(), Target::Feedback),
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-hints-versus-content-practice-inspection".into(), Target::Text("/dev/vde3 · /keep.txt: Willow practice retained archive\n".into())),
    Step::Click(Target::Control("inspect")),
    Step::Click(Target::Control("reset-practice")),
    Step::PracticeFresh,
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::Format { partition:"/dev/vde1".into(), format:work::FormatChoice::Ext4, label:"draft".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::Format { partition:"/dev/vde2".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], true),
    Step::Same("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("assignment")),
    Step::Completion(false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vda".into() }, false),
    Step::Remember("atomic-before-action".into(), Scope::Progress),
    Step::Click(Target::Control("create-gpt")),
    Step::Same("atomic-before-action".into(), Scope::Progress),
    Step::Notice(true),
    Step::Shot("lpic1-disk-hints-versus-content-atomic-refusal".into(), Target::Notice),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("capture".into()),
    Step::FieldValue("label", "capture".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Swap, label:"capture".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("paging".into()),
    Step::FieldValue("label", "paging".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Ext4, label:"paging".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,false,true], false),
    Step::Shot("lpic1-disk-hints-versus-content-wrong-applied-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("capture".into()),
    Step::FieldValue("label", "capture".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"capture".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,false,true], false),
    Step::Shot("lpic1-disk-hints-versus-content-partial-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Field("name")),
    Step::Remember("reset-before-undo".into(), Scope::Progress),
    Step::Key(egui::Key::Z, egui::Modifiers { ctrl:true, command:true, ..Default::default() }),
    Step::FieldValue("name", "".into()),
    Step::Same("reset-before-undo".into(), Scope::Progress),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("capture".into()),
    Step::FieldValue("label", "capture".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"capture".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("paging".into()),
    Step::FieldValue("label", "paging".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Swap, label:"paging".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-hints-versus-content-reference-complete".into(), Target::Feedback),
    Step::Save,
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-hints-versus-content-assignment-inspection".into(), Target::Text("/dev/vda3 · /keep.txt: Cedar handoff retained archive\n".into())),
    Step::Click(Target::Control("inspect")),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("paging".into()),
    Step::FieldValue("label", "paging".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000dd".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000dd".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Swap, label:"paging".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000dd".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("capture".into()),
    Step::FieldValue("label", "capture".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000de".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000de".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"capture".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000de".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-hints-versus-content-alternate-1".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("capture".into()),
    Step::FieldValue("label", "capture".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000e7".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000e7".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"capture".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000e7".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("paging".into()),
    Step::FieldValue("label", "paging".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000e8".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000e8".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Swap, label:"paging".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000e8".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-hints-versus-content-alternate-2".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("capture".into()),
    Step::FieldValue("label", "capture".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"capture".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("paging".into()),
    Step::FieldValue("label", "paging".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Swap, label:"paging".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("65536".into()),
    Step::FieldValue("start", "65536".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("67583".into()),
    Step::FieldValue("end", "67583".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"extra".into(), start:65536, end:67583, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,false], false),
    Step::Completion(false),
    Step::Shot("lpic1-disk-hints-versus-content-success-invalidated".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("capture".into()),
    Step::FieldValue("label", "capture".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"capture".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("paging".into()),
    Step::FieldValue("label", "paging".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda2".into(), format:work::FormatChoice::Swap, label:"paging".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Click(Target::Field("name")),
    Step::Replace("unfinished".into()),
    Step::FieldValue("name", "unfinished".into()),
    Step::RememberWarm("warm-return".into()),
    Step::Save,
    Step::Click(Target::Back),
    Step::Click(Target::Start(1)),
    Step::Task(1),
    Step::ResumeExact("warm-return".into(), false),
    Step::Remember("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Reset),
    Step::Click(Target::Cancel),
    Step::Same("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Field("name")),
    Step::Remember("bad-packet".into(), Scope::Progress),
    Step::InvalidPacket,
    Step::Same("bad-packet".into(), Scope::Progress),
    Step::Notice(true),
    Step::Click(Target::Field("name")),
    Step::Replace("resumed".into()),
    Step::FieldValue("name", "resumed".into()),
    Step::Notice(false),
    Step::Key(egui::Key::Tab, Default::default()),
    Step::Focus("start"),
    Step::Key(egui::Key::Tab, egui::Modifiers { shift:true,..Default::default() }),
    Step::Focus("name"),
    Step::Click(Target::Mode(true)),
    Step::Shot("lpic1-disk-hints-versus-content-challenge".into(), Target::Field("name")),
    Step::Click(Target::Mode(false)),
    Step::Click(Target::Control("assignment")),
    Step::Save,
    Step::Click(Target::Back),
    Step::Click(Target::Search),
    Step::Replace("Build a layout on eligible media".into()),
    Step::Click(Target::Start(2)),
    Step::Task(2),
    Step::Shot("lpic1-disk-eligible-media-introduction".into(), Target::Text("No disk-command knowledge is required. Familiarity with file paths is useful; read these concepts and try the independent practice before the full assignment. The introduction is skippable, and prerequisite suggestions never lock access.".into())),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,false,true], false),
    Step::Shot("lpic1-disk-eligible-media-whole-assignment".into(), Target::Goal),
    Step::Remember("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("practice")),
    Step::Shot("lpic1-disk-eligible-media-separate-practice".into(), Target::Goal),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdf".into() }, true),
    Step::Shot("lpic1-disk-eligible-media-filled-not-applied".into(), Target::Control("disk")),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdf".into(), name:"sample".into(), start:2048, end:8191, type_hint:core::TypeHint::LinuxData }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-2".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::Format { partition:"/dev/vdf1".into(), format:work::FormatChoice::Ext4, label:"sample".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-3".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdf".into(), name:"reserve".into(), start:10240, end:12287, type_hint:core::TypeHint::LinuxSwap }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-4".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::Format { partition:"/dev/vdf2".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], true),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdf".into())),
    Step::Selected("/dev/vdf".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("16384".into()),
    Step::FieldValue("start", "16384".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("18431".into()),
    Step::FieldValue("end", "18431".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdf".into(), name:"extra".into(), start:16384, end:18431, type_hint:core::TypeHint::LinuxData }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,false], true),
    Step::Shot("lpic1-disk-eligible-media-practice-experiment".into(), Target::Feedback),
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-eligible-media-practice-inspection".into(), Target::History(0)),
    Step::Click(Target::Control("inspect")),
    Step::Click(Target::Control("reset-practice")),
    Step::PracticeFresh,
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdf".into() }, true),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdf".into(), name:"sample".into(), start:2048, end:8191, type_hint:core::TypeHint::LinuxData }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-2".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::Format { partition:"/dev/vdf1".into(), format:work::FormatChoice::Ext4, label:"sample".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-3".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdf".into(), name:"reserve".into(), start:10240, end:12287, type_hint:core::TypeHint::LinuxSwap }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-4".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::Format { partition:"/dev/vdf2".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], true),
    Step::Same("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("assignment")),
    Step::Completion(false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vda".into() }, false),
    Step::Remember("atomic-before-action".into(), Scope::Progress),
    Step::Click(Target::Control("create-gpt")),
    Step::Same("atomic-before-action".into(), Scope::Progress),
    Step::Notice(true),
    Step::Shot("lpic1-disk-eligible-media-atomic-refusal".into(), Target::Notice),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("records".into()),
    Step::FieldValue("name", "records".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("18431".into()),
    Step::FieldValue("end", "18431".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"records".into(), start:2048, end:18431, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("wrong".into()),
    Step::FieldValue("label", "wrong".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Ext4, label:"wrong".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("overflow".into()),
    Step::FieldValue("name", "overflow".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("24575".into()),
    Step::FieldValue("end", "24575".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"overflow".into(), start:20480, end:24575, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("overflow".into()),
    Step::FieldValue("label", "overflow".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb2".into(), format:work::FormatChoice::Swap, label:"overflow".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,true,true], false),
    Step::Shot("lpic1-disk-eligible-media-wrong-applied-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("records".into()),
    Step::FieldValue("name", "records".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("18431".into()),
    Step::FieldValue("end", "18431".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"records".into(), start:2048, end:18431, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("records".into()),
    Step::FieldValue("label", "records".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Ext4, label:"records".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,false,true], false),
    Step::Shot("lpic1-disk-eligible-media-partial-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Field("name")),
    Step::Remember("reset-before-undo".into(), Scope::Progress),
    Step::Key(egui::Key::Z, egui::Modifiers { ctrl:true, command:true, ..Default::default() }),
    Step::FieldValue("name", "".into()),
    Step::Same("reset-before-undo".into(), Scope::Progress),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("records".into()),
    Step::FieldValue("name", "records".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("18431".into()),
    Step::FieldValue("end", "18431".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"records".into(), start:2048, end:18431, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("records".into()),
    Step::FieldValue("label", "records".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Ext4, label:"records".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("overflow".into()),
    Step::FieldValue("name", "overflow".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("24575".into()),
    Step::FieldValue("end", "24575".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"overflow".into(), start:20480, end:24575, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("overflow".into()),
    Step::FieldValue("label", "overflow".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb2".into(), format:work::FormatChoice::Swap, label:"overflow".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-eligible-media-reference-complete".into(), Target::Feedback),
    Step::Save,
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-eligible-media-assignment-inspection".into(), Target::History(0)),
    Step::Click(Target::Control("inspect")),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("overflow".into()),
    Step::FieldValue("name", "overflow".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("10239".into()),
    Step::FieldValue("end", "10239".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"overflow".into(), start:2048, end:10239, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("overflow".into()),
    Step::FieldValue("label", "overflow".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000f3".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000f3".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Swap, label:"overflow".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000f3".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("records".into()),
    Step::FieldValue("name", "records".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("12288".into()),
    Step::FieldValue("start", "12288".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("45055".into()),
    Step::FieldValue("end", "45055".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"records".into(), start:12288, end:45055, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("records".into()),
    Step::FieldValue("label", "records".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000f5".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000f5".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb2".into(), format:work::FormatChoice::Ext4, label:"records".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000f5".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-eligible-media-alternate-1".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("records".into()),
    Step::FieldValue("name", "records".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("4096".into()),
    Step::FieldValue("start", "4096".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("28671".into()),
    Step::FieldValue("end", "28671".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"records".into(), start:4096, end:28671, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("overflow".into()),
    Step::FieldValue("name", "overflow".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("32768".into()),
    Step::FieldValue("start", "32768".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("38911".into()),
    Step::FieldValue("end", "38911".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"overflow".into(), start:32768, end:38911, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("overflow".into()),
    Step::FieldValue("label", "overflow".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000fe".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000fe".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb2".into(), format:work::FormatChoice::Swap, label:"overflow".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000fe".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("records".into()),
    Step::FieldValue("label", "records".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-0000000000ff".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-0000000000ff".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Ext4, label:"records".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-0000000000ff".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-eligible-media-alternate-2".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("records".into()),
    Step::FieldValue("name", "records".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("18431".into()),
    Step::FieldValue("end", "18431".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"records".into(), start:2048, end:18431, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("records".into()),
    Step::FieldValue("label", "records".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Ext4, label:"records".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("overflow".into()),
    Step::FieldValue("name", "overflow".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("24575".into()),
    Step::FieldValue("end", "24575".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"overflow".into(), start:20480, end:24575, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("overflow".into()),
    Step::FieldValue("label", "overflow".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb2".into(), format:work::FormatChoice::Swap, label:"overflow".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("32768".into()),
    Step::FieldValue("start", "32768".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("34815".into()),
    Step::FieldValue("end", "34815".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"extra".into(), start:32768, end:34815, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,false], false),
    Step::Completion(false),
    Step::Shot("lpic1-disk-eligible-media-success-invalidated".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("records".into()),
    Step::FieldValue("name", "records".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("18431".into()),
    Step::FieldValue("end", "18431".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"records".into(), start:2048, end:18431, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("records".into()),
    Step::FieldValue("label", "records".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Ext4, label:"records".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("overflow".into()),
    Step::FieldValue("name", "overflow".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("20480".into()),
    Step::FieldValue("start", "20480".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("24575".into()),
    Step::FieldValue("end", "24575".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"overflow".into(), start:20480, end:24575, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-2".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("overflow".into()),
    Step::FieldValue("label", "overflow".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb2".into(), format:work::FormatChoice::Swap, label:"overflow".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Click(Target::Field("name")),
    Step::Replace("unfinished".into()),
    Step::FieldValue("name", "unfinished".into()),
    Step::RememberWarm("warm-return".into()),
    Step::Save,
    Step::Click(Target::Back),
    Step::Click(Target::Start(2)),
    Step::Task(2),
    Step::ResumeExact("warm-return".into(), false),
    Step::Remember("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Reset),
    Step::Click(Target::Cancel),
    Step::Same("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Field("name")),
    Step::Remember("bad-packet".into(), Scope::Progress),
    Step::InvalidPacket,
    Step::Same("bad-packet".into(), Scope::Progress),
    Step::Notice(true),
    Step::Click(Target::Field("name")),
    Step::Replace("resumed".into()),
    Step::FieldValue("name", "resumed".into()),
    Step::Notice(false),
    Step::Key(egui::Key::Tab, Default::default()),
    Step::Focus("start"),
    Step::Key(egui::Key::Tab, egui::Modifiers { shift:true,..Default::default() }),
    Step::Focus("name"),
    Step::Click(Target::Mode(true)),
    Step::Shot("lpic1-disk-eligible-media-challenge".into(), Target::Field("name")),
    Step::Click(Target::Mode(false)),
    Step::Click(Target::Control("assignment")),
    Step::Save,
    Step::Click(Target::Back),
    Step::Click(Target::Search),
    Step::Replace("Deliver prepared storage without disturbing retained data".into()),
    Step::Click(Target::Start(3)),
    Step::Task(3),
    Step::Shot("lpic1-disk-delivery-introduction".into(), Target::Text("No disk-command knowledge is required. Familiarity with file paths is useful; read these concepts and try the independent practice before the full assignment. The introduction is skippable, and prerequisite suggestions never lock access.".into())),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,false,true], false),
    Step::Shot("lpic1-disk-delivery-whole-assignment".into(), Target::Goal),
    Step::Remember("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("practice")),
    Step::Shot("lpic1-disk-delivery-separate-practice".into(), Target::Goal),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::Format { partition:"/dev/vde1".into(), format:work::FormatChoice::Ext4, label:"parcel".into(), uuid:None }, true),
    Step::Shot("lpic1-disk-delivery-filled-not-applied".into(), Target::Field("label")),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdf".into() }, true),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-2".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdf".into(), name:"buffer".into(), start:2048, end:6143, type_hint:core::TypeHint::LinuxSwap }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-3".into())),
    Step::Remember("before-fill".into(), Scope::Example),
    Step::Click(Target::Control("fill-example")),
    Step::Same("before-fill".into(), Scope::Example),
    Step::Draft(work::Operation::Format { partition:"/dev/vdf1".into(), format:work::FormatChoice::Swap, label:"buffer".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], true),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vde".into())),
    Step::Selected("/dev/vde".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("24576".into()),
    Step::FieldValue("start", "24576".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("26623".into()),
    Step::FieldValue("end", "26623".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vde".into(), name:"extra".into(), start:24576, end:26623, type_hint:core::TypeHint::LinuxData }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,false], true),
    Step::Shot("lpic1-disk-delivery-practice-experiment".into(), Target::Feedback),
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-delivery-practice-inspection".into(), Target::Text("/dev/vde2 · /keep.txt: Hazel practice retained archive\n".into())),
    Step::Click(Target::Control("inspect")),
    Step::Click(Target::Control("reset-practice")),
    Step::PracticeFresh,
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-0".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::Format { partition:"/dev/vde1".into(), format:work::FormatChoice::Ext4, label:"parcel".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-1".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdf".into() }, true),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-2".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdf".into(), name:"buffer".into(), start:2048, end:6143, type_hint:core::TypeHint::LinuxSwap }, true),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("example-picker")),
    Step::Click(Target::Choice("example-3".into())),
    Step::Click(Target::Control("fill-example")),
    Step::Draft(work::Operation::Format { partition:"/dev/vdf1".into(), format:work::FormatChoice::Swap, label:"buffer".into(), uuid:None }, true),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], true),
    Step::Same("assignment-before-practice".into(), Scope::Assignment),
    Step::Click(Target::Control("assignment")),
    Step::Completion(false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vda".into() }, false),
    Step::Remember("atomic-before-action".into(), Scope::Progress),
    Step::Click(Target::Control("create-gpt")),
    Step::Same("atomic-before-action".into(), Scope::Progress),
    Step::Notice(true),
    Step::Shot("lpic1-disk-delivery-atomic-refusal".into(), Target::Notice),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("dispatch".into()),
    Step::FieldValue("label", "dispatch".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Swap, label:"dispatch".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("reserve".into()),
    Step::FieldValue("name", "reserve".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("10239".into()),
    Step::FieldValue("end", "10239".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"reserve".into(), start:2048, end:10239, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("reserve".into()),
    Step::FieldValue("label", "reserve".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![false,true,true], false),
    Step::Shot("lpic1-disk-delivery-wrong-applied-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("dispatch".into()),
    Step::FieldValue("label", "dispatch".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"dispatch".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,false,true], false),
    Step::Shot("lpic1-disk-delivery-partial-result".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Field("name")),
    Step::Remember("reset-before-undo".into(), Scope::Progress),
    Step::Key(egui::Key::Z, egui::Modifiers { ctrl:true, command:true, ..Default::default() }),
    Step::FieldValue("name", "".into()),
    Step::Same("reset-before-undo".into(), Scope::Progress),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("dispatch".into()),
    Step::FieldValue("label", "dispatch".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"dispatch".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("reserve".into()),
    Step::FieldValue("name", "reserve".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("10239".into()),
    Step::FieldValue("end", "10239".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"reserve".into(), start:2048, end:10239, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("reserve".into()),
    Step::FieldValue("label", "reserve".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-delivery-reference-complete".into(), Target::Feedback),
    Step::Save,
    Step::Click(Target::Control("inspect")),
    Step::Shot("lpic1-disk-delivery-assignment-inspection".into(), Target::Text("/dev/vda2 · /keep.txt: Alder delivery retained archive\n".into())),
    Step::Click(Target::Control("inspect")),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdc".into())),
    Step::Selected("/dev/vdc".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdc".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdc".into())),
    Step::Selected("/dev/vdc".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("reserve".into()),
    Step::FieldValue("name", "reserve".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("4096".into()),
    Step::FieldValue("start", "4096".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("16383".into()),
    Step::FieldValue("end", "16383".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdc".into(), name:"reserve".into(), start:4096, end:16383, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdc".into())),
    Step::Selected("/dev/vdc".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("reserve".into()),
    Step::FieldValue("label", "reserve".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-000000000107".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-000000000107".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdc1".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-000000000107".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("dispatch".into()),
    Step::FieldValue("label", "dispatch".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-000000000108".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-000000000108".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"dispatch".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-000000000108".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-delivery-alternate-1".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("dispatch".into()),
    Step::FieldValue("label", "dispatch".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-00000000010f".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-00000000010f".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"dispatch".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-00000000010f".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("reserve".into()),
    Step::FieldValue("name", "reserve".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("8192".into()),
    Step::FieldValue("start", "8192".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("24575".into()),
    Step::FieldValue("end", "24575".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"reserve".into(), start:8192, end:24575, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("reserve".into()),
    Step::FieldValue("label", "reserve".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("bbbbbbbb-bbbb-4bbb-8bbb-000000000112".into()),
    Step::FieldValue("uuid", "bbbbbbbb-bbbb-4bbb-8bbb-000000000112".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:Some("bbbbbbbb-bbbb-4bbb-8bbb-000000000112".into()) }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Completion(true),
    Step::Shot("lpic1-disk-delivery-alternate-2".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("dispatch".into()),
    Step::FieldValue("label", "dispatch".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"dispatch".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("reserve".into()),
    Step::FieldValue("name", "reserve".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("10239".into()),
    Step::FieldValue("end", "10239".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"reserve".into(), start:2048, end:10239, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("reserve".into()),
    Step::FieldValue("label", "reserve".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("extra".into()),
    Step::FieldValue("name", "extra".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("65536".into()),
    Step::FieldValue("start", "65536".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("67583".into()),
    Step::FieldValue("end", "67583".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxData".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vda".into(), name:"extra".into(), start:65536, end:67583, type_hint:core::TypeHint::LinuxData }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,false], false),
    Step::Completion(false),
    Step::Shot("lpic1-disk-delivery-success-invalidated".into(), Target::Feedback),
    Step::Remember("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Reset),
    Step::Click(Target::Confirm),
    Step::Fresh,
    Step::Same("other-before-reset".into(), Scope::Others),
    Step::Click(Target::Control("assignment")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vda".into())),
    Step::Selected("/dev/vda".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Ext4".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("dispatch".into()),
    Step::FieldValue("label", "dispatch".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vda1".into(), format:work::FormatChoice::Ext4, label:"dispatch".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Draft(work::Operation::CreateGpt { disk:"/dev/vdb".into() }, false),
    Step::Click(Target::Control("create-gpt")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Field("name")),
    Step::Replace("reserve".into()),
    Step::FieldValue("name", "reserve".into()),
    Step::Click(Target::Field("start")),
    Step::Replace("2048".into()),
    Step::FieldValue("start", "2048".into()),
    Step::Click(Target::Field("end")),
    Step::Replace("10239".into()),
    Step::FieldValue("end", "10239".into()),
    Step::Click(Target::Control("hint")),
    Step::Click(Target::Choice("hint-LinuxSwap".into())),
    Step::Draft(work::Operation::AddPartition { disk:"/dev/vdb".into(), name:"reserve".into(), start:2048, end:10239, type_hint:core::TypeHint::LinuxSwap }, false),
    Step::Click(Target::Control("add")),
    Step::Click(Target::Control("disk")),
    Step::Click(Target::Choice("/dev/vdb".into())),
    Step::Selected("/dev/vdb".into()),
    Step::Click(Target::Control("partition")),
    Step::Click(Target::Choice("partition-1".into())),
    Step::Click(Target::Control("content-type")),
    Step::Click(Target::Choice("format-Swap".into())),
    Step::Click(Target::Field("label")),
    Step::Replace("reserve".into()),
    Step::FieldValue("label", "reserve".into()),
    Step::Click(Target::Field("uuid")),
    Step::Replace("".into()),
    Step::FieldValue("uuid", "".into()),
    Step::Draft(work::Operation::Format { partition:"/dev/vdb1".into(), format:work::FormatChoice::Swap, label:"reserve".into(), uuid:None }, false),
    Step::Click(Target::Control("format")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Click(Target::Field("name")),
    Step::Replace("unfinished".into()),
    Step::FieldValue("name", "unfinished".into()),
    Step::RememberWarm("warm-return".into()),
    Step::Save,
    Step::Click(Target::Back),
    Step::Click(Target::Start(3)),
    Step::Task(3),
    Step::ResumeExact("warm-return".into(), false),
    Step::Remember("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Reset),
    Step::Click(Target::Cancel),
    Step::Same("cancel-before".into(), Scope::Progress),
    Step::Click(Target::Field("name")),
    Step::Remember("bad-packet".into(), Scope::Progress),
    Step::InvalidPacket,
    Step::Same("bad-packet".into(), Scope::Progress),
    Step::Notice(true),
    Step::Click(Target::Field("name")),
    Step::Replace("resumed".into()),
    Step::FieldValue("name", "resumed".into()),
    Step::Notice(false),
    Step::Key(egui::Key::Tab, Default::default()),
    Step::Focus("start"),
    Step::Key(egui::Key::Tab, egui::Modifiers { shift:true,..Default::default() }),
    Step::Focus("name"),
    Step::Click(Target::Mode(true)),
    Step::Shot("lpic1-disk-delivery-challenge".into(), Target::Field("name")),
    Step::Click(Target::Mode(false)),
    Step::Click(Target::Control("assignment")),
    Step::Save,
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.00".into())),
    Step::Window(true),
    Step::Setting(1.0_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-100-light-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.00".into())),
    Step::Window(false),
    Step::Setting(1.0_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-100-light-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.00".into())),
    Step::Window(true),
    Step::Setting(1.0_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-100-dark-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.00".into())),
    Step::Window(false),
    Step::Setting(1.0_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-100-dark-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.15".into())),
    Step::Window(true),
    Step::Setting(1.15_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-115-light-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.15".into())),
    Step::Window(false),
    Step::Setting(1.15_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-115-light-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.15".into())),
    Step::Window(true),
    Step::Setting(1.15_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-115-dark-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.15".into())),
    Step::Window(false),
    Step::Setting(1.15_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-115-dark-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.25".into())),
    Step::Window(true),
    Step::Setting(1.25_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-125-light-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.25".into())),
    Step::Window(false),
    Step::Setting(1.25_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-125-light-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.25".into())),
    Step::Window(true),
    Step::Setting(1.25_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-125-dark-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.25".into())),
    Step::Window(false),
    Step::Setting(1.25_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-125-dark-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.50".into())),
    Step::Window(true),
    Step::Setting(1.5_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-150-light-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.50".into())),
    Step::Window(false),
    Step::Setting(1.5_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-150-light-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.50".into())),
    Step::Window(true),
    Step::Setting(1.5_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-150-dark-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:1.50".into())),
    Step::Window(false),
    Step::Setting(1.5_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-150-dark-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:2.00".into())),
    Step::Window(true),
    Step::Setting(2.0_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-200-light-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-light".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:2.00".into())),
    Step::Window(false),
    Step::Setting(2.0_f32, "dlms-light".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-200-light-narrow".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:2.00".into())),
    Step::Window(true),
    Step::Setting(2.0_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-200-dark-wide".into(), Target::Field("name")),
    Step::Click(Target::Theme),
    Step::Click(Target::HeaderChoice("theme:dlms-dark".into())),
    Step::Click(Target::Scale),
    Step::Click(Target::HeaderChoice("scale:2.00".into())),
    Step::Window(false),
    Step::Setting(2.0_f32, "dlms-dark".into()),
    Step::Geometry(0),
    Step::Geometry(1),
    Step::Geometry(2),
    Step::Geometry(3),
    Step::Geometry(4),
    Step::Geometry(5),
    Step::Geometry(6),
    Step::Geometry(7),
    Step::Geometry(8),
    Step::Geometry(9),
    Step::Geometry(10),
    Step::Geometry(11),
    Step::Geometry(12),
    Step::Geometry(13),
    Step::Click(Target::Field("name")),
    Step::Focus("name"),
    Step::Shot("disk-delivery-layout-200-dark-narrow".into(), Target::Field("name")),
    Step::Click(Target::Control("check")),
    Step::Rows(vec![true,true,true], false),
    Step::Save,
    Step::Done,
]
;assert!(p.len()<RESERVE);p})
}
#[derive(Clone)]
struct Warm {
    portable: Value,
    caller_receipts: Value,
    raw: Vec<u8>,
    raw_receipts: Value,
}
#[derive(Clone, Default)]
struct Qa {
    stage: usize,
    last_root: Option<u64>,
    phase: u8,
    polls: usize,
    began: f64,
    geometry: Option<egui::Rect>,
    stable: u8,
    wheels: usize,
    wheel_until: f64,
    snapshots: BTreeMap<String, Value>,
    warm: BTreeMap<String, Warm>,
    next_returns: usize,
    continues: usize,
    shots: usize,
    finished: bool,
    closing: Option<Value>,
    private_fixture: bool,
    catalog: Option<Value>,
    units: Option<Value>,
}
fn key() -> egui::Id {
    egui::Id::new("conditional-disk-native-state")
}
#[cfg(test)]
pub(in crate::ui) fn phase_bounds() -> (usize, usize) {
    (FIRST, FIRST + plan().len() - 1)
}
pub(in crate::ui) fn planned_captures() -> Vec<String> {
    plan()
        .iter()
        .filter_map(|s| {
            if let Step::Shot(n, _) = s {
                Some(n.clone())
            } else {
                None
            }
        })
        .collect()
}
#[cfg(test)]
pub(in crate::ui) fn completed(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<Qa>(key()))
        .is_some_and(|q| q.finished)
}
#[cfg(test)]
pub(in crate::ui) fn recency_observations(ctx: &egui::Context) -> Option<(usize, usize)> {
    ctx.data(|d| d.get_temp::<Qa>(key()))
        .map(|q| (q.next_returns, q.continues))
}
impl LabApp {
    pub(in crate::ui) fn disk_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("disk-preparation-fixture")).then_some(FIRST)
    }
    pub(in crate::ui) fn disk_capture_active(&self) -> bool {
        self.smoke && (FIRST..94_000).contains(&self.smoke_stage)
    }
    fn disk_session(&self) -> &work::Session {
        let source = &self.scenarios[self.selected.expect("Actual disk selection")];
        let a = &self.progress.attempts[&source.id];
        work::context(a, source).unwrap();
        let engine::State::PartitionFs { simulation } = &a.state else {
            panic!("Actual typed disk attempt")
        };
        simulation
    }
    fn disk_practice(&self) -> bool {
        let s = self.disk_session();
        !self.progress.attempts[&self.scenarios[self.selected.unwrap()].id].challenge
            && s.view == View::Example
    }
    fn disk_editor(&self) -> &work::Editor {
        self.disk_session().selected_editor(self.disk_practice())
    }
    fn disk_field(&self, key: &str) -> &str {
        let e = self.disk_editor();
        let d = &e.drafts[&e.selected];
        match key {
            "name" => &d.name,
            "start" => &d.start,
            "end" => &d.end,
            "label" => &d.label,
            "uuid" => &d.uuid,
            _ => panic!("Closed disk field"),
        }
    }
    fn disk_projection(&self, scope: Scope) -> Value {
        let id = &self.scenarios[self.selected.unwrap()].id;
        let a = &self.progress.attempts[id];
        let s = self.disk_session();
        match scope {
            Scope::Assignment => {
                json!({"current":s.current,"editor":s.assignment_editor,"transcript":s.transcript,"checked":s.checked,"hints":a.hints,"mode":a.challenge,"step":a.step,"visited":a.visited,"caller_receipts":self.progress.migration_receipts})
            }
            Scope::Example => {
                json!({"model":s.example,"transcript":s.example_transcript,"checked":s.example_checked})
            }
            Scope::Progress => {
                json!({"portable":self.progress,"caller_receipts":self.progress.migration_receipts})
            }
            Scope::Others => {
                let mut p = serde_json::to_value(&self.progress).unwrap();
                p["attempts"].as_object_mut().unwrap().remove(id);
                json!({"portable":p,"caller_receipts":self.progress.migration_receipts})
            }
        }
    }
    fn disk_source_fixture(&mut self) -> bool {
        assert!(
            self.smoke && !self.synthetic && !self.developer && self.progress.attempts.is_empty()
        );
        let present = IDS
            .iter()
            .filter(|id| self.scenarios.iter().any(|s| s.id == **id))
            .count();
        assert!(
            present == 0 || present == 4,
            "No partial private registration accepted"
        );
        let unit: course::Unit = serde_json::from_value(review()["unit_proposal"].clone()).unwrap();
        if present == 0 {
            let old = serde_json::to_value(&self.scenarios).unwrap();
            let old_units = serde_json::to_value(&self.units).unwrap();
            let count = self.scenarios.len();
            let mut texts = self
                .scenarios
                .chunks(16)
                .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
                .collect::<Vec<_>>();
            texts.push(include_str!("../../../scenarios/disk-preparation-v1.json").into());
            let catalog =
                pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
            assert_eq!(serde_json::to_value(&catalog[..count]).unwrap(), old);
            let mut units = self.units.clone();
            units.push(unit.clone());
            let units =
                course::load(&json!({"version":1,"units":units}).to_string(), &catalog).unwrap();
            assert_eq!(
                serde_json::to_value(&units[..units.len() - 1]).unwrap(),
                old_units
            );
            self.scenarios = catalog;
            self.units = units; // Source-only QA fixture; no Attempt/progress installed.
        }
        for (n, s) in sources().iter().enumerate() {
            let actual = self.scenarios.iter().find(|a| a.id == IDS[n]).unwrap();
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(s).unwrap()
            );
        }
        let actual = self.units.iter().find(|u| u.id == unit.id).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(&unit).unwrap()
        );
        assert!(self.progress.attempts.is_empty());
        present == 0
    }
    fn disk_draft_matches(&self, o: &work::Operation, practice: bool) {
        assert_eq!(self.disk_practice(), practice);
        let e = self.disk_editor();
        let d = &e.drafts[&e.selected];
        match o {
            work::Operation::CreateGpt { disk } => assert_eq!(&e.selected, disk),
            work::Operation::AddPartition {
                disk,
                name,
                start,
                end,
                type_hint,
            } => {
                assert_eq!(&e.selected, disk);
                assert_eq!(&d.name, name);
                assert_eq!(d.start, start.to_string());
                assert_eq!(d.end, end.to_string());
                assert_eq!(&d.type_hint, type_hint);
            }
            work::Operation::Format {
                partition,
                format,
                label,
                uuid,
            } => {
                assert_eq!(e.selected, &partition[..8]);
                assert_eq!(d.selected_partition, Some(partition[8..].parse().unwrap()));
                assert_eq!(&d.format, format);
                assert_eq!(&d.label, label);
                assert_eq!(d.uuid, uuid.clone().unwrap_or_default());
            }
        }
    }
    fn disk_target(&self, ctx: &egui::Context, t: &Target) -> (egui::Rect, egui::Rect) {
        let root = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let m = &self.metrics;
        match t {
            Target::Search => (m.library_search.expect("Actual library search"), root),
            Target::Start(n) => {
                let at = m
                    .card_verification
                    .iter()
                    .position(|r| r.0 == IDS[*n])
                    .expect("Actual fixed task card");
                (m.card_actions[at].0, root)
            }
            Target::Field(k) => {
                let r = m
                    .partition_fs
                    .fields
                    .iter()
                    .find(|r| r.0 == *k)
                    .expect("Actual native field");
                (r.1, r.3.intersect(root))
            }
            Target::Control("next") => (m.next_unit.expect("Actual shared Next"), root),
            Target::Control(k) => {
                let r = m
                    .partition_fs
                    .controls
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| panic!("Actual disk control{k}"));
                (r.1, r.2.intersect(root))
            }
            Target::Choice(k) => {
                let r = m
                    .partition_fs
                    .choices
                    .iter()
                    .find(|r| r.0 == *k)
                    .expect("Actual native popup choice");
                (r.1, r.2.intersect(ctx.content_rect()))
            }
            Target::Back => (m.back.unwrap(), root),
            Target::Reset => (m.reset.unwrap(), root),
            Target::Cancel => (m.reset_cancel.unwrap(), ctx.content_rect()),
            Target::Confirm => (m.confirm_reset.unwrap(), ctx.content_rect()),
            Target::Mode(challenge) => (
                if *challenge {
                    m.mode_challenge.unwrap()
                } else {
                    m.mode_learn.unwrap()
                },
                root,
            ),
            Target::Theme => (m.reopen_theme.unwrap(), ctx.content_rect()),
            Target::Scale => (m.reopen_scale.unwrap(), ctx.content_rect()),
            Target::HeaderChoice(k) => {
                let r = m
                    .physical_picker_choices
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap();
                (r.1, r.2.intersect(ctx.content_rect()))
            }
            Target::Goal => (m.partition_fs.goal.unwrap(), root),
            Target::Notice => (m.partition_fs.notice.expect("Actual action refusal"), root),
            Target::Feedback => {
                let r = m
                    .partition_fs
                    .rows
                    .iter()
                    .find(|r| !r.0)
                    .or_else(|| m.partition_fs.rows.first())
                    .expect("Actual current check row");
                (r.1, root)
            }
            Target::Text(wanted) => {
                let wanted = if wanted == "HELP" { work::HELP } else { wanted };
                let r = m
                    .partition_fs
                    .text_rects
                    .iter()
                    .find(|r| r.0 == wanted)
                    .expect("Actual requested inspection/help text");
                (r.1, r.2.intersect(root))
            }
            Target::ReferenceFirst => {
                let engine::Goal::PartitionFs { definition } =
                    &self.scenarios[self.selected.unwrap()].goal
                else {
                    panic!()
                };
                self.disk_target(ctx, &Target::Text(definition.reference[0].description()))
            }
            Target::History(n) => {
                let s = self.disk_session();
                let rows = if self.disk_practice() {
                    &s.example_transcript
                } else {
                    &s.transcript
                };
                let row = &rows[*n];
                self.disk_target(ctx, &Target::Text(format!("{}\n{}", row.input, row.output)))
            }
        }
    }
    fn disk_warm(&mut self) -> Warm {
        let portable = serde_json::to_value(&self.progress).unwrap();
        let caller_receipts = serde_json::to_value(&self.progress.migration_receipts).unwrap();
        self.save();
        assert!(!self.dirty && self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap();
        let mut saved: Value = serde_json::from_slice(&raw).unwrap();
        let raw_receipts = saved
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .expect("Real saved provenance");
        assert_eq!(saved, portable, "Every saved portable field is current");
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            portable,
            "Read-only load preserves the entire portable profile"
        );
        assert_eq!(
            serde_json::to_value(&loaded.migration_receipts).unwrap(),
            raw_receipts,
            "Real raw writer receipts are retained separately from skipped portable serialization"
        );
        assert_eq!(
            serde_json::to_value(&self.progress).unwrap(),
            portable,
            "Loading never installs replacement progress"
        );
        assert_eq!(
            serde_json::to_value(&self.progress.migration_receipts).unwrap(),
            caller_receipts,
            "Save/load never stamps caller receipts"
        );
        assert_eq!(
            progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap(),
            raw
        );
        Warm {
            portable,
            caller_receipts,
            raw,
            raw_receipts,
        }
    }
    fn disk_resume_exact(&mut self, before: &Warm, via_next: bool) {
        let i = self.selected.expect("Real returned activity");
        let source = &self.scenarios[i];
        let mut expected = before.portable.clone();
        let old_clock = expected["clock"].as_u64().unwrap();
        let clock = old_clock.checked_add(if via_next { 2 } else { 1 }).unwrap();
        expected["clock"] = json!(clock);
        assert!(
            expected["attempts"]
                .as_object()
                .unwrap()
                .contains_key(&source.id)
        );
        expected["attempts"][&source.id]["visited"] = json!(clock);
        if via_next {
            assert_eq!(source.id, IDS[0]);
            let peer = self.scenarios.iter().find(|s| s.id == IDS[1]).unwrap();
            assert_eq!(peer.id, IDS[1]);
            assert!(
                !expected["attempts"]
                    .as_object()
                    .unwrap()
                    .contains_key(&peer.id),
                "Next creates only the genuinely absent peer"
            );
            let mut fresh = progress::Attempt::new(peer, old_clock.checked_add(1).unwrap());
            work::binding::prepare(&mut fresh, peer).unwrap(); // Expected value only; never installed.
            work::context(&fresh, peer).unwrap();
            assert!(!progress::current_solved(&fresh, peer));
            expected["attempts"][&peer.id] = serde_json::to_value(&fresh).unwrap();
            assert!(
                !progress::current_solved(&self.progress.attempts[&peer.id], peer),
                "Next never invents peer mastery"
            );
        }
        assert_eq!(
            serde_json::to_value(&self.progress).unwrap(),
            expected,
            "Whole profile changes only exact clock/visit recency and the genuinely fresh Next peer"
        );
        assert_eq!(
            serde_json::to_value(&self.progress.migration_receipts).unwrap(),
            before.caller_receipts
        );
        let now = self.disk_warm();
        assert_eq!(now.portable, expected);
        assert_eq!(now.caller_receipts, before.caller_receipts);
        assert_eq!(now.raw_receipts, before.raw_receipts);
        let mut expected_raw: Value = serde_json::from_slice(&before.raw).unwrap();
        expected_raw["clock"] = expected["clock"].clone();
        expected_raw["attempts"] = expected["attempts"].clone();
        assert_eq!(
            serde_json::from_slice::<Value>(&now.raw).unwrap(),
            expected_raw,
            "Every persisted field including raw receipts follows the same exact recency law"
        );
    }
    fn disk_reveal(&mut self, ctx: &egui::Context, t: &Target, q: &mut Qa) -> bool {
        let now = ctx.input(|i| i.time);
        let (r, clip) = self.disk_target(ctx, t);
        let tolerance = 1. / ctx.pixels_per_point();
        if now < q.wheel_until {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(q.wheel_until - now));
            return false;
        }
        let same = q.geometry.is_some_and(|old| {
            (old.min - r.min).length() < tolerance * 0.25
                && (old.max - r.max).length() < tolerance * 0.25
        });
        q.stable = if same { q.stable.saturating_add(1) } else { 0 };
        q.geometry = Some(r);
        if q.stable < 2 {
            ctx.request_repaint();
            return false;
        }
        if smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return true;
        }
        assert!(
            r.left() >= clip.left() - tolerance && r.right() <= clip.right() + tolerance,
            "disk target horizontally clipped: {t:?}, {r:?}, {clip:?}"
        );
        assert!(
            q.wheels < 20,
            "Bounded disk native reveal failed: {t:?}, {r:?}, {clip:?}, stage{}",
            q.stage
        );
        q.wheels += 1;
        let popup = matches!(t, Target::HeaderChoice(_) | Target::Choice(_));
        let point = egui::pos2(
            if popup {
                r.center().x
            } else {
                clip.left() + 2.
            },
            clip.center().y,
        );
        self.pending_events.extend([
            egui::Event::PointerMoved(point),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (clip.center().y - r.center().y)
                        .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                ),
                modifiers: Default::default(),
            },
        ]);
        q.stable = 0;
        q.geometry = None;
        q.wheel_until = now + 0.15;
        ctx.request_repaint();
        false
    }
    fn disk_click(&mut self, ctx: &egui::Context, t: &Target, q: &mut Qa) -> bool {
        if q.phase == 2 {
            return true;
        }
        if !self.disk_reveal(ctx, t, q) {
            return false;
        }
        let (r, _) = self.disk_target(ctx, t);
        let point = r.center();
        let hover = ctx.input(|i| i.pointer.hover_pos());
        if q.phase == 0 || hover.is_none_or(|p| p.distance(point) > 0.25 / ctx.pixels_per_point()) {
            self.pending_events.push(egui::Event::PointerMoved(point));
            q.phase = 1;
            ctx.request_repaint();
            return false;
        }
        if ctx.input(|i| i.pointer.delta().length()) >= 0.25 / ctx.pixels_per_point() {
            ctx.request_repaint();
            return false;
        }
        assert!(
            ctx.input(|i| i.pointer.delta().length()) < 0.25 / ctx.pixels_per_point(),
            "Hover must settle before press, never create a scroll drag"
        );
        self.pending_events
            .extend(
                [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    }),
            );
        q.phase = 2;
        ctx.request_repaint();
        false
    }
    pub(in crate::ui) fn disk_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let mut q = ctx
            .data_mut(|d| d.remove_temp::<Qa>(key()))
            .unwrap_or_default();
        if q.finished {
            assert_eq!(
                q.closing.as_ref().unwrap(),
                &serde_json::to_value(&self.progress).unwrap(),
                "Delayed close may not change completed work"
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            ctx.data_mut(|d| d.insert_temp(key(), q));
            return (false, None);
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        if q.last_root == Some(root) {
            ctx.data_mut(|d| d.insert_temp(key(), q));
            return (false, None);
        }
        q.last_root = Some(root);
        let now = ctx.input(|i| i.time);
        if q.stage != stage {
            q.stage = stage;
            q.phase = 0;
            q.polls = 0;
            q.began = now;
            q.geometry = None;
            q.stable = 0;
            q.wheels = 0;
        }
        let waiting_for_wheel = now < q.wheel_until;
        if !waiting_for_wheel {
            q.polls += 1;
        }
        assert!(
            q.polls <= 300 && now - q.began < 15.,
            "disk native stage budget: {stage}, {:?}, phase{}, polls{}, elapsed{}, now{}, wheel_until{}, wheels{}, stable{}, geometry{:?}, back{:?}, main{:?}, scroll{:?}, content{:?}, pointer{:?}, pointer_delta{:?}, selected{:?}",
            plan().get(stage - FIRST),
            q.phase,
            q.polls,
            now - q.began,
            now,
            q.wheel_until,
            q.wheels,
            q.stable,
            q.geometry,
            self.metrics.back,
            self.metrics.main_viewport,
            self.metrics.main_scroll_offset,
            self.metrics.main_content_size,
            ctx.input(|i| i.pointer.hover_pos()),
            ctx.input(|i| i.pointer.delta()),
            self.selected
        );
        // A deliberate smooth-wheel deadline is not another geometry/click
        // attempt. Fast optimized frames must not consume all300 work polls
        // while waiting150ms; the original15-second wall bound still applies.
        if waiting_for_wheel {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(q.wheel_until - now));
            ctx.data_mut(|d| d.insert_temp(key(), q));
            return (false, None);
        }
        let step = plan()
            .get(stage - FIRST)
            .expect("Closed disk route inventory")
            .clone();
        let mut advance = true;
        let mut shot = None;
        match step {
            Step::Init => {
                q.private_fixture = self.disk_source_fixture();
                q.catalog = Some(serde_json::to_value(&self.scenarios).unwrap());
                q.units = Some(serde_json::to_value(&self.units).unwrap());
            }
            Step::Click(t) => advance = self.disk_click(ctx, &t, &mut q),
            Step::Replace(text) => {
                assert!(text.len() <= 160 && !text.contains(['\n', '\r', '\0']));
                self.pending_events
                    .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers {
                            ctrl: true,
                            command: true,
                            ..Default::default()
                        },
                    }));
                if text.is_empty() {
                    self.pending_events
                        .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                            key: egui::Key::Backspace,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: Default::default(),
                        }));
                } else {
                    self.pending_events.push(egui::Event::Paste(text));
                }
            }
            Step::FieldValue(key, text) => assert_eq!(self.disk_field(key), text),
            Step::Draft(op, practice) => self.disk_draft_matches(&op, practice),
            Step::Key(key, modifiers) => {
                self.pending_events
                    .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers,
                    }))
            }
            Step::Focus(key) => {
                let id = self
                    .metrics
                    .partition_fs
                    .fields
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap()
                    .2;
                if ctx.memory(|m| m.focused()) != Some(id) {
                    advance = false;
                    ctx.request_repaint();
                }
            }
            Step::Remember(name, scope) => {
                q.snapshots.insert(name, self.disk_projection(scope));
            }
            Step::Same(name, scope) => assert_eq!(
                &self.disk_projection(scope),
                q.snapshots.get(&name).expect("Actual preceding snapshot")
            ),
            Step::RememberWarm(name) => {
                q.warm.insert(name, self.disk_warm());
            }
            Step::ResumeExact(name, via_next) => {
                self.disk_resume_exact(q.warm.get(&name).unwrap(), via_next);
                if via_next {
                    q.next_returns += 1;
                } else {
                    q.continues += 1;
                }
            }
            Step::GuidanceDelta(name, count) => {
                let mut expected = q.snapshots.get(&name).unwrap().clone();
                let id = &self.scenarios[self.selected.unwrap()].id;
                let old = expected["portable"]["attempts"][id]["hints"]
                    .as_u64()
                    .unwrap();
                let next = old.checked_add(count as u64).unwrap();
                assert!(next <= self.scenarios[self.selected.unwrap()].hints.len() as u64);
                expected["portable"]["attempts"][id]["hints"] = json!(next);
                assert_eq!(
                    self.disk_projection(Scope::Progress),
                    expected,
                    "Only explicitly requested persisted hint changes"
                );
            }
            Step::Task(n) => {
                assert_eq!(
                    self.selected,
                    self.scenarios.iter().position(|s| s.id == IDS[n])
                );
                assert_eq!(self.scenarios[self.selected.unwrap()].id, IDS[n]);
            }
            Step::Selected(disk) => assert_eq!(self.disk_editor().selected, disk),
            Step::Answers(expected) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::PartitionFs { definition } = &s.goal else {
                    panic!()
                };
                let wanted = definition.reference[0].description();
                let shown = self
                    .metrics
                    .partition_fs
                    .text_rects
                    .iter()
                    .any(|r| r.0 == wanted);
                assert_eq!(
                    shown, expected,
                    "A hint alone does not request worked answers"
                );
            }
            Step::Rows(expected, practice) => {
                let s = self.disk_session();
                assert!(if practice {
                    s.example_checked
                } else {
                    s.checked
                });
                assert_eq!(
                    self.metrics
                        .partition_fs
                        .rows
                        .iter()
                        .map(|r| r.0)
                        .collect::<Vec<_>>(),
                    expected
                );
                if !practice {
                    assert_eq!(self.metrics.completion, expected.iter().all(|v| *v));
                }
            }
            Step::Completion(expected) => assert_eq!(self.metrics.completion, expected),
            Step::Notice(expected) => assert_eq!(self.notice.is_some(), expected),
            Step::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                work::context(a, s).unwrap();
                assert_eq!(a.state, s.initial);
                assert!(!a.challenge && a.hints == 0 && a.step == 0 && a.transcript.is_empty());
                assert!(!progress::current_solved(a, s));
            }
            Step::PracticeFresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::PartitionFs { definition } = &s.goal else {
                    panic!()
                };
                let w = self.disk_session();
                assert_eq!(w.example, definition.lesson.fixture);
                assert!(w.example_transcript.is_empty() && !w.example_checked);
            }
            Step::Save => {
                let _observed = self.disk_warm();
            }
            Step::InvalidPacket => {
                self.pending_events.push(egui::Event::Paste(format!(
                    "reports{}ignored-tail",
                    " ".repeat(36)
                )));
                self.pending_events
                    .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    }));
            }
            Step::Window(wide) => {
                if q.phase == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(wide));
                    if !wide {
                        let n = ctx.input(|i| i.viewport().native_pixels_per_point.unwrap_or(1.));
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1100. / n,
                            950. / n,
                        )));
                    }
                    q.phase = 1;
                    advance = false;
                } else {
                    let p = ctx.content_rect().size() * ctx.pixels_per_point();
                    if ctx.input(|i| i.viewport().fullscreen) != Some(wide)
                        || !(if wide { p.x > 1400. } else { p.x <= 1300. })
                    {
                        advance = false;
                        ctx.request_repaint();
                    }
                }
            }
            Step::Setting(scale, theme) => {
                assert_eq!(self.theme_id, theme);
                assert!((self.scale - scale).abs() < 0.001);
            }
            Step::Geometry(index) => {
                let targets = [
                    Target::Control("disk"),
                    Target::Field("name"),
                    Target::Field("start"),
                    Target::Field("end"),
                    Target::Control("hint"),
                    Target::Control("add"),
                    Target::Control("partition"),
                    Target::Control("content-type"),
                    Target::Field("label"),
                    Target::Field("uuid"),
                    Target::Control("format"),
                    Target::Control("check"),
                    Target::Control("inspect"),
                    Target::Control("help"),
                ];
                if !self.disk_reveal(ctx, &targets[index], &mut q) {
                    advance = false;
                } else {
                    let tolerance = 1. / ctx.pixels_per_point();
                    for (k, r, _, clip) in &self.metrics.partition_fs.fields {
                        assert!(
                            r.left() >= clip.left() - tolerance
                                && r.right() <= clip.right() + tolerance,
                            "Disk field{k} horizontal clipping"
                        );
                    }
                    for (k, r, clip) in &self.metrics.partition_fs.controls {
                        assert!(
                            r.left() >= clip.left() - tolerance
                                && r.right() <= clip.right() + tolerance,
                            "Disk control{k} horizontal clipping"
                        );
                    }
                }
            }
            Step::Shot(name, target) => {
                if !self.disk_reveal(ctx, &target, &mut q) {
                    advance = false;
                } else {
                    q.shots += 1;
                    assert!(q.shots <= planned_captures().len());
                    shot = Some(name);
                }
            }
            Step::Done => {
                assert_eq!(q.shots, planned_captures().len());
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[id], s));
                }
                assert_eq!(
                    self.progress.attempts.len(),
                    4,
                    "Only real visited tasks exist"
                );
                assert_eq!(
                    q.catalog.as_ref().unwrap(),
                    &serde_json::to_value(&self.scenarios).unwrap()
                );
                assert_eq!(
                    q.units.as_ref().unwrap(),
                    &serde_json::to_value(&self.units).unwrap()
                );
                q.closing = Some(serde_json::to_value(&self.progress).unwrap());
                q.finished = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                println!(
                    "PASS conditional disk native route: fixed_source_fixture={}, four actual Start/configuration tasks; screenshots{}; no host disks, normal registration, full utility, platform or novice claim",
                    q.private_fixture, q.shots
                );
                advance = false;
            }
        }
        ctx.data_mut(|d| d.insert_temp(key(), q));
        (advance, shot)
    }
}
