use ordered_float::OrderedFloat;
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::card::Card;
use crate::constants::{BASE_CONTRACT_POINTS, MAX_CARDS, MAX_POINTS};
use crate::contract::Contract;
use crate::deck::Deck;
use crate::errors::TarotErrorKind;
use crate::event::Event;
use crate::handle::Handle;
use crate::mode::Mode;
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
}

impl Deal {
    fn new(dog: Deck, players: Vec<PlayerInGame>, leader: usize) -> Self {
        let count = players.len();
        Self {
            dog,
            players,
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
    #[must_use]
    pub const fn dog(&self) -> &Deck {
        &self.dog
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
        (self.leader + self.turn.len()) % self.players.len()
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
    rng: StdRng,
    deals_left: u64,
    dealer: usize,
    players: Vec<Player>,
    deal: Option<Deal>,
    phase: Phase,
    pending: Option<Question>,
    events: Vec<Event>,
}

impl Engine {
    /// A game with the default seat names for `mode`.
    #[must_use]
    pub fn new(mode: Mode, deals: u64, seed: u64) -> Self {
        let names = (0..mode.players())
            .map(|index| {
                mode.player_name(index)
                    .unwrap_or("Player")
                    .to_string()
            })
            .collect();
        Self::with_names(mode, names, deals, seed)
    }

    /// A game whose seats are named by the caller.
    #[must_use]
    pub fn with_names(mode: Mode, names: Vec<String>, deals: u64, seed: u64) -> Self {
        let players = names.into_iter().map(Player::new).collect();
        let mut engine = Self {
            mode,
            rng: StdRng::seed_from_u64(seed),
            deals_left: deals,
            dealer: 0,
            players,
            deal: None,
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
        let question = self.pending.take().ok_or(TarotErrorKind::UnexpectedAnswer)?;
        self.apply(&question, answer)?;
        self.advance()
    }

    /// A legal answer picked at random, for a seat nobody is playing.
    pub fn random_answer(&mut self, question: &Question) -> Result<Answer, TarotErrorKind> {
        let pick = |rng: &mut StdRng, cards: &[Card]| -> Result<Card, TarotErrorKind> {
            let index = rng.random_range(0..cards.len().max(1));
            cards.get(index).copied().ok_or(TarotErrorKind::NoCard(index))
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
            | Question::Play { allowed, .. } => {
                pick(&mut self.rng, allowed).map(Answer::Card)
            }
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
        self.dealer = (self.dealer + 1) % self.mode.players();
    }

    fn advance(&mut self) -> Result<(), TarotErrorKind> {
        loop {
            if self.pending.is_some() {
                return Ok(());
            }
            match self.phase {
                Phase::Dealing => self.start_deal(),
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

    fn start_deal(&mut self) {
        if self.deals_left == 0 {
            self.events.push(Event::GameFinished);
            self.phase = Phase::Finished;
            return;
        }
        self.events.push(Event::DealStarted {
            deals_left: self.deals_left,
            dealer: self.dealer,
        });
        let mut deck = Deck::shuffled(&mut self.rng);
        let mut dog = deck.give(self.mode.dog_size());
        dog.sort();
        let mut players = Vec::with_capacity(self.mode.players());
        for _ in 0..self.mode.players() {
            let mut player_in_game = PlayerInGame::new(self.mode);
            player_in_game.extend_hand(&deck.give(self.mode.cards_per_player()));
            players.push(player_in_game);
        }
        // RULE: a hand whose only trump is the petit is dealt again
        if let Some(player) = players.iter().position(PlayerInGame::petit_sec) {
            self.events.push(Event::PetitSec { player });
            self.rotate_dealer();
            return;
        }
        let leader = (self.dealer + 1) % self.mode.players();
        self.deal = Some(Deal::new(dog, players, leader));
        self.phase = Phase::Bidding;
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
            self.deal = None;
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
                self.events.push(Event::DogGivenToDefense { player: defenser });
                self.start_playing()
            }
            Contract::Petite | Contract::Garde => {
                if !self.deal_ref()?.dog_given {
                    let deal = self.deal_mut()?;
                    deal.dog_given = true;
                    deal.player_mut(taker)?.extend_hand(&dog);
                    self.events.push(Event::DogGivenToTaker {
                        player: taker,
                        dog,
                    });
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
        let slam_running = deal.took_everything(Team::Attack) || deal.took_everything(Team::Defense);

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
            (Question::Discard { player, allowed, .. }, Answer::Card(card)) => {
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
        let players = self.mode.players();
        let deal = self.deal_mut()?;
        deal.bidder = (deal.bidder + 1) % players;
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

        self.deal = None;
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

#[cfg(test)]
#[allow(clippy::wildcard_enum_match_arm)]
mod tests {
    use super::Engine;
    use crate::card::Card;
    use crate::contract::Contract;
    use crate::event::Event;
    use crate::handle::Handle;
    use crate::mode::Mode;
    use crate::question::{Answer, Question};
    use crate::role::Role;
    use crate::suit::Suit;
    use crate::suit_value::SuitValue;
    use crate::team::Team;
    use crate::trump::Trump;
    use ordered_float::OrderedFloat;
    use strum::IntoEnumIterator;

    /// Play a whole game with random answers, collecting every event.
    fn autoplay(mode: Mode, deals: u64, seed: u64) -> (Engine, Vec<Event>) {
        let mut engine = Engine::new(mode, deals, seed);
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
                    taker, ally, points, ..
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
                    Question::Handle { allowed, .. } => assert!(!allowed.is_empty()),
                    Question::Call { allowed, .. }
                    | Question::Discard { allowed, .. }
                    | Question::Play { allowed, .. } => assert!(!allowed.is_empty()),
                    Question::TrimHandle {
                        shown, to_remove, ..
                    } => {
                        assert!(!shown.is_empty());
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
                            Event::SlamResult { realized: true, bonus, .. } => {
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
}
