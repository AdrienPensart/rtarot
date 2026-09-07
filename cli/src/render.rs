//! Turning engine values into colored terminal output.
//!
//! Nothing here is known to the engine: swapping this module for an HTML one
//! is all it takes to move the game to a browser.

use colored::{Color as TermColor, ColoredString, Colorize};
use rtarot_core::{Card, Color, Deck, Representation, Suit};
use std::collections::BTreeMap;
use std::fmt::Write;

#[must_use]
pub const fn term_color(color: Color) -> TermColor {
    match color {
        Color::Blue => TermColor::Blue,
        Color::Cyan => TermColor::Cyan,
        Color::Green => TermColor::Green,
        Color::Red => TermColor::Red,
        Color::Yellow => TermColor::Yellow,
        Color::Plain => TermColor::White,
    }
}

#[must_use]
pub fn card(card: Card) -> ColoredString {
    card.to_string().color(term_color(card.color()))
}

/// A hand, trumps first then one line per color.
#[must_use]
pub fn hand(deck: &Deck) -> String {
    let mut out = String::new();
    let (trumps, colors): (Vec<&Card>, Vec<&Card>) = deck.iter().partition(|c| c.is_trump());
    if !trumps.is_empty() {
        let _ = write!(
            out,
            "\n\t{}",
            trumps
                .iter()
                .map(|c| card(**c).to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    let mut last: Option<Suit> = None;
    for colored in colors {
        let Card::Normal(normal) = colored else {
            continue;
        };
        if last == Some(*normal.suit()) {
            let _ = write!(out, "{} ", card(*colored));
        } else {
            last = Some(*normal.suit());
            let _ = write!(out, "\n\t{} ", card(*colored));
        }
    }
    out
}

/// Ascii art of every card of a deck, side by side.
#[must_use]
pub fn art(deck: &Deck) -> String {
    let mut buffers: BTreeMap<usize, String> = BTreeMap::new();
    for one in deck.iter().copied() {
        let drawing = match one {
            Card::Normal(normal) => one.art().replace('*', normal.suit().symbol()),
            Card::Trump(_) => one.art().to_string(),
        };
        for (index, line) in drawing.lines().enumerate() {
            let painted = format!("{}\x1B[0m", line.color(term_color(one.color())));
            buffers
                .entry(index)
                .and_modify(|buffer| buffer.push_str(&painted))
                .or_insert(painted);
        }
    }
    buffers.values().cloned().collect::<Vec<_>>().join("\n")
}
