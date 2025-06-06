use derive_new::new;
use ordered_float::OrderedFloat;
use std::fmt;

use crate::mode::Mode;
use crate::options::Options;

#[derive(new, Eq, PartialEq, Clone, Debug)]
pub struct Player {
    name: String,
    mode: Mode,
    options: Options,
    #[new(default)]
    score: OrderedFloat<f64>,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (score: {})", self.name, self.score)
    }
}

impl Player {
    pub fn add_score(&mut self, points: OrderedFloat<f64>) {
        self.score += points;
    }
    #[must_use]
    pub const fn score(&self) -> OrderedFloat<f64> {
        self.score
    }
    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }
    #[must_use]
    pub const fn options(&self) -> &Options {
        &self.options
    }
}
