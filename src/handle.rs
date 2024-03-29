use crate::points::Points;
use ordered_float::OrderedFloat;
use strum::{Display, EnumIter};

#[derive(Default, Display, Eq, PartialEq, Debug, Copy, Clone, EnumIter)]
#[repr(u32)]
pub enum Handle {
    #[default]
    Refused,
    Simple,
    Double,
    Triple,
}

impl Points for Handle {
    fn points(&self) -> OrderedFloat<f64> {
        OrderedFloat(match self {
            Self::Refused => 0.0,
            Self::Simple => 20.0,
            Self::Double => 30.0,
            Self::Triple => 40.0,
        })
    }
}
