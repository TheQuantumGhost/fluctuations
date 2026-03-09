#![warn(clippy::all, rust_2018_idioms)]

mod intervalle_confiance;

mod app;
mod parsing;
pub use app::ProbaApp as App;
