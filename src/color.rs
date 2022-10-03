use std::fmt;
use std::str::FromStr;
use colored::{ColoredString, Colorize};
use crate::errors::TarotErrorKind;
use crate::traits::{Symbol, Representation, Colored};

#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Color {
    Heart,
    Spade,
    Diamond,
    Club,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl Symbol for Color {
    fn symbol(&self) -> ColoredString {
        match self {
            Self::Spade   => "♠".color(self.color()),
            Self::Diamond => "♦".color(self.color()),
            Self::Club  => "♣".color(self.color()),
            Self::Heart   => "♥".color(self.color()),
        }
    }
}

impl Representation for Color {
    fn repr(&self) -> ColoredString {
        self.symbol()
    }
}

impl Colored for Color {
    fn color(&self) -> &'static str {
        match self {
            Self::Spade   => "blue",
            Self::Diamond => "yellow",
            Self::Club  => "green",
            Self::Heart   => "red",
        }
    }
}

impl FromStr for Color {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Color, TarotErrorKind> {
        match s {
            "♥" => Ok(Self::Heart),
            "♠" => Ok(Self::Spade),
            "♦" => Ok(Self::Diamond),
            "♣" => Ok(Self::Club),
            _ => Err(TarotErrorKind::InvalidColor),
        }
    }
}
