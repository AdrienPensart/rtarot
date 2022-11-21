use clap::Parser;
use std::error;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::thread;
use strum::IntoEnumIterator;

pub mod card;
pub mod contract;
pub mod deck;
pub mod errors;
pub mod game;
pub mod game_distributed;
pub mod game_started;
pub mod handle;
pub mod helpers;
pub mod mode;
pub mod normal;
pub mod options;
pub mod player;
pub mod player_in_game;
pub mod points;
pub mod role;
pub mod suit;
pub mod suit_value;
pub mod team;
pub mod traits;
pub mod trump;
pub mod turn;

use crate::mode::Mode;
use crate::options::Options;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Opts {
    /// Players mode
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(["3", "4", "5"]), default_value = "4")]
    players: String,

    /// Number of deals per game
    #[arg(short = 'd', long = "deals", default_value_t = 10)]
    deals: u16,

    /// Attack mode
    #[arg(long = "attack")]
    attack: bool,

    /// Random playing mode
    #[arg(short = 'r', long = "random")]
    random: bool,

    /// Auto playing mode when possible
    #[arg(short = 'a', long = "auto")]
    auto: bool,

    /// Test mode
    #[arg(short = 't', long = "test")]
    test: bool,

    /// Quiet mode
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Forbid slam
    #[arg(long = "no-slam")]
    no_slam: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[arg(short, default_value_t = thread::available_parallelism().unwrap())]
    concurrency: NonZeroUsize,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = Opts::parse();
    let options = Options {
        random: opt.random,
        auto: opt.auto,
        quiet: opt.quiet,
        no_slam: opt.no_slam,
        attack: opt.attack,
        test: opt.test,
    };
    if opt.test {
        let mut children = vec![];
        if opt.concurrency == NonZeroUsize::new(1).unwrap() {
            for mode in Mode::iter().cycle() {
                match mode {
                    Mode::Three => {
                        helpers::test_game::<{ Mode::Three.players() }>(options, opt.deals)?
                    }
                    Mode::Four => {
                        helpers::test_game::<{ Mode::Four.players() }>(options, opt.deals)?
                    }
                    Mode::Five => {
                        helpers::test_game::<{ Mode::Five.players() }>(options, opt.deals)?
                    }
                }
            }
        } else {
            for _ in 0..opt.concurrency.get() {
                children.push(thread::spawn(move || {
                    println!("Spawned thread {:?}", thread::current());
                    for mode in Mode::iter().cycle() {
                        let result = match mode {
                            Mode::Three => {
                                helpers::test_game::<{ Mode::Three.players() }>(options, opt.deals)
                            }
                            Mode::Four => {
                                helpers::test_game::<{ Mode::Four.players() }>(options, opt.deals)
                            }
                            Mode::Five => {
                                helpers::test_game::<{ Mode::Five.players() }>(options, opt.deals)
                            }
                        };
                        if let Err(e) = result {
                            eprintln!("{:?} : {}", thread::current(), e);
                        }
                    }
                }));
            }
            for child in children {
                let _ = child.join();
            }
        }
    } else {
        let mode = Mode::from_str(&opt.players);
        match mode {
            Ok(Mode::Three) => {
                game::Game::<{ Mode::Three.players() }>::new(options)?.start(opt.deals)?
            }
            Ok(Mode::Four) => {
                game::Game::<{ Mode::Four.players() }>::new(options)?.start(opt.deals)?
            }
            Ok(Mode::Five) => {
                game::Game::<{ Mode::Five.players() }>::new(options)?.start(opt.deals)?
            }
            Err(e) => eprintln!("{}", e),
        };
    }
    Ok(())
}
