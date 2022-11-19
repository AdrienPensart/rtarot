use crate::points::HasPoints;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Colored, Discardable, Power, Representation, Symbol};
use colored::{ColoredString, Colorize};
use derive_more::Display;
use derive_new::new;
use ordered_float::OrderedFloat;
use regex::Regex;

#[derive(new, Display, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[display(fmt = "{}{}", value, suit)]
pub struct Normal {
    suit: Suit,
    value: SuitValue,
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
