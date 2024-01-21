use crate::points::Points;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::traits::{Discardable, Power, Representation};
use colored::{ColoredString, Colorize};
use derive_new::new;
use lazy_regex::regex;
use ordered_float::OrderedFloat;
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
    fn colored_symbol(&self) -> ColoredString {
        self.symbol().color(self.color())
    }
    fn color(&self) -> &'static str {
        self.suit.color()
    }
    fn repr(&self) -> ColoredString {
        self.colored_symbol()
    }
    #[allow(clippy::trivial_regex)]
    fn full_repr(&self) -> ColoredString {
        let repr_regex = regex!(r"[\*]");
        repr_regex
            .replace_all(&self.value.full_repr(), format!("{}", self.suit))
            .color(self.color())
    }
}
