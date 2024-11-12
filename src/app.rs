use crate::canvas_view::CanvasView;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct ConstraintApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    canvas_view: CanvasView,  
}


impl ConstraintApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        let mut default : Self = Default::default();
        default.canvas_view.setup_test_values_1();
        default.canvas_view.setup_test_values_2();
        default.canvas_view.setup_test_values_3();
        default.canvas_view.setup_test_values_4();

        default
    }
}

impl eframe::App for ConstraintApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.canvas_view.update(ui);

        });
    }
}
