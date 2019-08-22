use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Taker,
    Ally,
    Defenser,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Taker => write!(f, "taker"),
            Self::Ally => write!(f, "ally of taker"),
            Self::Defenser => write!(f, "defenser"),
        }
    }
}
