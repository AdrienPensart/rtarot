//! A bot that plays by a handful of rules of thumb.
//!
//! It is not clever, but it is not random either: it bids on the strength of
//! its hand, discards to create voids, follows with the cheapest card that
//! cannot win, takes with the cheapest card that can, leads kings and long
//! suits, saves the petit on a partner's trick and never wastes the fool in
//! the last trick. Everything it looks at is public: its own hand, the cards
//! on the table and the tricks already won.

use crate::card::Card;
use crate::contract::Contract;
use crate::deck::Deck;
use crate::engine::{Deal, Engine};
use crate::errors::TarotErrorKind;
use crate::handle::Handle;
use crate::mode::Mode;
use crate::player_in_game::PlayerInGame;
use crate::points::Points;
use crate::question::{Answer, Question};
use crate::role::Role;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::team::Team;
use crate::traits::Power;
use crate::trump::Trump;
use ordered_float::OrderedFloat;
use strum::IntoEnumIterator;

/// The reply a sensible, if unimaginative, player would give.
pub fn answer(engine: &Engine, question: &Question) -> Result<Answer, TarotErrorKind> {
    let deal = engine.deal().ok_or(TarotErrorKind::NoDeal)?;
    let mode = engine.mode();
    let seat = question.player();
    let me = deal.player(seat)?;
    let card = |choice: Option<Card>| {
        choice
            .map(Answer::Card)
            .ok_or_else(|| TarotErrorKind::InvalidCase("the bot has nothing to play".to_string()))
    };
    match question {
        Question::Contract { allowed, .. } => Ok(Answer::Contract(bid(me.hand(), mode, allowed))),
        Question::Slam { .. } => Ok(Answer::Slam(slam(me.hand(), mode))),
        Question::Handle { allowed, .. } => Ok(Answer::Handle(handle(me.hand(), allowed))),
        Question::Call { allowed, .. } => card(call(me.hand(), allowed)),
        Question::Discard { allowed, .. } => card(discard(me.hand(), allowed)),
        Question::TrimHandle { shown, .. } => card(trim(shown)),
        Question::Play { allowed, .. } => card(Table::new(deal, mode, seat, me).play(allowed)),
    }
}

// ---- weighing a hand -------------------------------------------------------

fn suit_len(hand: &Deck, suit: Suit) -> usize {
    hand.iter()
        .filter(|card| matches!(card, Card::Normal(n) if *n.suit() == suit))
        .count()
}

fn real_trumps(hand: &Deck) -> usize {
    hand.iter()
        .filter(|card| card.is_trump() && !card.is_fool())
        .count()
}

fn has(hand: &Deck, trump: Trump) -> bool {
    hand.has(&Card::Trump(trump))
}

/// How promising a hand is, as an attack. Around 36 is a Petite with 18 cards.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn strength(hand: &Deck) -> f64 {
    let trumps = real_trumps(hand);
    let mut score = 0.0;
    if has(hand, Trump::_21) {
        score += 10.0;
    }
    if has(hand, Trump::Fool) {
        score += 7.0;
    }
    if has(hand, Trump::Petit) {
        // RULE OF THUMB: a petit is only an asset when enough trumps protect it
        score += if trumps >= 5 { 5.0 } else { 1.0 };
    }
    score = 2.0f64.mul_add(trumps as f64, score);
    score += hand
        .iter()
        .filter(|card| matches!(card, Card::Trump(t) if (Trump::_16..=Trump::_20).contains(t)))
        .count() as f64;
    for card in hand.iter() {
        if let Card::Normal(normal) = card {
            score += match normal.value() {
                SuitValue::King => 6.0,
                SuitValue::Queen => 3.0,
                SuitValue::Knight => 2.0,
                SuitValue::Jack => 1.0,
                SuitValue::_1
                | SuitValue::_2
                | SuitValue::_3
                | SuitValue::_4
                | SuitValue::_5
                | SuitValue::_6
                | SuitValue::_7
                | SuitValue::_8
                | SuitValue::_9
                | SuitValue::_10 => 0.0,
            };
        }
    }
    for suit in Suit::iter() {
        score += match suit_len(hand, suit) {
            0 => 4.0,
            1 => 2.0,
            2..=4 => 0.0,
            long => 3.0 + (long - 5) as f64,
        };
    }
    score
}

/// The strength a hand needs before bidding `contract`.
///
/// Set from the distribution of random hands: a Petite is roughly the best
/// quarter of hands, a Garde the best eighth, a Garde Sans one hand in
/// twenty-five and a Garde Contre one in a hundred.
#[must_use]
pub const fn threshold(contract: Contract, mode: Mode) -> f64 {
    // Petite, Garde, Garde Sans, Garde Contre.
    let steps = match mode {
        Mode::Three => [48.0, 53.0, 59.0, 63.0],
        Mode::Four => [36.0, 40.0, 46.0, 51.0],
        Mode::Five => [30.0, 35.0, 40.0, 46.0],
    };
    match contract {
        Contract::Petite => steps[0],
        Contract::Garde => steps[1],
        Contract::GardeSans => steps[2],
        Contract::GardeContre => steps[3],
    }
}

fn bid(hand: &Deck, mode: Mode, allowed: &[Contract]) -> Option<Contract> {
    let score = strength(hand);
    allowed
        .iter()
        .copied()
        .filter(|contract| score >= threshold(*contract, mode))
        .max()
}

fn slam(hand: &Deck, mode: Mode) -> bool {
    has(hand, Trump::_21)
        && has(hand, Trump::Fool)
        && strength(hand) >= 1.5 * threshold(Contract::GardeContre, mode)
}

/// Showing trumps helps the opponents, so only a hand likely to win announces.
fn handle(hand: &Deck, allowed: &[Handle]) -> Handle {
    if has(hand, Trump::_21) || has(hand, Trump::Fool) {
        allowed
            .iter()
            .copied()
            .max_by_key(Points::points)
            .unwrap_or_default()
    } else {
        Handle::Refused
    }
}

/// Call in the longest suit whose king is not in hand, so that the suit becomes strong.
fn call(hand: &Deck, allowed: &[Card]) -> Option<Card> {
    allowed.iter().copied().max_by_key(|card| match card {
        Card::Normal(normal) => (!hand.has(card), suit_len(hand, *normal.suit())),
        Card::Trump(_) => (false, 0),
    })
}

/// Empty the shortest suit first, lowest cards first; trumps only when forced.
fn discard(hand: &Deck, allowed: &[Card]) -> Option<Card> {
    allowed.iter().copied().min_by_key(|card| match card {
        Card::Normal(normal) => (suit_len(hand, *normal.suit()), normal.power()),
        Card::Trump(trump) => (usize::MAX, *trump as usize),
    })
}

/// Drop the lowest trump that is not an oudler.
fn trim(shown: &[Card]) -> Option<Card> {
    shown
        .iter()
        .copied()
        .min_by_key(|card| (card.is_oudler(), card.power()))
}

// ---- playing a card --------------------------------------------------------

/// The cheapest card: fewest points, then lowest rank.
fn cheapest(cards: impl IntoIterator<Item = Card>) -> Option<Card> {
    cards
        .into_iter()
        .min_by_key(|card| (card.points(), card.power()))
}

struct Table<'a> {
    deal: &'a Deal,
    mode: Mode,
    seat: usize,
    me: &'a PlayerInGame,
}

impl Table<'_> {
    const fn new<'a>(deal: &'a Deal, mode: Mode, seat: usize, me: &'a PlayerInGame) -> Table<'a> {
        Table {
            deal,
            mode,
            seat,
            me,
        }
    }

    fn players(&self) -> usize {
        self.deal.players().len()
    }

    fn last_to_play(&self) -> bool {
        self.deal.turn().len() + 1 == self.players()
    }

    /// Whether `card` has already been seen on the table this deal.
    fn played(&self, card: Card) -> bool {
        self.deal.turn().cards().has(&card)
            || self
                .deal
                .players()
                .iter()
                .any(|player| player.owned().has(&card))
    }

    /// Whether the bot knows `other` to be on its side.
    ///
    /// With five players the ally is secret until the called king is played:
    /// the ally knows the taker, nobody else knows the ally.
    fn partner(&self, other: usize) -> bool {
        if other == self.seat {
            return true;
        }
        let Ok(theirs) = self.deal.player(other).map(PlayerInGame::team) else {
            return false;
        };
        let mine = self.me.team();
        if self.mode != Mode::Five {
            return mine.is_some() && mine == theirs;
        }
        let revealed = self.me.callee().is_some_and(|callee| self.played(callee));
        match self.me.role() {
            Some(Role::Ally) => theirs == Some(Team::Attack),
            Some(Role::Taker) => revealed && theirs == Some(Team::Attack),
            Some(Role::Defenser) | None => revealed && theirs == Some(Team::Defense),
        }
    }

    fn is_attack(&self) -> bool {
        self.me.team() == Some(Team::Attack)
    }

    /// Whether `card` would take the lead of the current trick.
    fn wins(&self, card: Card) -> bool {
        !card.is_fool()
            && self
                .deal
                .turn()
                .master_card()
                .is_none_or(|master| !master.master(card))
    }

    fn play(&self, allowed: &[Card]) -> Option<Card> {
        let fool = Card::Trump(Trump::Fool);
        let petit = Card::Trump(Trump::Petit);
        // RULE OF THUMB: a fool kept for the last trick is a fool lost.
        if self.me.hand().len() == 2 && allowed.contains(&fool) {
            return Some(fool);
        }
        let others: Vec<Card> = allowed
            .iter()
            .copied()
            .filter(|card| *card != fool)
            .collect();
        if others.is_empty() {
            return allowed.first().copied();
        }
        if self.deal.turn().master_card().is_none() {
            return self.lead(&others);
        }

        let partner_master = self.partner(self.deal.master());
        if partner_master {
            if self.last_to_play() {
                // The trick is ours: save the petit, or load it with points.
                if others.contains(&petit) {
                    return Some(petit);
                }
                let gift = others
                    .iter()
                    .copied()
                    .filter(|card| !card.is_trump())
                    .filter(
                        |card| !matches!(card, Card::Normal(n) if *n.value() == SuitValue::King),
                    )
                    .max_by_key(|card| (card.points(), card.power()));
                return gift.or_else(|| cheapest(others.iter().copied()));
            }
            return Self::give_up(&others, allowed.contains(&fool));
        }

        let winning: Vec<Card> = others
            .iter()
            .copied()
            .filter(|card| self.wins(*card))
            .collect();
        if winning.is_empty() {
            return Self::give_up(&others, allowed.contains(&fool));
        }
        // RULE OF THUMB: take with the cheapest card that takes.
        winning.into_iter().min_by_key(Power::power)
    }

    /// Nothing to win here: give as little as possible, and keep the petit alive.
    fn give_up(others: &[Card], fool_allowed: bool) -> Option<Card> {
        let petit = Card::Trump(Trump::Petit);
        let safe: Vec<Card> = others
            .iter()
            .copied()
            .filter(|card| *card != petit)
            .collect();
        let cheap = cheapest(safe.iter().copied());
        let costly = cheap.is_none_or(|card| card.points() >= OrderedFloat(2.5));
        if fool_allowed && costly {
            return Some(Card::Trump(Trump::Fool));
        }
        cheap.or(Some(petit))
    }

    /// Open a trick.
    fn lead(&self, others: &[Card]) -> Option<Card> {
        let hand = self.me.hand();
        let petit = Card::Trump(Trump::Petit);
        let colors: Vec<Card> = others
            .iter()
            .copied()
            .filter(|card| !card.is_trump())
            .collect();

        // RULE OF THUMB: with the 21 and plenty of trumps, the attack hunts the petit.
        let petit_out = !hand.has(&petit) && !self.played(petit);
        if self.is_attack()
            && petit_out
            && others.contains(&Card::Trump(Trump::_21))
            && real_trumps(hand) * 3 >= hand.len()
        {
            return Some(Card::Trump(Trump::_21));
        }
        // RULE OF THUMB: a king leads and takes, unless somebody cuts.
        if let Some(king) = colors
            .iter()
            .copied()
            .find(|card| matches!(card, Card::Normal(n) if *n.value() == SuitValue::King))
        {
            return Some(king);
        }
        // RULE OF THUMB: lead low in the longest suit.
        let longest = colors.iter().copied().max_by_key(|card| match card {
            Card::Normal(normal) => suit_len(hand, *normal.suit()),
            Card::Trump(_) => 0,
        });
        if let Some(Card::Normal(normal)) = longest {
            let suit = *normal.suit();
            return cheapest(
                colors
                    .iter()
                    .copied()
                    .filter(|card| matches!(card, Card::Normal(n) if *n.suit() == suit)),
            );
        }
        // Only trumps left: the lowest one, the petit last.
        cheapest(others.iter().copied().filter(|card| *card != petit)).or(Some(petit))
    }
}

#[cfg(test)]
mod tests {
    use super::{answer, strength, threshold};
    use crate::contract::Contract;
    use crate::deck::Deck;
    use crate::engine::Engine;
    use crate::event::Event;
    use crate::mode::Mode;
    use crate::suit::Suit;
    use crate::suit_value::SuitValue;
    use crate::{Card, Trump};
    use ordered_float::OrderedFloat;
    use strum::IntoEnumIterator;

    fn deck(cards: &[Card]) -> Deck {
        Deck::new(cards.to_vec())
    }

    #[test]
    fn a_hand_full_of_oudlers_and_kings_is_strong() {
        let strong = deck(&[
            Card::Trump(Trump::_21),
            Card::Trump(Trump::Fool),
            Card::Trump(Trump::Petit),
            Card::Trump(Trump::_20),
            Card::Trump(Trump::_19),
            Card::Trump(Trump::_12),
            Card::Trump(Trump::_7),
            Card::Trump(Trump::_3),
            Card::normal(Suit::Heart, SuitValue::King),
            Card::normal(Suit::Spade, SuitValue::King),
        ]);
        let weak = deck(&[
            Card::normal(Suit::Heart, SuitValue::_2),
            Card::normal(Suit::Spade, SuitValue::_3),
            Card::normal(Suit::Diamond, SuitValue::_4),
            Card::normal(Suit::Club, SuitValue::_5),
        ]);
        assert!(strength(&strong) > threshold(Contract::GardeContre, Mode::Four));
        assert!(strength(&weak) < threshold(Contract::Petite, Mode::Four));
    }

    /// Whole games with bots in every seat: deals get taken, and the attack
    /// wins often enough that the thresholds are neither timid nor reckless.
    #[test]
    fn bots_bid_and_win_a_reasonable_share() {
        for mode in Mode::iter() {
            let (mut bid, mut passed, mut won, mut lost) = (0u32, 0u32, 0u32, 0u32);
            for seed in 0..60 {
                let mut engine = Engine::new(mode, 5, seed);
                loop {
                    for event in engine.take_events() {
                        if matches!(event, Event::EveryonePassed) {
                            passed += 1;
                        }
                        if let Event::DealScored { points, .. } = event {
                            bid += 1;
                            if points > OrderedFloat(0.0) {
                                won += 1;
                            } else {
                                lost += 1;
                            }
                        }
                    }
                    let Some(question) = engine.pending().cloned() else {
                        break;
                    };
                    let reply = answer(&engine, &question).expect("the bot always answers");
                    engine.answer(reply).expect("the bot answers legally");
                }
                assert_eq!(engine.is_consistent(), Ok(()));
            }
            let bid_rate = f64::from(bid) / f64::from(bid + passed);
            let win_rate = f64::from(won) / f64::from(won + lost);
            eprintln!("{mode}: bid {bid_rate:.2} of deals, attack won {win_rate:.2}");
            assert!(
                (0.5..=1.0).contains(&bid_rate),
                "{mode}: bid rate {bid_rate}"
            );
            assert!(
                (0.3..=0.8).contains(&win_rate),
                "{mode}: win rate {win_rate}"
            );
        }
    }

    /// The rules must be worth something: one bot among random players ends up ahead.
    #[test]
    fn a_bot_outscores_random_players() {
        let mut total = OrderedFloat(0.0);
        for seed in 0..80 {
            let mut engine = Engine::new(Mode::Four, 5, seed);
            while let Some(question) = engine.pending().cloned() {
                let reply = if question.player() == 0 {
                    answer(&engine, &question).expect("the bot always answers")
                } else {
                    engine
                        .random_answer(&question)
                        .expect("a random answer exists")
                };
                engine.answer(reply).expect("a legal answer");
            }
            total += engine.player(0).expect("seat 0").score();
        }
        eprintln!("bot against random players: {total} over 80 games");
        assert!(total > OrderedFloat(0.0));
    }
}
