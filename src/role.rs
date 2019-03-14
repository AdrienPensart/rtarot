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
            Role::Taker => write!(f, "taker"),
            Role::Ally => write!(f, "ally of taker"),
            Role::Defenser => write!(f, "defenser"),
        }
    }
}
