use strum::Display;

#[derive(Display, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Taker,
    Ally,
    Defenser,
}
