use std::fmt;
use regex::Regex;
use colored::{ColoredString, Colorize};
use crate::traits::{Representation, Colored, Discardable, Power, Points};
use crate::color::Color;
use crate::color_value::*;
use crate::trump_value::*;

#[derive(Copy, Ord, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Card {
    Trump(TrumpValue),
    Color(Color, ColorValue)
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Trump(v) => write!(f, "{}", v),
            Self::Color(c, v) => write!(f, "{} : {}", c, v)
        }
    }
}

impl Points for Card {
    fn points(&self) -> f64 {
        match self {
            Self::Trump(v) => v.points(),
            Self::Color(_, v) => v.points()
        }
    }
}

impl Power for Card {
    fn power(&self) -> usize {
        match self {
            Self::Trump(v) => *v as usize + ColorValue::King as usize,
            Self::Color(_, v) => *v as usize,
        }
    }
}

impl Discardable for Card {
    fn discardable(&self) -> bool {
        match self {
            Self::Trump(t) => t.discardable(),
            Self::Color(_, v) => v.discardable()
        }
    }
    fn discardable_forced(&self) -> bool {
        match self {
            Self::Trump(t) => t.discardable_forced(),
            Self::Color(_, v) => v.discardable_forced()
        }
    }
}

impl Card {
    pub fn is_fool(self) -> bool {
        match self {
            Self::Trump(v) => v == TrumpValue::Fool,
            _ => false
        }
    }
    pub fn is_trump(self) -> bool {
        matches!(self, Self::Trump(_))
    }
    pub fn is_oudler(self) -> bool {
        match self {
            Self::Trump(c) => c.is_oudler(),
            _ => false
        }
    }
    pub fn master(self, arg: Card) -> bool {
        match (&self, &arg) {
            (Self::Trump(c), Self::Color(_, _)) => c != &TrumpValue::Fool,
            (Self::Color(_, _), Self::Trump(c)) => c == &TrumpValue::Fool,
            (Self::Color(c1, v1), Self::Color(c2, v2)) => c1 != c2 || v1 > v2,
            (Self::Trump(v1), Self::Trump(v2)) => v1 > v2,
        }
    }
}

impl Colored for Card {
    fn color(&self) -> &'static str {
        match self {
            Self::Color(c, _) => c.color(),
            Self::Trump(t) => t.color()
        }
    }
}

impl Representation for Card {
    fn repr(&self) -> ColoredString {
        match self {
            Self::Color(c, cv) => {
                let re = Regex::new(r"[\*]").unwrap();
                re.replace_all(&cv.repr(), format!("{}", c)).color(c.color())
            },
            Self::Trump(t) => t.repr()
        }
    }
}

#[test]
fn card_tests() {
    use std::f64::EPSILON;
    let trump_2 = Card::Trump(TrumpValue::_2);
    println!("{}", trump_2.repr());
    let petit = Card::Trump(TrumpValue::Petit);
    println!("{}", petit.repr());
    let fool = Card::Trump(TrumpValue::Fool);
    println!("{}", fool.repr());
    let unassailable = Card::Trump(TrumpValue::_21);
    let spade_1 = Card::Color(Color::Spade, ColorValue::_1);
    let spade_2 = Card::Color(Color::Spade, ColorValue::_2);
    let spade_3 = Card::Color(Color::Spade, ColorValue::_3);
    let spade_10 = Card::Color(Color::Spade, ColorValue::_10);
    println!("{}", spade_10.repr());
    let diamond_3 = Card::Color(Color::Diamond, ColorValue::_3);
    println!("{}", diamond_3.repr());
    let heart_4 = Card::Color(Color::Heart, ColorValue::_4);
    println!("{}", heart_4.repr());
    let club_king = Card::Color(Color::Club, ColorValue::King);
    println!("{}", club_king.repr());

    assert!(!spade_3.master(spade_10));
    assert!(!petit.master(trump_2));
    assert!(petit.master(spade_1));
    assert!(!spade_1.master(petit));
    assert!(spade_2.master(spade_1));
    assert!(diamond_3.master(spade_2));
    assert!(diamond_3.master(fool));
    assert!(!fool.master(spade_1));

    assert!(!petit.discardable());
    assert!(!fool.discardable());
    assert!(!unassailable.discardable());
    assert!(!petit.discardable_forced());
    assert!(!fool.discardable_forced());
    assert!(!unassailable.discardable_forced());
    assert!(unassailable.points() - 4.5 < EPSILON);
}

