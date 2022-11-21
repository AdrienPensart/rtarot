use colored::{ColoredString, Colorize};
use std::fmt;
use std::str::FromStr;
use strum::EnumIter;

use crate::errors::TarotErrorKind;
use crate::traits::Representation;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Suit {
    Heart,
    Spade,
    Diamond,
    Club,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl Representation for Suit {
    fn symbol(&self) -> &'static str {
        match self {
            Self::Spade => "♠",
            Self::Diamond => "♦",
            Self::Club => "♣",
            Self::Heart => "♥",
        }
    }
    fn colored_symbol(&self) -> ColoredString {
        self.symbol().color(self.color())
    }
    fn repr(&self) -> ColoredString {
        self.symbol().color(self.color())
    }
    fn full_repr(&self) -> ColoredString {
        self.repr()
    }
    fn color(&self) -> &'static str {
        match self {
            Self::Spade => "blue",
            Self::Diamond => "yellow",
            Self::Club => "green",
            Self::Heart => "red",
        }
    }
}

impl FromStr for Suit {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Self, TarotErrorKind> {
        match s {
            "♥" => Ok(Self::Heart),
            "♠" => Ok(Self::Spade),
            "♦" => Ok(Self::Diamond),
            "♣" => Ok(Self::Club),
            _ => Err(TarotErrorKind::InvalidColor),
        }
    }
}
