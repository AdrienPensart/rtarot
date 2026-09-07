use strum::Display;

#[derive(Display, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Team {
    Defense,
    Attack,
}
