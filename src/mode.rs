use std::fmt;
use std::str::FromStr;
use crate::errors::TarotErrorKind;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, EnumIter, clap::ValueEnum)]
pub enum Mode {
    Three = 3,
    #[default]
    Four = 4,
    Five = 5,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Three => write!(f, "{} players, 1 vs 2 (easy)", Mode::Three as usize),
            Self::Four  => write!(f, "{} players, 1 vs 3 (standard)", Mode::Four as usize),
            Self::Five  => write!(f, "{} players, 2 vs 3 (call a king)", Mode::Five as usize),
        }
    }
}

impl FromStr for Mode {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Mode, TarotErrorKind> {
        match s {
            "3" | "three" => Ok(Self::Three),
            "4" | "four" => Ok(Self::Four),
            "5" | "five" => Ok(Self::Five),
            _ => Err(TarotErrorKind::InvalidPlayers),
        }
    }
}

impl Mode {
    pub fn dog_size(self) -> usize {
        match self {
            Self::Five => 3,
            _ => 6
        }
    }
    pub fn cards_per_turn(self) -> usize {
        match self {
            Self::Three=> 4,
            _ => 3
        }
    }
    pub fn cards_per_player(self) -> usize {
        match self {
            Self::Three => 24,
            Self::Four  => 18,
            Self::Five  => 15,
        }
    }
}

#[test]
fn mode_tests() {
    let mode = Mode::default();
    println!("mode: {}", &mode);

    let three = Mode::from_str("3");
    assert!(three == Ok(Mode::Three));

    let four = Mode::from_str("4");
    assert!(four == Ok(Mode::Four));

    let five = Mode::from_str("5");
    assert!(five == Ok(Mode::Five));
}
