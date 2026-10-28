//! Read-only geometry sweep of every text-size option in the normal NFS lab.
//! No saved policy, live mount, file, result or completion is installed.
use super::*;
const FIRST: usize = 48000;
const SCALES: [f32; 5] = [1., 1.15, 1.25, 1.5, 2.];
impl LabApp {
    pub(in crate::ui) fn nfs_alignment_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("nfs-alignment")).then_some(FIRST)
    }
    pub(in crate::ui) fn nfs_alignment_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!(self.smoke && self._smoke_profile.is_some());
        let offset = stage - FIRST;
        if offset == 60 {
            assert_eq!(
                ctx.data(
                    |d| d.get_temp::<serde_json::Value>(egui::Id::new("nfs-alignment-progress"))
                )
                .unwrap(),
                serde_json::to_value(&self.progress).unwrap()
            );
            println!(
                "PASS native NFS alignment: all five supported100/115/125/150/200 sizes, wide/narrow, light/dark, both editor/Save bands, actual measured geometry, unchanged portable progress; no grade or model action installed."
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return (false, None);
        }
        let case = offset / 3;
        let phase = offset % 3;
        let scale = SCALES[case % 5];
        let light = (case / 5) % 2 == 1;
        let narrow = case >= 10;
        if phase == 0 {
            if case == 0 {
                assert_eq!((self.scenarios.len(), self.units.len()), (164, 42));
                assert!(self.progress.attempts.is_empty());
                let i = self
                    .scenarios
                    .iter()
                    .position(|s| s.id == "lpic2-nfs-reference")
                    .unwrap();
                self.launch(i);
                self.progress
                    .attempts
                    .get_mut("lpic2-nfs-reference")
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .nfs_learning
                    .as_mut()
                    .unwrap()
                    .view = teaching::View::Assignment;
            }
            self.scale = scale;
            self.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
            if narrow {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
            }
            return (true, None);
        }
        let pixels = ctx.content_rect().size() * ctx.pixels_per_point();
        let fullscreen = ctx.input(|input| input.viewport().fullscreen);
        let width_ready = if narrow {
            (650. ..=1300.).contains(&pixels.x)
        } else {
            pixels.x >= 1500.
        };
        if (ctx.zoom_factor() - scale).abs() > 0.01 || fullscreen != Some(!narrow) || !width_ready {
            let key = egui::Id::new(("nfs-alignment-geometry-wait", stage));
            let tries = ctx.data_mut(|data| {
                let n = data.get_temp::<usize>(key).unwrap_or(0) + 1;
                data.insert_temp(key, n);
                n
            });
            assert!(
                tries <= 60,
                "Native geometry did not settle: requested_narrow={narrow}, actual_pixels={pixels:?}, fullscreen={fullscreen:?}"
            );
            ctx.request_repaint();
            return (false, None);
        }
        let selected = self.selected.unwrap();
        let id = &self.scenarios[selected].id;
        assert_eq!(
            &self.progress.attempts[id].state,
            &self.scenarios[selected].initial
        );
        assert!(!progress::current_solved(
            &self.progress.attempts[id],
            &self.scenarios[selected]
        ));
        let metrics = &self.metrics.nfs;
        assert_eq!(metrics.documents.len(), 2);
        let exports = metrics
            .controls
            .iter()
            .find(|r| r.0 == "save-exports")
            .unwrap()
            .1;
        let fstab = metrics
            .controls
            .iter()
            .find(|r| r.0 == "save-fstab")
            .unwrap()
            .1;
        if metrics.columns {
            assert!(
                (exports.top() - fstab.top()).abs() <= 1. / ctx.pixels_per_point(),
                "Actual Save row alignment"
            );
        }
        let key = if phase == 1 { "exports" } else { "fstab" };
        let document = metrics.documents.iter().find(|r| r.0 == key).unwrap().1;
        let save = if phase == 1 { exports } else { fstab };
        let band = document.union(save);
        if !self.reveal_smoke_control(ctx, band) {
            return (false, None);
        }
        let snapshot = serde_json::to_value(&self.progress).unwrap();
        let pin = egui::Id::new("nfs-alignment-progress");
        if case == 0 && phase == 1 {
            ctx.data_mut(|d| d.insert_temp(pin, snapshot));
        } else {
            assert_eq!(
                ctx.data(|d| d.get_temp::<serde_json::Value>(pin)).unwrap(),
                snapshot
            );
        }
        let content = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let actual = ctx.zoom_factor();
        println!(
            "NFS alignment case{case}/{key}: actual_content={content:?} PPP={ppp} zoom={actual}, columns={}, exports={exports:?}, fstab={fstab:?}",
            self.metrics.nfs.columns
        );
        (
            true,
            Some(format!(
                "nfs-alignment-{}-{}-{}-{key}",
                if narrow { "narrow" } else { "wide" },
                if light { "light" } else { "dark" },
                (scale * 100.).round() as u32
            )),
        )
    }
}
