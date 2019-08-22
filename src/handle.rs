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
            Handle::Refused => write!(f, "refuse handle"),
            Handle::Simple  => write!(f, "simple handle"),
            Handle::Double  => write!(f, "double handle"),
            Handle::Triple  => write!(f, "triple handle"),
        }
    }
}

impl Handle {
    pub fn new(count: usize, mode: Mode) -> Option<Handle> {
        match mode {
            Mode::Three => {
                match count {
                    0 ..= 12 => None,
                    13 ..= 14 => Some(Handle::Simple),
                    15 ..= 17 => Some(Handle::Double),
                    _ => Some(Handle::Triple)
                }
            },
            Mode::Four => {
                match count {
                    0 ..= 9 => None,
                    10 ..= 12 => Some(Handle::Simple),
                    13 ..= 14 => Some(Handle::Double),
                    _ => Some(Handle::Triple)
                }
            },
            Mode::Five => {
                match count {
                    0 ..= 7 => None,
                    8 ..= 9 => Some(Handle::Simple),
                    10 ..= 12 => Some(Handle::Double),
                    _ => Some(Handle::Triple)
                }
            }
        }
    }
    pub fn limit(&self, mode: Mode) -> usize {
        match self {
            Handle::Refused => 0 as usize,
            Handle::Simple => {
                match mode {
                    Mode::Three => 13 as usize,
                    Mode::Four => 10 as usize,
                    Mode::Five => 8 as usize
                }
            },
            Handle::Double => {
                match mode {
                    Mode::Three => 15 as usize,
                    Mode::Four => 13 as usize,
                    Mode::Five => 10 as usize
                }
            }
            Handle::Triple => {
                match mode {
                    Mode::Three => 18 as usize,
                    Mode::Four => 15 as usize,
                    Mode::Five => 13 as usize
                }
            }
        }
    }
}
