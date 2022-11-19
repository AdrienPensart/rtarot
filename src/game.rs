use array_init::array_init;
use ordered_float::OrderedFloat;
use std::fmt;

use crate::deck::Deck;
use crate::errors::TarotErrorKind;
use crate::game_distributed::GameDistributed;
use crate::mode::Mode;
use crate::options::Options;
use crate::player::Player;
use crate::player_in_game::PlayerInGame;

#[derive(Debug, Clone)]
pub struct Game<const MODE: usize> {
    options: Options,
    mode: Mode,
    players: [Player; MODE],
    dealer: usize,
}

impl<const MODE: usize> fmt::Display for Game<MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Players : ")?;
        for player in self.players.iter() {
            writeln!(f, "\t{}", player)?;
        }
        Ok(())
    }
}

impl<const MODE: usize> Game<MODE> {
    pub fn new(options: Options) -> Result<Self, TarotErrorKind> {
        let mode: Mode = MODE.try_into()?;
        let players: [Player; MODE] =
            array_init(|i| Player::new(mode.player_name(i).to_string(), mode, options));
        Ok(Self {
            players,
            mode,
            options,
            dealer: 0,
        })
    }
    pub fn is_consistent(&self) -> Result<(), TarotErrorKind> {
        let mut sum = OrderedFloat(0.0);
        for player in &self.players {
            sum += player.score();
        }
        if sum != 0.0 {
            eprintln!("Inconsistent points sum : {}", sum);
            return Err(TarotErrorKind::InvalidScores(sum.to_string()));
        }
        Ok(())
    }
    pub fn mode(&self) -> &Mode {
        &self.mode
    }
    pub fn player(&self, index: usize) -> &Player {
        &self.players[index]
    }
    pub fn player_mut(&mut self, index: usize) -> &mut Player {
        &mut self.players[index]
    }
    pub fn players(&self) -> &[Player; MODE] {
        &self.players
    }
    pub fn start(mut self) -> Result<(), TarotErrorKind> {
        loop {
            let mut game_distributed = self.distribute()?;
            if let Some(mut game_started) = game_distributed.bidding_and_discard()? {
                while !game_started.finished() {
                    game_started.play()?;
                }
                game_started.count_points()?;
                break;
            } else if !self.options.quiet {
                println!("Everyone passed !");
            }
        }
        if !self.options.quiet {
            println!("GAME ENDED");
            println!("{}", self);
        }
        Ok(())
    }
    pub fn distribute(&mut self) -> Result<GameDistributed<MODE>, TarotErrorKind> {
        let mut players_in_game: [PlayerInGame; MODE] =
            array_init(|_| PlayerInGame::new(self.mode, self.options));

        let mut new_deck = Deck::random();
        let mut dog = new_deck.give(self.mode.dog_size());
        dog.sort();
        for player in players_in_game.iter_mut() {
            let buffer = new_deck.give(self.mode.cards_per_player());
            player.extend_hand(&buffer)
        }

        for player in players_in_game.iter() {
            if player.petit_sec() {
                // RULE: PetitSec cancel the game
                return Err(TarotErrorKind::PetitSec);
            }
        }

        let game_distributed = GameDistributed::new(self, dog, players_in_game, self.options);
        Ok(game_distributed)
    }
    pub fn rotate(&mut self) {
        if self.dealer == self.players.len() - 1 {
            self.dealer = 0;
        } else {
            self.dealer += 1;
        }
        self.players.rotate_left(self.dealer);
    }
}

#[test]
fn game_tests() {
    use crate::helpers::test_game;
    use crate::mode::Mode;
    let options = Options {
        random: true,
        ..Options::default()
    };
    test_game::<{ Mode::Three.players() }>(options).unwrap();
    test_game::<{ Mode::Four.players() }>(options).unwrap();
    test_game::<{ Mode::Five.players() }>(options).unwrap();
}
