// #[macro_use] extern crate failure;

use std::error;
use std::thread;
use std::str::FromStr;
use std::num::NonZeroUsize;
use clap::Parser;
use strum::IntoEnumIterator;

pub mod helpers;
pub mod traits;
pub mod constants;
pub mod errors;
pub mod color;
pub mod color_value;
pub mod trump_value;
pub mod card;
pub mod deck;
pub mod normal;
pub mod player;
pub mod role;
pub mod team;
pub mod turn;
pub mod handle;
pub mod mode;
pub mod contract;
pub mod game;

use crate::mode::Mode;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Opts {
    /// Players mode
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(["3", "4", "5"]), default_value = "4")]
    players: String,

    /// Random playing mode
    #[arg(short = 'r', long = "random")]
    random: bool,

    /// Auto playing mode when possible
    #[arg(short = 'a', long = "auto")]
    auto: bool,

    /// Test mode
    #[arg(short = 't', long = "test")]
    test: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[arg(short, default_value_t = thread::available_parallelism().unwrap())]
    concurrency: NonZeroUsize
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = Opts::parse();
    if opt.test {
        let mut children = vec![];
        for _ in 0..opt.concurrency.get() {
            children.push(thread::spawn(|| {
                for mode in Mode::iter().cycle() {
                    match mode {
                        Mode::Three => helpers::test_game::<{Mode::Three.players()}>().unwrap(),
                        Mode::Four => helpers::test_game::<{Mode::Four.players()}>().unwrap(),
                        Mode::Five => helpers::test_game::<{Mode::Five.players()}>().unwrap(),
                    }
                }
            }));
        }
        for child in children {
            let _ = child.join();
        }
    } else {
        let mode = Mode::from_str(&opt.players);
        match mode {
            Ok(Mode::Three) => game::Game::<{Mode::Three.players()}>::new(opt.random, opt.auto)?.start()?,
            Ok(Mode::Four) => game::Game::<{Mode::Four.players()}>::new(opt.random, opt.auto)?.start()?,
            Ok(Mode::Five) => game::Game::<{Mode::Five.players()}>::new(opt.random, opt.auto)?.start()?,
            Err(e) => eprintln!("{}", e),
        };
    }
    Ok(())
}
