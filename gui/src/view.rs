//! Drawing the table, and reading clicks on it.

use crate::cards::{self, HAND_CARD, SMALL_CARD, TRICK_CARD, paint_suit};
use crate::game::{Game, Line, Tone, Trick};
use crate::textures::Textures;
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui,
    pos2, vec2,
};
use ordered_float::OrderedFloat;
use rtarot_core::{Answer, Card, Contract, Deck, Points, Question, Representation, Role};
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
pub fn show(game: &mut Game, ui: &mut Ui, textures: &Textures) -> Outcome {
    let mut outcome = Outcome::Continue;
    let ctx = ui.ctx().clone();

    egui::Panel::top("bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            top_bar(game, ui);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Quit to menu").clicked() {
                    outcome = Outcome::Quit;
                }
                ui.toggle_value(&mut game.show_log, "Log");
                ui.toggle_value(&mut game.show_tricks, "Tricks won");
                ui.toggle_value(&mut game.show_last, "Last trick");
                ui.toggle_value(&mut game.right_to_left, "Sort right to left");
                ui.checkbox(&mut game.settings.auto, "Autoplay forced cards");
                ui.add(
                    egui::Slider::new(&mut game.settings.bot_delay, 0.0..=3.0)
                        .suffix(" s")
                        .fixed_decimals(1)
                        .text("Bot delay"),
                );
            });
        });
    });

    if game.show_log {
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
    }

    let felt = Frame::new().fill(FELT_DARK).inner_margin(12.0);
    let mut answer = None;
    egui::Panel::bottom("hand").frame(felt).show(ui, |ui| {
        answer = hand_panel(game, ui, textures);
    });

    let mut taken_back = None;
    egui::CentralPanel::default()
        .frame(Frame::new().fill(FELT))
        .show(ui, |ui| taken_back = table(game, ui, textures));
    if let Some(card) = taken_back {
        game.discard_pick.retain(|picked| *picked != card);
    }

    if let Some(answer) = answer {
        game.answer(answer);
    }

    if game.show_last {
        let mut open = true;
        egui::Window::new("Last trick")
            .open(&mut open)
            .resizable(false)
            .default_pos(ctx.content_rect().left_top() + vec2(20.0, 60.0))
            .show(&ctx, |ui| match &game.last_trick {
                Some(trick) => trick_entry(game, ui, "Last trick", trick, textures),
                None => {
                    ui.label("No trick finished yet");
                }
            });
        game.show_last = open;
    }

    if game.show_history {
        let mut open = true;
        egui::Window::new("The deal, trick by trick")
            .open(&mut open)
            .resizable(false)
            .default_pos(ctx.content_rect().left_top() + vec2(20.0, 60.0))
            .show(&ctx, |ui| history(game, ui, textures));
        game.show_history = open;
    }

    if game.show_tricks {
        let mut open = true;
        egui::Window::new("Tricks won")
            .open(&mut open)
            .resizable(false)
            .default_pos(ctx.content_rect().left_top() + vec2(20.0, 60.0))
            .show(&ctx, |ui| tricks_won(game, ui, textures));
        game.show_tricks = open;
    }

    if game.summary.is_none() && game.is_finished() {
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
    if let Some(deal) = game.deal() {
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

/// Returns the card clicked in the discard being built, to take it back in hand.
fn table(game: &Game, ui: &Ui, textures: &Textures) -> Option<Card> {
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

    // Once the taker starts making the dog, what it held is nobody's business.
    let discarding = game
        .engine
        .pending()
        .is_some_and(|question| matches!(question, Question::Discard { .. }));
    if game.trick.cards.is_empty() {
        if let (Some(last), Some(left)) = (&game.last_trick, game.last_trick_showing()) {
            ui.ctx().request_repaint_after(left);
            paint_trick(&painter, textures, last, trick_at, true);
            if let Some(winner) = last.winner {
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    format!("{} wins", game.name(winner)),
                    FontId::proportional(18.0),
                    ACCENT,
                );
            }
        } else if let Some(remaining) = game.picking() {
            return paint_pick(ui, game, textures, center, remaining);
        } else if let Some(dog) = &game.dog
            && !discarding
        {
            paint_dog(&painter, textures, center, dog);
        }
    } else {
        paint_trick(&painter, textures, &game.trick, trick_at, false);
    }
    None
}

/// The discard being built, one slot per card to give, each picked card
/// clickable to take it back. Returns the card clicked.
fn paint_pick(
    ui: &Ui,
    game: &Game,
    textures: &Textures,
    center: Pos2,
    remaining: usize,
) -> Option<Card> {
    let painter = ui.painter();
    let pick = &game.discard_pick;
    painter.text(
        center - vec2(0.0, SMALL_CARD.y.mul_add(0.5, 16.0)),
        Align2::CENTER_CENTER,
        format!(
            "Your discard, {} of {remaining}. Click a card to take it back.",
            pick.len()
        ),
        FontId::proportional(16.0),
        INK,
    );
    let step = SMALL_CARD.x + 6.0;
    let width = step.mul_add(remaining as f32, -6.0);
    let first = width.mul_add(-0.5, center.x) + SMALL_CARD.x / 2.0;
    let mut taken = None;
    for index in 0..remaining {
        let x = (index as f32).mul_add(step, first);
        let rect = Rect::from_center_size(pos2(x, center.y), SMALL_CARD);
        match pick.get(index) {
            Some(card) => {
                let response = ui.interact(rect, Id::new(("pick", index)), Sense::click());
                cards::paint_card(painter, rect, *card, textures);
                if response.hovered() {
                    cards::paint_highlight(painter, rect);
                }
                if response.clicked() {
                    taken = Some(*card);
                }
            }
            None => {
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(6),
                    Stroke::new(1.5, Color32::from_white_alpha(90)),
                    StrokeKind::Inside,
                );
            }
        }
    }
    taken
}

fn paint_trick(
    painter: &egui::Painter,
    textures: &Textures,
    trick: &Trick,
    trick_at: impl Fn(usize) -> Pos2,
    faded: bool,
) {
    for (seat, card) in &trick.cards {
        let rect = Rect::from_center_size(trick_at(*seat), TRICK_CARD);
        cards::paint_card(painter, rect, *card, textures);
        if faded {
            painter.rect_filled(rect, CornerRadius::same(6), Color32::from_black_alpha(60));
        }
    }
}

fn paint_dog(painter: &egui::Painter, textures: &Textures, center: Pos2, dog: &Deck) {
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
            textures,
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
    let Some(deal) = game.deal() else {
        return;
    };
    let Ok(player) = deal.player(seat) else {
        return;
    };
    let mut status: Vec<Piece> = Vec::new();
    // With five players the engine knows the sides at once, the table only
    // once the called king has been played: before that, say nothing.
    let sides_known = game.sides_known();
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

    // Card points taken so far in this deal: the side's once the sides are known.
    let points = match player.team() {
        Some(team) if sides_known => format!("{team} {} pts", game.team_points(team)),
        Some(_) | None => format!("{} pts", player.owned().points()),
    };
    painter.text(
        rect.right_bottom() - vec2(10.0, 6.0),
        Align2::RIGHT_BOTTOM,
        points,
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
fn hand_panel(game: &mut Game, ui: &mut Ui, textures: &Textures) -> Option<Answer> {
    if game.paused {
        deal_over(game, ui, textures);
        return None;
    }
    let human = game.human();
    let question = game.human_question().cloned();
    let mut answer = None;
    let mut confirm = false;
    let mut slam = game.slam_intent;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(game.name(human))
                .size(18.0)
                .color(ACCENT),
        );
        ui.separator();
        match &question {
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
                if !game.settings.no_slam {
                    ui.separator();
                    ui.checkbox(&mut slam, "Announce a slam");
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
                let picked = game.discard_pick.len();
                ui.label(format!(
                    "Put {remaining} card(s) in the dog, {picked} picked"
                ));
                if ui
                    .add_enabled(
                        picked == *remaining,
                        egui::Button::new("Confirm the discard"),
                    )
                    .clicked()
                {
                    confirm = true;
                }
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

    if let Some(Question::Call { allowed, .. }) = &question {
        ui.add_space(6.0);
        if let Some(card) = card_row(ui, allowed, Some(allowed), SMALL_CARD, "call", textures) {
            answer = Some(Answer::Card(card));
        }
    }

    let picking = matches!(question, Some(Question::Discard { .. }));
    let allowed: Option<&[Card]> = match &question {
        Some(Question::Discard {
            allowed, remaining, ..
        }) => (game.discard_pick.len() < *remaining).then_some(allowed.as_slice()),
        Some(Question::Play { allowed, .. }) => Some(allowed.as_slice()),
        Some(Question::TrimHandle { shown, .. }) => Some(shown.as_slice()),
        Some(
            Question::Contract { .. }
            | Question::Slam { .. }
            | Question::Handle { .. }
            | Question::Call { .. },
        )
        | None => None,
    };
    let hand = game
        .engine
        .deal()
        .and_then(|deal| deal.player(human).ok())
        .map(|player| player.hand().clone())
        .unwrap_or_default();
    let mut hand: Vec<Card> = hand
        .iter()
        .filter(|card| !game.discard_pick.contains(card))
        .copied()
        .collect();
    hand.sort();
    if game.right_to_left {
        hand.reverse();
    }
    ui.add_space(6.0);
    if let Some(card) = card_row(ui, &hand, allowed, HAND_CARD, "hand", textures) {
        if picking {
            game.discard_pick.push(card);
        } else {
            answer = Some(Answer::Card(card));
        }
    }
    game.slam_intent = slam;
    if confirm {
        game.commit_discard();
    }
    answer
}

/// The score sheet of the deal just played, shown where the hand was so that
/// the table, with its last trick, stays in view until the next deal is asked for.
fn deal_over(game: &mut Game, ui: &mut Ui, textures: &Textures) {
    let mut next = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.heading("Deal over");
            if let Some(gain) = game
                .summary
                .as_ref()
                .and_then(|summary| summary.gains.get(game.human()))
            {
                let (verdict, color) = if *gain >= OrderedFloat(0.0) {
                    ("win", Color32::from_rgb(120, 230, 120))
                } else {
                    ("lose", Color32::from_rgb(255, 110, 110))
                };
                ui.label(
                    egui::RichText::new(format!("You {verdict} {} points", gain.abs()))
                        .size(32.0)
                        .strong()
                        .color(color),
                );
            }
            if let Some(summary) = &game.summary {
                egui::Grid::new("summary")
                    .num_columns(2)
                    .spacing([24.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (label, value) in &summary.rows {
                            ui.label(label);
                            ui.label(value);
                            ui.end_row();
                        }
                    });
            }
            ui.add_space(8.0);
            let label = if game.is_finished() {
                "Final scores"
            } else {
                "Next deal"
            };
            if ui
                .add(egui::Button::new(egui::RichText::new(label).size(18.0)))
                .clicked()
            {
                next = true;
            }
            ui.add_space(4.0);
            ui.toggle_value(&mut game.show_history, "Review the tricks");
        });
        // A dog nobody saw is turned over once the deal is scored.
        if let Some(deal) = game.deal()
            && let Some(contract @ (Contract::GardeSans | Contract::GardeContre)) = deal.contract()
        {
            ui.separator();
            ui.vertical(|ui| {
                let fate = match contract {
                    Contract::GardeSans => "kept unseen by the taker",
                    Contract::GardeContre | Contract::Petite | Contract::Garde => {
                        "given unseen to the defense"
                    }
                };
                ui.label(format!("The dog, {fate}:"));
                let mut dog = deal.dog().clone();
                dog.sort();
                card_row(ui, &dog, None, SMALL_CARD, "final_dog", textures);
            });
        }
    });
    if next {
        game.next_deal();
    }
}

/// Every trick of the deal in order, your own cards framed in gold.
fn history(game: &Game, ui: &mut Ui, textures: &Textures) {
    if game.tricks.is_empty() {
        ui.label("No trick played yet");
        return;
    }
    egui::ScrollArea::vertical()
        .max_height(ui.ctx().content_rect().height() * 0.7)
        .show(ui, |ui| {
            for (number, trick) in game.tricks.iter().enumerate() {
                trick_entry(game, ui, &format!("Trick {}", number + 1), trick, textures);
            }
        });
}

/// One trick: who led and won, then its cards in order, your own framed in gold.
fn trick_entry(game: &Game, ui: &mut Ui, title: &str, trick: &Trick, textures: &Textures) {
    let human = game.human();
    ui.horizontal(|ui| {
        ui.label(title);
        if let Some((leader, _)) = trick.cards.first() {
            ui.label("led by");
            ui.colored_label(name_color(game, *leader), game.name(*leader));
        }
        if let Some(winner) = trick.winner {
            ui.label("won by");
            ui.colored_label(name_color(game, winner), game.name(winner));
        }
    });
    ui.horizontal(|ui| {
        for (seat, card) in &trick.cards {
            ui.vertical(|ui| {
                ui.colored_label(name_color(game, *seat), game.name(*seat));
                let (rect, _) = ui.allocate_exact_size(SMALL_CARD, Sense::hover());
                cards::paint_card(ui.painter(), rect, *card, textures);
                if *seat == human {
                    cards::paint_highlight(ui.painter(), rect);
                }
            });
        }
    });
    ui.add_space(6.0);
}

/// Every card won so far, player by player.
fn tricks_won(game: &Game, ui: &mut Ui, textures: &Textures) {
    ui.set_min_width(640.0);
    let Some(deal) = game.deal() else {
        ui.label("No deal in progress");
        return;
    };
    for (seat, player) in deal.players().iter().enumerate() {
        let mut won = player.owned().clone();
        won.sort();
        ui.horizontal(|ui| {
            ui.colored_label(name_color(game, seat), game.name(seat));
            ui.label(format!("{} cards, {} pts", won.len(), won.points()));
        });
        if won.is_empty() {
            ui.label("nothing yet");
        } else {
            card_row(ui, &won, None, SMALL_CARD, &format!("won{seat}"), textures);
        }
        ui.add_space(4.0);
    }
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
    textures: &Textures,
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
        cards::paint_card(painter, rect, *card, textures);
        if raised {
            cards::paint_highlight(painter, rect);
        }
        if allowed.is_some_and(|allowed| !allowed.contains(card)) {
            cards::paint_dim(painter, rect);
        }
    }
    clicked
}
