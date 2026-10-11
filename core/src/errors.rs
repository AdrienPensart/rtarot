use crate::card::Card;
use crate::deck::Deck;
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum TarotErrorKind {
    #[error("A deck contains only one trump: the petit.")]
    PetitSec,
    #[error("Card {0} is not available")]
    InvalidCard(Card),
    #[error("A cut at {0} cards does not leave enough cards on both sides")]
    InvalidCut(usize),
    #[error("Contract {0} is not available")]
    InvalidContract(String),
    #[error("Handle {0} is not available")]
    InvalidHandle(String),
    #[error("An answer was given while the engine asked nothing")]
    UnexpectedAnswer,
    #[error("Answer does not match the pending question")]
    MismatchedAnswer,
    #[error("There is no deal in progress")]
    NoDeal,
    #[error("Invalid number of players : {0}")]
    InvalidPlayers(String),
    #[error("Invalid mode")]
    InvalidMode,
    #[error("Invalid deck : {0}")]
    InvalidDeck(Deck),
    #[error("Invalid case : {0}")]
    InvalidCase(String),
    // #[error("Impossible case, taker cannot have all kings, queens, knights, jacks")]
    // AllSuitsError,
    #[error("Sum of score is not zero")]
    InvalidScores(String),
    #[error("Invalid number of oudlers : {0}")]
    InvalidOudlersCount(Deck),
    #[error("Invalid color")]
    InvalidColor,
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
    #[error("No taker or auctions not finished : {0}")]
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
