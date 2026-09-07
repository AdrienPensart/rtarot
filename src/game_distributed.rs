use crate::contract::Contract;
use crate::deck::Deck;
use crate::errors::TarotErrorKind;
use crate::game::Game;
use crate::game_started::GameStarted;
use crate::options::Options;
use crate::player::Player;
use crate::player_in_game::PlayerInGame;
use crate::role::Role;
use crate::team::Team;
use derive_new::new;
use std::fmt;
use strum::IntoEnumIterator;

#[derive(new)]
pub struct GameDistributed<'a, const MODE: usize> {
    game: &'a mut Game<MODE>,
    options: Options,
    dog: Deck,
    players_in_game: [PlayerInGame; MODE],
}

impl<'a, const MODE: usize> GameDistributed<'a, MODE> {
    pub const fn game(&mut self) -> &mut Game<MODE> {
        self.game
    }
    pub const fn players_and_their_game_mut(
        &mut self,
    ) -> (&[Player; MODE], &mut [PlayerInGame; MODE]) {
        (self.game.players(), &mut self.players_in_game)
    }
    pub fn player(&self, index: usize) -> Result<&Player, TarotErrorKind> {
        self.game.player(index)
    }
    pub fn player_mut(&mut self, index: usize) -> Result<&mut Player, TarotErrorKind> {
        self.game.player_mut(index)
    }
    pub fn player_and_his_game(
        &self,
        index: usize,
    ) -> Result<(&Player, &PlayerInGame), TarotErrorKind> {
        let (Ok(player), Some(player_in_game)) =
            (self.game.player(index), self.players_in_game.get(index))
        else {
            return Err(TarotErrorKind::NoPlayer(index));
        };
        Ok((player, player_in_game))
    }
    pub fn player_and_his_game_mut(
        &mut self,
        index: usize,
    ) -> Result<(&Player, &mut PlayerInGame), TarotErrorKind> {
        let (Ok(player), Some(player_in_game)) =
            (self.game.player(index), self.players_in_game.get_mut(index))
        else {
            return Err(TarotErrorKind::NoPlayer(index));
        };
        Ok((player, player_in_game))
    }
    pub fn finished(&self) -> bool {
        self.players_in_game.iter().all(PlayerInGame::last_turn)
    }
    pub const fn rotate_at(&mut self, index: usize) {
        self.players_in_game.rotate_left(index);
        self.game.rotate_at(index);
    }
    pub fn bidding_and_discard(
        &'a mut self,
    ) -> Result<Option<GameStarted<'a, MODE>>, TarotErrorKind> {
        let quiet = self.options.quiet;
        let mut contracts: Vec<Contract> = Contract::iter().collect();
        let mut taker_index: Option<usize> = None;
        let mut contract: Option<Contract> = None;

        // RULE: bidding goes around once, each player either passes or overbids
        for (current_player_index, current_player_in_game) in
            self.players_in_game.iter_mut().enumerate()
        {
            let current_player = self.game.player(current_player_index)?;
            let Some(player_contract) =
                current_player_in_game.choose_contract_among(current_player, &contracts)?
            else {
                continue;
            };
            if !quiet {
                println!(
                    "Player {} has chosen contract {player_contract}",
                    current_player.name()
                );
            }
            contracts.retain(|other_contract| {
                other_contract.multiplier() > player_contract.multiplier()
            });
            taker_index = Some(current_player_index);
            contract = Some(player_contract);
        }
        let (Some(contract), Some(taker_index)) = (contract, taker_index) else {
            return Ok(None);
        };

        let Some(taker) = self.players_in_game.get(taker_index) else {
            return Err(TarotErrorKind::NoTaker(taker_index));
        };
        let callee = taker.call()?;
        for (current_player_index, current_player) in self.players_in_game.iter_mut().enumerate() {
            current_player.set_callee(callee);
            if current_player_index == taker_index {
                current_player.set_team(Team::Attack);
                current_player.set_role(Role::Taker);
            } else if callee.is_some_and(|card| current_player.has(&card)) {
                current_player.set_team(Team::Attack);
                current_player.set_role(Role::Ally);
            } else {
                current_player.set_team(Team::Defense);
                current_player.set_role(Role::Defenser);
            }
        }

        // RULE: the slam is announced by the taker, before the dog is turned over
        let Some(taker_in_game) = self.players_in_game.get_mut(taker_index) else {
            return Err(TarotErrorKind::NoTaker(taker_index));
        };
        let slam_announced = taker_in_game.announce_slam()?;

        match contract {
            Contract::GardeSans => {
                // RULE: the dog belongs to the attack but is never seen
                if !quiet {
                    println!("Attacker keeps dog because GardeSans");
                }
                let Some(taker_in_game) = self.players_in_game.get_mut(taker_index) else {
                    return Err(TarotErrorKind::NoTaker(taker_index));
                };
                taker_in_game.set_discard(&self.dog);
            }
            Contract::GardeContre => {
                // RULE: the dog goes to the defense and is never seen
                let Some(first_defenser_index) = (0..MODE).find(|index| {
                    self.players_in_game
                        .get(*index)
                        .is_some_and(|player| !player.is_attack())
                }) else {
                    return Err(TarotErrorKind::NoDefenser(0));
                };
                if !quiet {
                    println!("Attacker gives dog to first defenser because GardeContre");
                }
                let Some(defenser) = self.players_in_game.get_mut(first_defenser_index) else {
                    return Err(TarotErrorKind::NoDefenser(first_defenser_index));
                };
                defenser.set_discard(&self.dog);
            }
            Contract::Petite | Contract::Garde => {
                if !quiet {
                    println!("In the dog, there was : {}", self.dog);
                    println!(
                        "Taker {} received the dog",
                        self.game.player(taker_index)?.name()
                    );
                }
                let Some(taker_in_game) = self.players_in_game.get_mut(taker_index) else {
                    return Err(TarotErrorKind::NoTaker(taker_index));
                };
                taker_in_game.extend_hand(&self.dog);
                taker_in_game.discard()?;
            }
        }

        // RULE: the player who announced a slam must start, so he is rotated to the first seat
        if slam_announced {
            if !quiet {
                println!(
                    "Chelem announced so {} must start.",
                    self.game.player(taker_index)?.name()
                );
            }
            self.rotate_at(taker_index);
        }

        let game_started = GameStarted::new(self, contract, self.options);
        Ok(Some(game_started))
    }
}

impl<const MODE: usize> fmt::Display for GameDistributed<'_, MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Game {} with dog {}", self.game.mode(), self.dog)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::constants::MAX_CARDS;
    use crate::deck::Deck;
    use crate::mode::Mode;
    use array_init::try_array_init;

    fn test_options() -> Options {
        Options {
            random: true,
            test: true,
            auto: true,
            quiet: true,
            no_slam: false,
            attack: false,
        }
    }

    fn distribute_game<const MODE: usize>(game: &mut Game<MODE>) -> GameDistributed<'_, MODE> {
        let mode = *game.mode();
        let options = *game.player(0).unwrap().options();
        let mut players_in_game: [PlayerInGame; MODE] =
            try_array_init(|i| -> Result<PlayerInGame, TarotErrorKind> {
                Ok(PlayerInGame::new(mode, *game.player(i).unwrap().options()))
            })
            .unwrap();

        let mut deck = Deck::random();
        let mut dog = deck.give(mode.dog_size());
        dog.sort();
        for player in &mut players_in_game {
            let buffer = deck.give(mode.cards_per_player());
            player.extend_hand(&buffer);
        }
        GameDistributed::new(game, options, dog, players_in_game)
    }

    // --- Three-player mode ---

    #[test]
    fn three_player_distribute_dog_size() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert_eq!(gd.dog.len(), Mode::Three.dog_size());
    }

    #[test]
    fn three_player_distribute_total_cards() {
        let mode = Mode::Three;
        let expected = mode.dog_size() + mode.players() * mode.cards_per_player();
        assert_eq!(expected, MAX_CARDS);
    }

    #[test]
    fn three_player_player_access() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert!(gd.player(0).is_ok());
        assert!(gd.player(1).is_ok());
        assert!(gd.player(2).is_ok());
        assert!(gd.player(3).is_err());
    }

    #[test]
    fn three_player_player_and_game_access() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        for i in 0..3 {
            let (player, _pig) = gd.player_and_his_game(i).unwrap();
            assert!(!player.name().is_empty());
        }
        assert!(gd.player_and_his_game(3).is_err());
    }

    #[test]
    fn three_player_not_finished_initially() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert!(!gd.finished());
    }

    #[test]
    fn three_player_rotate_at() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let original_name = gd.player(0).unwrap().name().to_string();
        gd.rotate_at(1);
        assert_ne!(gd.player(0).unwrap().name(), original_name);
    }

    #[test]
    fn three_player_full_rotation() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let original_name = gd.player(0).unwrap().name().to_string();
        gd.rotate_at(3);
        assert_eq!(gd.player(0).unwrap().name(), original_name);
    }

    #[test]
    fn three_player_bidding_produces_game_or_pass() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let result = gd.bidding_and_discard();
        assert!(result.is_ok());
        // Either a game started or everyone passed — both are valid
    }

    #[test]
    fn three_player_bidding_attack_mode() {
        let options = Options {
            attack: true,
            ..test_options()
        };
        let mut game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        let mut gd = distribute_game(&mut game);
        let result = gd.bidding_and_discard();
        assert!(result.is_ok());
    }

    // --- Four-player mode ---

    #[test]
    fn four_player_distribute_dog_size() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert_eq!(gd.dog.len(), Mode::Four.dog_size());
    }

    #[test]
    fn four_player_distribute_total_cards() {
        let mode = Mode::Four;
        let expected = mode.dog_size() + mode.players() * mode.cards_per_player();
        assert_eq!(expected, MAX_CARDS);
    }

    #[test]
    fn four_player_player_access() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert!(gd.player(0).is_ok());
        assert!(gd.player(1).is_ok());
        assert!(gd.player(2).is_ok());
        assert!(gd.player(3).is_ok());
        assert!(gd.player(4).is_err());
    }

    #[test]
    fn four_player_player_and_game_access() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        for i in 0..4 {
            let (player, _pig) = gd.player_and_his_game(i).unwrap();
            assert!(!player.name().is_empty());
        }
        assert!(gd.player_and_his_game(4).is_err());
    }

    #[test]
    fn four_player_not_finished_initially() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert!(!gd.finished());
    }

    #[test]
    fn four_player_rotate_at() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let original_name = gd.player(0).unwrap().name().to_string();
        gd.rotate_at(1);
        assert_ne!(gd.player(0).unwrap().name(), original_name);
    }

    #[test]
    fn four_player_full_rotation() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let original_name = gd.player(0).unwrap().name().to_string();
        gd.rotate_at(4);
        assert_eq!(gd.player(0).unwrap().name(), original_name);
    }

    #[test]
    fn four_player_bidding_produces_game_or_pass() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let result = gd.bidding_and_discard();
        assert!(result.is_ok());
    }

    #[test]
    fn four_player_bidding_attack_mode() {
        let options = Options {
            attack: true,
            ..test_options()
        };
        let mut game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        let mut gd = distribute_game(&mut game);
        let result = gd.bidding_and_discard();
        assert!(result.is_ok());
    }

    // --- Five-player mode ---

    #[test]
    fn five_player_distribute_dog_size() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert_eq!(gd.dog.len(), Mode::Five.dog_size());
    }

    #[test]
    fn five_player_distribute_total_cards() {
        let mode = Mode::Five;
        let expected = mode.dog_size() + mode.players() * mode.cards_per_player();
        assert_eq!(expected, MAX_CARDS);
    }

    #[test]
    fn five_player_player_access() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert!(gd.player(0).is_ok());
        assert!(gd.player(1).is_ok());
        assert!(gd.player(2).is_ok());
        assert!(gd.player(3).is_ok());
        assert!(gd.player(4).is_ok());
        assert!(gd.player(5).is_err());
    }

    #[test]
    fn five_player_player_and_game_access() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        for i in 0..5 {
            let (player, _pig) = gd.player_and_his_game(i).unwrap();
            assert!(!player.name().is_empty());
        }
        assert!(gd.player_and_his_game(5).is_err());
    }

    #[test]
    fn five_player_not_finished_initially() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let gd = distribute_game(&mut game);
        assert!(!gd.finished());
    }

    #[test]
    fn five_player_rotate_at() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let original_name = gd.player(0).unwrap().name().to_string();
        gd.rotate_at(1);
        assert_ne!(gd.player(0).unwrap().name(), original_name);
    }

    #[test]
    fn five_player_full_rotation() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let original_name = gd.player(0).unwrap().name().to_string();
        gd.rotate_at(5);
        assert_eq!(gd.player(0).unwrap().name(), original_name);
    }

    #[test]
    fn five_player_bidding_produces_game_or_pass() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let result = gd.bidding_and_discard();
        assert!(result.is_ok());
    }

    #[test]
    fn five_player_bidding_attack_mode() {
        let options = Options {
            attack: true,
            ..test_options()
        };
        let mut game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        let mut gd = distribute_game(&mut game);
        let result = gd.bidding_and_discard();
        assert!(result.is_ok());
    }

    // --- Cross-mode: rotation preserves player count ---

    #[test]
    fn rotate_preserves_player_count_three() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        gd.rotate_at(2);
        assert_eq!(gd.players_in_game.len(), 3);
        assert!(gd.player(0).is_ok());
        assert!(gd.player(2).is_ok());
    }

    #[test]
    fn rotate_preserves_player_count_four() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        gd.rotate_at(3);
        assert_eq!(gd.players_in_game.len(), 4);
        assert!(gd.player(0).is_ok());
        assert!(gd.player(3).is_ok());
    }

    #[test]
    fn rotate_preserves_player_count_five() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        gd.rotate_at(4);
        assert_eq!(gd.players_in_game.len(), 5);
        assert!(gd.player(0).is_ok());
        assert!(gd.player(4).is_ok());
    }

    // --- Player mut access ---

    #[test]
    fn three_player_mut_access() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        assert!(gd.player_mut(0).is_ok());
        assert!(gd.player_mut(2).is_ok());
        assert!(gd.player_mut(3).is_err());
    }

    #[test]
    fn four_player_mut_access() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        assert!(gd.player_mut(0).is_ok());
        assert!(gd.player_mut(3).is_ok());
        assert!(gd.player_mut(4).is_err());
    }

    #[test]
    fn five_player_mut_access() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        assert!(gd.player_mut(0).is_ok());
        assert!(gd.player_mut(4).is_ok());
        assert!(gd.player_mut(5).is_err());
    }

    // --- Player and game mut access ---

    #[test]
    fn three_player_and_game_mut_access() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        for i in 0..3 {
            let (player, _pig) = gd.player_and_his_game_mut(i).unwrap();
            assert!(!player.name().is_empty());
        }
        assert!(gd.player_and_his_game_mut(3).is_err());
    }

    #[test]
    fn four_player_and_game_mut_access() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        for i in 0..4 {
            let (player, _pig) = gd.player_and_his_game_mut(i).unwrap();
            assert!(!player.name().is_empty());
        }
        assert!(gd.player_and_his_game_mut(4).is_err());
    }

    #[test]
    fn five_player_and_game_mut_access() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        for i in 0..5 {
            let (player, _pig) = gd.player_and_his_game_mut(i).unwrap();
            assert!(!player.name().is_empty());
        }
        assert!(gd.player_and_his_game_mut(5).is_err());
    }

    // --- Players and their game mut ---

    #[test]
    fn three_players_and_their_game_mut() {
        let mut game = Game::<{ Mode::Three.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let (players, pigs) = gd.players_and_their_game_mut();
        assert_eq!(players.len(), 3);
        assert_eq!(pigs.len(), 3);
    }

    #[test]
    fn four_players_and_their_game_mut() {
        let mut game = Game::<{ Mode::Four.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let (players, pigs) = gd.players_and_their_game_mut();
        assert_eq!(players.len(), 4);
        assert_eq!(pigs.len(), 4);
    }

    #[test]
    fn five_players_and_their_game_mut() {
        let mut game = Game::<{ Mode::Five.players() }>::new(test_options()).unwrap();
        let mut gd = distribute_game(&mut game);
        let (players, pigs) = gd.players_and_their_game_mut();
        assert_eq!(players.len(), 5);
        assert_eq!(pigs.len(), 5);
    }
}
