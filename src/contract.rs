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
    pub const fn multiplier(self) -> f64 {
        match self {
            Self::Petite => 1.0,
            Self::Garde => 2.0,
            Self::GardeSans => 4.0,
            Self::GardeContre => 6.0,
        }
    }
}
