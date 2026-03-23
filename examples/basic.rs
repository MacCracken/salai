//! Basic salai usage — create editor and inspect entities.

use salai::EditorApp;

fn main() {
    let app = EditorApp::new();
    println!("Salai editor — {} entities", app.entity_count());
    println!("Play state: {:?}", app.state.play_state);
}
