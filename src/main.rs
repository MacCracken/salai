//! Salai — game editor binary.

use salai::SalaiApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let scene_path = std::env::args().nth(1);
    let app = SalaiApp::new(scene_path.as_deref());

    tracing::info!(
        entities = app.editor.entity_count(),
        "salai editor starting"
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Salai — Game Editor")
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native("Salai", options, Box::new(|_cc| Ok(Box::new(app))))
}
