use std::fmt;
use crate::traits::*;

pub const TRUMP_COLOR : char = '🂠';

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum TrumpValue {
    Fool = 0,
    Petit = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
    _16 = 16,
    _17 = 17,
    _18 = 18,
    _19 = 19,
    _20 = 20,
    _21 = 21
}

impl fmt::Display for TrumpValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Fool   => write!(f, "🃏"),
            _ => write!(f, "{0} : {1: <2}", TRUMP_COLOR, *self as usize)
        }
    }
}

impl TrumpValue {
    pub fn is_oudler(self) -> bool {
        matches!(self, Self::Fool | Self::Petit | Self::_21)
    }
}

impl Discardable for TrumpValue {
    fn discardable(&self) -> bool {
        // RULE: cant discard trumps
        false
    }
    fn discardable_forced(&self) -> bool {
        // RULE: if we have 4 kings and x trumps, we must discard some trumps, except oudlers
        !self.is_oudler()
    }
}

impl Points for TrumpValue {
    fn points(&self) -> f64 {
        if self.is_oudler() {
            4.5
        } else {
            0.5
        }
    }
}

