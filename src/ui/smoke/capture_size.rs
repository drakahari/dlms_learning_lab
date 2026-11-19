//! Monitor-aware geometry for isolated native capture fixtures.
use super::*;

fn fitted(monitor: egui::Vec2) -> Result<egui::Vec2, &'static str> {
    if !monitor.is_finite() {
        return Err("Native capture needs finite monitor geometry");
    }
    let size = egui::vec2(
        1268_f32.min(monitor.x - 96.),
        1308_f32.min(monitor.y - 120.),
    )
    .floor();
    if size.x < 720. || size.y < 560. {
        return Err("Monitor cannot fit the native capture fixture and window decorations");
    }
    Ok(size)
}

pub(super) fn fits(actual_pixels: egui::Vec2, requested_pixels: egui::Vec2, native: f32) -> bool {
    // Wayland's compositor owns final window geometry. A stable smaller window
    // still exercises narrow reflow; a fullscreen/oversized result does not.
    actual_pixels.is_finite()
        && requested_pixels.is_finite()
        && native.is_finite()
        && native > 0.
        && actual_pixels.x >= 720. * native
        && actual_pixels.y >= 560. * native
        && actual_pixels.x <= requested_pixels.x + 4.
        && actual_pixels.y <= requested_pixels.y + 4.
}

pub(super) fn request(ctx: &egui::Context) -> egui::Vec2 {
    let (monitor, native) = ctx.input(|input| {
        (
            input.viewport().monitor_size,
            input.viewport().native_pixels_per_point,
        )
    });
    let native = native.expect("Native capture needs the OS display scale");
    let pixels_per_point = ctx.pixels_per_point();
    assert!(
        native.is_finite() && native > 0. && pixels_per_point.is_finite() && pixels_per_point > 0.
    );
    let monitor_pixels =
        monitor.expect("Native capture needs reported monitor geometry") * pixels_per_point;
    let logical = fitted(monitor_pixels / native).expect("Monitor can fit the capture fixture");
    let pixels = logical * native;
    // egui viewport commands use UI points, including the current text zoom.
    // Request only after zoom/fullscreen have settled, then compare actual pixels.
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(pixels / pixels_per_point));
    println!(
        "NATIVE_CAPTURE_SIZE monitor_pixels={monitor_pixels:?} native_scale={native} text_zoom={} requested_pixels={pixels:?}",
        ctx.zoom_factor()
    );
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_targets_fit_hd_and_large_monitors_without_changing_text_scale() {
        assert_eq!(
            fitted(egui::vec2(1920., 1080.)).unwrap(),
            egui::vec2(1268., 960.)
        );
        assert_eq!(
            fitted(egui::vec2(2560., 1440.)).unwrap(),
            egui::vec2(1268., 1308.)
        );
        assert_eq!(
            fitted(egui::vec2(1366., 768.)).unwrap(),
            egui::vec2(1268., 648.)
        );
        assert!(fitted(egui::vec2(800., 600.)).is_err());
        assert!(fitted(egui::vec2(f32::NAN, 1080.)).is_err());
        assert!(fits(
            egui::vec2(946., 929.),
            egui::vec2(1458.2, 941.8),
            1.15
        ));
        assert!(fits(egui::vec2(1268., 956.), egui::vec2(1268., 960.), 1.));
        assert!(!fits(
            egui::vec2(1920., 1080.),
            egui::vec2(1458.2, 941.8),
            1.15
        ));
        assert!(!fits(egui::vec2(700., 500.), egui::vec2(1268., 960.), 1.));
        assert!(!fits(
            egui::vec2(946., 929.),
            egui::vec2(1458.2, 941.8),
            f32::NAN
        ));
    }

    #[test]
    fn actual_capture_command_converts_ui_zoom_and_hidpi_to_the_requested_pixels() {
        for native in [1., 2.] {
            let ctx = egui::Context::default();
            ctx.set_zoom_factor(2.);
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600., 400.),
                )),
                ..Default::default()
            };
            let info = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
            info.monitor_size = Some(egui::vec2(960., 540.));
            info.native_pixels_per_point = Some(native);
            let output = ctx.run(input, |ctx| {
                assert_eq!(ctx.zoom_factor(), 2.);
                assert_eq!(request(ctx), egui::vec2(1268., 960.) * native);
            });
            assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.iter().any(|command| {
                matches!(command,egui::ViewportCommand::InnerSize(size) if *size==egui::vec2(634.,480.))
            }));
        }
    }
}
