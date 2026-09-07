use derive_new::new;
use ordered_float::OrderedFloat;
use std::fmt;

use crate::constants::{BASE_CONTRACT_POINTS, MAX_CARDS, MAX_POINTS};
use crate::contract::Contract;
use crate::errors::TarotErrorKind;
use crate::game_distributed::GameDistributed;
use crate::mode::Mode;
use crate::options::Options;
use crate::player::Player;
use crate::player_in_game::PlayerInGame;
use crate::points::Points;
use crate::role::Role;
use crate::team::Team;
use crate::turn::Turn;

#[derive(new)]
pub struct GameStarted<'a, const MODE: usize> {
    game_distributed: &'a mut GameDistributed<'a, MODE>,
    contract: Contract,
    options: Options,

    #[new(default)]
    petit_au_bout_for_team: Option<Team>,
    #[new(default)]
    defense_cards: usize,
    #[new(default)]
    attack_cards: usize,
}

impl<const MODE: usize> fmt::Display for GameStarted<'_, MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Contract : {}", self.contract)?;
        if let Some(team) = &self.petit_au_bout_for_team {
            writeln!(f, "Petit au bout? : {team}")?;
        }
        writeln!(f, "Defense cards : {}", self.defense_cards)?;
        writeln!(f, "Attack cards : {}", self.attack_cards)?;
        writeln!(f, "Players : ")?;
        for index in 0..MODE {
            let Ok((player, player_in_game)) = self.player_and_his_game(index) else {
                return Err(fmt::Error);
            };
            writeln!(f, "\t{player} {player_in_game}")?;
        }
        Ok(())
    }
}

impl<const MODE: usize> GameStarted<'_, MODE> {
    pub fn is_consistent(&mut self) -> Result<(), TarotErrorKind> {
        self.game_distributed.game().is_consistent()
    }
    #[must_use]
    pub fn finished(&self) -> bool {
        self.game_distributed.finished()
    }
    pub const fn mode(&mut self) -> &Mode {
        self.game_distributed.game().mode()
    }
    pub fn player(&self, index: usize) -> Result<&Player, TarotErrorKind> {
        self.game_distributed.player(index)
    }
    fn player_mut(&mut self, index: usize) -> Result<&mut Player, TarotErrorKind> {
        self.game_distributed.player_mut(index)
    }
    pub fn player_and_his_game(
        &self,
        index: usize,
    ) -> Result<(&Player, &PlayerInGame), TarotErrorKind> {
        self.game_distributed.player_and_his_game(index)
    }
    pub fn player_and_his_game_mut(
        &mut self,
        index: usize,
    ) -> Result<(&Player, &mut PlayerInGame), TarotErrorKind> {
        self.game_distributed.player_and_his_game_mut(index)
    }
    pub const fn players_and_their_game_mut(
        &mut self,
    ) -> (&[Player; MODE], &mut [PlayerInGame; MODE]) {
        self.game_distributed.players_and_their_game_mut()
    }
    pub fn play(&mut self) -> Result<(), TarotErrorKind> {
        let mut turn = Turn::default();
        let mut master_player_index: usize = 0;
        let mut fool_kept_by_owner = false;
        let quiet = self.options.quiet;

        // A side is still on track for a slam as long as the other side took no card at all.
        let attack_took_everything = self.defense_cards == 0;
        let defense_took_everything = self.attack_cards == 0;

        for current_player_index in 0..MODE {
            let (current_player, current_player_in_game) =
                self.player_and_his_game_mut(current_player_index)?;
            let current_player_name = current_player.name();

            if !quiet {
                println!("Current player {current_player_name} (index : {current_player_index})");
            }
            let &Some(team) = current_player_in_game.team() else {
                return Err(TarotErrorKind::NoTeamForPlayer(
                    current_player.name().to_string(),
                ));
            };

            let card = current_player_in_game.play_card(current_player, &turn)?;
            if card.is_fool() {
                // RULE: the fool is preserved to his owner, except when played in the last turn,
                // where it is lost to the winner of that turn unless its side took every turn.
                let keep = !current_player_in_game.last_turn()
                    || match team {
                        Team::Attack => attack_took_everything,
                        Team::Defense => defense_took_everything,
                    };
                if keep {
                    fool_kept_by_owner = true;
                    current_player_in_game.push_owned(card);
                } else if !quiet {
                    println!("Fool played in the last turn, it is lost");
                }
                turn.put(card);
            } else {
                turn.put(card);
                if let Some(master) = turn.master_card() {
                    if master.master(card) {
                        if !quiet {
                            let master_player =
                                self.game_distributed
                                    .player(master_player_index)
                                    .map_err(|_| TarotErrorKind::NoMaster(master_player_index))?;
                            println!(
                                "Master card is {master}, so player {} stays master",
                                master_player.name()
                            );
                        }
                    } else {
                        if !quiet {
                            println!(
                                "Master card is {card}, so player {current_player_name} becomes master",
                            );
                        }
                        master_player_index = current_player_index;
                        turn.master_index = Some(turn.len() - 1);
                    }
                } else {
                    if !quiet {
                        println!(
                            "First card is {card}, so player {current_player_name} becomes master",
                        );
                    }
                    master_player_index = current_player_index;
                    turn.master_index = Some(turn.len() - 1);
                }
            }
            if !quiet {
                println!("{turn}");
            }
        }

        if !quiet {
            if attack_took_everything {
                println!("Attack is near slam!");
            }
            if defense_took_everything {
                println!("Defense is near slam!");
            }
        }

        let (players, players_in_game) = self.players_and_their_game_mut();
        let Some(master) = players.get(master_player_index) else {
            return Err(TarotErrorKind::NoMaster(master_player_index));
        };
        let master_name = master.name().to_string();
        if !quiet {
            println!("Player {master_name} has win turn");
        }

        let Some(master_player_in_game) = players_in_game.get_mut(master_player_index) else {
            return Err(TarotErrorKind::NoMaster(master_player_index));
        };
        // RULE: petit au bout works for last turn, or before last turn if a slam is occuring
        let last_turn = master_player_in_game.last_turn();
        if last_turn && !quiet {
            println!("Last turn detected");
        }
        let before_last_turn = master_player_in_game.before_last_turn();
        if before_last_turn && !quiet {
            println!("Before last turn detected");
        }

        let Some(master_player_team) = master_player_in_game.team() else {
            return Err(TarotErrorKind::NoTeamForPlayer(master_name));
        };
        let master_player_team = *master_player_team;

        let turn_cards = turn.take_cards(fool_kept_by_owner);
        let petit_au_bout = turn_cards.has_petit()
            && (last_turn
                || (before_last_turn && (attack_took_everything || defense_took_everything)));
        let won_cards = turn_cards.len();
        master_player_in_game.extend_owned(&turn_cards);

        if petit_au_bout {
            if !quiet {
                println!(
                    "{master_name} (team: {master_player_team}) has Petit in last turn (Petit au bout) : +10 points"
                );
            }
            self.petit_au_bout_for_team = Some(master_player_team);
        }
        self.game_distributed.rotate_at(master_player_index);
        match master_player_team {
            Team::Attack => self.attack_cards += won_cards,
            Team::Defense => self.defense_cards += won_cards,
        }
        Ok(())
    }

    /// Every card of the game must still be somewhere, and be worth the same total.
    fn check_deal_integrity(&self) -> Result<(), TarotErrorKind> {
        let mut total_cards = 0;
        let mut total_points = OrderedFloat(0.0);
        for index in 0..MODE {
            let (_, player_in_game) = self.player_and_his_game(index)?;
            let cards = player_in_game.all_cards();
            total_cards += cards.len();
            total_points += cards.points();
        }
        if total_cards != MAX_CARDS {
            return Err(TarotErrorKind::InvalidCase(format!(
                "{total_cards} cards in play instead of {MAX_CARDS}"
            )));
        }
        if total_points != MAX_POINTS {
            return Err(TarotErrorKind::InvalidCase(format!(
                "{total_points} points in play instead of {MAX_POINTS}"
            )));
        }
        Ok(())
    }

    pub fn count_points(&mut self) -> Result<(), TarotErrorKind> {
        let quiet = self.options.quiet;
        let mut taker_index: Option<usize> = None;
        let mut ally_index: Option<usize> = None;
        let mut defense: Vec<usize> = Vec::new();
        let mut announced_handles = OrderedFloat(0.0);
        let mut owning_card_player_index: Option<usize> = None;
        let mut missing_card_player_index: Option<usize> = None;

        // Seats are rotated after every turn, so roles are the only reliable way
        // to find out who the taker is at this point.
        for current_player_index in 0..MODE {
            let (current_player, current_player_in_game) =
                self.player_and_his_game(current_player_index)?;

            let Some(role) = *current_player_in_game.role() else {
                return Err(TarotErrorKind::NoRoleForPlayer(
                    current_player.name().to_string(),
                ));
            };

            if current_player_in_game.owe_card() {
                owning_card_player_index = Some(current_player_index);
            }
            if current_player_in_game.missing_card() {
                missing_card_player_index = Some(current_player_index);
            }
            if let Some(handle) = *current_player_in_game.handle() {
                let handle_bonus = handle.points();
                announced_handles += handle_bonus;
                if !quiet {
                    println!("Handle announced by role {role} : {handle_bonus}");
                }
            }
            match role {
                Role::Taker => taker_index = Some(current_player_index),
                Role::Ally => ally_index = Some(current_player_index),
                Role::Defenser => defense.push(current_player_index),
            }
        }
        let Some(taker_index) = taker_index else {
            return Err(TarotErrorKind::NoTaker(0));
        };

        // RULE: the owner of a kept fool gives a low card in exchange,
        // and loses the fool itself when he has no card to give.
        if let (Some(owning_index), Some(missing_index)) =
            (owning_card_player_index, missing_card_player_index)
        {
            let (players, players_in_game) = self.players_and_their_game_mut();
            let (Some(owning_player), Some(missing_player)) =
                (players.get(owning_index), players.get(missing_index))
            else {
                return Err(TarotErrorKind::NoPlayer(owning_index));
            };
            let Some(owning_player_in_game) = players_in_game.get_mut(owning_index) else {
                return Err(TarotErrorKind::NoPlayer(owning_index));
            };
            let compensation = owning_player_in_game
                .give_low()
                .or_else(|| owning_player_in_game.give_fool());
            if let Some(card) = compensation {
                if !quiet {
                    println!(
                        "Player {} owes a card to {}, giving a {card} in exchange",
                        owning_player.name(),
                        missing_player.name()
                    );
                }
                let Some(missing_player_in_game) = players_in_game.get_mut(missing_index) else {
                    return Err(TarotErrorKind::NoPlayer(missing_index));
                };
                missing_player_in_game.push_owned(card);
            }
        }

        self.check_deal_integrity()?;

        // RULE: the ally puts his cards with the taker's ones
        let (attack_cards, taker_announced_slam) = {
            let (_, players_in_game) = self.players_and_their_game_mut();
            let Some(taker_in_game) = players_in_game.get(taker_index) else {
                return Err(TarotErrorKind::NoTaker(taker_index));
            };
            let mut attack_cards = taker_in_game.all_cards();
            let taker_announced_slam = taker_in_game.announced_slam();
            if let Some(ally_index) = ally_index {
                let Some(ally_in_game) = players_in_game.get(ally_index) else {
                    return Err(TarotErrorKind::NoAlly(ally_index));
                };
                attack_cards.extend(&ally_in_game.all_cards());
            }
            (attack_cards, taker_announced_slam)
        };

        let attack_points = attack_cards.points();
        let points_for_oudlers = attack_cards.points_for_oudlers()?;
        let contract_succeeded = attack_points >= points_for_oudlers;

        if !quiet {
            println!("Attack owned points: {attack_points}");
            println!("Contract todo: {points_for_oudlers}");
            println!("Contract base: {BASE_CONTRACT_POINTS}");
            println!("Contract difference: {}", attack_points - points_for_oudlers);
        }

        let contract_points = if contract_succeeded {
            (attack_points - points_for_oudlers + BASE_CONTRACT_POINTS) * self.contract.multiplier()
        } else {
            (attack_points - points_for_oudlers - BASE_CONTRACT_POINTS) * self.contract.multiplier()
        };
        if !quiet {
            println!(
                "Taker contract: {} (x{})",
                self.contract,
                self.contract.multiplier()
            );
            println!("Taker contract points: {contract_points}");
        }

        // RULE: a slam is realized when a side took every single turn
        let attack_slam = self.attack_cards > 0 && self.defense_cards == 0;
        let defense_slam = self.defense_cards > 0 && self.attack_cards == 0;
        let mut slam_bonus = OrderedFloat(0.0);
        if attack_slam {
            slam_bonus += if taker_announced_slam {
                if !quiet {
                    println!("Chelem announced and realized !");
                }
                OrderedFloat(400.0)
            } else {
                if !quiet {
                    println!("Chelem not announced but realized !");
                }
                OrderedFloat(200.0)
            };
        } else {
            if taker_announced_slam {
                if !quiet {
                    println!("Chelem announced but not realized !");
                }
                slam_bonus -= OrderedFloat(200.0);
            }
            if defense_slam {
                if !quiet {
                    println!("Chelem realized by the defense !");
                }
                slam_bonus -= OrderedFloat(200.0);
            }
        }

        let points_petit_au_bout = OrderedFloat(10.0) * self.contract.multiplier();
        let petit_au_bout_bonus = match self.petit_au_bout_for_team {
            Some(Team::Defense) => {
                if !quiet {
                    println!("Petit au bout for defense: -{points_petit_au_bout}");
                }
                -points_petit_au_bout
            }
            Some(Team::Attack) => {
                if !quiet {
                    println!("Petit au bout for attack: {points_petit_au_bout}");
                }
                points_petit_au_bout
            }
            None => {
                if !quiet {
                    println!("No petit au bout bonus");
                }
                OrderedFloat(0.0)
            }
        };

        // RULE: handles are always paid to the winner of the deal, whoever announced them
        let handle_bonus = if contract_succeeded {
            announced_handles
        } else {
            -announced_handles
        };

        let ratio = self.mode().ratio(ally_index.is_some());
        let points = contract_points + petit_au_bout_bonus + handle_bonus + slam_bonus;
        if !quiet {
            println!("Total handle bonuses: {handle_bonus}");
            println!("Slam bonus: {slam_bonus}");
            println!("Attack points: {points}");
        }

        let Ok(taker) = self.player_mut(taker_index) else {
            return Err(TarotErrorKind::NoTaker(taker_index));
        };
        taker.add_score(ratio * points);
        if !quiet {
            println!("Taker {} total points: {}", taker.name(), taker.score());
        }

        if let Some(ally_index) = ally_index {
            let Ok(ally) = self.player_mut(ally_index) else {
                return Err(TarotErrorKind::NoAlly(ally_index));
            };
            ally.add_score(points);
            if !quiet {
                println!("Ally {} total points: {}", ally.name(), ally.score());
            }
        }

        for defenser_index in defense {
            let Ok(defenser) = self.player_mut(defenser_index) else {
                return Err(TarotErrorKind::NoDefenser(defenser_index));
            };
            defenser.add_score(-points);
            if !quiet {
                println!("Defenser {} total points: {}", defenser.name(), defenser.score());
            }
        }
        self.is_consistent()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::contract::Contract;
    use crate::game::{Game, launch};
    use crate::mode::Mode;
    use crate::options::Options;
    use ordered_float::OrderedFloat;
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

    // --- Three-player mode (1 vs 2) ---

    #[test]
    fn three_player_single_deal_consistent() {
        let options = default_test_options();
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.start(60), Ok(()));
    }

    #[test]
    fn three_player_multiple_deals_consistent() {
        let options = default_test_options();
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.start(5), Ok(()));
    }

    #[test]
    fn three_player_no_slam() {
        let options = Options {
            no_slam: false,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn three_player_scores_sum_to_zero() {
        let options = default_test_options();
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.start(5), Ok(()));
    }

    #[test]
    fn three_player_launch_integration() {
        assert_eq!(launch(Mode::Three, default_test_options(), 3), Ok(()));
    }

    // --- Four-player mode (1 vs 3, standard) ---

    #[test]
    fn four_player_single_deal_consistent() {
        let options = default_test_options();
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.start(60), Ok(()));
    }

    #[test]
    fn four_player_multiple_deals_consistent() {
        let options = default_test_options();
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.start(5), Ok(()));
    }

    #[test]
    fn four_player_no_slam() {
        let options = Options {
            no_slam: false,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn four_player_scores_sum_to_zero() {
        let options = default_test_options();
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.start(5), Ok(()));
    }

    #[test]
    fn four_player_launch_integration() {
        assert_eq!(launch(Mode::Four, default_test_options(), 3), Ok(()));
    }

    // --- Five-player mode (2 vs 3, call a king) ---

    #[test]
    fn five_player_single_deal_consistent() {
        let options = default_test_options();
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.start(60), Ok(()));
    }

    #[test]
    fn five_player_multiple_deals_consistent() {
        let options = default_test_options();
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.start(5), Ok(()));
    }

    #[test]
    fn five_player_no_slam() {
        let options = Options {
            no_slam: false,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn five_player_scores_sum_to_zero() {
        let options = default_test_options();
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.start(5), Ok(()));
    }

    #[test]
    fn five_player_launch_integration() {
        assert_eq!(launch(Mode::Five, default_test_options(), 3), Ok(()));
    }

    // --- Cross-mode tests ---

    #[test]
    fn all_modes_single_deal() {
        let options = default_test_options();
        for mode in Mode::iter() {
            assert_eq!(launch(mode, options, 1), Ok(()));
        }
    }

    #[test]
    fn all_modes_many_deals() {
        let options = default_test_options();
        for mode in Mode::iter() {
            assert_eq!(launch(mode, options, 10), Ok(()));
        }
    }

    #[test]
    fn all_modes_no_slam() {
        let options = Options {
            no_slam: false,
            ..default_test_options()
        };
        for mode in Mode::iter() {
            assert_eq!(launch(mode, options, 3), Ok(()));
        }
    }

    // --- Contract multiplier sanity ---

    #[test]
    fn contract_multipliers() {
        assert_eq!(Contract::Petite.multiplier(), OrderedFloat(1.0));
        assert_eq!(Contract::Garde.multiplier(), OrderedFloat(2.0));
        assert_eq!(Contract::GardeSans.multiplier(), OrderedFloat(4.0));
        assert_eq!(Contract::GardeContre.multiplier(), OrderedFloat(6.0));
    }

    // --- Attack mode (attack: true, random: true) ---

    #[test]
    fn three_player_attack_mode() {
        let options = Options {
            attack: true,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn four_player_attack_mode() {
        let options = Options {
            attack: true,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn five_player_attack_mode() {
        let options = Options {
            attack: true,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn all_modes_attack() {
        let options = Options {
            attack: true,
            ..default_test_options()
        };
        for mode in Mode::iter() {
            assert_eq!(launch(mode, options, 3), Ok(()));
        }
    }

    #[test]
    fn three_player_attack_no_slam() {
        let options = Options {
            attack: true,
            no_slam: false,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Three.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn four_player_attack_no_slam() {
        let options = Options {
            attack: true,
            no_slam: false,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Four.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    #[test]
    fn five_player_attack_no_slam() {
        let options = Options {
            attack: true,
            no_slam: false,
            ..default_test_options()
        };
        let game = Game::<{ Mode::Five.players() }>::new(options).unwrap();
        assert_eq!(game.start(3), Ok(()));
    }

    // --- Stress: large number of deals per mode ---

    #[test]
    fn three_player_stress() {
        assert_eq!(launch(Mode::Three, default_test_options(), 20), Ok(()));
    }

    #[test]
    fn four_player_stress() {
        assert_eq!(launch(Mode::Four, default_test_options(), 20), Ok(()));
    }

    #[test]
    fn five_player_stress() {
        assert_eq!(launch(Mode::Five, default_test_options(), 20), Ok(()));
    }
}


