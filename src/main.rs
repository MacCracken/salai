//! Salai — game editor binary.

use salai::EditorApp;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mut app = EditorApp::new();

    // Load scene from CLI args if provided
    if let Some(path) = std::env::args().nth(1)
        && let Err(e) = app.load_scene(&path)
    {
        tracing::error!(path, error = %e, "failed to load scene");
    }

    tracing::info!(entities = app.entity_count(), "salai editor started");

    // TODO: Wire into eframe event loop when UI is implemented
    // For now, just print state
    println!("Salai — game editor for Kiran");
    println!("  Entities: {}", app.entity_count());
    println!("  State: {:?}", app.state.play_state);
}
