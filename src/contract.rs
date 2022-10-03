use std::fmt;
use std::str::FromStr;
use crate::errors::TarotErrorKind;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, EnumIter)]
pub enum Contract {
    Pass,
    Petite,
    Garde,
    GardeSans,
    GardeContre,
}

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pass         => write!(f, "Passe. (x0)"),
            Self::Petite       => write!(f, "Petite (x1)"),
            Self::Garde        => write!(f, "Garde (x2)"),
            Self::GardeSans    => write!(f, "Garde Sans (x4)"),
            Self::GardeContre  => write!(f, "Garde Contre (x6)"),
        }
    }
}

impl Contract {
    pub const fn multiplier(self) -> f64 {
        match self {
            Self::Pass         => 0.0,
            Self::Petite       => 1.0,
            Self::Garde        => 2.0,
            Self::GardeSans    => 4.0,
            Self::GardeContre  => 6.0,
        }
    }
}

impl FromStr for Contract {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Contract, TarotErrorKind> {
        match s {
            "0" => Ok(Self::Pass),
            "1" => Ok(Self::Petite),
            "2" => Ok(Self::Garde),
            "4" => Ok(Self::GardeSans),
            "6" => Ok(Self::GardeContre),
            _ => Err(TarotErrorKind::InvalidContract),
        }
    }
}

#[test]
fn card_contracts() {
    let pass = Contract::from_str("0");
    assert!(pass == Ok(Contract::Pass) );
    let petite = Contract::from_str("1");
    assert!(petite == Ok(Contract::Petite) );
    let garde = Contract::from_str("2");
    assert!(garde == Ok(Contract::Garde) );
    let garde_sans = Contract::from_str("4");
    assert!(garde_sans == Ok(Contract::GardeSans) );
    let garde_contre = Contract::from_str("6");
    assert!(garde_contre == Ok(Contract::GardeContre) );
}
