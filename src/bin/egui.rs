//! Interface graphique de secours en fenêtre native (egui).

fn main() -> eframe::Result<()> {
    let app = mon_budget::gui::FinanceApp::new();
    eframe::run_native(
        "Mon Budget",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1150.0, 760.0])
                .with_min_inner_size([880.0, 600.0])
                .with_title("Mon Budget"),
            ..Default::default()
        },
        Box::new(|_creation_context| match app {
            Ok(app) => Ok(Box::new(app)),
            Err(error) => Err(Box::<dyn std::error::Error + Send + Sync>::from(error)),
        }),
    )
}
