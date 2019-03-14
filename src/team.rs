use std::fmt;
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Team {
    Defense,
    Attack,
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Team::Defense => write!(f, "defense"),
            Team::Attack  => write!(f, "attack"),
        }
    }
}
