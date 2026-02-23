use egui::{Color32, widgets};
use egui_plot::{Line, Plot, PlotPoints, Points};
use rand::{distr::Uniform, rngs::ThreadRng};

fn generate_sample(rng: &mut ThreadRng, p: f64, sample_size: usize) -> f64 {
    use rand::distr::Distribution as _;

    Uniform::new_inclusive(0f64, 1f64)
        .expect("This should always be a valid distribution")
        .sample_iter(rng)
        .take(sample_size)
        .filter(|item| (0f64..p).contains(item))
        .count() as f64
        / sample_size as f64
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Fluctuations {
    #[serde(skip)]
    rng: ThreadRng,
    p: f64,
    sample_size: usize,
    sample_number: usize,
    #[serde(skip)]
    f: f64,
    #[serde(skip)]
    show_exp: bool,
    #[serde(skip)]
    show_interval: bool,
    points: Vec<f64>,
}

impl Default for Fluctuations {
    fn default() -> Self {
        let mut rng = rand::rng();
        let p = 0.5;
        let sample_size = 100;
        let sample_number = 100;
        let f = 0.5;
        let points = (0..sample_number)
            .map(|_| generate_sample(&mut rng, p, sample_size))
            .collect();
        Self {
            rng,
            p,
            sample_size,
            sample_number,
            f,
            show_exp: false,
            show_interval: false,
            points,
        }
    }
}

impl Fluctuations {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

impl eframe::App for Fluctuations {
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

            ui.add(widgets::Slider::new(&mut self.p, 0.0..=1.0).text("Probabilité"));
            ui.add(widgets::Slider::new(&mut self.f, 0.0..=1.0).text("Experience"));
            ui.horizontal(|ui| {
                ui.add(widgets::Label::new("Taille d'un échantillon"));
                ui.add(
                    widgets::DragValue::new(&mut self.sample_size)
                        .range(0..=10000)
                        .speed(10),
                );
            });
            ui.horizontal(|ui| {
                ui.add(widgets::Label::new("Nombre d'échantillons"));
                ui.add(
                    widgets::DragValue::new(&mut self.sample_number)
                        .range(0..=10000)
                        .speed(10),
                );
            });
            ui.add(widgets::Checkbox::new(&mut self.show_exp, "Experience"));
            ui.add(widgets::Checkbox::new(
                &mut self.show_interval,
                "Intervalle",
            ));

            if ui.add(widgets::Button::new("Calculer")).clicked() {
                self.points = (0..self.sample_number)
                    .map(|_| generate_sample(&mut self.rng, self.p, self.sample_size))
                    .collect();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let points: PlotPoints<'_> = self
                .points
                .iter()
                .enumerate()
                .map(|(i, f)| [i as f64 / self.points.len() as f64, *f])
                .collect();
            let points = Points::new("Points", points)
                .shape(egui_plot::MarkerShape::Circle)
                .color(Color32::RED)
                .filled(true)
                .radius(5.);

            let main_point = Points::new("Experience", [0.5, self.f])
                .shape(egui_plot::MarkerShape::Circle)
                .color(Color32::GREEN)
                .filled(true)
                .radius(6.);

            let p_line = Line::new("Probabilité", vec![[-1f64, self.p], [2f64, self.p]])
                .color(Color32::BLUE);
            let delta = 1. / (self.sample_size as f64).sqrt();
            let h_line = Line::new(
                "Fluctuations",
                vec![
                    [-1f64, self.p + delta],
                    [2f64, self.p + delta],
                    [2f64, self.p - delta],
                    [-1f64, self.p - delta],
                ],
            );

            Plot::new("plot")
                .default_x_bounds(-0.05, 1.05)
                .default_y_bounds(-0.05, 1.05)
                .allow_scroll(false)
                .allow_axis_zoom_drag(false)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_boxed_zoom(false)
                .show(ui, |plot_ui| {
                    plot_ui.line(p_line);
                    if self.show_interval {
                        plot_ui.line(h_line);
                    }
                    plot_ui.add(points);

                    if self.show_exp {
                        plot_ui.add(main_point);
                    }
                });
        });
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

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
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}
