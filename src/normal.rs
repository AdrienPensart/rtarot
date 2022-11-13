use crate::points::HasPoints;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Colored, Discardable, Power, Representation, Symbol};
use colored::{ColoredString, Colorize};
use ordered_float::OrderedFloat;
use regex::Regex;
use std::fmt;

#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Normal {
    pub suit: Suit,
    pub value: SuitValue,
}

impl Normal {
    pub fn new(suit: Suit, value: SuitValue) -> Self {
        Self { suit, value }
    }
}

impl fmt::Display for Normal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value, self.suit)
    }
}

impl HasPoints for Normal {
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

impl Colored for Normal {
    fn color(&self) -> &'static str {
        self.suit.color()
    }
}

impl Symbol for Normal {
    fn symbol(&self) -> ColoredString {
        self.suit.symbol()
    }
}

impl Representation for Normal {
    fn repr(&self) -> ColoredString {
        let re = Regex::new(r"[\*]").unwrap();
        re.replace_all(&self.value.repr(), format!("{}", self.suit))
            .color(self.color())
    }
}
