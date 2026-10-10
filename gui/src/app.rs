//! The application: a setup screen, then a table.

use crate::game::{Game, Settings};
use crate::textures::Textures;
use crate::view::{self, Outcome};
use rtarot_core::Mode;
use strum::IntoEnumIterator;

enum Screen {
    Setup,
    Table(Box<Game>),
}

pub struct TarotApp {
    settings: Settings,
    screen: Screen,
    textures: Textures,
}

impl TarotApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        cc.egui_ctx.set_zoom_factor(1.0);
        Self {
            settings: Settings::default(),
            screen: Screen::Setup,
            textures: Textures::load(&cc.egui_ctx),
        }
    }

    /// The setup screen. Returns true when the player wants to start.
    fn setup(settings: &mut Settings, ui: &mut egui::Ui) -> bool {
        let mut start = false;
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("French Tarot").size(36.0));
                ui.add_space(24.0);
                egui::Grid::new("setup")
                    .num_columns(2)
                    .spacing([24.0, 12.0])
                    .show(ui, |ui| {
                        ui.label("Players");
                        ui.vertical(|ui| {
                            for mode in Mode::iter() {
                                if ui
                                    .selectable_value(&mut settings.mode, mode, mode.to_string())
                                    .changed()
                                {
                                    settings.seat = Settings::south(mode);
                                }
                            }
                        });
                        ui.end_row();

                        ui.label("Deals");
                        ui.add(egui::Slider::new(&mut settings.deals, 1..=30));
                        ui.end_row();

                        settings.seat = settings.seat.min(settings.mode.players() - 1);
                        ui.label("Your seat");
                        egui::ComboBox::from_id_salt("seat")
                            .selected_text(seat_name(settings.mode, settings.seat))
                            .show_ui(ui, |ui| {
                                for seat in 0..settings.mode.players() {
                                    ui.selectable_value(
                                        &mut settings.seat,
                                        seat,
                                        seat_name(settings.mode, seat),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label("Bots");
                        ui.vertical(|ui| {
                            ui.checkbox(
                                &mut settings.attack,
                                "Bots always pass, you play the attack",
                            );
                            ui.checkbox(&mut settings.no_slam, "Nobody ever announces a slam");
                            ui.checkbox(
                                &mut settings.auto,
                                "Play for me when only one card is legal",
                            );
                        });
                        ui.end_row();

                        ui.label("Bot thinking time");
                        ui.add(
                            egui::Slider::new(&mut settings.bot_delay, 0.0..=3.0)
                                .suffix(" s")
                                .fixed_decimals(1),
                        );
                        ui.end_row();

                        ui.label("Finished trick stays");
                        ui.add(
                            egui::Slider::new(&mut settings.trick_delay, 0.0..=6.0)
                                .suffix(" s")
                                .fixed_decimals(1),
                        );
                        ui.end_row();
                    });
                ui.add_space(24.0);
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("Deal the cards").size(20.0),
                    ))
                    .clicked()
                {
                    start = true;
                }
            });
        });
        start
    }
}

fn seat_name(mode: Mode, seat: usize) -> String {
    mode.player_name(seat)
        .map_or_else(|_| format!("Seat {seat}"), str::to_string)
}

impl eframe::App for TarotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let next = match &mut self.screen {
            Screen::Setup => Self::setup(&mut self.settings, ui)
                .then(|| Screen::Table(Box::new(Game::new(&self.settings)))),
            Screen::Table(game) => {
                game.tick(ui.ctx());
                match view::show(game, ui, &self.textures) {
                    Outcome::Continue => None,
                    Outcome::Quit => Some(Screen::Setup),
                }
            }
        };
        if let Some(screen) = next {
            self.screen = screen;
        }
    }
}
