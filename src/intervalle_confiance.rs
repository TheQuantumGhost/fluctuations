use egui::{Checkbox, Color32, DragValue, Label, Slider, Ui};
use egui_plot::{Line, Plot, PlotPoints, Points};
use rand::{distr::Bernoulli, rngs::ThreadRng};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Fluctuations {
    p: f64,
    sample_size: usize,
    sample_number: usize,
    f: f64,
    #[serde(skip)]
    show_exp: bool,
    #[serde(skip)]
    show_interval_90: bool,
    #[serde(skip)]
    show_interval_95: bool,
    #[serde(skip)]
    show_interval_99: bool,
    #[serde(skip)]
    points: Vec<f64>,
}

impl Default for Fluctuations {
    fn default() -> Self {
        Self {
            p: 0.5,
            sample_size: 100,
            sample_number: 100,
            f: 0.5,
            show_exp: false,
            show_interval_90: false,
            show_interval_95: false,
            show_interval_99: false,
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
        if ui
            .add(Slider::new(&mut self.p, 0.0..=1.0).text("Probabilité"))
            .changed()
        {
            self.gen_points(rng);
        }
        ui.add(Slider::new(&mut self.f, 0.0..=1.0).text("Experience"));
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
        ui.add(Checkbox::new(&mut self.show_exp, "Experience"));
        ui.add(Checkbox::new(
            &mut self.show_interval_90,
            "Intervalle à 90%",
        ));
        ui.add(Checkbox::new(
            &mut self.show_interval_95,
            "Intervalle à 95%",
        ));
        ui.add(Checkbox::new(
            &mut self.show_interval_99,
            "Intervalle à 99%",
        ));
    }

    pub fn show_plot(&self, ui: &mut Ui) {
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

        let p_line =
            Line::new("Probabilité", vec![[-1f64, self.p], [2f64, self.p]]).color(Color32::BLUE);

        let sigma = (self.p * (1. - self.p) / self.sample_size as f64).sqrt();
        let line_90 =
            interval_line("90%", self.p, 1.645 * sigma).color(Color32::from_rgb(51, 153, 102));
        let line_95 =
            interval_line("95%", self.p, 1.96 * sigma).color(Color32::from_rgb(51, 153, 51));
        let line_99 =
            interval_line("99%", self.p, 2.5758 * sigma).color(Color32::from_rgb(51, 153, 0));

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
                if self.show_interval_90 {
                    plot_ui.line(line_90);
                }
                if self.show_interval_95 {
                    plot_ui.line(line_95);
                }
                if self.show_interval_99 {
                    plot_ui.line(line_99);
                }

                plot_ui.add(points);

                if self.show_exp {
                    plot_ui.add(main_point);
                }
            });
    }
}

fn interval_line(name: &str, p: f64, delta: f64) -> Line<'_> {
    Line::new(
        name,
        vec![
            [-1f64, p + delta],
            [2f64, p + delta],
            [2f64, p - delta],
            [-1f64, p - delta],
        ],
    )
}
