use std::io;

#[allow(clippy::redundant_closure)]
pub fn read_index() -> usize {
    let mut input = String::new();
    loop {
        if io::stdin().read_line(&mut input).is_ok() {
            if let Ok(output) = input.trim().parse::<usize>(){
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

pub fn test_game<const MODE: usize>() {
    use crate::game::Game;
    use crate::errors::TarotErrorKind;
    loop {
        let mut game: Game<MODE> = Game::new(true, true);
        if let Err(fail) = game.distribute() {
            if let Some(cause) = fail.find_root_cause().downcast_ref::<TarotErrorKind>() {
               if cause == &TarotErrorKind::PetitSec {
                   continue
               } else {
                   panic!("Error is not PetitSec")
               }
            }
        }
        assert!(game.bidding().is_ok());
        if game.passed() {
            continue
        }
        assert!(game.discard().is_ok());
        while !game.finished() {
            assert!(game.play().is_ok());
        }
        assert!(game.count_points().is_ok());
        break
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
