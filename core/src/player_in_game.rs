use derive_new::new;
use ordered_float::OrderedFloat;
use std::fmt;

use crate::card::Card;
use crate::deck::Deck;
use crate::errors::TarotErrorKind;
use crate::handle::Handle;
use crate::mode::Mode;
use crate::points::Points;
use crate::role::Role;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::team::Team;
use crate::trump::Trump;
use crate::turn::Turn;
use itertools::Itertools;
use strum::IntoEnumIterator;

/// Everything a player owns for the duration of one deal.
///
/// This type only ever *computes* what is legal, it never chooses:
/// choosing is the job of whoever drives the [`crate::engine::Engine`].
#[derive(new, Eq, PartialEq, Clone, Debug)]
pub struct PlayerInGame {
    mode: Mode,
    #[new(default)]
    slam: bool,
    #[new(default)]
    team: Option<Team>,
    #[new(default)]
    role: Option<Role>,
    #[new(default)]
    discard: Deck,
    #[new(default)]
    hand: Deck,
    #[new(default)]
    owned: Deck,
    #[new(default)]
    callee: Option<Card>,
    #[new(default)]
    handle: Option<Handle>,
}

impl fmt::Display for PlayerInGame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(role) = &self.role {
            write!(f, "{role} ")?;
        }
        if let Some(team) = &self.team {
            write!(f, "({team})")?;
        }
        Ok(())
    }
}

impl Points for PlayerInGame {
    fn points(&self) -> OrderedFloat<f64> {
        // RULE: the dog kept by a side counts in its points
        self.all_cards().points()
    }
}

impl PlayerInGame {
    // ---- state ----------------------------------------------------------
    #[must_use]
    pub const fn hand(&self) -> &Deck {
        &self.hand
    }
    #[must_use]
    pub const fn owned(&self) -> &Deck {
        &self.owned
    }
    #[must_use]
    pub const fn discard(&self) -> &Deck {
        &self.discard
    }
    #[must_use]
    pub fn all_cards(&self) -> Deck {
        let mut all_cards = self.owned.clone();
        all_cards.extend(&self.discard);
        all_cards
    }
    #[must_use]
    pub const fn role(&self) -> Option<Role> {
        self.role
    }
    #[must_use]
    pub const fn team(&self) -> Option<Team> {
        self.team
    }
    #[must_use]
    pub const fn handle(&self) -> Option<Handle> {
        self.handle
    }
    #[must_use]
    pub const fn callee(&self) -> Option<Card> {
        self.callee
    }
    #[must_use]
    pub const fn announced_slam(&self) -> bool {
        self.slam
    }
    #[must_use]
    pub fn is_taker(&self) -> bool {
        self.role == Some(Role::Taker)
    }
    #[must_use]
    pub fn is_attack(&self) -> bool {
        self.team == Some(Team::Attack)
    }
    #[must_use]
    pub fn has(&self, card: &Card) -> bool {
        self.hand.has(card)
    }
    #[must_use]
    pub fn petit_sec(&self) -> bool {
        self.hand.petit_sec()
    }
    #[must_use]
    pub const fn is_first_turn(&self) -> bool {
        self.mode.cards_per_player() == self.hand.len()
    }
    #[must_use]
    pub const fn last_turn(&self) -> bool {
        self.hand.is_empty()
    }
    #[must_use]
    pub const fn before_last_turn(&self) -> bool {
        self.hand.len() == 1
    }

    // ---- mutations ------------------------------------------------------
    pub const fn set_callee(&mut self, callee: Option<Card>) {
        self.callee = callee;
    }
    pub const fn set_team(&mut self, team: Team) {
        self.team = Some(team);
    }
    pub const fn set_role(&mut self, role: Role) {
        self.role = Some(role);
    }
    pub const fn set_slam(&mut self, slam: bool) {
        self.slam = slam;
    }
    pub const fn set_handle(&mut self, handle: Handle) {
        self.handle = Some(handle);
    }
    pub fn set_discard(&mut self, discard: &Deck) {
        self.discard = discard.clone();
    }
    pub fn push_owned(&mut self, card: Card) {
        self.owned.push(card);
    }
    pub fn extend_hand(&mut self, deck: &Deck) {
        self.hand.extend(deck);
        self.hand.sort();
    }
    pub fn extend_owned(&mut self, deck: &Deck) {
        self.owned.extend(deck);
    }
    pub fn give_low(&mut self) -> Option<Card> {
        self.owned.give_low()
    }
    pub fn give_fool(&mut self) -> Option<Card> {
        self.owned.give_fool()
    }

    /// Take `card` out of the hand, refusing anything that is not there.
    pub fn take_from_hand(&mut self, card: Card) -> Result<Card, TarotErrorKind> {
        let index = self
            .hand
            .iter()
            .position(|candidate| candidate == &card)
            .ok_or(TarotErrorKind::InvalidCard(card))?;
        Ok(self.hand.remove(index))
    }

    /// Move `card` from the hand to the dog the taker gives back.
    pub fn discard_card(&mut self, card: Card) -> Result<(), TarotErrorKind> {
        let discarded = self.take_from_hand(card)?;
        self.discard.push(discarded);
        self.hand.sort();
        Ok(())
    }

    // ---- fool compensation ----------------------------------------------
    #[must_use]
    pub fn owe_card(&self) -> bool {
        self.owned.has_fool() && (self.owned.len() % self.mode.players()) == 1
    }
    #[must_use]
    pub fn missing_card(&self) -> bool {
        !self.owned.has_fool()
            && (self.owned.len() % self.mode.players()) == (self.mode.players() - 1)
    }

    /// A player owns a whole number of turns, plus the fool if he kept it,
    /// minus one card if he won the turn in which the fool was kept by its owner.
    pub fn check_owned(&self) -> Result<(), TarotErrorKind> {
        let players = self.mode.players();
        let remainder = self.owned.len() % players;
        if remainder == 0
            || (remainder == 1 && self.owned.has_fool())
            || (remainder == players - 1 && !self.owned.has_fool())
        {
            return Ok(());
        }
        Err(TarotErrorKind::InvalidDeck(self.owned.clone()))
    }

    // ---- legal moves ----------------------------------------------------
    /// Kings the taker may call to find an ally, in five players mode.
    pub fn callable(&self) -> Result<Vec<Card>, TarotErrorKind> {
        if self.mode != Mode::Five {
            return Ok(Vec::new());
        }
        let mut value_callable: Vec<SuitValue> = vec![SuitValue::King];
        // RULE: holding every king of a value opens the call to the value below
        if self.hand.count_tete(SuitValue::King) == 4 {
            value_callable.push(SuitValue::Queen);
            if self.hand.count_tete(SuitValue::Queen) == 4 {
                value_callable.push(SuitValue::Knight);
                if self.hand.count_tete(SuitValue::Knight) == 4 {
                    value_callable.push(SuitValue::Jack);
                    if self.hand.count_tete(SuitValue::Jack) == 4 {
                        return Err(TarotErrorKind::InvalidCase(
                            "Impossible case, taker cannot have all kings, queens, knights, jacks"
                                .to_string(),
                        ));
                    }
                }
            }
        }
        Ok(Suit::iter()
            .cartesian_product(value_callable.iter())
            .map(|(suit, value)| Card::normal(suit, *value))
            .collect())
    }

    /// Cards the taker may still put in the dog, knowing how many are left to give.
    #[must_use]
    pub fn discardable(&self, remaining: usize) -> Vec<Card> {
        self.hand
            .discardables(remaining)
            .iter()
            .filter_map(|index| self.hand.get(*index).copied())
            .collect()
    }

    /// Handles this player may announce, empty when he has too few trumps.
    #[must_use]
    pub fn handle_options(&self) -> Vec<Handle> {
        match self.mode.handle(self.hand.count_trumps()) {
            None | Some(Handle::Refused) => Vec::new(),
            Some(Handle::Simple) => vec![Handle::Refused, Handle::Simple],
            Some(Handle::Double) => vec![Handle::Refused, Handle::Simple, Handle::Double],
            Some(Handle::Triple) => vec![
                Handle::Refused,
                Handle::Simple,
                Handle::Double,
                Handle::Triple,
            ],
        }
    }

    /// Trumps that would be shown for `handle`, before the player trims the extra ones.
    #[must_use]
    pub fn handle_trumps(&self, handle: Handle) -> Vec<Card> {
        let mut trumps: Vec<Card> = self.hand.trumps().into_iter().copied().collect();
        let fool_in_hand = trumps.iter().any(|card| card.is_fool());
        let real_trumps = trumps.len() - usize::from(fool_in_hand);
        // RULE: the fool can only be shown in a handle when there are not enough real trumps
        if real_trumps >= self.mode.handle_limit(&handle) {
            trumps.retain(|card| !card.is_fool());
        }
        trumps
    }

    /// Cards this player may legally play in `turn`.
    pub fn playable(&self, turn: &Turn) -> Result<Vec<Card>, TarotErrorKind> {
        let card_at = |index: usize| self.hand.get(index).copied();
        // RULE: the holder of the called card must play it as soon as its color is asked
        if let Some(Card::Normal(callee_normal)) = self.callee
            && let Some(Card::Normal(called_normal)) = turn.called()
            && callee_normal.suit() == called_normal.suit()
            && self.hand.has(&Card::Normal(callee_normal))
        {
            let mut forced = vec![Card::Normal(callee_normal)];
            if self.hand.has_fool() {
                forced.push(Card::Trump(Trump::Fool));
            }
            return Ok(forced);
        }

        let mut and_fool: Option<usize> = None;
        let mut trumps = Vec::new();
        let mut trumps_less = Vec::new();
        let mut trumps_more = Vec::new();
        let mut other_colors = Vec::new();
        let mut same_color = Vec::new();
        let mut compatibles: Vec<usize> = match (turn.called(), turn.master_card()) {
            (Some(Card::Normal(called_normal)), Some(Card::Normal(_))) => {
                for (i, card) in self.hand.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &Trump::Fool {
                                and_fool = Some(i);
                            } else {
                                trumps.push(i);
                            }
                        }
                        Card::Normal(card_normal) => {
                            if card_normal.suit() == called_normal.suit() {
                                same_color.push(i);
                            } else {
                                other_colors.push(i);
                            }
                        }
                    }
                }
                // RULE: follow the asked color, else cut, else anything
                if same_color.is_empty() {
                    if trumps.is_empty() { other_colors } else { trumps }
                } else {
                    same_color
                }
            }
            (Some(Card::Normal(called_normal)), Some(Card::Trump(master_trump_value))) => {
                for (i, card) in self.hand.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &Trump::Fool {
                                and_fool = Some(i);
                            } else if card_trump_value > master_trump_value {
                                trumps_more.push(i);
                            } else {
                                trumps_less.push(i);
                            }
                        }
                        Card::Normal(card_normal) => {
                            if card_normal.suit() == called_normal.suit() {
                                same_color.push(i);
                            } else {
                                other_colors.push(i);
                            }
                        }
                    }
                }
                // RULE: follow the asked color, else overcut, else undercut, else anything
                if !same_color.is_empty() {
                    same_color
                } else if !trumps_more.is_empty() {
                    trumps_more
                } else if trumps_less.is_empty() {
                    other_colors
                } else {
                    trumps_less
                }
            }
            (Some(Card::Trump(_)), Some(Card::Trump(master_trump_value))) => {
                for (i, card) in self.hand.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &Trump::Fool {
                                and_fool = Some(i);
                            } else if card_trump_value > master_trump_value {
                                trumps_more.push(i);
                            } else {
                                trumps_less.push(i);
                            }
                        }
                        Card::Normal(_) => other_colors.push(i),
                    }
                }
                // RULE: a trump was asked, overcut if possible, else any trump, else anything
                if !trumps_more.is_empty() {
                    trumps_more
                } else if trumps_less.is_empty() {
                    other_colors
                } else {
                    trumps_less
                }
            }
            (Some(Card::Normal(_)), None) => {
                return Err(TarotErrorKind::InvalidCase(
                    "There cannot be a called color and no master card".to_string(),
                ));
            }
            (Some(Card::Trump(_)), Some(Card::Normal(_))) => {
                return Err(TarotErrorKind::InvalidCase(
                    "There cannot be a called trump and a master color".to_string(),
                ));
            }
            (Some(Card::Trump(_)), None) => {
                return Err(TarotErrorKind::InvalidCase(
                    "There cannot be a called trump and no master".to_string(),
                ));
            }
            (None, Some(_)) => {
                return Err(TarotErrorKind::InvalidCase(
                    "There cannot be no called color and a master".to_string(),
                ));
            }
            // RULE: the called color cannot be led on the first turn,
            // except by playing the called card itself
            (None, None) => match (self.is_first_turn(), self.mode) {
                (true, Mode::Five) => self
                    .hand
                    .iter()
                    .enumerate()
                    .filter(|(_, card)| match (card, self.callee) {
                        (Card::Normal(normal), Some(Card::Normal(callee_normal))) => {
                            callee_normal.suit() != normal.suit()
                                || normal.value() == callee_normal.value()
                        }
                        (Card::Normal(_) | Card::Trump(_), _) => true,
                    })
                    .map(|(i, _)| i)
                    .collect(),
                (true | false, Mode::Three | Mode::Four) | (false, Mode::Five) => {
                    (0..self.hand.len()).collect()
                }
            },
        };
        // RULE: the fool may always be played
        if let Some(fool_index) = and_fool {
            compatibles.push(fool_index);
        }
        Ok(compatibles.into_iter().filter_map(card_at).collect())
    }
}
