use std::fmt;
use std::str::FromStr;
use strum::EnumIter;

use crate::errors::TarotErrorKind;
use crate::traits::{Color, Representation};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Suit {
    Heart,
    Spade,
    Diamond,
    Club,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl Representation for Suit {
    fn symbol(&self) -> &'static str {
        match self {
            Self::Spade => "\u{2660}",
            Self::Diamond => "\u{2666}",
            Self::Club => "\u{2663}",
            Self::Heart => "\u{2665}",
        }
    }
    fn art(&self) -> &'static str {
        self.symbol()
    }
    fn color(&self) -> Color {
        match self {
            Self::Spade => Color::Blue,
            Self::Diamond => Color::Yellow,
            Self::Club => Color::Green,
            Self::Heart => Color::Red,
        }
    }
}

impl FromStr for Suit {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Self, TarotErrorKind> {
        match s {
            "\u{2665}" => Ok(Self::Heart),
            "\u{2660}" => Ok(Self::Spade),
            "\u{2666}" => Ok(Self::Diamond),
            "\u{2663}" => Ok(Self::Club),
            _ => Err(TarotErrorKind::InvalidColor),
        }
    }
}
