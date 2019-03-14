extern crate strum;
extern crate itertools;
extern crate rand;
extern crate ctrlc;
extern crate rayon;
#[macro_use] extern crate clap;
#[macro_use] extern crate strum_macros;
#[macro_use] extern crate failure;
#[macro_use] extern crate text_io;
#[macro_use] extern crate log;

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
//use rayon::prelude::*;
use rayon::current_num_threads;
use crate::mode::Mode;

fn main() -> Result<(), Box<error::Error>> {
    //use std::sync::atomic::{AtomicBool, Ordering};
    //use std::sync::Arc;
    //let running = Arc::new(AtomicBool::new(true));
    //let r = running.clone();
    //ctrlc::set_handler(move || {
    //    r.store(false, Ordering::SeqCst);
    //}).expect("Error setting Ctrl-C handler");
    //while running.load(Ordering::SeqCst) {

    let players_choices = ["3", "4", "5"];
    let default_concurrency = current_num_threads().to_string();
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
                Mode::iter().cycle().for_each(|m| helpers::test_game(m));
            }));
        }
        for child in children {
            let _ = child.join();
        }
        return Ok(());
    }

    debug!("players:{}", players as usize);
    debug!("{:?}", &matches);
    let mut game = game::Game::new(players, random, auto);

    loop {
        debug!("Distribute phase");
        game.distribute()?;
        debug!("Auctions phase");
        game.bidding()?;
        debug!("After auctions : {}", &game);
        if game.passed() {
            println!("Everyone passed !");
            continue
        }
        debug!("Discard phase");
        game.discard()?;
        debug!("Play phase");
        while !game.finished() {
            game.play()?;
        }
        debug!("Count points phase");
        game.count_points()?;
        break
    }
    println!("GAME ENDED");
    println!("{}", &game);
    Ok(())
}
