#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::Fluctuations as App;
pub use app::TemplateApp; // as App;
