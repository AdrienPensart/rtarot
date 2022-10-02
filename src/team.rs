use std::fmt;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Team {
    Defense,
    Attack,
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Defense => write!(f, "defense"),
            Self::Attack  => write!(f, "attack"),
        }
    }
}
