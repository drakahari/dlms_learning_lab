//! Fixed bootloader recovery fixtures using native controls and synthetic work.
use super::*;
#[derive(Clone, Copy)]
enum Fixture {
    Bios,
    Efi,
    Removable,
    Once,
}

struct DraftObservation<'a> {
    view: teaching::View,
    command: &'a str,
    checked: bool,
}

impl Fixture {
    fn id(self) -> &'static str {
        match self {
            Self::Bios => "lpic1-grub-bios-install",
            Self::Efi => "lpic1-grub-efi-registration",
            Self::Removable => "lpic2-efi-removable-recovery",
            Self::Once => "lpic2-grub-one-boot-recovery",
        }
    }
    fn prefix(self) -> &'static str {
        match self {
            Self::Bios => "BIOS_LOADER",
            Self::Efi => "EFI_LOADER",
            Self::Removable => "EFI_REMOVABLE",
            Self::Once => "GRUB_ONCE",
        }
    }
    fn controls(self, app: &LabApp) -> &[(String, egui::Rect, egui::Rect)] {
        match self {
            Self::Bios => &app.metrics.bios_loader.controls,
            Self::Efi | Self::Removable => &app.metrics.efi_loader.controls,
            Self::Once => &app.metrics.grub_once.controls,
        }
    }
    fn result(self, app: &LabApp) -> Option<bool> {
        match self {
            Self::Bios => app.metrics.bios_loader.result,
            Self::Efi | Self::Removable => app.metrics.efi_loader.result,
            Self::Once => app.metrics.grub_once.result,
        }
    }
    fn observe(self, app: &LabApp) -> DraftObservation<'_> {
        match &app.progress.attempts[self.id()].state {
            engine::State::BiosLoader { simulation } => {
                let draft = if simulation.view == teaching::View::Example {
                    &simulation.practice
                } else {
                    &simulation.assignment
                };
                DraftObservation {
                    view: simulation.view,
                    command: &draft.command,
                    checked: draft.current.checked.is_some(),
                }
            }
            engine::State::EfiLoader { simulation } => {
                let draft = if simulation.view == teaching::View::Example {
                    &simulation.practice
                } else {
                    &simulation.assignment
                };
                DraftObservation {
                    view: simulation.view,
                    command: &draft.command,
                    checked: draft.checked.is_some(),
                }
            }
            engine::State::GrubOnce { simulation } => {
                let draft = if simulation.view == teaching::View::Example {
                    &simulation.practice
                } else {
                    &simulation.assignment
                };
                DraftObservation {
                    view: simulation.view,
                    command: &draft.input.linux,
                    checked: draft.observed.is_some(),
                }
            }
            _ => panic!("Bootloader workspace expected"),
        }
    }
    fn worked_command(self, app: &LabApp, index: usize) -> &str {
        match &app.scenarios[app.selected.unwrap()].goal {
            engine::Goal::BiosLoader { definition } => &definition.example.reference[index],
            engine::Goal::EfiLoader { definition } => &definition.example.reference[index],
            engine::Goal::GrubOnce { definition } => &definition.example.reference.linux,
            _ => panic!("Bootloader task expected"),
        }
    }
    fn route(self) -> Vec<Op> {
        match self {
            Self::Bios => bios_route(),
            Self::Efi => efi_route(),
            Self::Removable => removable_route(),
            Self::Once => once_route(),
        }
    }
}
#[derive(Clone, Copy)]
enum Op {
    Style(f32, bool, bool),
    Open,
    View(&'static str),
    Click(&'static str),
    Worked,
    Insert(usize),
    Edit(&'static str),
    Run,
    Check(bool),
    Shot(&'static str, &'static str),
    Save,
    Back,
}
fn bios_route() -> Vec<Op> {
    use Op::*;
    vec![
        Style(1., false, false),
        Open,
        Shot("bios-loader-introduction", "top"),
        Shot("bios-loader-worked", "try"),
        View("example"),
        Shot("bios-loader-practice", "input"),
        Check(false),
        Edit("lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS"),
        Run,
        Edit("findmnt -T /workshop/boot"),
        Run,
        Edit("cat /workshop/boot/grub/grub.cfg"),
        Run,
        Worked,
        Insert(0),
        Shot("bios-loader-inert-insertion", "input"),
        Run,
        Check(true),
        Shot("bios-loader-practice-correct", "result"),
        Edit("grub-install /dev/vdc"),
        Run,
        Check(false),
        Shot("bios-loader-empty-menu-feedback", "result"),
        Edit("grub-install --boot-directory=/workshop/boot /dev/vdc"),
        Run,
        Check(true),
        View("assignment"),
        Shot("bios-loader-assignment", "input"),
        Check(false),
        Edit("grub-install /dev/vda --boot-directory '/target/boot' --target i386-pc"),
        Run,
        Check(true),
        Shot("bios-loader-complete", "result"),
        Edit("grub-install /dev/vda"),
        Run,
        Check(false),
        Shot("bios-loader-stale-success-cleared", "result"),
        Edit("grub-install --boot-directory=/target/boot /dev/vda"),
        Run,
        Check(true),
        Style(1.5, false, false),
        Shot("bios-loader-dark150-console", "input"),
        Style(2., true, true),
        Shot("bios-loader-light200-console", "input"),
        Shot("bios-loader-light200-result", "result"),
        Edit("grub-install --boot-directory='unfinished"),
        Save,
        Back,
    ]
}
fn efi_route() -> Vec<Op> {
    use Op::*;
    vec![
        Style(1., false, false),
        Open,
        Shot("efi-loader-introduction", "top"),
        Shot("efi-loader-worked", "try"),
        View("example"),
        Shot("efi-loader-practice", "input"),
        Check(false),
        Edit("lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS"),
        Run,
        Worked,
        Insert(0),
        Shot("efi-loader-inert-insertion", "input"),
        Run,
        Check(false),
        Shot("efi-loader-file-without-entry", "result"),
        Insert(1),
        Run,
        Check(true),
        Shot("efi-loader-practice-correct", "result"),
        View("assignment"),
        Shot("efi-loader-assignment", "input"),
        Check(false),
        Edit(
            "grub-install --no-nvram --boot-directory '/target/boot' --bootloader-id Recovered --target x86_64-efi --efi-directory '/rescue/esp'",
        ),
        Run,
        Check(false),
        Edit(r"efibootmgr -c -d /dev/vda -p 1 -L 'New Linux path' -l '\EFI\RECOVERED\GRUBX64.EFI'"),
        Run,
        Check(true),
        Shot("efi-loader-complete", "result"),
        Edit("efibootmgr --bootorder 0007,0000"),
        Run,
        Check(false),
        Shot("efi-loader-wrong-order", "result"),
        Edit("efibootmgr -o 0,7"),
        Run,
        Check(true),
        Style(1.5, false, false),
        Shot("efi-loader-dark150-console", "input"),
        Style(2., true, true),
        Shot("efi-loader-light200-console", "input"),
        Shot("efi-loader-light200-result", "result"),
        Edit("efibootmgr --bootorder"),
        Save,
        Back,
    ]
}

fn removable_route() -> Vec<Op> {
    use Op::*;
    vec![
        Style(1., false, false),
        Open,
        Shot("removable-efi-introduction", "top"),
        Shot("removable-efi-worked", "try"),
        View("example"),
        Shot("removable-efi-practice", "input"),
        Check(false),
        Shot("removable-efi-unavailable-directory", "result"),
        Edit("lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS"),
        Run,
        Worked,
        Insert(0),
        Run,
        Check(false),
        Shot("removable-efi-unavailable-root", "result"),
        Insert(1),
        Run,
        Check(true),
        Shot("removable-efi-practice-correct", "result"),
        View("assignment"),
        Shot("removable-efi-assignment", "input"),
        Check(false),
        Edit(
            "grub-install --removable --no-nvram --boot-directory '/rescue/recovery-boot' --target x86_64-efi --efi-directory /rescue/removable-esp",
        ),
        Run,
        Check(true),
        Shot("removable-efi-both-paths-pass", "result"),
        Edit(
            "grub-install --removable --boot-directory=/rescue/copied-boot --target=x86_64-efi --efi-directory=/rescue/removable-esp",
        ),
        Run,
        Check(false),
        Shot("removable-efi-stale-success-cleared", "result"),
        Edit(
            "grub-install --target=x86_64-efi --efi-directory=/rescue/removable-esp --boot-directory=/rescue/recovery-boot --removable",
        ),
        Run,
        Check(true),
        Style(1.5, false, false),
        Shot("removable-efi-dark150-console", "input"),
        Style(2., true, true),
        Shot("removable-efi-light200-console", "input"),
        Shot("removable-efi-light200-results", "result"),
        Edit("grub-install --removable --efi-directory"),
        Save,
        Back,
    ]
}

fn once_route() -> Vec<Op> {
    use Op::*;
    const ROOT: &str = "linux /maple-inspection root=UUID=7bcfe67b-fc52-4d25-90e9-0db0e316d598 ro";
    const REPAIR: &str = "linux '/ridge-inspection' systemd.unit=rescue.target ro root=UUID=985fa65c-bd81-4d48-8511-4504022931e5";
    const WRONG: &str = "linux '/ridge-inspection' systemd.unit=multi-user.target ro root=UUID=985fa65c-bd81-4d48-8511-4504022931e5";
    vec![
        Style(1., false, false),
        Open,
        Shot("grub-once-introduction", "top"),
        View("example"),
        Click("choice"),
        Click("menu-inspection"),
        Check(false),
        Shot("grub-once-example-root-blocked", "result"),
        Edit(ROOT),
        Check(false),
        Shot("grub-once-example-partial", "result"),
        Worked,
        Insert(0),
        Check(true),
        Shot("grub-once-example-passed", "result"),
        View("assignment"),
        Shot("grub-once-assignment", "input"),
        Click("choice"),
        Click("menu-inspection"),
        Edit(REPAIR),
        Check(true),
        Shot("grub-once-assignment-complete", "result"),
        Edit(WRONG),
        Shot("grub-once-stale-success-cleared", "test"),
        Check(false),
        Edit(REPAIR),
        Check(true),
        Style(1.5, false, false),
        Shot("grub-once-dark150-editor", "input"),
        Style(2., true, true),
        Shot("grub-once-light200-editor", "input"),
        Shot("grub-once-light200-results", "result"),
        Save,
        Back,
    ]
}

#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    next: f64,
    polls: usize,
    started: Option<Instant>,
    rect: Option<egui::Rect>,
    stable: usize,
}
fn target(app: &LabApp, fixture: Fixture, key: &str) -> Option<egui::Rect> {
    if key == "back" {
        app.metrics.back
    } else {
        fixture
            .controls(app)
            .iter()
            .find(|row| row.0 == key)
            .map(|row| row.1)
    }
}
fn click(app: &mut LabApp, ctx: &egui::Context, fixture: Fixture, key: &str) -> bool {
    app.click_visible_smoke_control(ctx, target(app, fixture, key))
}
fn capture(a: &mut LabApp, ctx: &egui::Context, name: &str) {
    a.capture_pending = true;
    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
        name.to_owned(),
    )));
}
impl LabApp {
    pub(in crate::ui) fn bios_loader_tick(&mut self, ctx: &egui::Context) {
        self.loader_install_tick(ctx, Fixture::Bios);
    }

    pub(in crate::ui) fn efi_loader_tick(&mut self, ctx: &egui::Context) {
        self.loader_install_tick(ctx, Fixture::Efi);
    }

    pub(in crate::ui) fn removable_efi_tick(&mut self, ctx: &egui::Context) {
        self.loader_install_tick(ctx, Fixture::Removable);
    }

    pub(in crate::ui) fn grub_once_tick(&mut self, ctx: &egui::Context) {
        self.loader_install_tick(ctx, Fixture::Once);
    }

    fn loader_install_tick(&mut self, ctx: &egui::Context, fixture: Fixture) {
        let key = egui::Id::new((fixture.id(), "loader-native-flow"));
        let mut f = ctx
            .data_mut(|d| d.get_temp::<Flow>(key))
            .unwrap_or_default();
        let ops = fixture.route();
        if f.at == ops.len() {
            println!(
                "{}_NATIVE_COMPLETE: {} actions; console, worked insertion, separate practice, boot checks, retry, invalidation, save and native captures",
                fixture.prefix(),
                ops.len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = match fixture {
            Fixture::Bios => 126000,
            Fixture::Efi => 127000,
            Fixture::Removable => 128000,
            Fixture::Once => 129000,
        } + f.at;
        let now = ctx.input(|i| i.time);
        let started = f.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut f.polls, now, f.next, started.elapsed());
        assert!(
            due.is_ok(),
            "{} action{} phase{}: {due:?}; elapsed{:?} capture_pending{} viewport{:?}",
            fixture.prefix(),
            f.at,
            f.phase,
            started.elapsed(),
            self.capture_pending,
            self.metrics.main_viewport
        );
        if !due.unwrap() {
            ctx.data_mut(|d| d.insert_temp(key, f));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let done = match ops[f.at] {
            Op::Style(scale, light, narrow) => {
                if f.phase == 0 {
                    self.scale = scale;
                    ctx.set_zoom_factor(scale);
                    self.theme_id = if light {
                        theme::inventory()
                            .iter()
                            .find(|t| t.light)
                            .unwrap()
                            .id
                            .clone()
                    } else {
                        theme::Preferences::default().theme
                    };
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                    if narrow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1268., 1308.,
                        )));
                    }
                    self.lesson_scroll_reset = true;
                    f.phase = 1;
                    false
                } else {
                    let rect = ctx.content_rect();
                    let ppp = ctx.pixels_per_point();
                    let size = if narrow {
                        (rect.width() * ppp - 1268.).abs() < 4.
                            && (rect.height() * ppp - 1308.).abs() < 4.
                    } else {
                        ctx.input(|i| i.viewport().fullscreen) != Some(false)
                    };
                    if size && (ctx.zoom_factor() - scale).abs() < 0.01 && f.rect == Some(rect) {
                        f.stable += 1;
                    } else {
                        f.stable = 0;
                    }
                    f.rect = Some(rect);
                    f.stable >= 2
                }
            }

            Op::Open => {
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                let index = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == fixture.id())
                    .unwrap();
                self.launch(index);
                true
            }
            Op::View(view) => {
                if f.phase == 0 {
                    if click(self, ctx, fixture, view) {
                        f.phase = 1;
                    }
                    false
                } else {
                    assert_eq!(
                        fixture.observe(self).view,
                        if view == "example" {
                            teaching::View::Example
                        } else {
                            teaching::View::Assignment
                        },
                    );
                    true
                }
            }
            Op::Click(control) => click(self, ctx, fixture, control),
            Op::Worked => click(self, ctx, fixture, "worked"),
            Op::Insert(index) => {
                if f.phase == 0 {
                    if click(self, ctx, fixture, &format!("insert-{index}")) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let observation = fixture.observe(self);
                    assert_eq!(observation.view, teaching::View::Example);
                    assert_eq!(observation.command, fixture.worked_command(self, index));
                    if matches!(fixture, Fixture::Once) {
                        assert!(!observation.checked);
                        assert_eq!(fixture.result(self), None);
                    } else {
                        assert!(observation.checked);
                        assert_eq!(fixture.result(self), Some(false));
                    }
                    assert!(!progress::current_solved(
                        &self.progress.attempts[fixture.id()],
                        &self.scenarios[self.selected.unwrap()],
                    ));
                    true
                }
            }
            Op::Edit(text) => {
                if f.phase == 0 {
                    if click(self, ctx, fixture, "input") {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    self.pending_events.extend([
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers {
                                command: true,
                                ..egui::Modifiers::CTRL
                            },
                        },
                        egui::Event::Paste(text.into()),
                    ]);
                    f.phase = 2;
                    false
                } else {
                    assert_eq!(fixture.observe(self).command, text);
                    true
                }
            }
            Op::Run => {
                if f.phase == 0 {
                    if click(self, ctx, fixture, "run") {
                        f.phase = 1;
                    }
                    false
                } else {
                    assert!(
                        fixture.observe(self).command.is_empty(),
                        "The supported command must run."
                    );
                    true
                }
            }
            Op::Check(pass) => {
                if f.phase == 0 {
                    if click(self, ctx, fixture, "test") {
                        f.phase = 1;
                    }
                    false
                } else {
                    assert_eq!(fixture.result(self), Some(pass));
                    assert_eq!(
                        progress::current_solved(
                            &self.progress.attempts[fixture.id()],
                            &self.scenarios[self.selected.unwrap()],
                        ),
                        pass && fixture.observe(self).view == teaching::View::Assignment,
                    );
                    true
                }
            }
            Op::Shot(name, control) => {
                if f.phase == 0 {
                    if control == "top"
                        || target(self, fixture, control)
                            .is_some_and(|r| self.reveal_smoke_control(ctx, r))
                    {
                        f.phase = 1;
                        f.next = now + 0.15;
                    }
                    false
                } else if f.phase == 1 {
                    capture(self, ctx, name);
                    f.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Back => {
                if f.phase == 0 {
                    if click(self, ctx, fixture, "back") {
                        f.phase = 1;
                    }
                    false
                } else {
                    assert!(self.selected.is_none());
                    true
                }
            }
        };
        if done {
            println!("{}_NATIVE_STEP {}", fixture.prefix(), f.at);
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.started = None;
            f.rect = None;
            f.stable = 0;
            f.next = now + 0.025;
        } else {
            f.next = f.next.max(now + 0.025);
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bios_loader_capture_does_not_reenter_context_lock() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let out = ctx.run(egui::RawInput::default(), |ctx| {
            capture(&mut a, ctx, "bios-loader-capture-regression")
        });
        assert!(a.capture_pending);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
        assert_eq!(
            bios_route()
                .iter()
                .filter(|op| matches!(op, Op::Shot(..)))
                .count(),
            12
        );
        assert_eq!(bios_route().len(), 48);
        assert_eq!(removable_route().len(), 41);
        assert_eq!(
            removable_route()
                .iter()
                .filter(|op| matches!(op, Op::Shot(..)))
                .count(),
            12
        );
        assert_eq!(once_route().len(), 34);
        assert_eq!(
            once_route()
                .iter()
                .filter(|op| matches!(op, Op::Shot(..)))
                .count(),
            10
        );
        assert_eq!(efi_route().len(), 44);
        assert_eq!(
            efi_route()
                .iter()
                .filter(|op| matches!(op, Op::Shot(..)))
                .count(),
            12
        );
    }
}
