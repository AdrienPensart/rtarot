//! Rules engine for the game of tarot, with no user interface attached.
//!
//! The engine never reads, never prints and never blocks. It is driven with two
//! calls: read the [`Question`] it is waiting on, hand back an [`Answer`].
//! Everything worth showing comes out as [`Event`]s. That makes the very same
//! engine usable from a terminal, from a browser through WebAssembly, or from a
//! bot, without a single line of it changing.
//!
//! ```
//! use rtarot_core::{Answer, Engine, Mode};
//!
//! let mut engine = Engine::new(Mode::Four, 1, 42);
//! while let Some(question) = engine.pending().cloned() {
//!     let answer = engine.random_answer(&question)?;
//!     engine.answer(answer)?;
//! }
//! assert!(engine.is_finished());
//! # Ok::<(), rtarot_core::TarotErrorKind>(())
//! ```

pub mod bot;
pub mod card;
pub mod constants;
pub mod contract;
pub mod deck;
pub mod engine;
pub mod errors;
pub mod event;
pub mod handle;
pub mod helpers;
pub mod mode;
pub mod normal;
pub mod player;
pub mod player_in_game;
pub mod points;
pub mod question;
pub mod role;
pub mod suit;
pub mod suit_value;
pub mod team;
pub mod traits;
pub mod trump;
pub mod turn;

pub use card::Card;
pub use contract::Contract;
pub use deck::Deck;
pub use engine::{Deal, Engine};
pub use errors::TarotErrorKind;
pub use event::Event;
pub use handle::Handle;
pub use mode::Mode;
pub use normal::Normal;
pub use player::Player;
pub use player_in_game::PlayerInGame;
pub use points::Points;
pub use question::{Answer, Question};
pub use role::Role;
pub use suit::Suit;
pub use suit_value::SuitValue;
pub use team::Team;
pub use traits::{Color, Discardable, Power, Representation};
pub use trump::Trump;
pub use turn::Turn;
