extern crate strum;
extern crate rand;
extern crate itertools;
#[macro_use] extern crate log;
#[macro_use] extern crate strum_macros;
#[macro_use] extern crate failure;

use std::error;
use std::thread;
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
    #[clap(value_enum, default_value_t = mode::Mode::default())]
    players: mode::Mode,

    /// Random playing mode
    #[clap(short = 'r', long = "random")]
    random: bool,

    /// Auto playing mode when possible
    #[clap(short = 'a', long = "auto")]
    auto: bool,

    /// Test mode
    #[clap(short = 't', long = "test")]
    test: bool,

    /// Concurrency in test mode, default is number of cpu on this machine
    #[clap(short, default_value_t = thread::available_parallelism().unwrap())]
    concurrency: NonZeroUsize
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = Opts::parse();
    if opt.test {
        let mut children = vec![];
        for _ in 0..opt.concurrency.get() {
            children.push(thread::spawn(move || {
                #[allow(clippy::infinite_iter)]
                Mode::iter().cycle().for_each(|m| {
                    match m {
                        Mode::Three => helpers::test_game::<3>(),
                        Mode::Four => helpers::test_game::<4>(),
                        Mode::Five => helpers::test_game::<5>(),
                    }
                });
            }));
        }
        for child in children {
            let _ = child.join();
        }
    } else {
        match opt.players {
            Mode::Three => game::Game::<3>::new(opt.random, opt.auto).start()?,
            Mode::Four => game::Game::<4>::new(opt.random, opt.auto).start()?,
            Mode::Five => game::Game::<5>::new(opt.random, opt.auto).start()?,
        };
    }
    Ok(())
}
