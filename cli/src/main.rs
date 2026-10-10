//! Terminal front end for the rtarot engine.
//!
//! All the game rules live in `rtarot-core`; this crate only decides how to ask
//! a human and how to draw the result.

use clap::Parser;
use rand::RngExt;
use rtarot_core::{Answer, Engine, Mode, Question, TarotErrorKind, bot};
use std::error;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::thread;
use strum::IntoEnumIterator;

mod render;
mod ui;

/// The only seat a human sits at by default.
const HUMAN_SEAT: &str = "South";

#[derive(Parser, Debug, Clone)]
#[clap(author, about, version)]
struct Opts {
    /// Players mode
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(["3", "4", "5"]), default_value = "4")]
    players: String,

    /// Number of deals per game
    #[arg(short = 'd', long = "deals", default_value_t = 10)]
    deals: u64,

    /// Bots always pass, so the human plays the attack
    #[arg(long = "attack")]
    attack: bool,

    /// Nobody is human, every seat is played by a bot
    #[arg(short = 'r', long = "random")]
    random: bool,

    /// Bots answer at random instead of following their rules of thumb
    #[arg(long = "dumb")]
    dumb: bool,

    /// Answer by itself when only one card is legal
    #[arg(short = 'a', long = "auto")]
    auto: bool,

    /// Play games forever, on every mode, to hunt for rule breakages
    #[arg(short = 't', long = "test")]
    test: bool,

    /// Quiet mode
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Forbid slam
    #[arg(long = "no-slam")]
    no_slam: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[arg(short, default_value_t = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN))]
    concurrency: NonZeroUsize,
}

impl Opts {
    /// Whether the engine, and not a human, answers for that seat.
    fn is_bot(&self, name: &str) -> bool {
        self.test || self.random || name != HUMAN_SEAT
    }
}

/// Decide what to reply to `question`, from a human or from the engine itself.
fn choose(engine: &mut Engine, question: &Question, opts: &Opts) -> Result<Answer, TarotErrorKind> {
    let player = question.player();
    let name = engine.player(player)?.name().to_string();

    if opts.no_slam && matches!(question, Question::Slam { .. }) {
        return Ok(Answer::Slam(false));
    }
    if !opts.is_bot(&name) {
        // Only one legal card left is not a decision worth a prompt.
        if opts.auto
            && let Some([only]) = single_choice(question)
        {
            return Ok(Answer::Card(*only));
        }
        return ui::ask(engine, question);
    }
    if opts.attack && matches!(question, Question::Contract { .. }) {
        return Ok(Answer::Contract(None));
    }
    // Test mode hunts rule breakages, which random play finds faster.
    if opts.dumb || opts.test {
        return engine.random_answer(question);
    }
    bot::answer(engine, question)
}

/// The allowed cards of a question, when it is a card question.
const fn single_choice(question: &Question) -> Option<&[rtarot_core::Card]> {
    match question {
        Question::Call { allowed, .. }
        | Question::Discard { allowed, .. }
        | Question::Play { allowed, .. } => Some(allowed.as_slice()),
        Question::TrimHandle { shown, .. } => Some(shown.as_slice()),
        Question::Contract { .. } | Question::Slam { .. } | Question::Handle { .. } => None,
    }
}

fn play(mode: Mode, opts: &Opts, seed: u64) -> Result<(), TarotErrorKind> {
    let mut engine = Engine::new(mode, opts.deals, seed);
    loop {
        let events = engine.take_events();
        if !opts.quiet {
            for event in &events {
                ui::show(&engine, event);
            }
        }
        let Some(question) = engine.pending().cloned() else {
            break;
        };
        let answer = choose(&mut engine, &question, opts)?;
        engine.answer(answer)?;
    }
    engine.is_consistent()
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opts = Opts::parse();
    if opts.test {
        let mut children = Vec::new();
        for _ in 0..opts.concurrency.get() {
            let opts = opts.clone();
            children.push(thread::spawn(move || {
                let mut rng = rand::rng();
                for mode in Mode::iter().cycle() {
                    if let Err(e) = play(mode, &opts, rng.random()) {
                        eprintln!("{:?} : {e}", thread::current().id());
                    }
                }
            }));
        }
        for child in children {
            drop(child.join());
        }
        return Ok(());
    }
    let mode = Mode::from_str(&opts.players)?;
    let seed = rand::rng().random();
    if let Err(e) = play(mode, &opts, seed) {
        eprintln!("{e}");
    }
    Ok(())
}
