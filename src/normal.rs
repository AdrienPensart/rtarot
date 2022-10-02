use std::fmt;
use regex::Regex;
use colored::{ColoredString, Colorize};
use crate::color::Color;
use crate::color_value::ColorValue;
use crate::traits::{Representation, Colored, Discardable, Power, Points};

#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Normal {
    pub color: Color,
    pub value: ColorValue
}

impl Normal {
    pub fn new(color: Color, value: ColorValue) -> Self {
        Normal {color, value}
    }
}

impl fmt::Display for Normal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", self.color, self.value)
    }
}

impl Points for Normal {
    fn points(&self) -> f64 {
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
        self.color.color()
    }
}

impl Representation for Normal {
    fn repr(&self) -> ColoredString {
        let re = Regex::new(r"[\*]").unwrap();
        re.replace_all(&self.value.repr(), format!("{}", self.color)).color(self.color())
    }
}
