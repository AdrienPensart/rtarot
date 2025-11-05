use array_init::try_array_init;
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

pub fn launch(mode: Mode, options: Options, deals: u64) -> Result<(), TarotErrorKind> {
    if mode == Mode::Three {
        Game::<{ Mode::Three.players() }>::new(options)?.start(deals)?;
        return Ok(());
    } else if mode == Mode::Four {
        Game::<{ Mode::Four.players() }>::new(options)?.start(deals)?;
        return Ok(());
    } else if mode == Mode::Five {
        Game::<{ Mode::Five.players() }>::new(options)?.start(deals)?;
        return Ok(());
    }
    Ok(())
}

impl<const MODE: usize> fmt::Display for Game<MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Players : ")?;
        for player in &self.players {
            writeln!(f, "\t{player}")?;
        }
        Ok(())
    }
}

impl<const MODE: usize> Game<MODE> {
    pub fn new(options: Options) -> Result<Self, TarotErrorKind> {
        let mode: Mode = MODE.try_into()?;
        let players: [Player; MODE] = try_array_init(|i| -> Result<Player, TarotErrorKind> {
            let name = mode.player_name(i)?;
            let random = options.test || name != "South";
            let player_options = Options { random, ..options };
            Ok(Player::new(name.to_string(), mode, player_options))
        })?;
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
            eprintln!("Inconsistent points sum : {sum}");
            return Err(TarotErrorKind::InvalidScores(sum.to_string()));
        }
        Ok(())
    }
    #[must_use]
    pub const fn mode(&self) -> &Mode {
        &self.mode
    }
    pub fn player(&self, index: usize) -> Result<&Player, TarotErrorKind> {
        self.players
            .get(index)
            .ok_or(TarotErrorKind::NoPlayer(index))
    }
    pub fn player_mut(&mut self, index: usize) -> Result<&mut Player, TarotErrorKind> {
        self.players
            .get_mut(index)
            .ok_or(TarotErrorKind::NoPlayer(index))
    }
    #[must_use]
    pub const fn players(&self) -> &[Player; MODE] {
        &self.players
    }
    pub fn start(mut self, mut deals: u64) -> Result<(), TarotErrorKind> {
        while deals > 0 {
            if !self.options.quiet {
                println!("Deals left : {deals}");
            }
            if let Ok(Some(mut game_distributed)) = self.distribute() {
                if let Some(mut game_started) = game_distributed.bidding_and_discard()? {
                    while !game_started.finished() {
                        game_started.play()?;
                    }
                    game_started.count_points()?;
                    deals -= 1;
                } else if !self.options.quiet {
                    println!("Everyone passed !");
                }
            }
            self.rotate_dealer();
        }
        if !self.options.quiet {
            println!("GAME ENDED");
            println!("{self}");
        }
        Ok(())
    }
    fn distribute(&'_ mut self) -> Result<Option<GameDistributed<'_, MODE>>, TarotErrorKind> {
        let mut players_in_game: [PlayerInGame; MODE] =
            try_array_init(|i| -> Result<PlayerInGame, TarotErrorKind> {
                Ok(PlayerInGame::new(self.mode, *self.player(i)?.options()))
            })?;

        let mut new_deck = Deck::random();
        let mut dog = new_deck.give(self.mode.dog_size());
        dog.sort();
        for player in &mut players_in_game {
            let buffer = new_deck.give(self.mode.cards_per_player());
            player.extend_hand(&buffer);
        }

        for player in &players_in_game {
            if player.petit_sec() {
                if !self.options.quiet {
                    dbg!("Petit sec, cancel the game");
                }
                return Ok(None);
            }
        }
        Ok(Some(GameDistributed::new(
            self,
            self.options,
            dog,
            players_in_game,
        )))
    }
    pub fn rotate_at(&mut self, index: usize) {
        self.players.rotate_left(index);
    }
    fn rotate_dealer(&mut self) {
        if self.dealer == self.players.len() - 1 {
            self.dealer = 0;
        } else {
            self.dealer += 1;
        }
        self.rotate_at(self.dealer);
    }
}

#[test]
fn game_tests() {
    use crate::mode::Mode;
    use strum::IntoEnumIterator;
    let options = Options {
        random: true,
        test: true,
        auto: true,
        quiet: true,
        no_slam: false,
        attack: false,
    };
    for mode in Mode::iter() {
        assert_eq!(launch(mode, options, 1), Ok(()));
    }
}
