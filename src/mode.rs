use crate::errors::TarotErrorKind;
use crate::handle::Handle;
use ordered_float::OrderedFloat;
use std::fmt;
use std::str::FromStr;
use strum::EnumIter;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, EnumIter)]
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
            Self::Four => write!(f, "{} players, 1 vs 3 (standard)", self.players()),
            Self::Five => write!(f, "{} players, 2 vs 3 (call a king)", self.players()),
        }
    }
}

impl TryFrom<usize> for Mode {
    type Error = TarotErrorKind;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            _ => Err(Self::Error::InvalidPlayers(value.to_string())),
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
            _ => Err(Self::Err::InvalidPlayers(s.into())),
        }
    }
}

impl Mode {
    #[must_use]
    pub const fn players(self) -> usize {
        match self {
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
        }
    }
    #[must_use]
    pub const fn ratio(&self, with_ally: bool) -> OrderedFloat<f64> {
        let ratio = match self {
            Self::Three => 2.0,
            Self::Four => 3.0,
            Self::Five => {
                if with_ally {
                    2.0
                } else {
                    4.0
                }
            }
        };
        OrderedFloat(ratio)
    }
    #[must_use]
    pub const fn dog_size(&self) -> usize {
        match self {
            Self::Five => 3,
            _ => 6,
        }
    }
    #[must_use]
    pub const fn cards_per_turn(&self) -> usize {
        match self {
            Self::Three => 4,
            _ => 3,
        }
    }
    pub const fn cards_per_player(&self) -> usize {
        match self {
            Self::Three => 24,
            Self::Four => 18,
            Self::Five => 15,
        }
    }
    #[must_use]
    pub const fn max_cards_for_taker(&self) -> usize {
        self.dog_size() + self.cards_per_player()
    }
    pub fn player_name(&self, index: usize) -> Result<&'static str, TarotErrorKind> {
        match self {
            Self::Three => match index {
                0 => Ok("East"),
                1 => Ok("North"),
                2 => Ok("South"),
                _ => Err(TarotErrorKind::InvalidCase(
                    "Mode with 3 players does not support more than 3 default names".to_string(),
                )),
            },
            Self::Four => match index {
                0 => Ok("East"),
                1 => Ok("North"),
                2 => Ok("South"),
                3 => Ok("West"),
                _ => Err(TarotErrorKind::InvalidCase(
                    "Mode with 4 players does not support more than 4 default names".to_string(),
                )),
            },
            Self::Five => match index {
                0 => Ok("East"),
                1 => Ok("North"),
                2 => Ok("South"),
                3 => Ok("West"),
                4 => Ok("Compass"),
                _ => Err(TarotErrorKind::InvalidCase(
                    "Mode with 5 players does not support more than 5 default names".to_string(),
                )),
            },
        }
    }
    #[must_use]
    pub const fn handle(&self, count: usize) -> Option<Handle> {
        match self {
            Self::Three => match count {
                0..=12 => None,
                13..=14 => Some(Handle::Simple),
                15..=17 => Some(Handle::Double),
                _ => Some(Handle::Triple),
            },
            Self::Four => match count {
                0..=9 => None,
                10..=12 => Some(Handle::Simple),
                13..=14 => Some(Handle::Double),
                _ => Some(Handle::Triple),
            },
            Self::Five => match count {
                0..=7 => None,
                8..=9 => Some(Handle::Simple),
                10..=12 => Some(Handle::Double),
                _ => Some(Handle::Triple),
            },
        }
    }
    #[must_use]
    pub const fn handle_limit(&self, handle: &Handle) -> usize {
        match handle {
            Handle::Refused => 0,
            Handle::Simple => match self {
                Self::Three => 13,
                Self::Four => 10,
                Self::Five => 8,
            },
            Handle::Double => match self {
                Self::Three => 15,
                Self::Four => 13,
                Self::Five => 10,
            },
            Handle::Triple => match self {
                Self::Three => 18,
                Self::Four => 15,
                Self::Five => 13,
            },
        }
    }
}

#[test]
fn mode_tests() {
    let mode = Mode::default();
    println!("mode: {}", &mode);

    let three = Mode::from_str("3");
    assert_eq!(three, Ok(Mode::Three));

    let four = Mode::from_str("4");
    assert_eq!(four, Ok(Mode::Four));

    let five = Mode::from_str("5");
    assert_eq!(five, Ok(Mode::Five));
}
