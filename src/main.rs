use crate::editor::StarEditor;

mod editor;
mod save;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Star Editor",
        options,
        Box::new(|_cc| Box::new(StarEditor::default())),
    )
}