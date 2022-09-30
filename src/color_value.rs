use std::fmt;
use crate::traits::*;

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
            Self::Jack => write!(f, "V"),
            Self::Knight => write!(f, "C"),
            Self::Queen => write!(f, "Q"),
            Self::King => write!(f, "K"),
            _ => write!(f, "{}", *self as usize),
        }
    }
}

impl Discardable for ColorValue {
    fn discardable(&self) -> bool {
        // RULE: cant discard kings
        self != &Self::King
    }
    fn discardable_forced(&self) -> bool {
        // RULE: cant discard kings
        self != &Self::King
    }
}

impl Points for ColorValue {
    fn points(&self) -> f64 {
        match self {
            Self::Jack => 1.5,
            Self::Knight => 2.5,
            Self::Queen => 3.5,
            Self::King => 4.5,
            _  => 0.5
        }
    }
}
