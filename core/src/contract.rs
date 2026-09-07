use ordered_float::OrderedFloat;
use strum::{Display, EnumIter};

#[derive(Display, Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, EnumIter)]
pub enum Contract {
    Petite,
    Garde,
    #[strum(serialize = "Garde Sans")]
    GardeSans,
    #[strum(serialize = "Garde Contre")]
    GardeContre,
}

impl Contract {
    #[must_use]
    pub const fn multiplier(self) -> OrderedFloat<f64> {
        match self {
            Self::Petite => OrderedFloat(1.0),
            Self::Garde => OrderedFloat(2.0),
            Self::GardeSans => OrderedFloat(4.0),
            Self::GardeContre => OrderedFloat(6.0),
        }
    }
}
