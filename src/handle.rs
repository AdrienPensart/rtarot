use std::fmt;
use crate::traits::Points;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Handle {
    Refused  = 0,
    Simple  = 20,
    Double  = 30,
    Triple  = 40,
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Refused => write!(f, "refuse handle"),
            Self::Simple  => write!(f, "simple handle"),
            Self::Double  => write!(f, "double handle"),
            Self::Triple  => write!(f, "triple handle"),
        }
    }
}

impl Points for Handle {
    fn points(&self) -> f64 {
        match self {
            Self::Refused => 0.0,
            Self::Simple  => 20.0,
            Self::Double  => 30.0,
            Self::Triple  => 40.0,
        }
    }
}
