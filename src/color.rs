use std::fmt;
use std::str::FromStr;
use crate::errors::TarotErrorKind;
use crate::traits::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Color {
    Heart,
    Spade,
    Diamond,
    Club,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Color::Spade   => write!(f, "♠"),
            Color::Diamond => write!(f, "♦"),
            Color::Club  => write!(f, "♣"),
            Color::Heart   => write!(f, "♥"),
        }
    }
}

impl FromStr for Color {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Color, TarotErrorKind> {
        match s {
            "1" => Ok(Color::Heart),
            "2" => Ok(Color::Spade),
            "3" => Ok(Color::Diamond),
            "4" => Ok(Color::Club),
            _ => Err(TarotErrorKind::InvalidColor),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum ColorValue {
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    Jack = 11,
    Knight = 12,
    Queen = 13,
    King  = 14,
}

impl fmt::Display for ColorValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColorValue::Jack => write!(f, "V"),
            ColorValue::Knight => write!(f, "C"),
            ColorValue::Queen => write!(f, "Q"),
            ColorValue::King => write!(f, "K"),
            _ => write!(f, "{}", *self as usize),
        }
    }
}

impl Discardable for ColorValue {
    fn discardable(&self) -> bool {
        // RULE: cant discard kings
        self != &ColorValue::King
    }
    fn discardable_forced(&self) -> bool {
        // RULE: cant discard kings
        self != &ColorValue::King
    }
}

impl Points for ColorValue {
    fn points(&self) -> f64 {
        match self {
            ColorValue::Jack => 1.5,
            ColorValue::Knight => 2.5,
            ColorValue::Queen => 3.5,
            ColorValue::King => 4.5,
            _  => 0.5
        }
    }
}
