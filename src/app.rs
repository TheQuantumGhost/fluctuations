use rand::rngs::ThreadRng;

use crate::intervalle_confiance::Fluctuations;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ProbaApp {
    #[serde(skip)]
    rng: ThreadRng,
    selected_demo: Demo,
    fluctuations: Fluctuations,
}

#[derive(Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum Demo {
    #[default]
    Fluctuations,
    Intervalles,
}

impl Default for ProbaApp {
    fn default() -> Self {
        Self {
            rng: rand::rng(),
            selected_demo: Demo::default(),
            fluctuations: Fluctuations::default(),
        }
    }
}

impl ProbaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

impl eframe::App for ProbaApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
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

            self.fluctuations.show_ui(&mut self.rng, ui);
        });

        //egui::SidePanel::left("selection panel")
        //    .resizable(true)
        //    .show(ctx, |ui| {
        //        ui.vertical(|ui| {
        //            ui.selectable_value(
        //                &mut self.selected_demo,
        //                Demo::Fluctuations,
        //                "Fréquences empiriques",
        //            );
        //            ui.selectable_value(
        //                &mut self.selected_demo,
        //                Demo::Intervalles,
        //                "Intervalles empiriques",
        //            );
        //        });
        //    });

        egui::CentralPanel::default().show(ctx, |ui| self.fluctuations.show_plot(ui));
    }
}
