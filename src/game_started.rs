use derive_new::new;
use ordered_float::OrderedFloat;
use std::fmt;

use crate::contract::Contract;
use crate::errors::TarotErrorKind;
use crate::game_distributed::GameDistributed;
use crate::helpers::wait_input;
use crate::mode::Mode;
use crate::options::Options;
use crate::player::Player;
use crate::player_in_game::PlayerInGame;
use crate::points::{HasPoints, BASE_CONTRACT_POINTS, MAX_CARDS};
use crate::role::Role;
use crate::team::Team;
use crate::turn::Turn;

#[derive(new)]
pub struct GameStarted<'a, const MODE: usize> {
    game_distributed: &'a mut GameDistributed<'a, MODE>,
    taker_index: usize,
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
            writeln!(f, "Petit au bout? : {}", team)?;
        }
        writeln!(f, "Defense cards : {}", self.defense_cards)?;
        writeln!(f, "Attack cards : {}", self.attack_cards)?;
        writeln!(f, "Players : ")?;
        for index in 0..MODE {
            let (player, player_in_game) = self.player_and_his_game(index);
            writeln!(f, "\t{} {}", player, player_in_game)?;
        }
        Ok(())
    }
}

impl<'a, const MODE: usize> GameStarted<'a, MODE> {
    pub fn is_consistent(&mut self) -> Result<(), TarotErrorKind> {
        self.game_distributed.game().is_consistent()
    }
    pub fn finished(&mut self) -> bool {
        self.game_distributed.finished()
    }
    pub fn mode(&mut self) -> &Mode {
        self.game_distributed.game().mode()
    }
    pub fn player(&self, index: usize) -> &Player {
        self.game_distributed.player(index)
    }
    pub fn player_and_his_game(&self, index: usize) -> (&Player, &PlayerInGame) {
        self.game_distributed.player_and_his_game(index)
    }
    pub fn player_and_his_game_mut(&mut self, index: usize) -> (&Player, &mut PlayerInGame) {
        self.game_distributed.player_and_his_game_mut(index)
    }
    pub fn players_and_their_game_mut(&mut self) -> (&[Player; MODE], &mut [PlayerInGame; MODE]) {
        self.game_distributed.players_and_their_game_mut()
    }
    pub fn play(&mut self) -> Result<(), TarotErrorKind> {
        let mut turn = Turn::default();
        let mut master_player_index: usize = 0;
        let quiet = self.options.quiet;
        for current_player_index in 0..MODE {
            let (current_player, current_player_in_game) =
                self.player_and_his_game_mut(current_player_index);
            let current_player_name = current_player.name();

            if !quiet {
                println!("Current player {current_player_name} (index : {current_player_index})");
            }
            let &Some(team) = current_player_in_game.team() else {
                return Err(TarotErrorKind::NoTeamForPlayer(current_player.name().to_string()));
            };

            let card = current_player_in_game.play_card(current_player, &mut turn)?;
            if card.is_fool() {
                if !current_player_in_game.last_turn() {
                    // RULE: the fool is always preserved to his owner
                    current_player_in_game.push_owned(card);
                    turn.put(card);
                } else {
                    // RULE: exception in the last turn, the fool is in game and can be lost
                    turn.put(card);
                    match team {
                        Team::Attack => {
                            if self.attack_cards == MAX_CARDS - self.mode().dog_size() {
                                turn.master_index = Some(turn.len() - 1);
                                master_player_index = current_player_index;
                            }
                        }
                        Team::Defense => {
                            if self.defense_cards == MAX_CARDS - self.mode().dog_size() {
                                turn.master_index = Some(turn.len() - 1);
                                master_player_index = current_player_index;
                            }
                        }
                    }
                }
            } else {
                turn.put(card);
                if let Some(master) = turn.master_card() {
                    if master.master(card) {
                        if !quiet {
                            let master_player_name = self.player(master_player_index).name();
                            println!(
                                "Master card is {master}, so player {master_player_name} stays master",
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
            if !self.options.quiet {
                println!("{turn}");
            }
        }

        let mode = *self.mode();
        let attack_near_slam = self.attack_cards == MAX_CARDS - mode.dog_size() - mode.players();
        if attack_near_slam && !quiet {
            println!("Attack is near slam!");
        }
        let defense_near_slam = self.defense_cards == MAX_CARDS - mode.dog_size() - mode.players();
        if defense_near_slam && !quiet {
            println!("Defense is near slam!");
        }

        let (players, players_in_game) = self.players_and_their_game_mut();
        let master_player_name = players[master_player_index].name();
        let master_player_in_game = &mut players_in_game[master_player_index];
        if !quiet {
            println!("Player {master_player_name} has win turn");
        }
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
            return Err(TarotErrorKind::NoTeamForPlayer(master_player_name.to_string()));
        };

        let turn_cards = turn.take_cards_except_fool();
        let petit_au_bout_for_team = if turn_cards.has_petit()
            && (last_turn || (before_last_turn && (attack_near_slam || defense_near_slam)))
        {
            if !quiet {
                println!(
                    "{master_player_name} (team: {master_player_team}) has Petit in last turn (Petit au bout) : +10 points",
                );
            }
            wait_input();
            Some(*master_player_team)
        } else {
            None
        };

        let mut attack_cards = 0;
        let mut defense_cards = 0;
        match master_player_team {
            Team::Attack => attack_cards = turn_cards.len(),
            Team::Defense => defense_cards = turn_cards.len(),
        }
        master_player_in_game.extend_owned(&turn_cards);
        self.game_distributed.rotate_at(master_player_index);
        self.petit_au_bout_for_team = petit_au_bout_for_team;
        self.attack_cards += attack_cards;
        self.defense_cards += defense_cards;
        Ok(())
    }
    pub fn count_points(&mut self) -> Result<(), TarotErrorKind> {
        let mut ally_index: Option<usize> = None;
        let mut attack: Vec<usize> = Vec::new();
        let mut defense: Vec<usize> = Vec::new();
        let mut owning_card_player_index: Option<usize> = None;
        let mut missing_card_player_index: Option<usize> = None;
        let mut handle_bonus = OrderedFloat(0.0);
        let quiet = self.options.quiet;
        for current_player_index in 0..MODE {
            let (current_player, current_player_in_game) =
                self.player_and_his_game_mut(current_player_index);

            if current_player_in_game.owe_card() {
                owning_card_player_index = Some(current_player_index);
            }
            if current_player_in_game.missing_card() {
                missing_card_player_index = Some(current_player_index);
            }
            if let Some(handle) = &current_player_in_game.handle() {
                handle_bonus = handle.points();
                if !quiet {
                    println!("Handle bonus: {handle_bonus}");
                }
            }
            match current_player_in_game.role() {
                Some(Role::Taker) => {
                    attack.push(current_player_index);
                }
                Some(Role::Ally) => {
                    assert!(ally_index.is_none());
                    ally_index = Some(current_player_index);
                    attack.push(current_player_index);
                }
                Some(Role::Defenser) => {
                    defense.push(current_player_index);
                }
                None => {
                    return Err(TarotErrorKind::NoRoleForPlayer(
                        current_player.name().to_string(),
                    ));
                }
            }
        }
        match self.game_distributed.game().mode() {
            Mode::Three => assert_eq!(defense.len(), 2),
            Mode::Four => assert_eq!(defense.len(), 3),
            Mode::Five => {
                if ally_index.is_some() {
                    assert_eq!(defense.len(), 3)
                } else {
                    assert_eq!(defense.len(), 4)
                }
            }
        };

        // give a low card if someone owe a card to someone else
        if let (Some(owning_card_player_index), Some(missing_card_player_index)) =
            (owning_card_player_index, missing_card_player_index)
        {
            let (players, players_in_game) = self.players_and_their_game_mut();
            let owning_card_player_name = players[owning_card_player_index].name();
            let low_card = players_in_game[owning_card_player_index].give_low();
            if let Some(low_card) = low_card {
                let missing_card_player_name = players[missing_card_player_index].name();
                let missing_card_player_in_game = &mut players_in_game[missing_card_player_index];
                missing_card_player_in_game.push_owned(low_card);
                if !quiet {
                    println!("Player {owning_card_player_name} own a card to {missing_card_player_name}, giving a {low_card} in exchange");
                }
            } else if !quiet {
                println!("Player {owning_card_player_name} cannot give a low card");
            }
        }

        let taker_index = self.taker_index;
        if let Some(ally_index) = ally_index {
            let (players, players_in_game) = self.players_and_their_game_mut();
            let ally = &players[ally_index].name();
            let ally_in_game = &mut players_in_game[ally_index];
            let ally_cards = ally_in_game.all_cards();
            let taker = &players[taker_index].name();
            let taker_in_game = &mut players_in_game[taker_index];
            if !quiet {
                println!("{ally} gives his card to {taker}")
            }
            taker_in_game.extend_owned(&ally_cards)
        }

        let (taker, taker_in_game) = self.player_and_his_game_mut(self.taker_index);
        let slam_bonus = taker_in_game.slam_bonus();
        let taker_points = taker_in_game.points();
        let points_for_oudlers = taker_in_game.points_for_oudlers()?;

        if !quiet {
            println!("Taker {taker} slam bonus: {slam_bonus}");
            println!("Taker {taker} owned points: {taker_points}");
            println!("Contract todo: {points_for_oudlers}");
            println!("Contract base: {BASE_CONTRACT_POINTS}");
            let difference = taker_points - points_for_oudlers;
            println!("Contract difference: {difference}");
        }

        let contract_points = if taker_points >= points_for_oudlers {
            if !quiet {
                let total = taker_points - points_for_oudlers + BASE_CONTRACT_POINTS;
                println!("Contract total: {total}");
            }
            (taker_points - points_for_oudlers + BASE_CONTRACT_POINTS) * self.contract.multiplier()
        } else {
            if !self.options.quiet {
                let total = taker_points - points_for_oudlers - BASE_CONTRACT_POINTS;
                println!("Contract total: {total}");
            }
            (taker_points - points_for_oudlers - BASE_CONTRACT_POINTS) * self.contract.multiplier()
        };
        if !self.options.quiet {
            println!(
                "Taker contract: {} (x{})",
                self.contract,
                self.contract.multiplier()
            );
            println!("Taker contract points: {contract_points}");
        }

        let points_petit_au_bout = 10.0 * self.contract.multiplier();
        let petit_au_bout_bonus = match self.petit_au_bout_for_team {
            Some(Team::Defense) => {
                if !self.options.quiet {
                    println!("Petit au bout for defense: -{points_petit_au_bout}");
                }
                -points_petit_au_bout
            }
            Some(Team::Attack) => {
                if !self.options.quiet {
                    println!("Petit au bout for attack: {points_petit_au_bout}");
                }
                points_petit_au_bout
            }
            None => {
                if !self.options.quiet {
                    println!("No petit au bout bonus");
                }
                0.0
            }
        };

        let ratio = self.mode().ratio(ally_index.is_some());
        let points = contract_points + petit_au_bout_bonus + handle_bonus + slam_bonus;

        if contract_points >= OrderedFloat(0.0) {
            self.game_distributed
                .game()
                .player_mut(self.taker_index)
                .add_score(ratio * points);
        } else {
            handle_bonus *= -1.0;
            self.game_distributed
                .game()
                .player_mut(self.taker_index)
                .add_score(-ratio * points);
        }

        if !self.options.quiet {
            println!("Attack handle bonus: {}", handle_bonus.abs());
            println!("Taker points: {}", points);
            println!(
                "Taker total points: {}",
                self.game_distributed
                    .game()
                    .player(self.taker_index)
                    .score()
            );
        }

        if let Some(ally_index) = ally_index {
            if contract_points >= OrderedFloat(0.0) {
                self.game_distributed
                    .game()
                    .player_mut(ally_index)
                    .add_score(points);
            } else {
                self.game_distributed
                    .game()
                    .player_mut(ally_index)
                    .add_score(-points);
            }
            if !self.options.quiet {
                println!(
                    "Ally total points: {}",
                    self.game_distributed.game().player(ally_index).score()
                );
            }
        }

        for defenser_index in defense {
            if contract_points >= OrderedFloat(0.0) {
                self.game_distributed
                    .game()
                    .player_mut(defenser_index)
                    .add_score(-points);
            } else {
                self.game_distributed
                    .game()
                    .player_mut(defenser_index)
                    .add_score(points);
            }
            if !self.options.quiet {
                let defenser = self.game_distributed.game().player(defenser_index);
                println!("Defenser : {}", defenser);
            }
        }
        //if handle_bonus != 0.0  && petit_au_bout_bonus != 0.0 && slam_bonus != 0.0 && ratio == 4.0 {
        //    helpers::wait_input();
        //}
        self.is_consistent()
    }
}
