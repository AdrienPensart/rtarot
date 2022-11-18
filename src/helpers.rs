use crate::errors::TarotErrorKind;
use crate::options::Options;
use std::io;

#[allow(clippy::redundant_closure)]
pub fn read_index() -> usize {
    let mut input = String::new();
    loop {
        if io::stdin().read_line(&mut input).is_ok() {
            if let Ok(output) = input.trim().parse::<usize>() {
                return output;
            }
        }
    }
}

// pub fn wait_input() {
//     use std::io::prelude::*;
//     let mut stdin = io::stdin();
//     let _ = stdin.read(&mut [0u8]).unwrap();
// }

pub fn test_game<const MODE: usize>(options: Options) -> Result<(), TarotErrorKind> {
    use crate::game::Game;
    loop {
        let mut game: Game<MODE> = Game::new(options)?;
        match game.distribute() {
            Err(fail) => {
                if fail == TarotErrorKind::PetitSec {
                    continue;
                } else {
                    return Err(fail);
                }
            }
            Ok(mut game_distributed) => {
                if let Some(mut game_started) = game_distributed.bidding_and_discard()? {
                    while !game_started.finished() {
                        game_started.play()?;
                    }
                    game_started.count_points()?;
                }
            }
        };
        return Ok(());
    }
}

pub fn binomial(mut n: usize, mut k: usize) -> usize {
    if k > n {
        return 0;
    }
    if k > (n / 2) {
        k = n - k;
    }
    let mut result = 1;
    for d in 1..=k {
        result *= n;
        n -= 1;
        result /= d;
    }
    result
}

#[test]
fn helpers_tests() {
    assert_eq!(binomial(24, 6), 134_596);
}
