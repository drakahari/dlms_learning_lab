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
    let options = eframe::NativeOptions {
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
