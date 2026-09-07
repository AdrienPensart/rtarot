use crate::points::Points;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Color, Discardable, Power, Representation};
use derive_new::new;
use ordered_float::OrderedFloat;
use std::fmt;

#[derive(new, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Normal {
    suit: Suit,
    value: SuitValue,
}

impl fmt::Display for Normal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value, self.suit)
    }
}

impl Normal {
    #[must_use]
    pub const fn new_const(suit: Suit, value: SuitValue) -> Self {
        Self { suit, value }
    }
    #[must_use]
    pub const fn suit(&self) -> &Suit {
        &self.suit
    }
    #[must_use]
    pub const fn value(&self) -> &SuitValue {
        &self.value
    }
}

impl Points for Normal {
    fn points(&self) -> OrderedFloat<f64> {
        self.value.points()
    }
}

impl Power for Normal {
    fn power(&self) -> usize {
        self.value as usize
    }
}

impl Discardable for Normal {
    fn discardable(&self) -> bool {
        self.value.discardable()
    }
    fn discardable_forced(&self) -> bool {
        self.value.discardable_forced()
    }
}

impl Representation for Normal {
    fn symbol(&self) -> &'static str {
        self.suit.symbol()
    }
    fn color(&self) -> Color {
        self.suit.color()
    }
    /// The art of the value, in which `*` stands for the suit symbol.
    fn art(&self) -> &'static str {
        self.value.art()
    }
}
