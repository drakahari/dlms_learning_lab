//! Optional render-only normal MAC matrix, separate from the full behavioral flow.
use super::*;
use engine::State;
const START: usize = 20000;
const IDS: [&str; 4] = [
    "ccna-mac-learn-paths",
    "ccna-mac-stale-location",
    "ccna-mac-source-refresh",
    "ccna-mac-handover",
];
#[derive(Clone)]
struct Case {
    id: &'static str,
    theme: &'static str,
    scale: f32,
    fullscreen: bool,
}
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for fullscreen in [true, false] {
        for id in IDS {
            cases.push(Case {
                id,
                theme: "dlms-dark",
                scale: 1.,
                fullscreen,
            });
        }
    }
    for (scale, theme) in [(1.5, "dlms-light"), (2., "dlms-dark")] {
        for fullscreen in [true, false] {
            cases.push(Case {
                id: IDS[3],
                theme,
                scale,
                fullscreen,
            });
        }
    }
    cases
}
impl LabApp {
    pub(super) fn mac_layout_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let cases = cases();
        let index = (stage - START) / 5;
        let phase = (stage - START) % 5;
        if index == cases.len() {
            println!(
                "PASS scoped MAC render-only matrix: four unsolved assignments100 fullscreen/windowed and capstone150Light/200Dark fullscreen/windowed;24 actual captures; requested and actual geometry logged separately. No model actions/reference State publication/fullbehavior/novice/otherplatform claim."
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return (true, None);
        }
        let case = &cases[index];
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == case.id)
            .expect("normal registered MAC lesson");
        if phase == 0 {
            assert!(self.smoke && self._smoke_profile.is_some());
            assert!(
                self.path
                    .starts_with(self._smoke_profile.as_ref().unwrap().path())
            );
            self.launch(i);
            self.set_mac_view(i, teaching::View::Assignment);
            assert_eq!(
                self.progress.attempts[case.id].state,
                self.scenarios[i].initial
            );
            self.theme_id = case.theme.into();
            self.scale = case.scale;
            theme::apply(ctx, case.theme);
            self.lesson_scroll_reset = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(case.fullscreen));
            if !case.fullscreen {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 850.)));
            }
            return (true, None);
        }
        let State::MacLearning { .. } = &self.progress.attempts[case.id].state else {
            panic!()
        };
        assert_eq!(
            self.progress.attempts[case.id].state, self.scenarios[i].initial,
            "render-only matrix cannot alter applied State"
        );
        let requested = if case.fullscreen {
            "fullscreen"
        } else {
            "windowed"
        };
        let name = format!(
            "mac-layout-{index}-{}-{requested}-{}",
            case.id,
            (case.scale * 100.) as u16
        );
        match phase {
            1 => {
                let rect = ctx.content_rect();
                let ppp_bits = ctx.pixels_per_point().to_bits();
                let actual_fullscreen = ctx.input(|i| i.viewport().fullscreen);
                let (previous, ppp, stable, tries) = ctx
                    .data(|d| {
                        d.get_temp::<(egui::Rect, u32, u8, u8)>(egui::Id::new((
                            "mac-layout-settle",
                            stage,
                        )))
                    })
                    .unwrap_or((egui::Rect::NOTHING, 0, 0, 0));
                let stable = if previous == rect && ppp == ppp_bits {
                    stable + 1
                } else {
                    0
                };
                let tries = tries + 1;
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("mac-layout-settle", stage)),
                        (rect, ppp_bits, stable, tries),
                    )
                });
                assert!(
                    tries <= 30,
                    "layout viewport must settle: actual={rect:?} fullscreen={actual_fullscreen:?} requested={requested}"
                );
                if stable < 3
                    || (ctx.zoom_factor() - case.scale).abs() > 0.01
                    || actual_fullscreen != Some(case.fullscreen)
                {
                    return (false, None);
                }
                eprintln!(
                    "MAC layout settled {}: actual_content={rect:?}, ppp={}, fullscreen={actual_fullscreen:?}; window size only requested, not asserted",
                    case.id,
                    ctx.pixels_per_point()
                );
            }
            2 => return (true, Some(format!("{name}-top"))),
            3 => {
                let rect = if case.id == IDS[3] {
                    self.metrics.vlan_named_fields.last().map(|(_, _, _, r)| *r)
                } else if case.id == IDS[2] {
                    self.metrics.mac_aging
                } else {
                    self.metrics.mac_send
                }
                .expect("actual relevant native control");
                if !self.reveal_smoke_control(ctx, rect) {
                    return (false, None);
                }
            }
            4 => {
                let native = self
                    .metrics
                    .vlan_named_fields
                    .iter()
                    .find(|(s, p, k, _)| s == "core" && p == "right" && k == &"native")
                    .map(|(_, _, _, r)| *r);
                let allowed = self
                    .metrics
                    .vlan_named_fields
                    .iter()
                    .find(|(s, p, k, _)| s == "core" && p == "right" && k == &"allowed")
                    .map(|(_, _, _, r)| *r);
                eprintln!(
                    "MAC layout controls {name}: actual_content={:?} viewport={:?} native={native:?} allowed={allowed:?}; same-row indicates columns, separated rows indicate stacking",
                    ctx.content_rect(),
                    self.metrics.main_viewport
                );
                return (true, Some(format!("{name}-controls")));
            }
            _ => unreachable!(),
        }
        (true, None)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mac_layout_matrix_is_twelve_bounded_render_only_cases_with_wide_and_windowed_pairs() {
        let cases = cases();
        assert_eq!(cases.len(), 12);
        assert!(START + cases.len() * 5 < 20100);
        for id in IDS {
            assert!(
                cases
                    .iter()
                    .any(|c| c.id == id && c.scale == 1. && c.fullscreen)
            );
            assert!(
                cases
                    .iter()
                    .any(|c| c.id == id && c.scale == 1. && !c.fullscreen)
            );
        }
        for scale in [1.5, 2.] {
            for full in [false, true] {
                assert!(
                    cases
                        .iter()
                        .any(|c| c.id == IDS[3] && c.scale == scale && c.fullscreen == full)
                );
            }
        }
    }
    #[test]
    fn mac_layout_phase_one_records_real_context_measurements_without_lock_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false; // Only this explicit phase call, never the automatic smoke dispatcher.
        let i = app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
        app.launch(i);
        app.set_mac_view(i, teaching::View::Assignment);
        let original = app.progress.attempts[IDS[0]].state.clone();
        let mut completed = false;
        for expected_try in 1..=4 {
            let mut raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 850.),
                )),
                ..Default::default()
            };
            raw.viewports
                .entry(egui::ViewportId::ROOT)
                .or_default()
                .fullscreen = Some(true);
            let _ = ctx.run(raw, |ctx| {
                let (advance, capture) = app.mac_layout_native(ctx, START + 1);
                assert!(capture.is_none());
                completed = advance;
            });
            let sample = ctx
                .data(|d| {
                    d.get_temp::<(egui::Rect, u32, u8, u8)>(egui::Id::new((
                        "mac-layout-settle",
                        START + 1,
                    )))
                })
                .expect("actual phase1 measurement sample");
            assert_eq!(sample.0, ctx.content_rect());
            assert_eq!(sample.1, ctx.pixels_per_point().to_bits());
            assert_eq!(sample.3, expected_try);
            assert_eq!(sample.2, expected_try - 1);
            assert_eq!(app.progress.attempts[IDS[0]].state, original);
        }
        assert!(
            completed,
            "sameactualviewport reaches3stableticks without nestedContext read"
        );
    }
}
