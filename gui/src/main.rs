//! Desktop front end for the rtarot engine, drawn with egui.
//!
//! All the rules live in `rtarot-core`; this crate only decides how to show a
//! table and how to turn clicks into answers. egui is immediate mode: every
//! frame, the whole screen is described again from the engine state, so there
//! is no widget tree to keep in sync with the game.

// Geometry turns card counts into pixels, where losing precision is harmless.
#![allow(clippy::cast_precision_loss)]

mod app;
mod cards;
mod game;
mod log;
mod textures;
mod view;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rtarot")
            .with_inner_size([1280.0, 840.0])
            .with_min_inner_size([980.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rtarot",
        options,
        Box::new(|cc| Ok(Box::new(app::TarotApp::new(cc)))),
    )
}
