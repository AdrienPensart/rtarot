use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum TarotErrorKind {
    #[error("A deck contains only one outsider: the petit.")]
    PetitSec,
    #[error("Card is invalid")]
    InvalidCard,
    #[error("Invalid number of players")]
    InvalidPlayers,
    #[error("No contract")]
    NoContract,
    #[error("Invalid mode")]
    InvalidMode,
    #[error("Invalid contract")]
    InvalidContract,
    #[error("Invalid case")]
    InvalidCase,
    #[error("Invalid color")]
    InvalidColor,
    #[error("No taker or auctions not finished")]
    NoTaker,
    #[error("A player shoud belongs to a team")]
    NoTeam,
}
