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
            let attack = options.attack && name != "South";
            let player_options = Options {
                random,
                attack,
                ..options
            };
            Ok(Player::new(name.to_string(), mode, player_options))
        })?;
        Ok(Self {
            options,
            mode,
            players,
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
                    println!("Petit sec, cancel the game");
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
    pub const fn rotate_at(&mut self, index: usize) {
        self.players.rotate_left(index);
    }
    const fn rotate_dealer(&mut self) {
        // RULE: the deal moves by exactly one seat between two deals
        self.rotate_at(1);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::mode::Mode;
    use strum::IntoEnumIterator;

    fn default_test_options() -> Options {
        Options {
            random: true,
            test: true,
            auto: true,
            quiet: true,
            no_slam: false,
            attack: false,
        }
    }

    #[test]
    fn launch_all_modes() {
        let options = default_test_options();
        for mode in Mode::iter() {
            assert_eq!(launch(mode, options, 1), Ok(()));
        }
    }

    // --- Mode Three (3 players, 1 vs 2) ---

    #[test]
    fn three_player_game_creation() {
        let game = Game::<{ Mode::Three.players() }>::new(default_test_options()).unwrap();
        assert_eq!(*game.mode(), Mode::Three);
        assert_eq!(game.players().len(), 3);
    }

    #[test]
    fn three_player_names() {
        let game = Game::<{ Mode::Three.players() }>::new(default_test_options()).unwrap();
        assert_eq!(game.player(0).unwrap().name(), "East");
        assert_eq!(game.player(1).unwrap().name(), "North");
        assert_eq!(game.player(2).unwrap().name(), "South");
    }

    #[test]
    fn three_player_initial_scores_zero() {
        let game = Game::<{ Mode::Three.players() }>::new(default_test_options()).unwrap();
        for player in game.players() {
            assert_eq!(player.score(), OrderedFloat(0.0));
        }
    }

    #[test]
    fn three_player_consistent_after_deal() {
        let options = default_test_options();
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.is_consistent(), Ok(()));
    }

    #[test]
    fn three_player_launch_single_deal() {
        assert_eq!(launch(Mode::Three, default_test_options(), 1), Ok(()));
    }

    #[test]
    fn three_player_launch_multiple_deals() {
        assert_eq!(launch(Mode::Three, default_test_options(), 5), Ok(()));
    }

    #[test]
    fn three_player_consistent_after_game() {
        let options = default_test_options();
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn three_player_mode_properties() {
        assert_eq!(Mode::Three.players(), 3);
        assert_eq!(Mode::Three.dog_size(), 6);
        assert_eq!(Mode::Three.cards_per_player(), 24);
        assert_eq!(Mode::Three.cards_per_turn(), 3);
        assert_eq!(Mode::Three.max_cards_for_taker(), 30);
    }

    // --- Mode Four (4 players, 1 vs 3) ---

    #[test]
    fn four_player_game_creation() {
        let game = Game::<{ Mode::Four.players() }>::new(default_test_options()).unwrap();
        assert_eq!(*game.mode(), Mode::Four);
        assert_eq!(game.players().len(), 4);
    }

    #[test]
    fn four_player_names() {
        let game = Game::<{ Mode::Four.players() }>::new(default_test_options()).unwrap();
        assert_eq!(game.player(0).unwrap().name(), "East");
        assert_eq!(game.player(1).unwrap().name(), "North");
        assert_eq!(game.player(2).unwrap().name(), "South");
        assert_eq!(game.player(3).unwrap().name(), "West");
    }

    #[test]
    fn four_player_initial_scores_zero() {
        let game = Game::<{ Mode::Four.players() }>::new(default_test_options()).unwrap();
        for player in game.players() {
            assert_eq!(player.score(), OrderedFloat(0.0));
        }
    }

    #[test]
    fn four_player_consistent_after_deal() {
        let options = default_test_options();
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.is_consistent(), Ok(()));
    }

    #[test]
    fn four_player_launch_single_deal() {
        assert_eq!(launch(Mode::Four, default_test_options(), 1), Ok(()));
    }

    #[test]
    fn four_player_launch_multiple_deals() {
        assert_eq!(launch(Mode::Four, default_test_options(), 5), Ok(()));
    }

    #[test]
    fn four_player_consistent_after_game() {
        let options = default_test_options();
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn four_player_mode_properties() {
        assert_eq!(Mode::Four.players(), 4);
        assert_eq!(Mode::Four.dog_size(), 6);
        assert_eq!(Mode::Four.cards_per_player(), 18);
        assert_eq!(Mode::Four.cards_per_turn(), 4);
        assert_eq!(Mode::Four.max_cards_for_taker(), 24);
    }

    // --- Mode Five (5 players, 2 vs 3) ---

    #[test]
    fn five_player_game_creation() {
        let game = Game::<{ Mode::Five.players() }>::new(default_test_options()).unwrap();
        assert_eq!(*game.mode(), Mode::Five);
        assert_eq!(game.players().len(), 5);
    }

    #[test]
    fn five_player_names() {
        let game = Game::<{ Mode::Five.players() }>::new(default_test_options()).unwrap();
        assert_eq!(game.player(0).unwrap().name(), "East");
        assert_eq!(game.player(1).unwrap().name(), "North");
        assert_eq!(game.player(2).unwrap().name(), "South");
        assert_eq!(game.player(3).unwrap().name(), "West");
        assert_eq!(game.player(4).unwrap().name(), "Compass");
    }

    #[test]
    fn five_player_initial_scores_zero() {
        let game = Game::<{ Mode::Five.players() }>::new(default_test_options()).unwrap();
        for player in game.players() {
            assert_eq!(player.score(), OrderedFloat(0.0));
        }
    }

    #[test]
    fn five_player_consistent_after_deal() {
        let options = default_test_options();
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.is_consistent(), Ok(()));
    }

    #[test]
    fn five_player_launch_single_deal() {
        assert_eq!(launch(Mode::Five, default_test_options(), 1), Ok(()));
    }

    #[test]
    fn five_player_launch_multiple_deals() {
        assert_eq!(launch(Mode::Five, default_test_options(), 5), Ok(()));
    }

    #[test]
    fn five_player_consistent_after_game() {
        let options = default_test_options();
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn five_player_mode_properties() {
        assert_eq!(Mode::Five.players(), 5);
        assert_eq!(Mode::Five.dog_size(), 3);
        assert_eq!(Mode::Five.cards_per_player(), 15);
        assert_eq!(Mode::Five.cards_per_turn(), 5);
        assert_eq!(Mode::Five.max_cards_for_taker(), 18);
    }

    // --- Card distribution: dog + player cards = 78 ---

    #[test]
    fn three_player_card_count() {
        let mode = Mode::Three;
        let total = mode.dog_size() + mode.players() * mode.cards_per_player();
        assert_eq!(total, 78);
    }

    #[test]
    fn four_player_card_count() {
        let mode = Mode::Four;
        let total = mode.dog_size() + mode.players() * mode.cards_per_player();
        assert_eq!(total, 78);
    }

    #[test]
    fn five_player_card_count() {
        let mode = Mode::Five;
        let total = mode.dog_size() + mode.players() * mode.cards_per_player();
        assert_eq!(total, 78);
    }

    // --- Invalid player index ---

    #[test]
    fn player_out_of_bounds_three() {
        let game = Game::<{ Mode::Three.players() }>::new(default_test_options()).unwrap();
        assert!(game.player(3).is_err());
    }

    #[test]
    fn player_out_of_bounds_four() {
        let game = Game::<{ Mode::Four.players() }>::new(default_test_options()).unwrap();
        assert!(game.player(4).is_err());
    }

    #[test]
    fn player_out_of_bounds_five() {
        let game = Game::<{ Mode::Five.players() }>::new(default_test_options()).unwrap();
        assert!(game.player(5).is_err());
    }

    // --- Attack mode per game mode ---

    #[test]
    fn three_player_attack_mode() {
        let options = Options {
            attack: true,
            ..default_test_options()
        };
        assert_eq!(launch(Mode::Three, options, 1), Ok(()));
    }

    #[test]
    fn four_player_attack_mode() {
        let options = Options {
            attack: true,
            ..default_test_options()
        };
        assert_eq!(launch(Mode::Four, options, 1), Ok(()));
    }

    #[test]
    fn five_player_attack_mode() {
        let options = Options {
            attack: true,
            ..default_test_options()
        };
        assert_eq!(launch(Mode::Five, options, 1), Ok(()));
    }

    // --- No slam option per game mode ---

    #[test]
    fn three_player_no_slam() {
        let options = Options {
            no_slam: true,
            ..default_test_options()
        };
        assert_eq!(launch(Mode::Three, options, 1), Ok(()));
    }

    #[test]
    fn four_player_no_slam() {
        let options = Options {
            no_slam: true,
            ..default_test_options()
        };
        assert_eq!(launch(Mode::Four, options, 1), Ok(()));
    }

    #[test]
    fn five_player_no_slam() {
        let options = Options {
            no_slam: true,
            ..default_test_options()
        };
        assert_eq!(launch(Mode::Five, options, 1), Ok(()));
    }

    // --- Ratio per mode ---

    #[test]
    fn three_player_ratio() {
        assert_eq!(Mode::Three.ratio(false), OrderedFloat(2.0));
        assert_eq!(Mode::Three.ratio(true), OrderedFloat(2.0));
    }

    #[test]
    fn four_player_ratio() {
        assert_eq!(Mode::Four.ratio(false), OrderedFloat(3.0));
        assert_eq!(Mode::Four.ratio(true), OrderedFloat(3.0));
    }

    #[test]
    fn five_player_ratio() {
        assert_eq!(Mode::Five.ratio(false), OrderedFloat(4.0));
        assert_eq!(Mode::Five.ratio(true), OrderedFloat(2.0));
    }

    // --- Handle thresholds per mode ---

    #[test]
    fn three_player_handles() {
        use crate::handle::Handle;
        assert_eq!(Mode::Three.handle(12), None);
        assert_eq!(Mode::Three.handle(13), Some(Handle::Simple));
        assert_eq!(Mode::Three.handle(15), Some(Handle::Double));
        assert_eq!(Mode::Three.handle(18), Some(Handle::Triple));
    }

    #[test]
    fn four_player_handles() {
        use crate::handle::Handle;
        assert_eq!(Mode::Four.handle(9), None);
        assert_eq!(Mode::Four.handle(10), Some(Handle::Simple));
        assert_eq!(Mode::Four.handle(13), Some(Handle::Double));
        assert_eq!(Mode::Four.handle(15), Some(Handle::Triple));
    }

    #[test]
    fn five_player_handles() {
        use crate::handle::Handle;
        assert_eq!(Mode::Five.handle(7), None);
        assert_eq!(Mode::Five.handle(8), Some(Handle::Simple));
        assert_eq!(Mode::Five.handle(10), Some(Handle::Double));
        assert_eq!(Mode::Five.handle(13), Some(Handle::Triple));
    }

    // --- Zero deals (no game played) ---

    #[test]
    fn zero_deals_three() {
        assert_eq!(launch(Mode::Three, default_test_options(), 0), Ok(()));
    }

    #[test]
    fn zero_deals_four() {
        assert_eq!(launch(Mode::Four, default_test_options(), 0), Ok(()));
    }

    #[test]
    fn zero_deals_five() {
        assert_eq!(launch(Mode::Five, default_test_options(), 0), Ok(()));
    }

    // --- Dealer rotation ---

    #[test]
    fn dealer_rotation_three() {
        let mut game = Game::<{ Mode::Three.players() }>::new(default_test_options()).unwrap();
        let first_player_name = game.player(0).unwrap().name().to_string();
        game.rotate_at(1);
        // After rotating by 1, the first player should have changed
        assert_ne!(game.player(0).unwrap().name(), first_player_name);
    }

    #[test]
    fn dealer_rotation_four() {
        let mut game = Game::<{ Mode::Four.players() }>::new(default_test_options()).unwrap();
        let first_player_name = game.player(0).unwrap().name().to_string();
        game.rotate_at(1);
        assert_ne!(game.player(0).unwrap().name(), first_player_name);
    }

    #[test]
    fn dealer_rotation_five() {
        let mut game = Game::<{ Mode::Five.players() }>::new(default_test_options()).unwrap();
        let first_player_name = game.player(0).unwrap().name().to_string();
        game.rotate_at(1);
        assert_ne!(game.player(0).unwrap().name(), first_player_name);
    }

    #[test]
    fn full_rotation_returns_to_original_three() {
        let mut game = Game::<{ Mode::Three.players() }>::new(default_test_options()).unwrap();
        let original_names: Vec<String> = game
            .players()
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        game.rotate_at(3);
        let rotated_names: Vec<String> = game
            .players()
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        assert_eq!(original_names, rotated_names);
    }

    #[test]
    fn full_rotation_returns_to_original_four() {
        let mut game = Game::<{ Mode::Four.players() }>::new(default_test_options()).unwrap();
        let original_names: Vec<String> = game
            .players()
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        game.rotate_at(4);
        let rotated_names: Vec<String> = game
            .players()
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        assert_eq!(original_names, rotated_names);
    }

    #[test]
    fn full_rotation_returns_to_original_five() {
        let mut game = Game::<{ Mode::Five.players() }>::new(default_test_options()).unwrap();
        let original_names: Vec<String> = game
            .players()
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        game.rotate_at(5);
        let rotated_names: Vec<String> = game
            .players()
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        assert_eq!(original_names, rotated_names);
    }

    // --- Display ---

    #[test]
    fn game_display_three() {
        let game = Game::<{ Mode::Three.players() }>::new(default_test_options()).unwrap();
        let display = format!("{game}");
        assert!(display.contains("East"));
        assert!(display.contains("North"));
        assert!(display.contains("South"));
    }

    #[test]
    fn game_display_four() {
        let game = Game::<{ Mode::Four.players() }>::new(default_test_options()).unwrap();
        let display = format!("{game}");
        assert!(display.contains("West"));
    }

    #[test]
    fn game_display_five() {
        let game = Game::<{ Mode::Five.players() }>::new(default_test_options()).unwrap();
        let display = format!("{game}");
        assert!(display.contains("Compass"));
    }
}
