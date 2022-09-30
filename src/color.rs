use std::fmt;
use std::str::FromStr;
use crate::errors::TarotErrorKind;

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
            Self::Spade   => write!(f, "♠"),
            Self::Diamond => write!(f, "♦"),
            Self::Club  => write!(f, "♣"),
            Self::Heart   => write!(f, "♥"),
        }
    }
}

impl FromStr for Color {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Color, TarotErrorKind> {
        match s {
            "1" => Ok(Self::Heart),
            "2" => Ok(Self::Spade),
            "3" => Ok(Self::Diamond),
            "4" => Ok(Self::Club),
            _ => Err(TarotErrorKind::InvalidColor),
        }
    }
}
