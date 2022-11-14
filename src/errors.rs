use crate::deck::Deck;
use rand_distr::WeightedError;
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
    #[error("Invalid deck : {0}")]
    InvalidDeck(Deck),
    #[error("Invalid case")]
    InvalidCase,
    #[error("Sum of score is not zero")]
    InvalidScores(String),
    #[error("Invalid number of oudlers : {0}")]
    InvalidOudlersCount(Deck),
    #[error("No role for player : {0}")]
    NoRoleForPlayer(String),
    #[error("Invalid color")]
    InvalidColor,
    #[error("No taker or auctions not finished")]
    NoTaker,
    #[error("A player shoud belongs to a team")]
    NoTeamForPlayer(String),
    #[error("Random number distribution error")]
    WeightedError(#[from] WeightedError),
}
