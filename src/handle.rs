use std::fmt;
use crate::mode::Mode;

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

impl Handle {
    pub fn new(count: usize, mode: Mode) -> Option<Handle> {
        match mode {
            Mode::Three => {
                match count {
                    0 ..= 12 => None,
                    13 ..= 14 => Some(Self::Simple),
                    15 ..= 17 => Some(Self::Double),
                    _ => Some(Self::Triple)
                }
            },
            Mode::Four => {
                match count {
                    0 ..= 9 => None,
                    10 ..= 12 => Some(Self::Simple),
                    13 ..= 14 => Some(Self::Double),
                    _ => Some(Self::Triple)
                }
            },
            Mode::Five => {
                match count {
                    0 ..= 7 => None,
                    8 ..= 9 => Some(Self::Simple),
                    10 ..= 12 => Some(Self::Double),
                    _ => Some(Self::Triple)
                }
            }
        }
    }
    pub fn limit(&self, mode: Mode) -> usize {
        match self {
            Self::Refused => 0 as usize,
            Self::Simple => {
                match mode {
                    Mode::Three => 13 as usize,
                    Mode::Four => 10 as usize,
                    Mode::Five => 8 as usize
                }
            },
            Self::Double => {
                match mode {
                    Mode::Three => 15 as usize,
                    Mode::Four => 13 as usize,
                    Mode::Five => 10 as usize
                }
            }
            Self::Triple => {
                match mode {
                    Mode::Three => 18 as usize,
                    Mode::Four => 15 as usize,
                    Mode::Five => 13 as usize
                }
            }
        }
    }
}
