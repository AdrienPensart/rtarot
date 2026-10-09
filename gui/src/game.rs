//! Driving the engine from a window: bots answer on a timer, humans by clicking.

use crate::log;
use egui::Context;
use rand::RngExt;
use rtarot_core::{
    Answer, Card, Contract, Deal, Deck, Engine, Event, Mode, PlayerInGame, Question, TarotErrorKind,
};
use std::time::{Duration, Instant};

/// Everything chosen on the setup screen.
#[derive(Clone, Debug)]
pub struct Settings {
    pub mode: Mode,
    pub deals: u64,
    /// The seat played by the human.
    pub seat: usize,
    /// Bots always pass, so the human plays the attack.
    pub attack: bool,
    /// Nobody, human or bot, ever announces a slam.
    pub no_slam: bool,
    /// Answer by itself when only one card is legal.
    pub auto: bool,
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
            seat: Self::south(Mode::Four),
            attack: false,
            no_slam: false,
            auto: true,
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
}

pub struct Game {
    pub settings: Settings,
    pub engine: Engine,
    pub log: Vec<Line>,
    /// The taker's partner, once the called king has been seen on the table.
    pub ally: Option<usize>,
    /// What each seat said during the auction: nothing yet, a pass, or a contract.
    pub bids: Vec<Bid>,
    pub trick: Trick,
    pub last_trick: Option<Trick>,
    /// The dog, from the moment it is turned over until the first card is played.
    pub dog: Option<Deck>,
    pub summary: Option<Summary>,
    pub error: Option<String>,
    /// Bots stay still while a summary is being read.
    pub paused: bool,
    next_bot_at: Instant,
}

impl Game {
    pub fn new(settings: &Settings) -> Self {
        let seed = rand::rng().random();
        let engine = Engine::new(settings.mode, settings.deals, seed);
        let mut game = Self {
            settings: settings.clone(),
            engine,
            log: Vec::new(),
            trick: Trick::default(),
            ally: None,
            bids: vec![Bid::Silent; settings.mode.players()],
            last_trick: None,
            dog: None,
            summary: None,
            error: None,
            paused: false,
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

    pub fn taker(&self) -> Option<usize> {
        self.engine.deal().and_then(Deal::taker)
    }

    /// The king the taker called, five players only.
    pub fn callee(&self) -> Option<Card> {
        self.engine
            .deal()
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
        if let Some(deal) = self.engine.deal()
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
        let mut turn_won = false;
        for event in self.engine.take_events() {
            match &event {
                Event::DealStarted { .. } => {
                    self.trick = Trick::default();
                    self.last_trick = None;
                    self.dog = None;
                    self.ally = None;
                    self.bids.iter_mut().for_each(|bid| *bid = Bid::Silent);
                }
                Event::DogGivenToTaker { dog, .. } => self.dog = Some(dog.clone()),
                Event::CardPlayed { player, card } => {
                    self.dog = None;
                    self.trick.cards.push((*player, *card));
                    // The partner stays secret until the called king is put down.
                    if Some(*card) == self.callee() && Some(*player) != self.taker() {
                        self.ally = Some(*player);
                    }
                }
                Event::TurnWon { player, .. } => {
                    let mut done = std::mem::take(&mut self.trick);
                    done.winner = Some(*player);
                    self.last_trick = Some(done);
                    turn_won = true;
                }
                Event::DealScored { .. } => {
                    self.summary = Some(log::summary(&self.engine, &event));
                    self.paused = true;
                }
                Event::Bid { player, contract } => {
                    if let Some(bid) = self.bids.get_mut(*player) {
                        *bid = contract.map_or(Bid::Pass, Bid::Contract);
                    }
                }
                Event::PetitSec { .. }
                | Event::EveryonePassed
                | Event::KingCalled { .. }
                | Event::SlamAnnounced { .. }
                | Event::DogKeptByAttack { .. }
                | Event::DogGivenToDefense { .. }
                | Event::Discarded { .. }
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
        }
        turn_won
    }

    /// Hand an answer to the engine, whoever it comes from.
    pub fn answer(&mut self, answer: Answer) {
        if let Err(error) = self.engine.answer(answer) {
            self.error = Some(error.to_string());
        }
        let turn_won = self.drain_events();
        // A finished trick stays on the table twice as long as a single card.
        let factor = if turn_won { 2 } else { 1 };
        self.next_bot_at = Instant::now() + self.delay() * factor;
    }

    /// What the human does not even need to be asked.
    fn automatic(&self, question: &Question) -> Option<Answer> {
        match question {
            Question::Slam { .. } => self.settings.no_slam.then_some(Answer::Slam(false)),
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

    fn bot_answer(&mut self, question: &Question) -> Result<Answer, TarotErrorKind> {
        if self.settings.no_slam && matches!(question, Question::Slam { .. }) {
            return Ok(Answer::Slam(false));
        }
        if self.settings.attack && matches!(question, Question::Contract { .. }) {
            return Ok(Answer::Contract(None));
        }
        self.engine.random_answer(question)
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
    use rtarot_core::Mode;
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
                    game.summary = None;
                    game.paused = false;
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
            let text = |line: &Vec<Span>| {
                line.iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>()
            };
            assert!(game.log.iter().any(|line| text(line).contains("Game over")));
            assert_eq!(game.engine.is_consistent(), Ok(()));
        }
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
