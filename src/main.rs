#![forbid(unsafe_code)]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
mod startup_trace;
mod ui;
fn main() -> eframe::Result<()> {
    startup_trace::mark("main");
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|s| s == "--version" || s == "-V") {
        println!("{}", dlms_learning_lab::build_identity());
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("--validate-content") {
        match dlms_learning_lab::pack::bundled().and_then(|labs| {
            dlms_learning_lab::course::bundled(&labs).map(|units| (labs.len(), units.len()))
        }) {
            Ok((labs, units)) => println!("Validated {labs} authored labs and {units} units"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("--validate-pack") {
        let result = args
            .get(2)
            .ok_or("Expected JSON path".to_string())
            .and_then(|p| dlms_learning_lab::pack::load_file(std::path::Path::new(p)));
        match result {
            Ok(s) => println!("Validated {} scenarios", s.len()),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    if args
        .iter()
        .skip(1)
        .any(|a| !matches!(a.as_str(), "--ui-smoke" | "--developer"))
    {
        eprintln!(
            "Unsupported launch option. Open without flags for the normal app; --ui-smoke uses isolated test data."
        );
        std::process::exit(2);
    }
    // Mirror pinned winit's backend selection before it creates the loop.
    // Linux Wayland requests interval 0 to avoid the observed EGL wait; other
    // backends retain the default. This is not a claim of a universal driver fix.
    let display = std::env::var_os("WAYLAND_DISPLAY");
    let socket = std::env::var_os("WAYLAND_SOCKET");
    let wayland = wayland_env_selected(display.as_deref(), socket.as_deref());
    let diagnostic = std::env::var_os("DLMS_SMOKE_NO_VSYNC");
    let vsync = native_vsync(
        args.iter().any(|a| a == "--ui-smoke"),
        diagnostic.as_deref(),
        wayland,
    );
    if !vsync {
        startup_trace::mark("app-swapinterval-zero-requested");
    }
    let options = eframe::NativeOptions {
        vsync,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("org.dlms.LearningLab")
            .with_inner_size([1200., 850.])
            .with_min_inner_size([720., 560.]),
        ..Default::default()
    };
    startup_trace::mark("enter-run-native");
    eframe::run_native(
        "DLMS Learning Lab",
        options,
        Box::new(move |cc| {
            startup_trace::mark("creation-context-ready");
            let app = ui::LabApp::new(
                cc,
                args.contains(&"--ui-smoke".into()),
                args.contains(&"--developer".into()),
            );
            startup_trace::mark("application-ready");
            Ok(Box::new(app))
        }),
    )
}

/// winit0.30.13 selects Wayland for either nonempty Unicode variable. This app
/// has no forced-backend hook. Session type alone does not select its backend.
fn wayland_env_selected(
    display: Option<&std::ffi::OsStr>,
    socket: Option<&std::ffi::OsStr>,
) -> bool {
    [display, socket]
        .into_iter()
        .flatten()
        .any(|value| value.to_str().is_some_and(|s| !s.is_empty()))
}
fn native_vsync(smoke: bool, diagnostic: Option<&std::ffi::OsStr>, wayland: bool) -> bool {
    !(cfg!(target_os = "linux")
        && (wayland || (smoke && diagnostic == Some(std::ffi::OsStr::new("1")))))
}

#[cfg(test)]
mod startup_options_tests {
    use super::*;
    #[test]
    fn vsync_policy_preserves_other_backends_and_exact_smoke_override() {
        for value in [None, Some(""), Some("0"), Some("1"), Some("true")] {
            let value = value.map(std::ffi::OsStr::new);
            assert!(native_vsync(false, value, false));
            assert_eq!(
                native_vsync(true, value, false),
                !(cfg!(target_os = "linux") && value == Some(std::ffi::OsStr::new("1")))
            );
            for smoke in [false, true] {
                assert_eq!(native_vsync(smoke, value, true), !cfg!(target_os = "linux"));
            }
        }
        assert!(eframe::NativeOptions::default().vsync);
    }
    #[test]
    fn wayland_selection_matches_pinned_nonempty_unicode_rule() {
        let empty = std::ffi::OsStr::new("");
        let display = std::ffi::OsStr::new("wayland-0");
        let socket = std::ffi::OsStr::new("3");
        assert!(!wayland_env_selected(None, None));
        assert!(!wayland_env_selected(Some(empty), Some(empty)));
        for (a, b) in [
            (Some(display), None),
            (None, Some(socket)),
            (Some(empty), Some(socket)),
            (Some(display), Some(empty)),
        ] {
            assert!(wayland_env_selected(a, b));
        }
        assert!(wayland_env_selected(Some(std::ffi::OsStr::new(" ")), None));
    }
    #[cfg(unix)]
    #[test]
    fn nonunicode_signal_is_not_wayland_selection() {
        use std::os::unix::ffi::OsStrExt;
        let invalid = std::ffi::OsStr::from_bytes(b"\xff");
        assert!(!wayland_env_selected(Some(invalid), None));
        assert!(!wayland_env_selected(None, Some(invalid)));
        assert!(wayland_env_selected(
            Some(invalid),
            Some(std::ffi::OsStr::new("3"))
        ));
    }
}
