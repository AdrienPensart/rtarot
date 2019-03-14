use crate::mode::Mode;

#[allow(clippy::redundant_closure)]
pub fn read_index() -> Result<usize, text_io::Error> {
    let index: Result<usize, _> = try_read!();
    index
}

pub fn wait_input() {
    use std::io;
    use std::io::prelude::*;
    let mut stdin = io::stdin();
    let _ = stdin.read(&mut [0u8]).unwrap();
}

pub fn test_game(mode: Mode) {
    use crate::game::Game;
    use crate::errors::TarotErrorKind;
    loop {
        let mut game = Game::new(mode, true, true);
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
