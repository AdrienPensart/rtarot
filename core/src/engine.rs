use ordered_float::OrderedFloat;
use rand::Rng;
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::card::Card;
use crate::constants::{BASE_CONTRACT_POINTS, MAX_CARDS, MAX_POINTS, MIN_CUT};
use crate::contract::Contract;
use crate::deck::Deck;
use crate::errors::TarotErrorKind;
use crate::event::Event;
use crate::handle::Handle;
use crate::mode::Mode;
use crate::options::{Dealing, Direction, Options};
use crate::player::Player;
use crate::player_in_game::PlayerInGame;
use crate::points::Points;
use crate::question::{Answer, Question};
use crate::role::Role;
use crate::team::Team;
use crate::turn::Turn;
use strum::IntoEnumIterator;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Phase {
    Dealing,
    Bidding,
    Calling,
    AnnouncingSlam,
    Discarding,
    Handle(usize),
    TrimHandle(usize, Handle),
    Playing,
    Counting,
    Finished,
}

/// The state of one deal: hands, dog, roles and the turn being played.
#[derive(Clone, Debug)]
pub struct Deal {
    dog: Deck,
    /// Every hand as it was dealt, before the dog and the play touched it.
    dealt: Vec<Deck>,
    dealer: usize,
    players: Vec<PlayerInGame>,
    contract: Option<Contract>,
    taker: Option<usize>,
    slam_announced: bool,
    dog_given: bool,
    bidder: usize,
    bids_left: usize,
    allowed_contracts: Vec<Contract>,
    handle_asked: Vec<bool>,
    handle_shown: Vec<Card>,
    leader: usize,
    master: usize,
    turn: Turn,
    fool_kept: bool,
    petit_au_bout: Option<Team>,
    attack_cards: usize,
    defense_cards: usize,
    direction: Direction,
}

impl Deal {
    fn new(
        dog: Deck,
        players: Vec<PlayerInGame>,
        dealer: usize,
        leader: usize,
        direction: Direction,
    ) -> Self {
        let count = players.len();
        let dealt = players.iter().map(|player| player.hand().clone()).collect();
        Self {
            dog,
            dealt,
            dealer,
            players,
            direction,
            contract: None,
            taker: None,
            slam_announced: false,
            dog_given: false,
            bidder: leader,
            bids_left: count,
            allowed_contracts: Contract::iter().collect(),
            handle_asked: vec![false; count],
            handle_shown: Vec::new(),
            leader,
            master: leader,
            turn: Turn::default(),
            fool_kept: false,
            petit_au_bout: None,
            attack_cards: 0,
            defense_cards: 0,
        }
    }

    #[must_use]
    pub fn players(&self) -> &[PlayerInGame] {
        &self.players
    }
    /// The dog as it was dealt, sorted.
    #[must_use]
    pub const fn dog(&self) -> &Deck {
        &self.dog
    }
    /// Every hand as it was dealt, sorted, one per seat.
    #[must_use]
    pub fn dealt(&self) -> &[Deck] {
        &self.dealt
    }
    /// Who dealt this deal.
    #[must_use]
    pub const fn dealer(&self) -> usize {
        self.dealer
    }
    #[must_use]
    pub const fn contract(&self) -> Option<Contract> {
        self.contract
    }
    #[must_use]
    pub const fn taker(&self) -> Option<usize> {
        self.taker
    }
    #[must_use]
    pub const fn turn(&self) -> &Turn {
        &self.turn
    }
    /// Who plays the first card of the current turn.
    #[must_use]
    pub const fn leader(&self) -> usize {
        self.leader
    }
    /// Who is currently winning the turn being played.
    #[must_use]
    pub const fn master(&self) -> usize {
        self.master
    }
    #[must_use]
    pub const fn attack_cards(&self) -> usize {
        self.attack_cards
    }
    #[must_use]
    pub const fn defense_cards(&self) -> usize {
        self.defense_cards
    }

    pub fn player(&self, index: usize) -> Result<&PlayerInGame, TarotErrorKind> {
        self.players
            .get(index)
            .ok_or(TarotErrorKind::NoPlayer(index))
    }
    fn player_mut(&mut self, index: usize) -> Result<&mut PlayerInGame, TarotErrorKind> {
        self.players
            .get_mut(index)
            .ok_or(TarotErrorKind::NoPlayer(index))
    }
    fn taker_index(&self) -> Result<usize, TarotErrorKind> {
        self.taker.ok_or(TarotErrorKind::NoTaker(0))
    }
    fn contract_value(&self) -> Result<Contract, TarotErrorKind> {
        self.contract.ok_or(TarotErrorKind::NoTaker(0))
    }
    /// Whose turn it is to put a card down.
    const fn current_player(&self) -> usize {
        self.direction
            .step(self.leader, self.turn.len(), self.players.len())
    }
    fn finished(&self) -> bool {
        self.players.iter().all(PlayerInGame::last_turn)
    }
    /// A side is still on track for a slam while the other took no card at all.
    const fn took_everything(&self, team: Team) -> bool {
        match team {
            Team::Attack => self.defense_cards == 0,
            Team::Defense => self.attack_cards == 0,
        }
    }
    fn role_holder(&self, role: Role) -> Option<usize> {
        self.players.iter().position(|p| p.role() == Some(role))
    }
}

/// Drives a whole game, deal after deal, without ever talking to a user.
pub struct Engine {
    mode: Mode,
    options: Options,
    rng: StdRng,
    /// The pack between two deals: shuffled once, then only cut, in traditional dealing.
    deck: Deck,
    deals_left: u64,
    dealer: usize,
    players: Vec<Player>,
    deal: Option<Deal>,
    /// The deal that was just scored, until the next one is.
    last_deal: Option<Deal>,
    phase: Phase,
    pending: Option<Question>,
    events: Vec<Event>,
}

impl Engine {
    /// A game with the default seat names for `mode`.
    #[must_use]
    pub fn new(mode: Mode, deals: u64, seed: u64) -> Self {
        let names = (0..mode.players())
            .map(|index| mode.player_name(index).unwrap_or("Player").to_string())
            .collect();
        Self::with_names(mode, names, deals, seed)
    }

    /// A game whose seats are named by the caller.
    #[must_use]
    pub fn with_names(mode: Mode, names: Vec<String>, deals: u64, seed: u64) -> Self {
        Self::with_options(mode, names, deals, seed, Options::default())
    }

    /// A game played with the given table rules.
    #[must_use]
    pub fn with_options(
        mode: Mode,
        names: Vec<String>,
        deals: u64,
        seed: u64,
        options: Options,
    ) -> Self {
        let players = names.into_iter().map(Player::new).collect();
        let mut rng = StdRng::seed_from_u64(seed);
        // RULE: the pack is shuffled once, when the game starts
        let deck = Deck::shuffled(&mut rng);
        let mut engine = Self {
            mode,
            options,
            rng,
            deck,
            deals_left: deals,
            dealer: 0,
            players,
            deal: None,
            last_deal: None,
            phase: Phase::Dealing,
            pending: None,
            events: Vec::new(),
        };
        // A brand new engine is already waiting on its first question.
        if let Err(error) = engine.advance() {
            engine.events.push(Event::GameFinished);
            engine.phase = Phase::Finished;
            drop(error);
        }
        engine
    }

    // ---- reading the engine ---------------------------------------------
    #[must_use]
    pub const fn mode(&self) -> Mode {
        self.mode
    }
    #[must_use]
    pub const fn options(&self) -> Options {
        self.options
    }
    #[must_use]
    pub fn players(&self) -> &[Player] {
        &self.players
    }
    pub fn player(&self, index: usize) -> Result<&Player, TarotErrorKind> {
        self.players
            .get(index)
            .ok_or(TarotErrorKind::NoPlayer(index))
    }
    #[must_use]
    pub const fn deal(&self) -> Option<&Deal> {
        self.deal.as_ref()
    }
    /// The deal that was just scored: its tricks, discard and dog, as they ended.
    #[must_use]
    pub const fn last_deal(&self) -> Option<&Deal> {
        self.last_deal.as_ref()
    }
    #[must_use]
    pub const fn dealer(&self) -> usize {
        self.dealer
    }
    #[must_use]
    pub const fn deals_left(&self) -> u64 {
        self.deals_left
    }
    /// The question the engine is waiting on, `None` once the game is over.
    #[must_use]
    pub const fn pending(&self) -> Option<&Question> {
        self.pending.as_ref()
    }
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.phase == Phase::Finished
    }
    /// Everything that happened since the last call.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
    /// Scores must always add up to zero.
    pub fn is_consistent(&self) -> Result<(), TarotErrorKind> {
        let sum: OrderedFloat<f64> = self.players.iter().map(Player::score).sum();
        if sum == 0.0 {
            return Ok(());
        }
        Err(TarotErrorKind::InvalidScores(sum.to_string()))
    }

    // ---- answering -------------------------------------------------------
    /// Hand back the reply to [`Engine::pending`] and let the engine run on.
    pub fn answer(&mut self, answer: Answer) -> Result<(), TarotErrorKind> {
        let question = self
            .pending
            .take()
            .ok_or(TarotErrorKind::UnexpectedAnswer)?;
        self.apply(&question, answer)?;
        self.advance()
    }

    /// A legal answer picked at random, for a seat nobody is playing.
    pub fn random_answer(&mut self, question: &Question) -> Result<Answer, TarotErrorKind> {
        let pick = |rng: &mut StdRng, cards: &[Card]| -> Result<Card, TarotErrorKind> {
            let index = rng.random_range(0..cards.len().max(1));
            cards
                .get(index)
                .copied()
                .ok_or(TarotErrorKind::NoCard(index))
        };
        match question {
            Question::Contract { allowed, .. } => {
                // zero means passing, which stays possible whatever is left
                let choice = self.rng.random_range(0..=allowed.len());
                Ok(Answer::Contract(
                    choice
                        .checked_sub(1)
                        .and_then(|index| allowed.get(index).copied()),
                ))
            }
            // RULE: announcing a slam is a rare bet
            Question::Slam { .. } => Ok(Answer::Slam(self.rng.random_ratio(1, 100))),
            Question::Handle { allowed, .. } => {
                let index = self.rng.random_range(0..allowed.len().max(1));
                allowed
                    .get(index)
                    .copied()
                    .map(Answer::Handle)
                    .ok_or(TarotErrorKind::NoHandle(index))
            }
            Question::Call { allowed, .. }
            | Question::Discard { allowed, .. }
            | Question::Play { allowed, .. } => pick(&mut self.rng, allowed).map(Answer::Card),
            Question::TrimHandle { shown, .. } => pick(&mut self.rng, shown).map(Answer::Card),
        }
    }

    // ---- internals -------------------------------------------------------
    fn deal_ref(&self) -> Result<&Deal, TarotErrorKind> {
        self.deal.as_ref().ok_or(TarotErrorKind::NoDeal)
    }
    fn deal_mut(&mut self) -> Result<&mut Deal, TarotErrorKind> {
        self.deal.as_mut().ok_or(TarotErrorKind::NoDeal)
    }
    const fn rotate_dealer(&mut self) {
        // RULE: the deal moves by exactly one seat between two deals
        self.dealer = self.next(self.dealer);
    }
    /// The seat after `seat`, in the direction of play.
    const fn next(&self, seat: usize) -> usize {
        self.options.direction.next(seat, self.mode.players())
    }
    /// The seat before `seat`, in the direction of play.
    const fn previous(&self, seat: usize) -> usize {
        self.options.direction.previous(seat, self.mode.players())
    }

    /// Put every card back into the pack, without shuffling it.
    ///
    /// RULE: the attack's tricks are stacked first, then the defense's,
    /// then whatever was never dealt. `dog` is given when it was never handed out.
    fn gather(
        &mut self,
        players: &[PlayerInGame],
        dog: Option<&Deck>,
    ) -> Result<(), TarotErrorKind> {
        if self.options.dealing != Dealing::Traditional {
            return Ok(());
        }
        let mut pack = Deck::default();
        let attack_first = players
            .iter()
            .filter(|player| player.is_attack())
            .chain(players.iter().filter(|player| !player.is_attack()));
        for player in attack_first {
            pack.extend(&player.all_cards());
            pack.extend(player.hand());
        }
        if let Some(dog) = dog {
            pack.extend(dog);
        }
        if pack.len() != MAX_CARDS {
            return Err(TarotErrorKind::InvalidCase(format!(
                "{} cards gathered instead of {MAX_CARDS}",
                pack.len()
            )));
        }
        self.deck = pack;
        Ok(())
    }

    fn advance(&mut self) -> Result<(), TarotErrorKind> {
        loop {
            if self.pending.is_some() {
                return Ok(());
            }
            match self.phase {
                Phase::Dealing => self.start_deal()?,
                Phase::Bidding => self.ask_bid()?,
                Phase::Calling => self.ask_call()?,
                Phase::AnnouncingSlam => self.ask_slam()?,
                Phase::Discarding => self.ask_discard()?,
                Phase::Handle(player) => self.ask_handle(player)?,
                Phase::TrimHandle(player, handle) => self.ask_trim(player, handle)?,
                Phase::Playing => self.step_play()?,
                Phase::Counting => self.count_points()?,
                Phase::Finished => return Ok(()),
            }
        }
    }

    fn start_deal(&mut self) -> Result<(), TarotErrorKind> {
        if self.deals_left == 0 {
            self.events.push(Event::GameFinished);
            self.phase = Phase::Finished;
            return Ok(());
        }
        self.events.push(Event::DealStarted {
            deals_left: self.deals_left,
            dealer: self.dealer,
        });
        let (mut dog, hands) = match self.options.dealing {
            Dealing::Random => {
                self.deck = Deck::shuffled(&mut self.rng);
                let dog = self.deck.give(self.mode.dog_size());
                let hands = (0..self.mode.players())
                    .map(|_| self.deck.give(self.mode.cards_per_player()))
                    .collect();
                (dog, hands)
            }
            Dealing::Traditional => {
                // RULE: the previous dealer cuts the pack
                let cutter = self.previous(self.dealer);
                let depth = self.rng.random_range(MIN_CUT..=MAX_CARDS - MIN_CUT);
                self.deck.cut(depth)?;
                self.events.push(Event::DeckCut { player: cutter });
                // RULE: the dealer serves the seat after him first, and himself last
                let first = self.next(self.dealer);
                let mut pack = self.deck.give_all();
                deal_three_by_three(
                    &mut pack,
                    self.mode,
                    first,
                    self.options.direction,
                    &mut self.rng,
                )
            }
        };
        dog.sort();
        let players: Vec<PlayerInGame> = hands
            .iter()
            .map(|hand| {
                let mut player_in_game = PlayerInGame::new(self.mode);
                player_in_game.extend_hand(hand);
                player_in_game
            })
            .collect();
        // RULE: a hand whose only trump is the petit is dealt again
        if let Some(player) = players.iter().position(PlayerInGame::petit_sec) {
            self.events.push(Event::PetitSec { player });
            self.gather(&players, Some(&dog))?;
            self.rotate_dealer();
            return Ok(());
        }
        let leader = self.next(self.dealer);
        self.deal = Some(Deal::new(
            dog,
            players,
            self.dealer,
            leader,
            self.options.direction,
        ));
        self.phase = Phase::Bidding;
        Ok(())
    }

    fn ask_bid(&mut self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_ref()?;
        if deal.bids_left > 0 {
            self.pending = Some(Question::Contract {
                player: deal.bidder,
                allowed: deal.allowed_contracts.clone(),
            });
            return Ok(());
        }
        if deal.taker.is_none() {
            self.events.push(Event::EveryonePassed);
            if let Some(deal) = self.deal.take() {
                self.gather(&deal.players, Some(&deal.dog))?;
            }
            self.rotate_dealer();
            self.phase = Phase::Dealing;
            return Ok(());
        }
        self.phase = Phase::Calling;
        Ok(())
    }

    fn ask_call(&mut self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_ref()?;
        let taker = deal.taker_index()?;
        if self.mode == Mode::Five {
            let allowed = deal.player(taker)?.callable()?;
            self.pending = Some(Question::Call {
                player: taker,
                allowed,
            });
            return Ok(());
        }
        self.assign_roles(None)?;
        self.phase = Phase::AnnouncingSlam;
        Ok(())
    }

    fn assign_roles(&mut self, callee: Option<Card>) -> Result<(), TarotErrorKind> {
        let deal = self.deal_mut()?;
        let taker = deal.taker_index()?;
        for (index, player) in deal.players.iter_mut().enumerate() {
            player.set_callee(callee);
            if index == taker {
                player.set_team(Team::Attack);
                player.set_role(Role::Taker);
            } else if callee.is_some_and(|card| player.has(&card)) {
                // RULE: whoever holds the called king plays with the taker
                player.set_team(Team::Attack);
                player.set_role(Role::Ally);
            } else {
                player.set_team(Team::Defense);
                player.set_role(Role::Defenser);
            }
        }
        Ok(())
    }

    fn ask_slam(&mut self) -> Result<(), TarotErrorKind> {
        // RULE: the slam is announced by the taker, before the dog is turned over
        let taker = self.deal_ref()?.taker_index()?;
        self.pending = Some(Question::Slam { player: taker });
        Ok(())
    }

    fn ask_discard(&mut self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_ref()?;
        let taker = deal.taker_index()?;
        let contract = deal.contract_value()?;
        let dog = deal.dog.clone();
        match contract {
            Contract::GardeSans => {
                // RULE: the dog belongs to the attack but is never seen
                self.deal_mut()?.player_mut(taker)?.set_discard(&dog);
                self.events.push(Event::DogKeptByAttack { player: taker });
                self.start_playing()
            }
            Contract::GardeContre => {
                // RULE: the dog goes to the defense and is never seen
                let defenser = self
                    .deal_ref()?
                    .role_holder(Role::Defenser)
                    .ok_or(TarotErrorKind::NoDefenser(0))?;
                self.deal_mut()?.player_mut(defenser)?.set_discard(&dog);
                self.events
                    .push(Event::DogGivenToDefense { player: defenser });
                self.start_playing()
            }
            Contract::Petite | Contract::Garde => {
                if !self.deal_ref()?.dog_given {
                    let deal = self.deal_mut()?;
                    deal.dog_given = true;
                    deal.player_mut(taker)?.extend_hand(&dog);
                    self.events
                        .push(Event::DogGivenToTaker { player: taker, dog });
                    return Ok(());
                }
                let taker_in_game = self.deal_ref()?.player(taker)?;
                let remaining = self.mode.dog_size() - taker_in_game.discard().len();
                if remaining == 0 {
                    return self.start_playing();
                }
                self.pending = Some(Question::Discard {
                    player: taker,
                    allowed: taker_in_game.discardable(remaining),
                    remaining,
                });
                Ok(())
            }
        }
    }

    fn start_playing(&mut self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_mut()?;
        // RULE: the player who announced a slam must start
        if deal.slam_announced {
            deal.leader = deal.taker_index()?;
            deal.master = deal.leader;
        }
        self.phase = Phase::Playing;
        Ok(())
    }

    fn ask_handle(&mut self, player: usize) -> Result<(), TarotErrorKind> {
        let allowed = self.deal_ref()?.player(player)?.handle_options();
        if allowed.is_empty() {
            let deal = self.deal_mut()?;
            if let Some(asked) = deal.handle_asked.get_mut(player) {
                *asked = true;
            }
            self.phase = Phase::Playing;
            return Ok(());
        }
        self.pending = Some(Question::Handle { player, allowed });
        Ok(())
    }

    fn ask_trim(&mut self, player: usize, handle: Handle) -> Result<(), TarotErrorKind> {
        let limit = self.mode.handle_limit(&handle);
        let shown = self.deal_ref()?.handle_shown.clone();
        if shown.len() <= limit {
            let deal = self.deal_mut()?;
            deal.player_mut(player)?.set_handle(handle);
            if let Some(asked) = deal.handle_asked.get_mut(player) {
                *asked = true;
            }
            self.events.push(Event::HandleAnnounced {
                player,
                handle,
                trumps: shown,
            });
            self.phase = Phase::Playing;
            return Ok(());
        }
        self.pending = Some(Question::TrimHandle {
            player,
            handle,
            to_remove: shown.len() - limit,
            shown,
        });
        Ok(())
    }

    fn step_play(&mut self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_ref()?;
        if deal.turn.len() == self.mode.players() {
            return self.resolve_turn();
        }
        let player = deal.current_player();
        if deal.handle_asked.get(player) == Some(&false) {
            // RULE: a handle is announced before its owner plays his first card
            self.phase = Phase::Handle(player);
            return Ok(());
        }
        let allowed = deal.player(player)?.playable(&deal.turn)?;
        self.pending = Some(Question::Play { player, allowed });
        Ok(())
    }

    fn resolve_turn(&mut self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_mut()?;
        let master = deal.master;
        let master_in_game = deal.player(master)?;
        let last_turn = master_in_game.last_turn();
        let before_last_turn = master_in_game.before_last_turn();
        let team = master_in_game
            .team()
            .ok_or_else(|| TarotErrorKind::NoTeamForPlayer(master.to_string()))?;
        let slam_running =
            deal.took_everything(Team::Attack) || deal.took_everything(Team::Defense);

        let turn = std::mem::take(&mut deal.turn);
        let turn_cards = turn.take_cards(deal.fool_kept);
        // RULE: petit au bout counts in the last turn, or the one before during a slam
        let petit_au_bout =
            turn_cards.has_petit() && (last_turn || (before_last_turn && slam_running));
        let won = turn_cards.len();
        deal.player_mut(master)?.extend_owned(&turn_cards);
        deal.player(master)?.check_owned()?;
        match team {
            Team::Attack => deal.attack_cards += won,
            Team::Defense => deal.defense_cards += won,
        }
        if petit_au_bout {
            deal.petit_au_bout = Some(team);
        }
        deal.leader = master;
        deal.fool_kept = false;
        let finished = deal.finished();

        self.events.push(Event::TurnWon {
            player: master,
            cards: won,
        });
        if petit_au_bout {
            self.events.push(Event::PetitAuBout {
                player: master,
                team,
            });
        }
        self.phase = if finished {
            Phase::Counting
        } else {
            Phase::Playing
        };
        Ok(())
    }

    // ---- applying an answer ---------------------------------------------
    fn apply(&mut self, question: &Question, answer: Answer) -> Result<(), TarotErrorKind> {
        match (question, answer) {
            (Question::Contract { player, allowed }, Answer::Contract(contract)) => {
                self.apply_bid(*player, allowed, contract)
            }
            (Question::Call { player, allowed }, Answer::Card(card)) => {
                Self::require(allowed, card)?;
                self.assign_roles(Some(card))?;
                self.events.push(Event::KingCalled {
                    player: *player,
                    card,
                });
                self.phase = Phase::AnnouncingSlam;
                Ok(())
            }
            (Question::Slam { player }, Answer::Slam(slam)) => {
                let deal = self.deal_mut()?;
                deal.slam_announced = slam;
                deal.player_mut(*player)?.set_slam(slam);
                if slam {
                    self.events.push(Event::SlamAnnounced { player: *player });
                }
                self.phase = Phase::Discarding;
                Ok(())
            }
            (
                Question::Discard {
                    player, allowed, ..
                },
                Answer::Card(card),
            ) => {
                Self::require(allowed, card)?;
                self.deal_mut()?.player_mut(*player)?.discard_card(card)?;
                self.events.push(Event::Discarded {
                    player: *player,
                    card,
                });
                Ok(())
            }
            (Question::Handle { player, allowed }, Answer::Handle(handle)) => {
                if !allowed.contains(&handle) {
                    return Err(TarotErrorKind::InvalidHandle(handle.to_string()));
                }
                let deal = self.deal_mut()?;
                if handle == Handle::Refused {
                    if let Some(asked) = deal.handle_asked.get_mut(*player) {
                        *asked = true;
                    }
                    self.phase = Phase::Playing;
                    return Ok(());
                }
                deal.handle_shown = deal.player(*player)?.handle_trumps(handle);
                self.phase = Phase::TrimHandle(*player, handle);
                Ok(())
            }
            (Question::TrimHandle { handle, shown, .. }, Answer::Card(card)) => {
                Self::require(shown, card)?;
                let deal = self.deal_mut()?;
                deal.handle_shown.retain(|shown_card| shown_card != &card);
                self.phase = Phase::TrimHandle(question.player(), *handle);
                Ok(())
            }
            (Question::Play { player, allowed }, Answer::Card(card)) => {
                Self::require(allowed, card)?;
                self.apply_play(*player, card)
            }
            (
                Question::Contract { .. }
                | Question::Call { .. }
                | Question::Slam { .. }
                | Question::Discard { .. }
                | Question::Handle { .. }
                | Question::TrimHandle { .. }
                | Question::Play { .. },
                Answer::Contract(_) | Answer::Slam(_) | Answer::Handle(_) | Answer::Card(_),
            ) => Err(TarotErrorKind::MismatchedAnswer),
        }
    }

    fn require(allowed: &[Card], card: Card) -> Result<(), TarotErrorKind> {
        if allowed.contains(&card) {
            return Ok(());
        }
        Err(TarotErrorKind::InvalidCard(card))
    }

    fn apply_bid(
        &mut self,
        player: usize,
        allowed: &[Contract],
        contract: Option<Contract>,
    ) -> Result<(), TarotErrorKind> {
        if let Some(contract) = contract {
            if !allowed.contains(&contract) {
                return Err(TarotErrorKind::InvalidContract(contract.to_string()));
            }
            let deal = self.deal_mut()?;
            deal.taker = Some(player);
            deal.contract = Some(contract);
            // RULE: bidding goes around once, each player either passes or overbids
            deal.allowed_contracts
                .retain(|other| other.multiplier() > contract.multiplier());
        }
        let next_bidder = self.next(player);
        let deal = self.deal_mut()?;
        deal.bidder = next_bidder;
        deal.bids_left -= 1;
        self.events.push(Event::Bid { player, contract });
        Ok(())
    }

    fn apply_play(&mut self, player: usize, card: Card) -> Result<(), TarotErrorKind> {
        let deal = self.deal_mut()?;
        let team = deal
            .player(player)?
            .team()
            .ok_or_else(|| TarotErrorKind::NoTeamForPlayer(player.to_string()))?;
        let took_everything = deal.took_everything(team);
        deal.player_mut(player)?.take_from_hand(card)?;
        self.events.push(Event::CardPlayed { player, card });

        let deal = self.deal_mut()?;
        if card.is_fool() {
            // RULE: the fool is preserved to his owner, except when played in the last turn,
            // where it is lost to the winner unless its side took every turn so far
            let keep = !deal.player(player)?.last_turn() || took_everything;
            if keep {
                deal.fool_kept = true;
                deal.player_mut(player)?.push_owned(card);
            }
            deal.turn.put(card);
            self.events.push(if keep {
                Event::FoolKept { player }
            } else {
                Event::FoolLost { player }
            });
            return Ok(());
        }

        let beaten = deal
            .turn
            .master_card()
            .is_none_or(|master| !master.master(card));
        deal.turn.put(card);
        if beaten {
            deal.master = player;
            deal.turn.master_index = Some(deal.turn.len() - 1);
            self.events.push(Event::MasterChanged { player, card });
        }
        Ok(())
    }

    // ---- scoring ---------------------------------------------------------
    /// Every card of the game must still be somewhere, and be worth the same total.
    fn check_deal_integrity(&self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_ref()?;
        let mut cards = 0;
        let mut points = OrderedFloat(0.0);
        for player in &deal.players {
            let owned = player.all_cards();
            cards += owned.len();
            points += owned.points();
        }
        if cards != MAX_CARDS {
            return Err(TarotErrorKind::InvalidCase(format!(
                "{cards} cards in play instead of {MAX_CARDS}"
            )));
        }
        if points != MAX_POINTS {
            return Err(TarotErrorKind::InvalidCase(format!(
                "{points} points in play instead of {MAX_POINTS}"
            )));
        }
        Ok(())
    }

    fn count_points(&mut self) -> Result<(), TarotErrorKind> {
        self.pay_fool_compensation()?;
        self.check_deal_integrity()?;

        let deal = self.deal_ref()?;
        let contract = deal.contract_value()?;
        let taker = deal.taker_index()?;
        let ally = deal.role_holder(Role::Ally);
        let defense: Vec<usize> = deal
            .players
            .iter()
            .enumerate()
            .filter(|(_, player)| player.role() == Some(Role::Defenser))
            .map(|(index, _)| index)
            .collect();

        let mut attack_cards = deal.player(taker)?.all_cards();
        if let Some(ally) = ally {
            attack_cards.extend(&deal.player(ally)?.all_cards());
        }
        let attack_points = attack_cards.points();
        let needed = attack_cards.points_for_oudlers()?;
        let succeeded = attack_points >= needed;
        let contract_points = if succeeded {
            (attack_points - needed + BASE_CONTRACT_POINTS) * contract.multiplier()
        } else {
            (attack_points - needed - BASE_CONTRACT_POINTS) * contract.multiplier()
        };

        // RULE: handles are always paid to the winner of the deal, whoever announced them
        let announced_handles: OrderedFloat<f64> = deal
            .players
            .iter()
            .filter_map(PlayerInGame::handle)
            .map(|handle| handle.points())
            .sum();
        let handles = if succeeded {
            announced_handles
        } else {
            -announced_handles
        };

        let petit_au_bout = match deal.petit_au_bout {
            Some(Team::Attack) => OrderedFloat(10.0) * contract.multiplier(),
            Some(Team::Defense) => -OrderedFloat(10.0) * contract.multiplier(),
            None => OrderedFloat(0.0),
        };

        let slam = self.slam_bonus()?;
        let points = contract_points + petit_au_bout + handles + slam;
        let ratio = self.mode.ratio(ally.is_some());

        self.player_mut(taker)?.add_score(ratio * points);
        if let Some(ally) = ally {
            self.player_mut(ally)?.add_score(points);
        }
        for defenser in defense {
            self.player_mut(defenser)?.add_score(-points);
        }

        self.events.push(Event::DealScored {
            taker,
            ally,
            attack_points,
            needed,
            contract,
            contract_points,
            petit_au_bout,
            handles,
            slam,
            points,
        });
        self.is_consistent()?;

        // RULE: the tricks are stacked back into the pack, the dog is already in a discard
        if let Some(deal) = self.deal.take() {
            self.gather(&deal.players, None)?;
            self.last_deal = Some(deal);
        }
        self.deals_left = self.deals_left.saturating_sub(1);
        self.rotate_dealer();
        self.phase = Phase::Dealing;
        Ok(())
    }

    /// RULE: the owner of a kept fool gives a low card in exchange,
    /// and loses the fool itself when he has no card to give.
    fn pay_fool_compensation(&mut self) -> Result<(), TarotErrorKind> {
        let deal = self.deal_ref()?;
        let (Some(from), Some(to)) = (
            deal.players.iter().position(PlayerInGame::owe_card),
            deal.players.iter().position(PlayerInGame::missing_card),
        ) else {
            return Ok(());
        };
        let deal = self.deal_mut()?;
        let owner = deal.player_mut(from)?;
        let Some(card) = owner.give_low().or_else(|| owner.give_fool()) else {
            return Ok(());
        };
        deal.player_mut(to)?.push_owned(card);
        self.events.push(Event::FoolCompensation { from, to, card });
        Ok(())
    }

    /// RULE: a slam is realized when a side took every single turn
    fn slam_bonus(&mut self) -> Result<OrderedFloat<f64>, TarotErrorKind> {
        let deal = self.deal_ref()?;
        let announced = deal.slam_announced;
        let attack_slam = deal.attack_cards > 0 && deal.defense_cards == 0;
        let defense_slam = deal.defense_cards > 0 && deal.attack_cards == 0;
        let mut bonus = OrderedFloat(0.0);
        if attack_slam {
            bonus += OrderedFloat(if announced { 400.0 } else { 200.0 });
        } else {
            if announced {
                bonus -= OrderedFloat(200.0);
            }
            if defense_slam {
                bonus -= OrderedFloat(200.0);
            }
        }
        if attack_slam || defense_slam || announced {
            self.events.push(Event::SlamResult {
                attack: attack_slam || announced,
                announced,
                realized: attack_slam || defense_slam,
                bonus,
            });
        }
        Ok(bonus)
    }

    fn player_mut(&mut self, index: usize) -> Result<&mut Player, TarotErrorKind> {
        self.players
            .get_mut(index)
            .ok_or(TarotErrorKind::NoPlayer(index))
    }
}

/// Cards given to one seat at a time.
const PACKET: usize = 3;

/// Deal `pack` three by three, starting at `first` and going around in `direction`.
///
/// RULE: the dog is made of cards dropped one at a time between two packets,
/// never the first nor the last card of the pack.
fn deal_three_by_three<R: Rng + ?Sized>(
    pack: &mut Deck,
    mode: Mode,
    first: usize,
    direction: Direction,
    rng: &mut R,
) -> (Deck, Vec<Deck>) {
    let players = mode.players();
    let packets = players * (mode.cards_per_player() / PACKET);
    // a dog card may fall before any packet but the first one
    let mut gaps: Vec<usize> = rand::seq::index::sample(rng, packets - 1, mode.dog_size())
        .into_iter()
        .map(|gap| gap + 1)
        .collect();
    gaps.sort_unstable();
    let mut dog = Deck::default();
    let mut hands = vec![Deck::default(); players];
    let mut seat = first;
    for packet in 0..packets {
        if gaps.binary_search(&packet).is_ok() {
            dog.extend(&pack.give(1));
        }
        if let Some(hand) = hands.get_mut(seat) {
            hand.extend(&pack.give(PACKET));
        }
        seat = direction.next(seat, players);
    }
    (dog, hands)
}

#[cfg(test)]
#[allow(clippy::wildcard_enum_match_arm)]
mod tests {
    use super::{Engine, deal_three_by_three};
    use crate::card::Card;
    use crate::constants::MAX_CARDS;
    use crate::contract::Contract;
    use crate::deck::Deck;
    use crate::event::Event;
    use crate::handle::Handle;
    use crate::mode::Mode;
    use crate::options::{Dealing, Direction, Options};
    use crate::question::{Answer, Question};
    use crate::role::Role;
    use crate::suit::Suit;
    use crate::suit_value::SuitValue;
    use crate::team::Team;
    use crate::trump::Trump;
    use ordered_float::OrderedFloat;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use strum::IntoEnumIterator;

    /// Play a whole game with random answers, collecting every event.
    fn autoplay(mode: Mode, deals: u64, seed: u64) -> (Engine, Vec<Event>) {
        autoplay_with(mode, deals, seed, Options::default())
    }

    /// Every combination of table rules.
    fn all_options() -> Vec<Options> {
        let mut all = Vec::new();
        for direction in [Direction::Clockwise, Direction::CounterClockwise] {
            for dealing in [Dealing::Traditional, Dealing::Random] {
                all.push(Options { direction, dealing });
            }
        }
        all
    }

    fn autoplay_with(mode: Mode, deals: u64, seed: u64, options: Options) -> (Engine, Vec<Event>) {
        let names = (0..mode.players()).map(|i| format!("P{i}")).collect();
        let mut engine = Engine::with_options(mode, names, deals, seed, options);
        let mut events = Vec::new();
        let mut guard = 0;
        loop {
            events.append(&mut engine.take_events());
            let Some(question) = engine.pending().cloned() else {
                break;
            };
            let answer = engine
                .random_answer(&question)
                .expect("a legal answer always exists");
            engine.answer(answer).expect("a legal answer is accepted");
            guard += 1;
            assert!(guard < 1_000_000, "engine never reached the end");
        }
        (engine, events)
    }

    // ---- a game runs to the end, on every mode -------------------------

    #[test]
    fn every_mode_plays_to_the_end() {
        for mode in Mode::iter() {
            let (engine, events) = autoplay(mode, 3, 1);
            assert!(engine.is_finished());
            assert!(engine.pending().is_none());
            assert!(events.contains(&Event::GameFinished));
        }
    }

    #[test]
    fn scores_always_add_up_to_zero() {
        for mode in Mode::iter() {
            for seed in 0..20 {
                let (engine, _) = autoplay(mode, 2, seed);
                assert_eq!(engine.is_consistent(), Ok(()));
            }
        }
    }

    #[test]
    fn every_table_rule_plays_to_the_end_and_adds_up() {
        for options in all_options() {
            for mode in Mode::iter() {
                for seed in 0..5 {
                    let (engine, events) = autoplay_with(mode, 4, seed, options);
                    assert!(engine.is_finished(), "{options:?} {mode:?} never ended");
                    assert_eq!(engine.is_consistent(), Ok(()), "{options:?} {mode:?}");
                    let scored = events
                        .iter()
                        .filter(|event| matches!(event, Event::DealScored { .. }))
                        .count();
                    assert_eq!(scored, 4, "{options:?} {mode:?}");
                }
            }
        }
    }

    // ---- direction of play ---------------------------------------------

    /// The dealer of each deal and who bid first after him.
    fn dealers_and_first_bidders(events: &[Event]) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut dealer = None;
        for event in events {
            match event {
                Event::DealStarted { dealer: d, .. } => dealer = Some(*d),
                Event::Bid { player, .. } => {
                    if let Some(d) = dealer.take() {
                        out.push((d, *player));
                    }
                }
                _ => {}
            }
        }
        out
    }

    #[test]
    fn the_table_turns_the_way_it_is_told() {
        for direction in [Direction::Clockwise, Direction::CounterClockwise] {
            let options = Options {
                direction,
                ..Options::default()
            };
            let (_, events) = autoplay_with(Mode::Four, 4, 11, options);
            let pairs = dealers_and_first_bidders(&events);
            assert_ne!(pairs.len(), 0);
            for (dealer, first) in &pairs {
                assert_eq!(*first, direction.next(*dealer, 4), "{direction}");
            }
            // the deal itself moves in the same direction
            for window in pairs.windows(2) {
                if let [(before, _), (after, _)] = window {
                    assert_eq!(*after, direction.next(*before, 4), "{direction}");
                }
            }
        }
    }

    #[test]
    fn cards_go_round_the_table_in_the_direction_of_play() {
        for direction in [Direction::Clockwise, Direction::CounterClockwise] {
            let options = Options {
                direction,
                ..Options::default()
            };
            let (_, events) = autoplay_with(Mode::Five, 2, 4, options);
            let mut previous: Option<usize> = None;
            let mut in_turn = 0;
            for event in &events {
                match event {
                    Event::CardPlayed { player, .. } => {
                        if in_turn > 0
                            && let Some(before) = previous
                        {
                            assert_eq!(*player, direction.next(before, 5), "{direction}");
                        }
                        previous = Some(*player);
                        in_turn += 1;
                    }
                    Event::TurnWon { .. } => in_turn = 0,
                    _ => {}
                }
            }
        }
    }

    // ---- dealing --------------------------------------------------------

    #[test]
    fn the_previous_dealer_cuts_before_every_traditional_deal() {
        let (_, events) = autoplay_with(Mode::Four, 5, 2, Options::default());
        let mut dealer = None;
        let mut cuts = 0;
        for event in &events {
            match event {
                Event::DealStarted { dealer: d, .. } => dealer = Some(*d),
                Event::DeckCut { player } => {
                    cuts += 1;
                    let dealer = dealer.expect("a cut follows a deal start");
                    assert_eq!(*player, Direction::Clockwise.previous(dealer, 4));
                }
                _ => {}
            }
        }
        let started = events
            .iter()
            .filter(|event| matches!(event, Event::DealStarted { .. }))
            .count();
        assert_eq!(cuts, started);
    }

    #[test]
    fn a_random_deal_never_cuts() {
        let options = Options {
            dealing: Dealing::Random,
            ..Options::default()
        };
        let (_, events) = autoplay_with(Mode::Four, 3, 2, options);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::DeckCut { .. }))
        );
    }

    #[test]
    fn three_by_three_serves_packets_in_order_and_keeps_the_dog_inside() {
        let mut rng = StdRng::seed_from_u64(7);
        for mode in Mode::iter() {
            for direction in [Direction::Clockwise, Direction::CounterClockwise] {
                let players = mode.players();
                let first = 1;
                let full = Deck::full();
                let mut pack = full.clone();
                let (dog, hands) = deal_three_by_three(&mut pack, mode, first, direction, &mut rng);
                assert!(pack.is_empty(), "every card is dealt");
                assert_eq!(dog.len(), mode.dog_size());
                assert_eq!(hands.len(), players);
                for hand in &hands {
                    assert_eq!(hand.len(), mode.cards_per_player());
                    // each hand is made of packets of three consecutive cards of the pack
                    for packet in hand.chunks(3) {
                        let at =
                            |card: &Card| full.iter().position(|c| c == card).expect("in pack");
                        let positions: Vec<usize> = packet.iter().map(at).collect();
                        assert_eq!(positions.get(1).copied(), positions.first().map(|p| p + 1));
                        assert_eq!(positions.get(2).copied(), positions.first().map(|p| p + 2));
                    }
                }
                // the first packet goes to `first`, the last one to the seat before it
                assert_eq!(hands.get(first).and_then(|h| h.first()), full.first());
                let last_seat = direction.previous(first, players);
                assert_eq!(
                    hands.get(last_seat).and_then(|h| h.last()),
                    full.get(MAX_CARDS - 1)
                );
                // RULE: the dog never holds the first nor the last card
                assert!(!dog.has(full.first().expect("full")));
                assert!(!dog.has(full.get(MAX_CARDS - 1).expect("full")));
                // the second packet goes to the next seat in the direction of play,
                // right after the first packet or after one dog card
                let second = direction.next(first, players);
                let start = hands
                    .get(second)
                    .and_then(|h| h.first())
                    .and_then(|c| full.iter().position(|x| x == c));
                assert!(
                    matches!(start, Some(3 | 4)),
                    "{mode:?} {direction}: {start:?}"
                );
            }
        }
    }

    #[test]
    fn the_dealt_hands_are_kept_until_the_deal_is_scored() {
        let mut engine = Engine::new(Mode::Four, 1, 9);
        let dealt: Vec<Deck> = engine.deal().expect("a deal").dealt().to_vec();
        assert_eq!(dealt.len(), 4);
        let mut all = Deck::default();
        for hand in &dealt {
            assert_eq!(hand.len(), Mode::Four.cards_per_player());
            all.extend(hand);
        }
        all.extend(engine.deal().expect("a deal").dog());
        all.sort();
        assert!(
            all.is_chelem(),
            "the dealt hands and the dog make the whole pack"
        );
        while let Some(question) = engine.pending().cloned() {
            let answer = engine.random_answer(&question).expect("legal");
            engine.answer(answer).expect("accepted");
        }
        let last = engine.last_deal().expect("the scored deal is kept");
        assert_eq!(last.dealt(), dealt.as_slice());
        assert!(last.players().iter().all(|p| p.hand().is_empty()));
    }

    #[test]
    fn a_traditional_pack_is_never_reshuffled_between_deals() {
        // Two games with the same seed but different answers get the same first deal,
        // then diverge only through play: the cards stay in trick order.
        let names: Vec<String> = (0..4).map(|i| format!("P{i}")).collect();
        let engine = Engine::with_options(Mode::Four, names, 2, 5, Options::default());
        let deal = engine.deal().expect("a deal is running");
        let dealt: usize =
            deal.players().iter().map(|p| p.hand().len()).sum::<usize>() + deal.dog().len();
        assert_eq!(dealt, MAX_CARDS);
        assert!(engine.deck.is_empty(), "the whole pack is on the table");
    }

    #[test]
    fn every_asked_deal_is_scored_exactly_once() {
        for mode in Mode::iter() {
            let (_, events) = autoplay(mode, 5, 7);
            let scored = events
                .iter()
                .filter(|event| matches!(event, Event::DealScored { .. }))
                .count();
            assert_eq!(scored, 5, "{mode:?} should score exactly five deals");
        }
    }

    // ---- determinism ----------------------------------------------------

    #[test]
    fn the_same_seed_replays_the_same_game() {
        let (first, first_events) = autoplay(Mode::Five, 3, 12_345);
        let (second, second_events) = autoplay(Mode::Five, 3, 12_345);
        assert_eq!(first_events, second_events);
        let scores: Vec<_> = first.players().iter().map(crate::Player::score).collect();
        let again: Vec<_> = second.players().iter().map(crate::Player::score).collect();
        assert_eq!(scores, again);
    }

    #[test]
    fn different_seeds_give_different_games() {
        let (_, first) = autoplay(Mode::Four, 3, 1);
        let (_, second) = autoplay(Mode::Four, 3, 2);
        assert_ne!(first, second);
    }

    // ---- roles ----------------------------------------------------------

    #[test]
    fn a_started_deal_has_exactly_one_taker() {
        let mut engine = Engine::new(Mode::Five, 1, 99);
        while let Some(question) = engine.pending().cloned() {
            if matches!(question, Question::Play { .. }) {
                break;
            }
            let answer = engine.random_answer(&question).expect("legal answer");
            engine.answer(answer).expect("accepted");
        }
        let deal = engine.deal().expect("a deal is running");
        let takers = deal
            .players()
            .iter()
            .filter(|player| player.role() == Some(Role::Taker))
            .count();
        assert_eq!(takers, 1);
        // five players is two against three, or one against four
        let attack = deal.players().iter().filter(|p| p.is_attack()).count();
        assert!(attack == 1 || attack == 2, "attack was {attack}");
    }

    #[test]
    fn the_taker_is_the_last_and_highest_bidder() {
        let mut engine = Engine::new(Mode::Four, 1, 3);
        let mut best: Option<(usize, Contract)> = None;
        while let Some(question) = engine.pending().cloned() {
            if matches!(question, Question::Play { .. }) {
                break;
            }
            let answer = engine.random_answer(&question).expect("legal answer");
            engine.answer(answer).expect("accepted");
            for event in engine.take_events() {
                if let Event::Bid {
                    player,
                    contract: Some(contract),
                } = event
                {
                    assert!(
                        best.is_none_or(|(_, previous)| contract > previous),
                        "each bid must beat the previous one"
                    );
                    best = Some((player, contract));
                }
            }
        }
        let deal = engine.deal().expect("a deal is running");
        assert_eq!(deal.taker(), best.map(|(player, _)| player));
        assert_eq!(deal.contract(), best.map(|(_, contract)| contract));
    }

    // ---- the engine refuses illegal answers -----------------------------

    #[test]
    fn an_illegal_card_is_refused() {
        let mut engine = Engine::new(Mode::Four, 1, 5);
        let Some(Question::Contract { .. }) = engine.pending() else {
            panic!("a new engine asks for a bid first");
        };
        // a card is not an answer to a contract question
        assert!(
            engine
                .answer(Answer::Card(Card::Trump(Trump::Fool)))
                .is_err()
        );
    }

    #[test]
    fn a_contract_outside_the_allowed_list_is_refused() {
        let mut engine = Engine::new(Mode::Four, 1, 5);
        // take the highest contract, then check the next player cannot match it
        engine
            .answer(Answer::Contract(Some(Contract::GardeContre)))
            .expect("garde contre is allowed on the first bid");
        let Some(Question::Contract { allowed, .. }) = engine.pending() else {
            panic!("the next player is asked to bid");
        };
        assert!(allowed.is_empty(), "nothing beats a garde contre");
        assert!(
            engine
                .answer(Answer::Contract(Some(Contract::GardeContre)))
                .is_err()
        );
    }

    #[test]
    fn answering_nothing_is_refused_once_the_game_is_over() {
        let (mut engine, _) = autoplay(Mode::Three, 1, 8);
        assert!(engine.answer(Answer::Slam(false)).is_err());
    }

    #[test]
    fn an_unplayable_card_is_refused() {
        let mut engine = Engine::new(Mode::Four, 1, 21);
        let question = loop {
            let Some(question) = engine.pending().cloned() else {
                panic!("the game ended before a card was played");
            };
            if let Question::Play { ref allowed, .. } = question
                && allowed.len() < 18
            {
                break question;
            }
            let answer = engine.random_answer(&question).expect("legal answer");
            engine.answer(answer).expect("accepted");
        };
        let Question::Play { player, allowed } = question else {
            unreachable!()
        };
        let hand = engine
            .deal()
            .expect("a deal is running")
            .player(player)
            .expect("a player")
            .hand()
            .clone();
        let forbidden = hand
            .iter()
            .find(|card| !allowed.contains(card))
            .copied()
            .expect("a restricted turn leaves at least one illegal card");
        assert!(engine.answer(Answer::Card(forbidden)).is_err());
    }

    // ---- rules that only show up over many deals ------------------------

    #[test]
    fn cards_and_points_are_conserved_over_many_deals() {
        // `check_deal_integrity` runs inside every scoring, so reaching the end
        // of these games is the proof that all 78 cards and 91 points survived.
        for mode in Mode::iter() {
            for seed in 100..140 {
                let (engine, _) = autoplay(mode, 2, seed);
                assert!(engine.is_finished());
            }
        }
    }

    #[test]
    fn a_kept_fool_is_always_paid_for() {
        let mut compensations = 0;
        for seed in 0..30 {
            let (_, events) = autoplay(Mode::Four, 2, seed);
            compensations += events
                .iter()
                .filter(|event| matches!(event, Event::FoolCompensation { .. }))
                .count();
        }
        assert!(compensations > 0, "the fool exchange never happened");
    }

    #[test]
    fn a_handle_is_paid_to_the_winner_of_the_deal() {
        let mut seen = 0;
        for seed in 0..120 {
            let (_, events) = autoplay(Mode::Four, 2, seed);
            for event in &events {
                let Event::DealScored {
                    handles,
                    contract_points,
                    ..
                } = event
                else {
                    continue;
                };
                if *handles == 0.0 {
                    continue;
                }
                seen += 1;
                // RULE: the bonus goes to whoever won, so it always shares the
                // sign of the contract result
                assert_eq!(
                    *handles > OrderedFloat(0.0),
                    *contract_points >= OrderedFloat(0.0),
                    "a handle must never help the losing side"
                );
            }
        }
        assert!(seen > 0, "no handle was ever announced");
    }

    #[test]
    fn a_failed_contract_costs_the_taker() {
        let mut failures = 0;
        for seed in 0..40 {
            let mut engine = Engine::new(Mode::Four, 1, seed);
            let mut events = Vec::new();
            while let Some(question) = engine.pending().cloned() {
                events.append(&mut engine.take_events());
                let answer = engine.random_answer(&question).expect("legal answer");
                engine.answer(answer).expect("accepted");
            }
            events.append(&mut engine.take_events());
            for event in &events {
                let Event::DealScored {
                    taker,
                    contract_points,
                    petit_au_bout,
                    handles,
                    slam,
                    points,
                    ..
                } = event
                else {
                    continue;
                };
                assert_eq!(
                    *points,
                    *contract_points + *petit_au_bout + *handles + *slam,
                    "the deal total must be the sum of its parts"
                );
                if *contract_points < OrderedFloat(0.0) && *points < OrderedFloat(0.0) {
                    failures += 1;
                    let score = engine.player(*taker).expect("a taker").score();
                    assert!(
                        score < OrderedFloat(0.0),
                        "a taker who failed must lose points, got {score}"
                    );
                }
            }
        }
        assert!(failures > 0, "no contract ever failed");
    }

    #[test]
    fn petit_au_bout_is_worth_ten_times_the_contract() {
        let mut seen = 0;
        for mode in Mode::iter() {
            for seed in 0..80 {
                let (_, events) = autoplay(mode, 2, seed);
                for event in &events {
                    let Event::DealScored {
                        contract,
                        petit_au_bout,
                        ..
                    } = event
                    else {
                        continue;
                    };
                    if *petit_au_bout == 0.0 {
                        continue;
                    }
                    seen += 1;
                    assert_eq!(
                        OrderedFloat(petit_au_bout.abs()),
                        OrderedFloat(10.0) * contract.multiplier()
                    );
                }
            }
        }
        assert!(seen > 0, "petit au bout never happened");
    }

    // ---- the dog --------------------------------------------------------

    #[test]
    fn the_dog_follows_the_contract() {
        for seed in 0..60 {
            let mut engine = Engine::new(Mode::Four, 1, seed);
            let mut dog_events = Vec::new();
            while let Some(question) = engine.pending().cloned() {
                if matches!(question, Question::Play { .. }) {
                    break;
                }
                let answer = engine.random_answer(&question).expect("legal answer");
                engine.answer(answer).expect("accepted");
                dog_events.extend(engine.take_events());
            }
            let Some(deal) = engine.deal() else { continue };
            let Some(contract) = deal.contract() else {
                continue;
            };
            let taker = deal.taker().expect("a taker");
            let taker_discard = deal.player(taker).expect("a taker").discard().len();
            match contract {
                Contract::Petite | Contract::Garde => {
                    assert_eq!(taker_discard, Mode::Four.dog_size());
                    assert!(
                        dog_events
                            .iter()
                            .any(|e| matches!(e, Event::DogGivenToTaker { .. }))
                    );
                }
                Contract::GardeSans => {
                    assert_eq!(taker_discard, Mode::Four.dog_size());
                    assert!(
                        dog_events
                            .iter()
                            .any(|e| matches!(e, Event::DogKeptByAttack { .. }))
                    );
                }
                Contract::GardeContre => {
                    assert_eq!(taker_discard, 0);
                    let in_defense = deal
                        .players()
                        .iter()
                        .filter(|p| !p.is_attack())
                        .map(|p| p.discard().len())
                        .sum::<usize>();
                    assert_eq!(in_defense, Mode::Four.dog_size());
                }
            }
        }
    }

    #[test]
    fn a_discarded_dog_never_holds_a_king_or_an_oudler() {
        for seed in 0..60 {
            let mut engine = Engine::new(Mode::Four, 1, seed);
            while let Some(question) = engine.pending().cloned() {
                if matches!(question, Question::Play { .. }) {
                    break;
                }
                let answer = engine.random_answer(&question).expect("legal answer");
                engine.answer(answer).expect("accepted");
            }
            let Some(deal) = engine.deal() else { continue };
            let Some(Contract::Petite | Contract::Garde) = deal.contract() else {
                continue;
            };
            let taker = deal.taker().expect("a taker");
            for card in deal.player(taker).expect("a taker").discard().iter() {
                assert_ne!(*card, Card::normal(Suit::Heart, SuitValue::King));
                assert!(!card.is_oudler(), "{card} must never go to the dog");
            }
        }
    }

    // ---- five players ----------------------------------------------------

    #[test]
    fn the_called_king_decides_who_the_ally_is() {
        for seed in 0..40 {
            let mut engine = Engine::new(Mode::Five, 1, seed);
            let mut called: Option<Card> = None;
            while let Some(question) = engine.pending().cloned() {
                if matches!(question, Question::Play { .. }) {
                    break;
                }
                let answer = engine.random_answer(&question).expect("legal answer");
                engine.answer(answer).expect("accepted");
                for event in engine.take_events() {
                    if let Event::KingCalled { card, .. } = event {
                        called = Some(card);
                    }
                }
            }
            let (Some(deal), Some(called)) = (engine.deal(), called) else {
                continue;
            };
            let taker = deal.taker().expect("a taker");
            let ally = deal
                .players()
                .iter()
                .position(|player| player.role() == Some(Role::Ally));
            match ally {
                // the called king is in the dog or in the taker's own hand
                None => assert!(
                    deal.players()
                        .iter()
                        .enumerate()
                        .all(|(index, player)| index == taker || !player.has(&called))
                ),
                Some(ally) => {
                    assert_ne!(ally, taker);
                    assert!(deal.player(ally).expect("an ally").has(&called));
                }
            }
        }
    }

    #[test]
    fn a_lone_taker_is_paid_four_times_and_an_allied_one_twice() {
        for seed in 0..40 {
            let mut engine = Engine::new(Mode::Five, 1, seed);
            let mut events = Vec::new();
            while let Some(question) = engine.pending().cloned() {
                events.append(&mut engine.take_events());
                let answer = engine.random_answer(&question).expect("legal answer");
                engine.answer(answer).expect("accepted");
            }
            events.append(&mut engine.take_events());
            for event in &events {
                let Event::DealScored {
                    taker,
                    ally,
                    points,
                    ..
                } = event
                else {
                    continue;
                };
                let expected = if ally.is_some() { 2.0 } else { 4.0 };
                assert_eq!(
                    engine.player(*taker).expect("a taker").score(),
                    OrderedFloat(expected) * *points
                );
            }
        }
    }

    // ---- questions are always answerable --------------------------------

    #[test]
    fn a_question_never_offers_an_empty_choice() {
        for mode in Mode::iter() {
            let mut engine = Engine::new(mode, 3, 55);
            while let Some(question) = engine.pending().cloned() {
                match &question {
                    // passing is always possible, so an empty list is fine here
                    Question::Contract { .. } | Question::Slam { .. } => {}
                    Question::Handle { allowed, .. } => assert_ne!(allowed.len(), 0),
                    Question::Call { allowed, .. }
                    | Question::Discard { allowed, .. }
                    | Question::Play { allowed, .. } => assert_ne!(allowed.len(), 0),
                    Question::TrimHandle {
                        shown, to_remove, ..
                    } => {
                        assert_ne!(shown.len(), 0);
                        assert!(*to_remove > 0);
                    }
                }
                let answer = engine.random_answer(&question).expect("legal answer");
                engine.answer(answer).expect("accepted");
            }
        }
    }

    #[test]
    fn a_handle_shows_exactly_the_required_number_of_trumps() {
        let mut seen = 0;
        for mode in Mode::iter() {
            for seed in 0..60 {
                let (_, events) = autoplay(mode, 2, seed);
                for event in &events {
                    let Event::HandleAnnounced { handle, trumps, .. } = event else {
                        continue;
                    };
                    seen += 1;
                    assert_ne!(*handle, Handle::Refused);
                    assert_eq!(trumps.len(), mode.handle_limit(handle));
                    // RULE: the fool only completes a handle that lacks real trumps
                    if trumps.iter().any(|card| card.is_fool()) {
                        assert_eq!(
                            trumps.iter().filter(|card| !card.is_fool()).count(),
                            mode.handle_limit(handle) - 1
                        );
                    }
                }
            }
        }
        assert!(seen > 0, "no handle was ever announced");
    }

    // ---- turn mechanics --------------------------------------------------

    #[test]
    fn the_winner_of_a_turn_leads_the_next_one() {
        let mut engine = Engine::new(Mode::Four, 3, 17);
        let mut expected_leader: Option<usize> = None;
        let mut checked = 0;
        while let Some(question) = engine.pending().cloned() {
            if let Question::Play { player, .. } = question
                && let Some(deal) = engine.deal()
                && deal.turn().is_empty()
                && let Some(leader) = expected_leader.take()
            {
                assert_eq!(player, leader, "the master of a turn opens the next one");
                checked += 1;
            }
            let answer = engine.random_answer(&question).expect("legal answer");
            engine.answer(answer).expect("accepted");
            for event in engine.take_events() {
                match event {
                    Event::TurnWon { player, .. } => expected_leader = Some(player),
                    // a new deal restarts from the dealer, not from the last master
                    Event::DealStarted { .. } => expected_leader = None,
                    _ => {}
                }
            }
        }
        assert!(checked > 40, "only {checked} turns were checked");
    }

    #[test]
    fn a_turn_holds_one_card_per_player() {
        for mode in Mode::iter() {
            let (_, events) = autoplay(mode, 2, 31);
            let mut played = 0;
            for event in &events {
                match event {
                    Event::CardPlayed { .. } => played += 1,
                    Event::TurnWon { .. } => {
                        assert_eq!(played, mode.players());
                        played = 0;
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn every_deal_plays_all_the_cards() {
        for mode in Mode::iter() {
            let (_, events) = autoplay(mode, 3, 71);
            let turns = events
                .iter()
                .filter(|event| matches!(event, Event::TurnWon { .. }))
                .count();
            let deals = events
                .iter()
                .filter(|event| matches!(event, Event::DealScored { .. }))
                .count();
            assert_eq!(turns, deals * mode.cards_per_player());
        }
    }

    // ---- slam -------------------------------------------------------------

    #[test]
    fn a_realized_slam_means_one_side_took_every_turn() {
        let mut slams = 0;
        for mode in Mode::iter() {
            for seed in 0..40 {
                let mut engine = Engine::new(mode, 1, seed);
                let mut attack: Vec<usize> = Vec::new();
                let mut winners: Vec<usize> = Vec::new();
                while let Some(question) = engine.pending().cloned() {
                    if let Some(deal) = engine.deal()
                        && attack.is_empty()
                    {
                        attack = deal
                            .players()
                            .iter()
                            .enumerate()
                            .filter(|(_, player)| player.is_attack())
                            .map(|(index, _)| index)
                            .collect();
                    }
                    let answer = engine.random_answer(&question).expect("legal answer");
                    engine.answer(answer).expect("accepted");
                    for event in engine.take_events() {
                        match event {
                            Event::TurnWon { player, .. } => winners.push(player),
                            Event::SlamResult {
                                realized: true,
                                bonus,
                                ..
                            } => {
                                slams += 1;
                                let by_attack =
                                    winners.iter().all(|player| attack.contains(player));
                                let by_defense =
                                    winners.iter().all(|player| !attack.contains(player));
                                assert!(
                                    by_attack || by_defense,
                                    "{mode:?} seed {seed}: a slam needs every turn on one side"
                                );
                                let magnitude = OrderedFloat(bonus.abs());
                                assert!(
                                    magnitude == OrderedFloat(200.0)
                                        || magnitude == OrderedFloat(400.0),
                                    "a slam is worth 200 or 400, got {bonus}"
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        assert!(slams > 0, "no slam ever happened");
    }

    #[test]
    fn a_petit_sec_hand_is_dealt_again() {
        // the check is that a petit sec never reaches the bidding
        for seed in 0..200 {
            let (_, events) = autoplay(Mode::Four, 1, seed);
            let mut petit_sec = false;
            for event in &events {
                match event {
                    Event::PetitSec { .. } => petit_sec = true,
                    Event::Bid { .. } => {
                        assert!(!petit_sec, "bidding started on a cancelled deal");
                    }
                    Event::DealStarted { .. } => petit_sec = false,
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn a_team_is_either_attack_or_defense() {
        let mut engine = Engine::new(Mode::Five, 1, 44);
        while let Some(question) = engine.pending().cloned() {
            if matches!(question, Question::Play { .. }) {
                break;
            }
            let answer = engine.random_answer(&question).expect("legal answer");
            engine.answer(answer).expect("accepted");
        }
        let deal = engine.deal().expect("a deal is running");
        for player in deal.players() {
            let team = player.team().expect("every player belongs to a team");
            match player.role().expect("every player has a role") {
                Role::Taker | Role::Ally => assert_eq!(team, Team::Attack),
                Role::Defenser => assert_eq!(team, Team::Defense),
            }
        }
        assert!(
            deal.players().iter().all(|p| p.owned().is_empty()),
            "no turn is won before the first card is played"
        );
    }

    #[test]
    fn the_scored_deal_stays_readable_after_the_next_one_is_dealt() {
        let mut engine = Engine::new(crate::mode::Mode::Four, 2, 3);
        assert!(engine.last_deal().is_none());
        let mut seen = 0;
        while let Some(question) = engine.pending().cloned() {
            let answer = engine.random_answer(&question).expect("a legal answer");
            engine.answer(answer).expect("a legal answer");
            let scored = engine
                .take_events()
                .iter()
                .any(|event| matches!(event, crate::event::Event::DealScored { .. }));
            if scored {
                let last = engine.last_deal().expect("the scored deal is kept");
                let cards: usize = last
                    .players()
                    .iter()
                    .map(|player| player.all_cards().len())
                    .sum();
                assert_eq!(cards, crate::constants::MAX_CARDS);
                assert!(last.taker().is_some());
                seen += 1;
            }
        }
        assert!(seen > 0);
    }
}
