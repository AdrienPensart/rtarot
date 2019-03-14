use failure::Context;
#[derive(Debug)]
pub struct TarotError {
    inner: Context<TarotErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum TarotErrorKind {
    #[fail(display = "A deck contains only one outsider: the petit.")]
    PetitSec,
    #[fail(display = "Card is invalid")]
    InvalidCard,
    #[fail(display = "Invalid number of players")]
    InvalidPlayers,
    #[fail(display = "No contract")]
    NoContract,
    #[fail(display = "Invalid mode")]
    InvalidMode,
    #[fail(display = "Invalid contract")]
    InvalidContract,
    #[fail(display = "Invalid case")]
    InvalidCase,
    #[fail(display = "Invalid color")]
    InvalidColor,
    #[fail(display = "No taker or auctions not finished")]
    NoTaker,
    #[fail(display = "A player shoud belongs to a team")]
    NoTeam,
}
