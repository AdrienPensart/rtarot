use ordered_float::OrderedFloat;
use std::fmt;

use crate::mode::Mode;
use crate::options::Options;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Player {
    name: String,
    mode: Mode,
    options: Options,
    score: OrderedFloat<f64>,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (score: {})", self.name, self.score)
    }
}

impl Player {
    pub fn new(name: String, mode: Mode, options: Options) -> Self {
        Self {
            name,
            mode,
            options,
            score: OrderedFloat(0.0),
        }
    }
    pub fn add_score(&mut self, points: OrderedFloat<f64>) {
        self.score += points
    }
    pub fn score(&self) -> OrderedFloat<f64> {
        self.score
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
