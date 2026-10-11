//! Driving the engine from a window: bots answer on a timer, humans by clicking.

use crate::log;
use egui::Context;
use ordered_float::OrderedFloat;
use rand::RngExt;
use rtarot_core::bot;
use rtarot_core::{
    Answer, Card, Contract, Deal, Dealing, Deck, Direction, Engine, Event, Mode, Options,
    PlayerInGame, Points, Question, TarotErrorKind, Team,
};
use std::time::{Duration, Instant};

/// Everything chosen on the setup screen.
#[derive(Clone, Debug)]
pub struct Settings {
    pub mode: Mode,
    pub deals: u64,
    /// Which way the table turns.
    pub direction: Direction,
    /// Whether the pack is cut and dealt three by three, or shuffled every deal.
    pub dealing: Dealing,
    /// The seat played by the human.
    pub seat: usize,
    /// Bots always pass, so the human plays the attack.
    pub attack: bool,
    /// Nobody, human or bot, ever announces a slam.
    pub no_slam: bool,
    /// Answer by itself when only one card is legal.
    pub auto: bool,
    /// Seconds a finished trick stays on the table before it is folded.
    pub trick_delay: f32,
    /// Seconds a bot waits before answering, so that a human can follow.
    pub bot_delay: f32,
}

impl Settings {
    /// The seat a human takes by default in `mode`: south, the bottom of the table.
    pub fn south(mode: Mode) -> usize {
        (0..mode.players())
            .find(|&seat| mode.player_name(seat) == Ok("South"))
            .unwrap_or_else(|| mode.players() - 1)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Four,
            deals: 10,
            direction: Direction::default(),
            dealing: Dealing::default(),
            seat: Self::south(Mode::Four),
            attack: false,
            no_slam: false,
            auto: false,
            trick_delay: 2.0,
            bot_delay: 0.8,
        }
    }
}

/// Who a piece of text talks about, so that a name can be painted in its side's color.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tone {
    Plain,
    Taker,
    Ally,
}

/// A run of text in one tone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub text: String,
    pub tone: Tone,
}

/// One line of the table talk.
pub type Line = Vec<Span>;

/// What a seat said during the auction.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Bid {
    #[default]
    Silent,
    Pass,
    Contract(Contract),
}

/// The cards on the table, in the order they were put down.
#[derive(Clone, Debug, Default)]
pub struct Trick {
    pub cards: Vec<(usize, Card)>,
    pub winner: Option<usize>,
}

/// What a finished deal was worth, ready to be displayed.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub rows: Vec<(String, String)>,
    /// What each seat won or lost on the deal.
    pub gains: Vec<OrderedFloat<f64>>,
}

pub struct Game {
    pub settings: Settings,
    pub engine: Engine,
    /// What the engine was seeded with: the same seed deals the same cards again.
    seed: u64,
    pub log: Vec<Line>,
    /// The taker's partner, once the called king has been seen on the table.
    pub ally: Option<usize>,
    /// Whether the table has seen that the called king is the taker's own:
    /// it was in the dog, or the taker played it.
    alone_seen: bool,
    /// What each seat said during the auction: nothing yet, a pass, or a contract.
    pub bids: Vec<Bid>,
    pub trick: Trick,
    pub last_trick: Option<Trick>,
    /// Every trick of the deal so far, in order.
    pub tricks: Vec<Trick>,
    /// The dog, from the moment it is turned over until the taker starts discarding.
    pub dog: Option<Deck>,
    /// The cards the human has set aside for the dog, not yet given to the engine.
    pub discard_pick: Vec<Card>,
    /// Whether the human ticked the slam box while bidding.
    pub slam_intent: bool,
    pub summary: Option<Summary>,
    pub error: Option<String>,
    /// Bots stay still while a summary is being read.
    pub paused: bool,
    /// What happened after the deal was scored, held back until the next deal is asked for.
    deferred: Vec<Event>,
    /// Whether the window listing the tricks won is open.
    pub show_tricks: bool,
    /// Whether the window showing the last trick is open.
    pub show_last: bool,
    /// Whether the table talk panel is shown.
    pub show_log: bool,
    /// Whether the hand is laid out from the highest card on the left down to the
    /// lowest on the right. Off for every new game: the hand reads left to right.
    pub right_to_left: bool,
    /// Until when the last trick stays on the table.
    last_trick_until: Instant,
    /// Whether the window replaying the deal trick by trick is open.
    pub show_history: bool,
    /// Whether the window showing every hand as it was dealt is open.
    pub show_dealt: bool,
    next_bot_at: Instant,
}

impl Game {
    pub fn new(settings: &Settings) -> Self {
        Self::with_seed(settings, rand::rng().random())
    }

    /// The same game again: same seed, so the same cards are dealt first.
    ///
    /// With traditional dealing the later deals depend on how the tricks fell,
    /// so only the first deal is sure to come back identical.
    pub fn replay(&self) -> Self {
        Self::with_seed(&self.settings, self.seed)
    }

    // pub const fn seed(&self) -> u64 {
    //     self.seed
    // }

    pub fn with_seed(settings: &Settings, seed: u64) -> Self {
        let names = (0..settings.mode.players())
            .map(|index| {
                settings
                    .mode
                    .player_name(index)
                    .unwrap_or("Player")
                    .to_string()
            })
            .collect();
        let options = Options {
            direction: settings.direction,
            dealing: settings.dealing,
        };
        let engine = Engine::with_options(settings.mode, names, settings.deals, seed, options);
        let mut game = Self {
            settings: settings.clone(),
            engine,
            seed,
            log: Vec::new(),
            trick: Trick::default(),
            ally: None,
            alone_seen: false,
            bids: vec![Bid::Silent; settings.mode.players()],
            last_trick: None,
            tricks: Vec::new(),
            dog: None,
            discard_pick: Vec::new(),
            slam_intent: false,
            summary: None,
            error: None,
            paused: false,
            deferred: Vec::new(),
            show_tricks: false,
            show_last: false,
            show_log: false,
            right_to_left: false,
            last_trick_until: Instant::now(),
            show_history: false,
            show_dealt: false,
            next_bot_at: Instant::now(),
        };
        game.drain_events();
        game
    }

    pub const fn human(&self) -> usize {
        self.settings.seat
    }

    pub fn name(&self, index: usize) -> String {
        self.engine
            .player(index)
            .map_or_else(|_| format!("player {index}"), |p| p.name().to_string())
    }

    /// The deal on the table: the one being played, or the one just scored
    /// while its score sheet is being read.
    pub const fn deal(&self) -> Option<&Deal> {
        if self.paused {
            self.engine.last_deal()
        } else {
            self.engine.deal()
        }
    }

    pub fn taker(&self) -> Option<usize> {
        self.deal().and_then(Deal::taker)
    }

    /// Whether everybody knows who plays with whom: at once with three or four
    /// players, once the called king has been seen with five.
    pub fn sides_known(&self) -> bool {
        self.engine.mode() != Mode::Five || self.ally.is_some() || self.alone()
    }

    /// Whether the taker is known to play alone, having called a king of their own.
    ///
    /// The table knows once the king shows up in the dog or is played by the taker.
    /// The human knows at once when they are the taker and hold it.
    pub fn alone(&self) -> bool {
        if self.alone_seen {
            return true;
        }
        let Some(taker) = self.taker() else {
            return false;
        };
        if taker != self.human() {
            return false;
        }
        let Some(callee) = self.callee() else {
            return false;
        };
        self.deal()
            .and_then(|deal| deal.player(taker).ok())
            .is_some_and(|player| player.has(&callee))
    }

    /// Card points taken so far by a side.
    pub fn team_points(&self, team: Team) -> OrderedFloat<f64> {
        self.deal().map_or(OrderedFloat(0.0), |deal| {
            deal.players()
                .iter()
                .filter(|player| player.team() == Some(team))
                .map(|player| player.owned().points())
                .sum()
        })
    }

    /// How many cards the human has to put in the dog, while that is being asked.
    pub fn picking(&self) -> Option<usize> {
        if let Some(Question::Discard { remaining, .. }) = self.human_question() {
            Some(*remaining)
        } else {
            None
        }
    }

    /// Hand the picked discard to the engine, one card at a time.
    pub fn commit_discard(&mut self) {
        while let Some(Question::Discard { allowed, .. }) = self.human_question().cloned() {
            let Some(index) = self
                .discard_pick
                .iter()
                .position(|card| allowed.contains(card))
            else {
                break;
            };
            let card = self.discard_pick.remove(index);
            self.answer(Answer::Card(card));
            if self.error.is_some() {
                break;
            }
        }
    }

    /// Whether the last trick is still on the table, and for how long.
    pub fn last_trick_showing(&self) -> Option<Duration> {
        self.last_trick_until.checked_duration_since(Instant::now())
    }

    /// Read the score sheet, then let the next deal start.
    pub fn next_deal(&mut self) {
        self.summary = None;
        self.show_history = false;
        self.show_dealt = false;
        self.paused = false;
        let deferred = std::mem::take(&mut self.deferred);
        self.process(deferred);
    }

    /// The king the taker called, five players only.
    pub fn callee(&self) -> Option<Card> {
        self.deal()
            .and_then(|deal| deal.players().first())
            .and_then(PlayerInGame::callee)
    }

    /// Names worth painting in a color right now.
    fn marks(&self) -> Vec<(String, Tone)> {
        [(self.taker(), Tone::Taker), (self.ally, Tone::Ally)]
            .into_iter()
            .filter_map(|(seat, tone)| seat.map(|seat| (self.name(seat), tone)))
            .collect()
    }

    /// What to write next to a name: the contract once the auction is over,
    /// else what that seat bid so far.
    pub fn bid_label(&self, seat: usize) -> Option<String> {
        if let Some(deal) = self.deal()
            && let (Some(contract), Some(taker)) = (deal.contract(), deal.taker())
        {
            return (taker == seat).then(|| contract.to_string());
        }
        match self.bids.get(seat).copied().unwrap_or_default() {
            Bid::Silent => None,
            Bid::Pass => Some("passes".to_string()),
            Bid::Contract(contract) => Some(contract.to_string()),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.engine.is_finished()
    }

    /// The question waiting for the human, if any.
    pub fn human_question(&self) -> Option<&Question> {
        self.engine
            .pending()
            .filter(|question| question.player() == self.human())
    }

    fn delay(&self) -> Duration {
        Duration::from_secs_f32(self.settings.bot_delay.max(0.0))
    }

    /// Pull everything that happened out of the engine. Returns whether a turn ended.
    fn drain_events(&mut self) -> bool {
        let events = self.engine.take_events();
        self.process(events)
    }

    /// Apply `events` to the table. Whatever follows a scored deal waits for [`Self::next_deal`].
    fn process(&mut self, events: Vec<Event>) -> bool {
        let mut turn_won = false;
        let mut events = events.into_iter();
        while let Some(event) = events.next() {
            let alone_before = self.alone_seen;
            match &event {
                Event::DealStarted { .. } => {
                    self.trick = Trick::default();
                    self.last_trick = None;
                    self.tricks.clear();
                    self.dog = None;
                    self.discard_pick.clear();
                    self.slam_intent = false;
                    self.ally = None;
                    self.alone_seen = false;
                    self.bids.iter_mut().for_each(|bid| *bid = Bid::Silent);
                }
                Event::DogGivenToTaker { dog, .. } => {
                    self.dog = Some(dog.clone());
                    // RULE: a called king found in the dog leaves the taker alone
                    if let Some(callee) = self.callee()
                        && dog.has(&callee)
                    {
                        self.alone_seen = true;
                    }
                }
                Event::CardPlayed { player, card } => {
                    self.dog = None;
                    self.trick.cards.push((*player, *card));
                    // The partner stays secret until the called king is put down.
                    if Some(*card) == self.callee() {
                        if Some(*player) == self.taker() {
                            self.alone_seen = true;
                        } else {
                            self.ally = Some(*player);
                        }
                    }
                }
                Event::TurnWon { player, .. } => {
                    let mut done = std::mem::take(&mut self.trick);
                    done.winner = Some(*player);
                    self.tricks.push(done.clone());
                    self.last_trick = Some(done);
                    self.last_trick_until = Instant::now()
                        + Duration::from_secs_f32(self.settings.trick_delay.max(0.0));
                    turn_won = true;
                }
                Event::DealScored { .. } => {
                    self.summary = Some(log::summary(&self.engine, &event));
                    self.paused = true;
                }
                Event::Discarded { .. } => self.dog = None,
                Event::Bid { player, contract } => {
                    if let Some(bid) = self.bids.get_mut(*player) {
                        *bid = contract.map_or(Bid::Pass, Bid::Contract);
                    }
                }
                Event::DeckCut { .. }
                | Event::PetitSec { .. }
                | Event::EveryonePassed
                | Event::KingCalled { .. }
                | Event::SlamAnnounced { .. }
                | Event::DogKeptByAttack { .. }
                | Event::DogGivenToDefense { .. }
                | Event::HandleAnnounced { .. }
                | Event::FoolKept { .. }
                | Event::FoolLost { .. }
                | Event::MasterChanged { .. }
                | Event::PetitAuBout { .. }
                | Event::FoolCompensation { .. }
                | Event::SlamResult { .. }
                | Event::GameFinished => {}
            }
            let marks = if let Event::DealScored { taker, ally, .. } = &event {
                // The deal is already over, so the table no longer knows them.
                [(Some(*taker), Tone::Taker), (*ally, Tone::Ally)]
                    .into_iter()
                    .filter_map(|(seat, tone)| seat.map(|seat| (self.name(seat), tone)))
                    .collect()
            } else {
                self.marks()
            };
            self.log
                .push(tag(&log::describe(&self.engine, &event), &marks));
            if self.alone_seen
                && !alone_before
                && let Some(taker) = self.taker()
            {
                let line = format!(
                    "{} called a king of their own and plays alone",
                    self.name(taker)
                );
                self.log.push(tag(&line, &marks));
            }
            if matches!(event, Event::DealScored { .. }) {
                self.deferred.extend(events.by_ref());
                break;
            }
        }
        turn_won
    }

    /// Hand an answer to the engine, whoever it comes from.
    pub fn answer(&mut self, answer: Answer) {
        if let Err(error) = self.engine.answer(answer) {
            self.error = Some(error.to_string());
        }
        let turn_won = self.drain_events();
        // A finished trick stays on the table before the bots go on.
        let wait = if turn_won {
            self.delay()
                .max(Duration::from_secs_f32(self.settings.trick_delay.max(0.0)))
        } else {
            self.delay()
        };
        self.next_bot_at = Instant::now() + wait;
    }

    /// What the human does not even need to be asked.
    const fn automatic(&self, question: &Question) -> Option<Answer> {
        match question {
            // The slam was decided with the bid, by a checkbox.
            Question::Slam { .. } => Some(Answer::Slam(self.slam_intent && !self.settings.no_slam)),
            Question::Call { allowed, .. }
            | Question::Discard { allowed, .. }
            | Question::Play { allowed, .. } => match allowed.as_slice() {
                [only] if self.settings.auto => Some(Answer::Card(*only)),
                _ => None,
            },
            Question::TrimHandle { shown, .. } => match shown.as_slice() {
                [only] if self.settings.auto => Some(Answer::Card(*only)),
                _ => None,
            },
            Question::Contract { .. } | Question::Handle { .. } => None,
        }
    }

    fn bot_answer(&self, question: &Question) -> Result<Answer, TarotErrorKind> {
        if self.settings.no_slam && matches!(question, Question::Slam { .. }) {
            return Ok(Answer::Slam(false));
        }
        if self.settings.attack && matches!(question, Question::Contract { .. }) {
            return Ok(Answer::Contract(None));
        }
        bot::answer(&self.engine, question)
    }

    /// Let the bots play, as far as the clock allows. Called once per frame.
    pub fn tick(&mut self, ctx: &Context) {
        loop {
            if self.error.is_some() || self.paused {
                return;
            }
            let Some(question) = self.engine.pending().cloned() else {
                return;
            };
            if question.player() == self.human() {
                match self.automatic(&question) {
                    Some(answer) => {
                        self.answer(answer);
                        continue;
                    }
                    None => return,
                }
            }
            let now = Instant::now();
            if now < self.next_bot_at {
                ctx.request_repaint_after(self.next_bot_at - now);
                return;
            }
            match self.bot_answer(&question) {
                Ok(answer) => self.answer(answer),
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            }
        }
    }
}

/// Split a sentence so that every name in `marks` gets its tone.
fn tag(text: &str, marks: &[(String, Tone)]) -> Line {
    let mut line = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let hit = marks
            .iter()
            .filter(|(name, _)| !name.is_empty())
            .filter_map(|(name, tone)| rest.find(name.as_str()).map(|at| (at, name, *tone)))
            .min_by_key(|(at, ..)| *at);
        let Some((at, name, tone)) = hit else {
            line.push(Span {
                text: rest.to_string(),
                tone: Tone::Plain,
            });
            break;
        };
        let (before, after) = rest.split_at(at);
        if !before.is_empty() {
            line.push(Span {
                text: before.to_string(),
                tone: Tone::Plain,
            });
        }
        line.push(Span {
            text: name.clone(),
            tone,
        });
        rest = after.get(name.len()..).unwrap_or_default();
    }
    line
}

#[cfg(test)]
mod tests {
    use super::{Game, Settings, Span, Tone, tag};
    use rtarot_core::{Mode, Question};
    use strum::IntoEnumIterator;

    /// A whole game where the human answers at random, with instant bots.
    #[test]
    fn a_game_runs_to_the_end_in_every_mode() {
        for mode in Mode::iter() {
            let settings = Settings {
                mode,
                seat: Settings::south(mode),
                deals: 3,
                bot_delay: 0.0,
                trick_delay: 0.0,
                ..Settings::default()
            };
            let mut game = Game::new(&settings);
            let ctx = egui::Context::default();
            let mut guard = 0;
            while !game.is_finished() {
                guard += 1;
                assert!(guard < 10_000, "the game does not end in mode {mode}");
                game.tick(&ctx);
                assert_eq!(game.error, None);
                if game.paused {
                    assert!(game.deal().is_some(), "the scored deal stays on the table");
                    game.next_deal();
                    continue;
                }
                if let Some(Question::Discard {
                    allowed, remaining, ..
                }) = game.human_question().cloned()
                {
                    // The human picks the whole discard first, then confirms it.
                    game.discard_pick
                        .extend(allowed.iter().take(remaining).copied());
                    game.commit_discard();
                    assert!(game.discard_pick.is_empty(), "the discard is given whole");
                    continue;
                }
                if let Some(question) = game.human_question().cloned() {
                    let answer = game
                        .engine
                        .random_answer(&question)
                        .expect("a legal answer");
                    game.answer(answer);
                }
            }
            // The last score sheet still waits to be read when the human plays last.
            if game.paused {
                game.next_deal();
            }
            let text = |line: &Vec<Span>| {
                line.iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>()
            };
            assert!(game.log.iter().any(|line| text(line).contains("Game over")));
            assert_eq!(game.engine.is_consistent(), Ok(()));
        }
    }

    /// A taker who calls a king they hold is shown alone once that king is played.
    #[test]
    fn a_taker_calling_their_own_king_is_labelled_alone() {
        let settings = Settings {
            mode: Mode::Five,
            seat: Settings::south(Mode::Five),
            deals: 1,
            bot_delay: 0.0,
            trick_delay: 0.0,
            ..Settings::default()
        };
        // Deal until the taker holds a king, whoever the taker is.
        let (mut game, king) = loop {
            let mut game = Game::new(&settings);
            let found = loop {
                let Some(question) = game.engine.pending().cloned() else {
                    break None;
                };
                if let Question::Call { player, allowed } = &question {
                    let hand = game
                        .engine
                        .deal()
                        .and_then(|deal| deal.player(*player).ok())
                        .map(|p| p.hand().clone())
                        .expect("the taker has a hand");
                    break allowed.iter().find(|king| hand.has(king)).copied();
                }
                let answer = game.engine.random_answer(&question).expect("legal");
                game.answer(answer);
            };
            if let Some(king) = found {
                break (game, king);
            }
        };
        assert!(!game.sides_known(), "nothing is known before the call");
        game.answer(rtarot_core::Answer::Card(king));
        let taker = game.taker().expect("a taker");
        let human_knows = taker == game.human();
        assert_eq!(
            game.alone(),
            human_knows,
            "only the human taker knows at once"
        );
        // Play on until the taker puts the king down.
        let mut seen = false;
        while let Some(question) = game.engine.pending().cloned() {
            if game.alone_seen {
                seen = true;
                break;
            }
            let answer = game.engine.random_answer(&question).expect("legal");
            game.answer(answer);
            assert_eq!(game.error, None);
        }
        assert!(seen, "the table sees the self-call once the king is played");
        assert!(game.alone());
        assert!(game.sides_known());
        assert_eq!(game.ally, None);
        let text = |line: &Vec<Span>| line.iter().map(|s| s.text.as_str()).collect::<String>();
        assert!(
            game.log
                .iter()
                .any(|line| text(line).contains("plays alone"))
        );
    }

    #[test]
    fn a_replay_deals_the_same_cards_again() {
        let settings = Settings {
            bot_delay: 0.0,
            trick_delay: 0.0,
            ..Settings::default()
        };
        let mut game = Game::new(&settings);
        let dealt = game.engine.deal().expect("a deal").dealt().to_vec();
        let dog = game.engine.deal().expect("a deal").dog().clone();
        // play a bit, so that the replay really starts over
        let ctx = egui::Context::default();
        for _ in 0..3 {
            game.tick(&ctx);
            if let Some(question) = game.human_question().cloned() {
                let answer = game.engine.random_answer(&question).expect("legal");
                game.answer(answer);
            }
        }
        let again = game.replay();
        // assert_eq!(again.seed(), game.seed());
        let deal = again.engine.deal().expect("a fresh deal");
        assert_eq!(deal.dealt(), dealt.as_slice());
        assert_eq!(deal.dog(), &dog);
        assert_eq!(again.engine.deals_left(), settings.deals);
        assert!(again.tricks.is_empty());
    }

    #[test]
    fn names_get_their_tone() {
        let marks = vec![
            ("North".to_string(), Tone::Taker),
            ("East".to_string(), Tone::Ally),
        ];
        let line = tag("East follows North, North wins", &marks);
        let tones: Vec<(&str, Tone)> = line
            .iter()
            .map(|span| (span.text.as_str(), span.tone))
            .collect();
        assert_eq!(
            tones,
            vec![
                ("East", Tone::Ally),
                (" follows ", Tone::Plain),
                ("North", Tone::Taker),
                (", ", Tone::Plain),
                ("North", Tone::Taker),
                (" wins", Tone::Plain),
            ]
        );
        assert_eq!(tag("Everyone passed", &marks).len(), 1);
    }
}
