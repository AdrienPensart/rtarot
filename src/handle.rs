use crate::points::HasPoints;
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

impl HasPoints for Handle {
    fn points(&self) -> OrderedFloat<f64> {
        let points = match self {
            Self::Refused => 0.0,
            Self::Simple => 20.0,
            Self::Double => 30.0,
            Self::Triple => 40.0,
        };
        OrderedFloat(points)
    }
}
