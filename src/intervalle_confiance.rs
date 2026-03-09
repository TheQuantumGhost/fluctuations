use egui::{Checkbox, Color32, DragValue, Label, Slider, Ui};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints, Points};
use rand::{distr::Bernoulli, rngs::ThreadRng};

const Q90: f64 = 1.645;
const Q95: f64 = 1.96;
const Q99: f64 = 2.578;

const COLOR_BLUE: Color32 = Color32::from_rgb(102, 102, 204);
const COLOR_GREEN: Color32 = Color32::from_rgb(102, 204, 102);
const COLOR_RED: Color32 = Color32::from_rgb(204, 102, 102);

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Fluctuations {
    p: f64,
    sample_size: usize,
    sample_number: usize,
    f: f64,
    display_style: DisplayMode,
    #[serde(skip)]
    show_exp: bool,
    #[serde(skip)]
    interval_q: Option<f64>,
    #[serde(skip)]
    points: Vec<f64>,
}

#[derive(Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum DisplayMode {
    #[default]
    Points,
    Intervalles,
}

impl Default for Fluctuations {
    fn default() -> Self {
        Self {
            p: 0.5,
            sample_size: 100,
            sample_number: 100,
            f: 0.5,
            display_style: DisplayMode::default(),
            show_exp: false,
            interval_q: None,
            points: Vec::default(),
        }
    }
}

fn generate_sample(rng: &mut ThreadRng, p: f64, sample_size: usize) -> f64 {
    use rand::distr::Distribution as _;
    Bernoulli::new(p)
        .expect("This should always be a valid distribution")
        .sample_iter(rng)
        .take(sample_size)
        .filter(|b| *b)
        .count() as f64
        / sample_size as f64
}

impl Fluctuations {
    fn gen_points(&mut self, rng: &mut ThreadRng) {
        self.points = (0..self.sample_number)
            .map(|_| generate_sample(rng, self.p, self.sample_size))
            .collect();
    }

    pub fn show_ui(&mut self, rng: &mut ThreadRng, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.display_style, DisplayMode::Points, "Points");
            ui.selectable_value(
                &mut self.display_style,
                DisplayMode::Intervalles,
                "Intervalles",
            );
        });

        let probability_response = ui.add(Slider::new(&mut self.p, 0.0..=1.0).text("Probabilité"));
        if probability_response.changed() {
            self.gen_points(rng);
        }

        if self.display_style != DisplayMode::Intervalles {
            ui.add(Slider::new(&mut self.f, 0.0..=1.0).text("Experience"));
            ui.add(Checkbox::new(&mut self.show_exp, "Experience"));
        }

        ui.horizontal(|ui| {
            ui.add(Label::new("Taille d'un échantillon"));
            if ui
                .add(
                    DragValue::new(&mut self.sample_size)
                        .range(0..=10000)
                        .speed(10),
                )
                .changed()
            {
                self.gen_points(rng);
            }
        });
        ui.horizontal(|ui| {
            ui.add(Label::new("Nombre d'échantillons"));
            if ui
                .add(
                    DragValue::new(&mut self.sample_number)
                        .range(0..=10000)
                        .speed(10),
                )
                .changed()
            {
                self.gen_points(rng);
            }
        });

        match self.display_style {
            DisplayMode::Points => ui.horizontal(|ui| {
                ui.selectable_value(&mut self.interval_q, None, "Rien");
                ui.selectable_value(&mut self.interval_q, Some(Q90), "90%");
                ui.selectable_value(&mut self.interval_q, Some(Q95), "95%");
                ui.selectable_value(&mut self.interval_q, Some(Q99), "99%");
            }),
            DisplayMode::Intervalles => ui.horizontal(|ui| {
                ui.selectable_value(&mut self.interval_q, Some(Q90), "90%");
                ui.selectable_value(&mut self.interval_q, Some(Q95), "95%");
                ui.selectable_value(&mut self.interval_q, Some(Q99), "99%");
            }),
        };
    }

    pub fn show_plot(&self, ui: &mut Ui) {
        match self.display_style {
            DisplayMode::Points => self.show_plot_points(ui),
            DisplayMode::Intervalles => self.show_plot_intervalles(ui),
        }
    }

    pub fn show_plot_points(&self, ui: &mut Ui) {
        let points: PlotPoints<'_> = self
            .points
            .iter()
            .enumerate()
            .map(|(i, f)| [i as f64 / self.points.len() as f64, *f])
            .collect();
        let points = Points::new("Points", points)
            .shape(egui_plot::MarkerShape::Circle)
            .color(COLOR_RED)
            .filled(true)
            .radius(5.);

        let main_point = Points::new("Experience", [0.5, self.f])
            .shape(egui_plot::MarkerShape::Circle)
            .color(COLOR_GREEN)
            .filled(true)
            .radius(6.);

        let p_line =
            Line::new("Probabilité", vec![[-1f64, self.p], [2f64, self.p]]).color(COLOR_BLUE);

        Plot::new("plot_points")
            .default_x_bounds(-0.05, 1.05)
            .default_y_bounds(-0.05, 1.05)
            .allow_scroll(false)
            .allow_axis_zoom_drag(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .show(ui, |plot_ui| {
                plot_ui.line(p_line);
                if let Some(q) = self.interval_q {
                    let sigma = (self.p * (1. - self.p) / self.sample_size as f64).sqrt();
                    let delta = q * sigma;
                    plot_ui.add(
                        double_line("Intervalle", self.p - delta, self.p + delta)
                            .color(Color32::from_rgb(51, 153, 102)),
                    );
                }

                plot_ui.add(points);

                if self.show_exp {
                    plot_ui.add(main_point);
                }
            });
    }
    pub fn show_plot_intervalles(&self, ui: &mut Ui) {
        let bars = self
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let sigma = (p * (1. - p) / self.sample_size as f64).sqrt();
                let delta = self.interval_q.unwrap_or_default() * sigma;

                Bar::new(i as f64 / self.sample_number as f64, 2. * delta).base_offset(p - delta)
            })
            .collect();
        let bars = BarChart::new("Intervalles", bars)
            .width(0.1 / self.sample_number as f64)
            .color(COLOR_RED);

        let p_line =
            Line::new("Probabilité", vec![[-1f64, self.p], [2f64, self.p]]).color(COLOR_BLUE);

        Plot::new("plot_intervalles")
            .default_x_bounds(-0.05, 1.05)
            .default_y_bounds(-0.05, 1.05)
            .allow_scroll(false)
            .allow_axis_zoom_drag(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .show(ui, |plot_ui| {
                plot_ui.add(bars);

                plot_ui.add(p_line);
            });
    }
}

fn double_line(name: &str, low: f64, high: f64) -> Line<'_> {
    Line::new(name, vec![[-1., low], [2., low], [2., high], [-1., high]])
}
