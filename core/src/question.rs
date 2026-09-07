use crate::card::Card;
use crate::contract::Contract;
use crate::handle::Handle;

/// Something the engine needs to know before it can go on.
///
/// A question never blocks: the driver reads it, obtains an answer however it
/// likes (a terminal prompt, a click, a bot) and hands it back with
/// [`crate::engine::Engine::answer`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Question {
    /// Bid or pass. Passing is always allowed, even when `allowed` is empty.
    Contract {
        player: usize,
        allowed: Vec<Contract>,
    },
    /// Call a king to find an ally, five players only.
    Call { player: usize, allowed: Vec<Card> },
    /// Announce a slam, before the dog is turned over.
    Slam { player: usize },
    /// Put one card back in the dog.
    Discard {
        player: usize,
        allowed: Vec<Card>,
        remaining: usize,
    },
    /// Announce a handle, before playing the first card of the deal.
    Handle {
        player: usize,
        allowed: Vec<Handle>,
    },
    /// Drop one trump from a handle that shows more trumps than needed.
    TrimHandle {
        player: usize,
        handle: Handle,
        shown: Vec<Card>,
        to_remove: usize,
    },
    /// Play one card in the current turn.
    Play { player: usize, allowed: Vec<Card> },
}

impl Question {
    /// Whose turn it is to answer.
    #[must_use]
    pub const fn player(&self) -> usize {
        match self {
            Self::Contract { player, .. }
            | Self::Call { player, .. }
            | Self::Slam { player }
            | Self::Discard { player, .. }
            | Self::Handle { player, .. }
            | Self::TrimHandle { player, .. }
            | Self::Play { player, .. } => *player,
        }
    }
}

/// The reply to a [`Question`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Answer {
    /// Reply to [`Question::Contract`], `None` means pass.
    Contract(Option<Contract>),
    /// Reply to [`Question::Slam`].
    Slam(bool),
    /// Reply to [`Question::Handle`].
    Handle(Handle),
    /// Reply to [`Question::Call`], [`Question::Discard`], [`Question::TrimHandle`]
    /// and [`Question::Play`]: cards are unique, so they name themselves.
    Card(Card),
}
