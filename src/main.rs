#![feature(tool_lints)]
extern crate strum;
extern crate itertools;
extern crate rand;
extern crate num_cpus;
#[macro_use] extern crate clap;
#[macro_use] extern crate strum_macros;
#[macro_use] extern crate failure;

pub mod helpers;
pub mod traits;
pub mod errors;
pub mod color;
pub mod trump;
pub mod card;
pub mod deck;
pub mod player;
pub mod role;
pub mod team;
pub mod turn;
pub mod handle;
pub mod mode;
pub mod contract;
pub mod game;

use std::error;
use std::thread;
use clap::{App, Arg};
use strum::IntoEnumIterator;
use crate::mode::Mode;

fn main() -> Result<(), Box<error::Error>> {
    let players_choices = ["3", "4", "5"];
    let default_concurrency = num_cpus::get().to_string();
    let matches = App::new("RTarot")
                     .version("1.0")
                     .about("Tarot simulation")
                     .author("Adrien P. <crunchengine@gmail.com>")
                     .arg(Arg::with_name("players")
                         .short("p")
                         .takes_value(true)
                         .required(true)
                         .help("Number of players")
                         .possible_values(&players_choices)
                         .default_value("4")
                     ).arg(Arg::with_name("random")
                          .short("r")
                          .help("Random playing mode")
                     ).arg(Arg::with_name("auto")
                          .short("a")
                          .help("Auto playing mode when possible")
                     ).arg(Arg::with_name("test")
                          .short("t")
                          .help("Test mode")
                     ).arg(Arg::with_name("concurrency")
                          .short("c")
                          .takes_value(true)
                          .required(true)
                          .help("Concurrency in test mode")
                          .default_value(default_concurrency.as_ref())
                     ).get_matches();
    let players = value_t!(matches.value_of("players"), mode::Mode)?;
    let concurrency = value_t!(matches.value_of("concurrency"), u32)?;
    let random = matches.is_present("random");
    let auto = matches.is_present("auto");
    let test = matches.is_present("test");
    if test {
        let mut children = vec![];
        for _ in 0..concurrency {
            children.push(thread::spawn(move || {
                #[allow(clippy::infinite_iter)]
                Mode::iter().cycle().for_each(helpers::test_game);
            }));
        }
        for child in children {
            let _ = child.join();
        }
    } else {
        let mut game = game::Game::new(players, random, auto);
        loop {
            game.distribute()?;
            game.bidding()?;
            if game.passed() {
                println!("Everyone passed !");
                continue
            }
            game.discard()?;
            while !game.finished() {
                game.play()?;
            }
            game.count_points()?;
            break
        }
        println!("GAME ENDED");
        println!("{}", &game);
    }
    Ok(())
}
