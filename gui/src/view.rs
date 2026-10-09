//! Drawing the table, and reading clicks on it.

use crate::cards::{self, HAND_CARD, SMALL_CARD, TRICK_CARD, paint_suit};
use crate::game::{Game, Line, Tone, Trick};
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui,
    pos2, vec2,
};
use rtarot_core::{Answer, Card, Deck, Mode, Points, Question, Representation, Role};
use std::f32::consts::{FRAC_PI_2, TAU};

const FELT: Color32 = Color32::from_rgb(28, 92, 58);
const FELT_DARK: Color32 = Color32::from_rgb(20, 70, 44);
const INK: Color32 = Color32::from_rgb(235, 235, 225);
const ACCENT: Color32 = Color32::from_rgb(255, 214, 90);
const BADGE: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 110);
/// The taker's name, wherever it is written.
const TAKER: Color32 = Color32::from_rgb(255, 150, 70);
/// The ally's name, once everyone knows it.
const ALLY: Color32 = Color32::from_rgb(120, 200, 255);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Continue,
    Quit,
}

/// The whole table for one frame.
pub fn show(game: &mut Game, ui: &mut Ui) -> Outcome {
    let mut outcome = Outcome::Continue;
    let ctx = ui.ctx().clone();

    egui::Panel::top("bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            top_bar(game, ui);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Quit to menu").clicked() {
                    outcome = Outcome::Quit;
                }
                ui.add(
                    egui::Slider::new(&mut game.settings.bot_delay, 0.0..=3.0)
                        .suffix(" s")
                        .fixed_decimals(1)
                        .text("Bot thinking time"),
                );
            });
        });
    });

    egui::Panel::right("log")
        .default_size(320.0)
        .show(ui, |ui| {
            ui.heading("Table talk");
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink(false)
                .show(ui, |ui| {
                    for line in &game.log {
                        ui.label(layout(ui, line));
                    }
                });
        });

    let felt = Frame::new().fill(FELT_DARK).inner_margin(12.0);
    let mut answer = None;
    egui::Panel::bottom("hand").frame(felt).show(ui, |ui| {
        answer = hand_panel(game, ui);
    });

    egui::CentralPanel::default()
        .frame(Frame::new().fill(FELT))
        .show(ui, |ui| table(game, ui));

    if let Some(answer) = answer {
        game.answer(answer);
    }

    if let Some(summary) = game.summary.clone() {
        egui::Window::new("Deal over")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(&ctx, |ui| {
                egui::Grid::new("summary")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (label, value) in &summary.rows {
                            ui.label(label);
                            ui.label(value);
                            ui.end_row();
                        }
                    });
                ui.add_space(8.0);
                let label = if game.is_finished() {
                    "Final scores"
                } else {
                    "Next deal"
                };
                if ui.button(label).clicked() {
                    game.summary = None;
                    game.paused = false;
                }
            });
    } else if game.is_finished() {
        egui::Window::new("Game over")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(&ctx, |ui| {
                egui::Grid::new("final")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for player in game.engine.players() {
                            ui.label(player.name());
                            ui.label(player.score().to_string());
                            ui.end_row();
                        }
                    });
                ui.add_space(8.0);
                if ui.button("Back to menu").clicked() {
                    outcome = Outcome::Quit;
                }
            });
    }
    outcome
}

fn top_bar(game: &Game, ui: &mut Ui) {
    ui.label(format!("Deals left: {}", game.engine.deals_left()));
    ui.separator();
    ui.label(format!("Dealer: {}", game.name(game.engine.dealer())));
    if let Some(deal) = game.engine.deal() {
        ui.separator();
        match (deal.contract(), deal.taker()) {
            (Some(contract), Some(taker)) => {
                ui.colored_label(TAKER, game.name(taker));
                ui.label(format!("plays a {contract}"));
                if let Some(called) = deal
                    .player(taker)
                    .ok()
                    .and_then(rtarot_core::PlayerInGame::callee)
                {
                    ui.label(format!("calling {called}"));
                }
                if let Some(ally) = game.ally {
                    ui.label("with");
                    ui.colored_label(ALLY, game.name(ally));
                }
            }
            (Some(_) | None, None) | (None, Some(_)) => {
                ui.label("Bidding");
            }
        }
    }
    if let Some(error) = &game.error {
        ui.separator();
        ui.colored_label(
            Color32::from_rgb(255, 90, 90),
            format!("Engine error: {error}"),
        );
    }
}

/// Where a seat sits on the screen: the human at the bottom, then the seats
/// clockwise in playing order.
fn seat_direction(game: &Game, seat: usize) -> egui::Vec2 {
    let count = game.engine.mode().players();
    let offset = (seat + count - game.human()) % count;
    let angle = (offset as f32).mul_add(TAU / count as f32, FRAC_PI_2);
    vec2(angle.cos(), angle.sin())
}

/// The color a name is written in: the attack stands out, once it is known.
fn name_color(game: &Game, seat: usize) -> Color32 {
    if game.taker() == Some(seat) {
        TAKER
    } else if game.ally == Some(seat) {
        ALLY
    } else {
        INK
    }
}

/// A log line laid out with its names in color.
fn layout(ui: &Ui, line: &Line) -> egui::text::LayoutJob {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let plain = ui.visuals().text_color();
    let mut job = egui::text::LayoutJob::default();
    for span in line {
        let color = match span.tone {
            Tone::Plain => plain,
            Tone::Taker => TAKER,
            Tone::Ally => ALLY,
        };
        job.append(
            &span.text,
            0.0,
            egui::TextFormat::simple(font.clone(), color),
        );
    }
    job
}

fn table(game: &Game, ui: &Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    let center = rect.center();
    let radius = vec2(rect.width() * 0.40, rect.height() * 0.38);

    let count = game.engine.mode().players();
    for seat in 0..count {
        let direction = seat_direction(game, seat);
        let position = center + vec2(direction.x * radius.x, direction.y * radius.y);
        seat_badge(game, &painter, position, seat);
    }

    let trick_at = |seat: usize| -> Pos2 {
        let direction = seat_direction(game, seat);
        center + vec2(direction.x * radius.x * 0.42, direction.y * radius.y * 0.42)
    };

    if game.trick.cards.is_empty() {
        if let Some(last) = &game.last_trick {
            paint_trick(&painter, last, trick_at, true);
            if let Some(winner) = last.winner {
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    format!("{} wins", game.name(winner)),
                    FontId::proportional(18.0),
                    ACCENT,
                );
            }
        } else if let Some(dog) = &game.dog {
            paint_dog(&painter, center, dog);
        }
    } else {
        paint_trick(&painter, &game.trick, trick_at, false);
    }
}

fn paint_trick(
    painter: &egui::Painter,
    trick: &Trick,
    trick_at: impl Fn(usize) -> Pos2,
    faded: bool,
) {
    for (seat, card) in &trick.cards {
        let rect = Rect::from_center_size(trick_at(*seat), TRICK_CARD);
        cards::paint_card(painter, rect, *card);
        if faded {
            painter.rect_filled(rect, CornerRadius::same(6), Color32::from_black_alpha(60));
        }
    }
}

fn paint_dog(painter: &egui::Painter, center: Pos2, dog: &Deck) {
    let step = SMALL_CARD.x + 6.0;
    let width = step.mul_add(dog.len() as f32, -6.0);
    painter.text(
        center - vec2(0.0, SMALL_CARD.y.mul_add(0.5, 16.0)),
        Align2::CENTER_CENTER,
        "The dog",
        FontId::proportional(16.0),
        INK,
    );
    for (index, card) in dog.iter().enumerate() {
        let first = width.mul_add(-0.5, center.x) + SMALL_CARD.x / 2.0;
        let x = (index as f32).mul_add(step, first);
        cards::paint_card(
            painter,
            Rect::from_center_size(pos2(x, center.y), SMALL_CARD),
            *card,
        );
    }
}

fn seat_badge(game: &Game, painter: &egui::Painter, position: Pos2, seat: usize) {
    let size = vec2(210.0, 66.0);
    let rect = Rect::from_center_size(position, size);
    let is_turn = game.engine.pending().is_some_and(|q| q.player() == seat);
    let edge = if is_turn {
        ACCENT
    } else {
        Color32::TRANSPARENT
    };
    painter.rect(
        rect,
        CornerRadius::same(8),
        BADGE,
        Stroke::new(2.0, edge),
        StrokeKind::Inside,
    );

    let name = game.name(seat);
    let score = game
        .engine
        .player(seat)
        .map_or_else(|_| "0".to_string(), |p| p.score().to_string());
    let dealer = if game.engine.dealer() == seat {
        " (dealer)"
    } else {
        ""
    };
    let name_rect = painter.text(
        rect.min + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        format!("{name}{dealer}"),
        FontId::proportional(17.0),
        name_color(game, seat),
    );
    if let Some(bid) = game.bid_label(seat) {
        painter.text(
            pos2(name_rect.max.x + 8.0, name_rect.center().y),
            Align2::LEFT_CENTER,
            bid,
            FontId::proportional(15.0),
            ACCENT,
        );
    }
    painter.text(
        rect.right_top() + vec2(-10.0, 6.0),
        Align2::RIGHT_TOP,
        score,
        FontId::proportional(17.0),
        INK,
    );
    let Some(deal) = game.engine.deal() else {
        return;
    };
    let Ok(player) = deal.player(seat) else {
        return;
    };
    let mut status: Vec<Piece> = Vec::new();
    // With five players the engine knows the sides at once, the table only
    // once the called king has been played: before that, say nothing.
    let sides_known = game.engine.mode() != Mode::Five || game.ally.is_some();
    match player.role() {
        Some(Role::Taker) => {
            status.push(Piece::Text("Taker".to_string()));
            // With five players, the call is public: the king's value, then
            // its suit painted in the suit's color.
            if let Some(Card::Normal(called)) = game.callee() {
                status.push(Piece::Text(format!("calls {}", called.value().symbol())));
                status.push(Piece::Suit(*called.suit()));
            }
        }
        Some(Role::Ally) if sides_known => status.push(Piece::Text("Ally".to_string())),
        Some(Role::Defenser) if sides_known => status.push(Piece::Text("Defender".to_string())),
        Some(Role::Ally | Role::Defenser) | None => {}
    }
    if let Some(handle) = player.handle() {
        status.push(Piece::Text(format!("{handle} handle")));
    }
    if player.announced_slam() {
        status.push(Piece::Text("Slam!".to_string()));
    }
    status.push(Piece::Text(format!("{} cards", player.hand().len())));
    paint_pieces(painter, rect.min + vec2(10.0, 36.0), &status);

    // Card points taken so far in this deal.
    painter.text(
        rect.right_bottom() - vec2(10.0, 6.0),
        Align2::RIGHT_BOTTOM,
        format!("{} pts", player.owned().points()),
        FontId::proportional(13.0),
        INK,
    );
}

/// A bit of a status line: words, or a suit symbol painted in its color.
enum Piece {
    Text(String),
    Suit(rtarot_core::Suit),
}

/// Paint `pieces` on one line from `start`, separated by dots, a suit symbol
/// following its words without a separator.
fn paint_pieces(painter: &egui::Painter, start: Pos2, pieces: &[Piece]) {
    let font = FontId::proportional(13.0);
    let mut x = start.x;
    let mut first = true;
    for piece in pieces {
        match piece {
            Piece::Text(text) => {
                let text = if first {
                    text.clone()
                } else {
                    format!(" · {text}")
                };
                x = painter
                    .text(
                        pos2(x, start.y),
                        Align2::LEFT_CENTER,
                        text,
                        font.clone(),
                        INK,
                    )
                    .max
                    .x;
            }
            Piece::Suit(suit) => {
                x += 8.0;
                paint_suit(
                    painter,
                    pos2(x, start.y),
                    12.0,
                    *suit,
                    cards::rgb(suit.color()),
                );
                x += 8.0;
            }
        }
        first = false;
    }
}

/// The prompt and the human's hand. Returns the answer clicked, if any.
fn hand_panel(game: &Game, ui: &mut Ui) -> Option<Answer> {
    let human = game.human();
    let question = if game.paused {
        None
    } else {
        game.human_question()
    };
    let mut answer = None;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(game.name(human))
                .size(18.0)
                .color(ACCENT),
        );
        ui.separator();
        match question {
            None if game.paused => {
                ui.label("Deal over");
            }
            None => {
                let waiting = game.engine.pending().map_or_else(
                    || "Game over".to_string(),
                    |q| format!("Waiting for {}…", game.name(q.player())),
                );
                ui.label(waiting);
            }
            Some(Question::Contract { allowed, .. }) => {
                ui.label("Your bid:");
                if ui.button("Pass").clicked() {
                    answer = Some(Answer::Contract(None));
                }
                for contract in allowed {
                    if ui
                        .button(format!("{contract} (x{})", contract.multiplier()))
                        .clicked()
                    {
                        answer = Some(Answer::Contract(Some(*contract)));
                    }
                }
            }
            Some(Question::Slam { .. }) => {
                ui.label("Announce a slam?");
                if ui.button("No").clicked() {
                    answer = Some(Answer::Slam(false));
                }
                if ui.button("Yes, slam!").clicked() {
                    answer = Some(Answer::Slam(true));
                }
            }
            Some(Question::Handle { allowed, .. }) => {
                ui.label("Announce a handle?");
                for handle in allowed {
                    if ui
                        .button(format!("{handle} ({} pts)", handle.points()))
                        .clicked()
                    {
                        answer = Some(Answer::Handle(*handle));
                    }
                }
            }
            Some(Question::Call { .. }) => {
                ui.label("Call a king to find your partner:");
            }
            Some(Question::Discard { remaining, .. }) => {
                ui.label(format!("Discard {remaining} more card(s) into the dog"));
            }
            Some(Question::TrimHandle { to_remove, .. }) => {
                ui.label(format!("Drop {to_remove} trump(s) from your handle"));
            }
            Some(Question::Play { .. }) => {
                let called = game
                    .engine
                    .deal()
                    .and_then(|deal| deal.turn().called().copied());
                match called {
                    Some(called) => ui.label(format!("Play a card, following {called}")),
                    None => ui.label("Lead a card"),
                };
            }
        }
    });

    if let Some(Question::Call { allowed, .. }) = question {
        ui.add_space(6.0);
        if let Some(card) = card_row(ui, allowed, Some(allowed), SMALL_CARD, "call") {
            answer = Some(Answer::Card(card));
        }
    }

    let allowed: Option<&[Card]> = match question {
        Some(Question::Discard { allowed, .. } | Question::Play { allowed, .. }) => {
            Some(allowed.as_slice())
        }
        Some(Question::TrimHandle { shown, .. }) => Some(shown.as_slice()),
        Some(
            Question::Contract { .. }
            | Question::Slam { .. }
            | Question::Handle { .. }
            | Question::Call { .. },
        )
        | None => None,
    };
    let mut hand = game
        .engine
        .deal()
        .and_then(|deal| deal.player(human).ok())
        .map(|player| player.hand().clone())
        .unwrap_or_default();
    hand.sort();
    ui.add_space(6.0);
    if let Some(card) = card_row(ui, &hand, allowed, HAND_CARD, "hand") {
        answer = Some(Answer::Card(card));
    }
    answer
}

/// A row of cards, overlapping when the row is too narrow. Returns the card clicked.
///
/// `allowed` is `None` when nothing can be clicked, else the cards that can.
fn card_row(
    ui: &mut Ui,
    cards: &[Card],
    allowed: Option<&[Card]>,
    size: egui::Vec2,
    salt: &str,
) -> Option<Card> {
    let lift = 12.0;
    let width = ui.available_width();
    let (area, _) = ui.allocate_exact_size(vec2(width, size.y + lift + 4.0), Sense::hover());
    if cards.is_empty() {
        return None;
    }
    let gap = 6.0;
    let natural = (size.x + gap).mul_add(cards.len() as f32, -gap);
    let step = if natural <= width {
        size.x + gap
    } else {
        (width - size.x) / (cards.len() - 1).max(1) as f32
    };
    let total = step.mul_add((cards.len() - 1) as f32, size.x);
    let start = total.mul_add(-0.5, area.center().x);
    let top = area.min.y + lift;

    let mut clicked = None;
    let mut hovered = None;
    // Hit testing first, so that a card lifted out of the row is drawn on top.
    for (index, card) in cards.iter().enumerate() {
        let rect = Rect::from_min_size(pos2((index as f32).mul_add(step, start), top), size);
        let legal = allowed.is_some_and(|allowed| allowed.contains(card));
        let response = ui.interact(rect, Id::new((salt, index)), Sense::click());
        if legal && response.hovered() {
            hovered = Some(index);
        }
        if legal && response.clicked() {
            clicked = Some(*card);
        }
    }
    let painter = ui.painter();
    for (index, card) in cards.iter().enumerate() {
        let raised = hovered == Some(index);
        let y = if raised { top - lift } else { top };
        let rect = Rect::from_min_size(pos2((index as f32).mul_add(step, start), y), size);
        cards::paint_card(painter, rect, *card);
        if raised {
            cards::paint_highlight(painter, rect);
        }
        if allowed.is_some_and(|allowed| !allowed.contains(card)) {
            cards::paint_dim(painter, rect);
        }
    }
    clicked
}
