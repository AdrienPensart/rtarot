//! Painting cards: the embedded images when they are loaded, shapes and text otherwise.

use crate::textures::Textures;
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2, pos2,
    vec2,
};
use rtarot_core::{Card, Color, Normal, Representation, Suit, SuitValue, Trump};

// Tarot cards are tall: the images are 250 by 481 pixels.
pub const HAND_CARD: Vec2 = vec2(82.0, 144.0);
pub const TRICK_CARD: Vec2 = vec2(70.0, 124.0);
pub const SMALL_CARD: Vec2 = vec2(53.0, 94.0);

const RADIUS: CornerRadius = CornerRadius::same(6);
const FACE: Color32 = Color32::from_rgb(250, 247, 238);
const EDGE: Color32 = Color32::from_rgb(70, 70, 70);
const GOLD: Color32 = Color32::from_rgb(214, 170, 40);

/// The egui color for a color name suggested by the engine.
pub const fn rgb(color: Color) -> Color32 {
    match color {
        Color::Blue => Color32::from_rgb(30, 70, 190),
        Color::Cyan => Color32::from_rgb(0, 125, 145),
        Color::Green => Color32::from_rgb(25, 130, 60),
        Color::Red => Color32::from_rgb(200, 30, 50),
        Color::Yellow => Color32::from_rgb(205, 145, 0),
        Color::Plain => Color32::from_rgb(30, 30, 30),
    }
}

/// A suit symbol of height `size`, drawn from circles and polygons.
pub fn paint_suit(painter: &Painter, center: Pos2, size: f32, suit: Suit, color: Color32) {
    let at = |x: f32, y: f32| center + vec2(x * size, y * size);
    let polygon = |points: Vec<Pos2>| {
        painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    };
    let stem = || polygon(vec![at(-0.14, 0.5), at(0.14, 0.5), at(0.0, 0.1)]);
    match suit {
        Suit::Diamond => polygon(vec![
            at(0.0, -0.5),
            at(0.38, 0.0),
            at(0.0, 0.5),
            at(-0.38, 0.0),
        ]),
        Suit::Heart => {
            painter.circle_filled(at(-0.22, -0.18), 0.27 * size, color);
            painter.circle_filled(at(0.22, -0.18), 0.27 * size, color);
            polygon(vec![at(-0.47, -0.08), at(0.47, -0.08), at(0.0, 0.5)]);
        }
        Suit::Spade => {
            polygon(vec![at(0.0, -0.5), at(0.47, 0.1), at(-0.47, 0.1)]);
            painter.circle_filled(at(-0.23, 0.1), 0.25 * size, color);
            painter.circle_filled(at(0.23, 0.1), 0.25 * size, color);
            stem();
        }
        Suit::Club => {
            painter.circle_filled(at(0.0, -0.24), 0.23 * size, color);
            painter.circle_filled(at(-0.24, 0.08), 0.23 * size, color);
            painter.circle_filled(at(0.24, 0.08), 0.23 * size, color);
            stem();
        }
    }
}

/// A card, face up: its image when there is one, else a drawing.
pub fn paint_card(painter: &Painter, rect: Rect, card: Card, textures: &Textures) {
    if let Some(texture) = textures.face(card) {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        // The images carry their own border and rounded corners.
        painter.image(texture.id(), rect, uv, Color32::WHITE);
        return;
    }
    painter.rect(
        rect,
        RADIUS,
        FACE,
        Stroke::new(1.5, EDGE),
        StrokeKind::Inside,
    );
    match card {
        Card::Normal(normal) => paint_normal(painter, rect, normal),
        Card::Trump(trump) => paint_trump(painter, rect, trump),
    }
}

fn paint_normal(painter: &Painter, rect: Rect, normal: Normal) {
    let suit = *normal.suit();
    let value = *normal.value();
    let color = rgb(suit.color());
    let corner = rect.height() * 0.19;
    let font = FontId::proportional(corner);
    let symbol = value.symbol();
    // Index in both corners, the lower one upside down would need a rotated galley;
    // mirroring the position is enough to read the card from either side.
    painter.text(
        rect.min + vec2(5.0, 3.0),
        Align2::LEFT_TOP,
        symbol,
        font.clone(),
        color,
    );
    let offset = vec2(corner.mul_add(0.42, 5.0), corner.mul_add(1.45, 3.0));
    paint_suit(painter, rect.min + offset, corner * 0.6, suit, color);
    painter.text(
        rect.max - vec2(5.0, 3.0),
        Align2::RIGHT_BOTTOM,
        symbol,
        font,
        color,
    );
    paint_suit(painter, rect.max - offset, corner * 0.6, suit, color);

    let center = rect.center();
    match value {
        SuitValue::Jack | SuitValue::Knight | SuitValue::Queen | SuitValue::King => {
            let big = FontId::proportional(rect.height() * 0.4);
            painter.text(
                center - vec2(0.0, rect.height() * 0.04),
                Align2::CENTER_CENTER,
                symbol,
                big,
                color,
            );
            paint_suit(
                painter,
                center + vec2(0.0, rect.height() * 0.24),
                rect.height() * 0.16,
                suit,
                color,
            );
        }
        SuitValue::_1
        | SuitValue::_2
        | SuitValue::_3
        | SuitValue::_4
        | SuitValue::_5
        | SuitValue::_6
        | SuitValue::_7
        | SuitValue::_8
        | SuitValue::_9
        | SuitValue::_10 => paint_suit(painter, center, rect.height() * 0.34, suit, color),
    }
}

fn paint_trump(painter: &Painter, rect: Rect, trump: Trump) {
    let color = rgb(trump.color());
    let inner = rect.shrink(4.0);
    let edge = if trump.is_oudler() { GOLD } else { color };
    painter.rect_stroke(
        inner,
        CornerRadius::same(4),
        Stroke::new(1.5, edge),
        StrokeKind::Inside,
    );
    let center = rect.center();
    if trump == Trump::Fool {
        let font = FontId::proportional(rect.height() * 0.15);
        painter.text(
            center - vec2(0.0, rect.height() * 0.1),
            Align2::CENTER_CENTER,
            "EXCUSE",
            font,
            color,
        );
        paint_star(
            painter,
            center + vec2(0.0, rect.height() * 0.14),
            rect.height() * 0.11,
            GOLD,
        );
        return;
    }
    let corner = FontId::proportional(rect.height() * 0.17);
    painter.text(
        rect.min + vec2(6.0, 4.0),
        Align2::LEFT_TOP,
        trump.symbol(),
        corner.clone(),
        color,
    );
    painter.text(
        rect.max - vec2(6.0, 4.0),
        Align2::RIGHT_BOTTOM,
        trump.symbol(),
        corner,
        color,
    );
    let big = FontId::proportional(rect.height() * 0.36);
    painter.text(
        center - vec2(0.0, rect.height() * 0.04),
        Align2::CENTER_CENTER,
        trump.symbol(),
        big,
        color,
    );
    let small = FontId::proportional(rect.height() * 0.1);
    painter.text(
        center + vec2(0.0, rect.height() * 0.22),
        Align2::CENTER_CENTER,
        "ATOUT",
        small,
        color,
    );
}

fn paint_star(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let point = |index: u8, r: f32| {
        let angle = -std::f32::consts::FRAC_PI_2 + f32::from(index) * std::f32::consts::PI / 5.0;
        pos2(
            r.mul_add(angle.cos(), center.x),
            r.mul_add(angle.sin(), center.y),
        )
    };
    for index in 0..5u8 {
        let outer = point(index * 2, radius);
        let next = point(index * 2 + 2, radius);
        painter.add(egui::Shape::convex_polygon(
            vec![center, outer, next],
            color,
            Stroke::NONE,
        ));
    }
    for index in 0..5u8 {
        let a = point(index * 2, radius);
        let b = point(index * 2 + 1, radius * 0.45);
        let c = point(index * 2 + 2, radius);
        painter.add(egui::Shape::convex_polygon(
            vec![a, b, c],
            color,
            Stroke::NONE,
        ));
    }
}

/// Grey out a card that cannot be chosen right now.
pub fn paint_dim(painter: &Painter, rect: Rect) {
    painter.rect_filled(rect, RADIUS, Color32::from_black_alpha(130));
}

/// A glowing border around a card that is being pointed at.
pub fn paint_highlight(painter: &Painter, rect: Rect) {
    painter.rect_stroke(
        rect.expand(2.0),
        CornerRadius::same(8),
        Stroke::new(3.0, GOLD),
        StrokeKind::Outside,
    );
}
