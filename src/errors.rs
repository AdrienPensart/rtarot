use crate::deck::Deck;
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum TarotErrorKind {
    #[error("A deck contains only one trump: the petit.")]
    PetitSec,
    #[error("Card is invalid")]
    InvalidCard,
    #[error("Invalid number of players : {0}")]
    InvalidPlayers(String),
    #[error("Invalid mode")]
    InvalidMode,
    #[error("Invalid deck : {0}")]
    InvalidDeck(Deck),
    #[error("Invalid case")]
    InvalidCase(String),
    // #[error("Impossible case, taker cannot have all kings, queens, knights, jacks")]
    // AllSuitsError,
    #[error("Sum of score is not zero")]
    InvalidScores(String),
    #[error("Invalid number of oudlers : {0}")]
    InvalidOudlersCount(Deck),
    #[error("Invalid color")]
    InvalidColor,
    #[error("Random number distribution error")]
    WeightedError(#[from] rand_distr::weighted::Error),

    #[error("No handle at index {0}")]
    NoHandle(usize),
    #[error("No slam at index {0}")]
    NoSlam(usize),
    #[error("No contract at index {0}")]
    NoContract(usize),
    #[error("No card at index : {0}")]
    NoCard(usize),
    #[error("No player at index : {0}")]
    NoPlayer(usize),
    #[error("No role for player : {0}")]
    NoRoleForPlayer(String),
    #[error("No taker or auctions not finished")]
    NoTaker(usize),
    #[error("No ally at index {0}")]
    NoAlly(usize),
    #[error("No defenser at index {0}")]
    NoDefenser(usize),
    #[error("No attacker at index {0}")]
    NoAttacker(usize),
    #[error("No master in turn at index {0}")]
    NoMaster(usize),
    #[error("A player shoud belongs to a team")]
    NoTeamForPlayer(String),
}
