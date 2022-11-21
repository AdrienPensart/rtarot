use crate::points::HasPoints;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Discardable, Power, Representation};
use colored::{ColoredString, Colorize};
use derive_new::new;
use ordered_float::OrderedFloat;
use regex::Regex;
use std::fmt;

#[derive(new, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Normal {
    suit: Suit,
    value: SuitValue,
}

impl fmt::Display for Normal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value(), self.colored_symbol())
    }
}

impl Normal {
    pub const fn suit(&self) -> &Suit {
        &self.suit
    }
    pub const fn value(&self) -> &SuitValue {
        &self.value
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

impl Representation for Normal {
    fn symbol(&self) -> &'static str {
        self.suit.symbol()
    }
    fn colored_symbol(&self) -> ColoredString {
        self.symbol().color(self.color())
    }
    fn color(&self) -> &'static str {
        self.suit.color()
    }
    fn repr(&self) -> ColoredString {
        self.colored_symbol()
    }
    fn full_repr(&self) -> ColoredString {
        let re = Regex::new(r"[\*]").unwrap();
        re.replace_all(&self.value.full_repr(), format!("{}", self.suit))
            .color(self.color())
    }
}
