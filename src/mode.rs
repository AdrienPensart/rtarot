use std::fmt;
use std::str::FromStr;
use crate::errors::TarotErrorKind;
use crate::handle::Handle;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Mode {
    Three,
    #[default]
    Four,
    Five,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Three => write!(f, "{} players, 1 vs 2 (easy)", self.players()),
            Self::Four  => write!(f, "{} players, 1 vs 3 (standard)", self.players()),
            Self::Five  => write!(f, "{} players, 2 vs 3 (call a king)", self.players()),
        }
    }
}

impl From<usize> for Mode {
    fn from(value: usize) -> Self {
        match value {
            3 => Self::Three,
            4 => Self::Four,
            5 => Self::Five,
            _ => panic!("Unable to convert value to Mode")
        }
    }
}

impl TryFrom<u8> for Mode {
    type Error = TarotErrorKind;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            _ => Err(TarotErrorKind::InvalidPlayers)
        }
    }
}

impl FromStr for Mode {
    type Err = TarotErrorKind;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "3" | "three" => Ok(Self::Three),
            "4" | "four" => Ok(Self::Four),
            "5" | "five" => Ok(Self::Five),
            _ => Err(TarotErrorKind::InvalidPlayers),
        }
    }
}

impl Mode {
    pub const fn players(self) -> usize {
        match self {
            Self::Three => 3,
            Self::Four  => 4,
            Self::Five  => 5,
        }
    }

    pub const fn default() -> Self {
        Self::Four
    }

    pub fn dog_size(&self) -> usize {
        match self {
            Self::Five => 3,
            _ => 6
        }
    }
    pub fn cards_per_turn(&self) -> usize {
        match self {
            Self::Three=> 4,
            _ => 3
        }
    }
    pub fn cards_per_player(&self) -> usize {
        match self {
            Self::Three => 24,
            Self::Four  => 18,
            Self::Five  => 15,
        }
    }
    pub fn player_name(&self, index: usize) -> &'static str {
        match self {
            Self::Three => match index {
                0 => "East",
                1 => "North",
                2 => "South",
                _ => panic!("Mode with 3 players does not support more than 3 default names")
            },
            Self::Four => match index {
                0 => "East",
                1 => "North",
                2 => "South",
                3 => "West",
                _ => panic!("Mode with 4 players does not support more than 4 default names")
            },
            Self::Five => match index {
                0 => "East",
                1 => "North",
                2 => "South",
                3 => "West",
                4 => "Compass",
                _ => panic!("Mode with 5 players does not support more than 5 default names")
            }
        }
    }
    pub fn handle(&self, count: usize) -> Option<Handle> {
        match self {
            Self::Three => {
                match count {
                    0 ..= 12 => None,
                    13 ..= 14 => Some(Handle::Simple),
                    15 ..= 17 => Some(Handle::Double),
                    _ => Some(Handle::Triple)
                }
            },
            Self::Four => {
                match count {
                    0 ..= 9 => None,
                    10 ..= 12 => Some(Handle::Simple),
                    13 ..= 14 => Some(Handle::Double),
                    _ => Some(Handle::Triple)
                }
            },
            Self::Five => {
                match count {
                    0 ..= 7 => None,
                    8 ..= 9 => Some(Handle::Simple),
                    10 ..= 12 => Some(Handle::Double),
                    _ => Some(Handle::Triple)
                }
            }
        }
    }
    pub fn handle_limit(&self, handle: &Handle) -> usize {
        match handle {
            Handle::Refused => 0_usize,
            Handle::Simple => {
                match self {
                    Self::Three => 13_usize,
                    Self::Four => 10_usize,
                    Self::Five => 8_usize
                }
            },
            Handle::Double => {
                match self {
                    Self::Three => 15_usize,
                    Self::Four => 13_usize,
                    Self::Five => 10_usize
                }
            }
            Handle::Triple => {
                match self {
                    Self::Three => 18_usize,
                    Self::Four => 15_usize,
                    Self::Five => 13_usize
                }
            }
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
