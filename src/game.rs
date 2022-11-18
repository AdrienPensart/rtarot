use array_init::array_init;
use itertools::{Either, Itertools};
use ordered_float::OrderedFloat;
use std::fmt;
use strum::IntoEnumIterator;

use crate::card::Card;
use crate::contract::Contract;
use crate::deck::Deck;
use crate::errors::TarotErrorKind;
use crate::mode::Mode;
use crate::options::Options;
use crate::player::{Player, PlayerInGame};
use crate::points::{HasPoints, BASE_CONTRACT_POINTS, MAX_CARDS};
use crate::role::Role;
use crate::team::Team;
use crate::turn::Turn;

#[derive(Debug, Clone)]
pub struct Game<const MODE: usize> {
    options: Options,
    mode: Mode,
    players: [Player; MODE],
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

pub struct GameDistributed<'a, const MODE: usize> {
    game: &'a mut Game<MODE>,
    options: Options,
    dog: Deck,
    players_in_game: [PlayerInGame; MODE],
}

pub struct GameStarted<'a, const MODE: usize> {
    game_distributed: &'a mut GameDistributed<'a, MODE>,
    taker_index: usize,
    options: Options,
    contract: Contract,
    petit_au_bout: Option<Team>,
    defense_cards: usize,
    attack_cards: usize,
}

impl<const MODE: usize> fmt::Display for GameDistributed<'_, MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Game {} with dog {}", self.game.mode, self.dog)
    }
}

impl<const MODE: usize> fmt::Display for GameStarted<'_, MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Contract : {}", self.contract)?;
        if let Some(petit_au_bout) = &self.petit_au_bout {
            writeln!(f, "Petit au bout? : {}", petit_au_bout)?;
        }
        writeln!(f, "Defense cards : {}", self.defense_cards)?;
        writeln!(f, "Attack cards : {}", self.attack_cards)?;
        writeln!(f, "Players : ")?;
        for (i, player) in self.game_distributed.game.players.iter().enumerate() {
            writeln!(f, "\t{} {}", player, self.game_distributed.players_in_game[i])?;
        }
        Ok(())
    }
}

impl<const MODE: usize> GameStarted<'_, MODE> {
    pub fn play(&mut self) -> Result<(), TarotErrorKind> {
        let mut turn = Turn::default();
        let mut master_player_index: usize = 0;
        let game = &mut self.game_distributed.game;
        for current_player_index in 0..self.game_distributed.players_in_game.len() {
            let current_player_in_game = &mut self.game_distributed.players_in_game[current_player_index];
            let current_player = &game.players[current_player_index];

            if !self.options.quiet {
                println!("current player index : {}", current_player_index);
            }
            let Some(team) = current_player_in_game.team() else {
                return Err(TarotErrorKind::NoTeamForPlayer(current_player.name().to_string()));
            };

            let card = current_player_in_game.play_card(current_player, &mut turn)?;
            if card.is_fool() {
                if !self.game_distributed.players_in_game[current_player_index].last_turn() {
                    // RULE: the fool is always preserved to his owner
                    self.game_distributed.players_in_game[current_player_index].push_owned(card);
                    turn.put(card);
                } else {
                    // RULE: exception in the last turn, the fool is in game and can be lost
                    turn.put(card);
                    match team {
                        Team::Attack => {
                            if self.attack_cards == MAX_CARDS - game.mode.dog_size() {
                                turn.master_index = Some(turn.len() - 1);
                                master_player_index = current_player_index;
                            }
                        }
                        Team::Defense => {
                            if self.defense_cards == MAX_CARDS - game.mode.dog_size() {
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
                        if !self.options.quiet {
                            println!(
                                "Master card is {}, so player {} stays master",
                                master,
                                current_player.name()
                            );
                        }
                    } else {
                        if !self.options.quiet {
                            println!(
                                "Master card is {}, so player {} becomes master",
                                card,
                                current_player.name()
                            );
                        }
                        master_player_index = current_player_index;
                        turn.master_index = Some(turn.len() - 1);
                    }
                } else {
                    if !self.options.quiet {
                        println!(
                            "First card is {}, so player {} becomes master",
                            card,
                            game.players[current_player_index].name()
                        );
                    }
                    master_player_index = current_player_index;
                    turn.master_index = Some(turn.len() - 1);
                }
            }
            if !self.options.quiet {
                println!("{}", &turn);
            }
        }

        let cards = turn.take();
        if !self.options.quiet {
            println!("Winner is player {}", game.players[master_player_index]);
        }
        // RULE: petit au bout works for last turn, or before last turn if a slam is occuring
        let last_turn = self.game_distributed.players_in_game[master_player_index].last_turn();
        let before_last_turn = self.game_distributed.players_in_game[master_player_index].before_last_turn();
        let attack_near_slam =
            self.attack_cards == MAX_CARDS - game.mode.dog_size() - game.mode.players();
        let defense_near_slam =
            self.defense_cards == MAX_CARDS - game.mode.dog_size() - game.mode.players();
        if cards.has_petit()
            && (last_turn || (before_last_turn && (attack_near_slam || defense_near_slam)))
        {
            if !self.options.quiet {
                println!(
                    "{} has Petit in last turn (Petit au bout) : +10 points",
                    game.players[master_player_index]
                );
            }
            self.petit_au_bout = self.game_distributed.players_in_game[master_player_index].team();
        }
        let Some(master_player_team) = self.game_distributed.players_in_game[master_player_index].team() else {
            return Err(TarotErrorKind::NoTeamForPlayer(game.players[master_player_index].name().to_string()));
        };
        match master_player_team {
            Team::Attack => self.attack_cards += cards.len(),
            Team::Defense => self.defense_cards += cards.len(),
        }
        self.game_distributed.players_in_game[master_player_index].append_owned(&cards);
        self.game_distributed.players_in_game.rotate_left(master_player_index);
        Ok(())
    }
    pub fn count_points(&mut self) -> Result<(), TarotErrorKind> {
        let mut ally_index: Option<usize> = None;
        let mut attack: Vec<usize> = Vec::new();
        let mut defense: Vec<usize> = Vec::new();
        let mut owning_card_player_index: Option<usize> = None;
        let mut missing_card_player_index: Option<usize> = None;
        let mut handle_bonus = OrderedFloat(0.0);
        let game_distributed = &mut self.game_distributed;
        for player_index in 0..game_distributed.players_in_game.len() {
            if game_distributed.players_in_game[player_index].owe_card() {
                owning_card_player_index = Some(player_index);
            }
            if game_distributed.players_in_game[player_index].missing_card() {
                missing_card_player_index = Some(player_index);
            }
            if let Some(handle) = &game_distributed.players_in_game[player_index].handle() {
                handle_bonus = handle.points();
                if !self.options.quiet {
                    println!("Handle bonus: {}", handle_bonus);
                }
            }
            match game_distributed.players_in_game[player_index].role() {
                Some(Role::Taker) => {
                    attack.push(player_index);
                }
                Some(Role::Ally) => {
                    assert!(ally_index.is_none());
                    ally_index = Some(player_index);
                    attack.push(player_index);
                }
                Some(Role::Defenser) => {
                    defense.push(player_index);
                }
                None => {
                    return Err(TarotErrorKind::NoRoleForPlayer(
                        game_distributed.game.players[player_index].name().to_string(),
                    ));
                }
            }
        }
        match game_distributed.game.mode {
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
        if let Some(owning_index) = owning_card_player_index {
            let low_card = game_distributed.players_in_game[owning_index].give_low();
            if let Some(low) = low_card {
                if let Some(missing_index) = missing_card_player_index {
                    game_distributed.players_in_game[missing_index].push_owned(low);
                }
            }
        }
        if let Some(ally_index) = ally_index {
            let ally_cards = game_distributed.players_in_game[ally_index].owned();
            game_distributed.players_in_game[self.taker_index].append_owned(&ally_cards)
        }

        let slam_bonus = game_distributed.players_in_game[self.taker_index].slam_bonus();
        let taker_points = game_distributed.players_in_game[self.taker_index].points();
        let points_for_oudlers = game_distributed.players_in_game[self.taker_index].points_for_oudlers()?;

        if !self.options.quiet {
            println!("Taker slam bonus: {}", slam_bonus);
            println!("Taker owned points: {}", taker_points);
            println!("Contract todo: {}", points_for_oudlers);
            println!("Contract base: {}", BASE_CONTRACT_POINTS);
            println!("Contract difference: {}", taker_points - points_for_oudlers);
        }

        let contract_points = if taker_points >= points_for_oudlers {
            if !self.options.quiet {
                println!(
                    "Contract total: {}",
                    taker_points - points_for_oudlers + BASE_CONTRACT_POINTS
                );
            }
            (taker_points - points_for_oudlers + BASE_CONTRACT_POINTS) * self.contract.multiplier()
        } else {
            if !self.options.quiet {
                println!(
                    "Contract total: {}",
                    taker_points - points_for_oudlers - BASE_CONTRACT_POINTS
                );
            }
            (taker_points - points_for_oudlers - BASE_CONTRACT_POINTS) * self.contract.multiplier()
        };
        if !self.options.quiet {
            println!("Taker contract: {} (x{})", self.contract, self.contract.multiplier());
            println!("Taker contract points: {}", contract_points);
        }

        let points_petit_au_bout = 10.0 * self.contract.multiplier();
        let petit_au_bout_bonus = match self.petit_au_bout {
            Some(Team::Defense) => {
                if !self.options.quiet {
                    println!("Petit au bout for defense: -{}", points_petit_au_bout);
                }
                -points_petit_au_bout
            }
            Some(Team::Attack) => {
                if !self.options.quiet {
                    println!("Petit au bout for attack: {}", points_petit_au_bout);
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

        let ratio = game_distributed.game.mode.ratio(ally_index.is_some());
        let points = contract_points + petit_au_bout_bonus + handle_bonus + slam_bonus;

        if contract_points >= OrderedFloat(0.0) {
            game_distributed.game.players[self.taker_index].add_score(ratio * points);
        } else {
            handle_bonus *= -1.0;
            game_distributed.game.players[self.taker_index].add_score(-ratio * points);
        }

        if !self.options.quiet {
            println!("Attack handle bonus: {}", handle_bonus);
            println!("Taker points: {}", points);
            println!("Taker total points: {}", game_distributed.game.players[self.taker_index].score());
        }

        if let Some(ally_index) = ally_index {
            if contract_points >= OrderedFloat(0.0) {
                game_distributed.game.players[ally_index].add_score(points);
            } else {
                game_distributed.game.players[ally_index].add_score(-points);
            }
            if !self.options.quiet {
                println!("Ally total points: {}", game_distributed.game.players[ally_index].score());
            }
        }

        for defenser_index in defense {
            if contract_points >= OrderedFloat(0.0) {
                game_distributed.game.players[defenser_index].add_score(-points);
            } else {
                game_distributed.game.players[defenser_index].add_score(points);
            }
            if !self.options.quiet {
                println!(
                    "Defenser {} total points: {}",
                    game_distributed.game.players[defenser_index].name(),
                    game_distributed.game.players[defenser_index].score()
                );
            }
        }
        //if handle_bonus != 0.0  && petit_au_bout_bonus != 0.0 && slam_bonus != 0.0 && ratio == 4.0 {
        //    helpers::wait_input();
        //}
        self.is_consistent()
    }
    pub fn finished(&self) -> bool {
        self.game_distributed.players_in_game.iter().all(|player| player.last_turn())
    }
    fn is_consistent(&self) -> Result<(), TarotErrorKind> {
        let mut sum = OrderedFloat(0.0);
        for player in &self.game_distributed.game.players {
            sum += player.score();
        }
        if sum != 0.0 {
            eprintln!("Inconsistent points sum : {}", sum);
            return Err(TarotErrorKind::InvalidScores(sum.to_string()));
        }
        Ok(())
    }
}

impl<'a, const MODE: usize> GameDistributed<'a, MODE> {
    pub fn bidding_and_discard(&'a mut self) -> Result<Option<GameStarted<'a, MODE>>, TarotErrorKind> {
        let mut contracts: Vec<Contract> = Contract::iter().collect();
        let mut slam_index: Option<usize> = None;
        let mut taker_index: Option<usize> = None;
        let mut contract: Option<Contract> = None;

        for (current_player_index, current_player_in_game) in self.players_in_game.iter_mut().enumerate() {
            let current_player = &self.game.players[current_player_index];
            let player_contract = current_player_in_game.choose_contract_among(current_player, &contracts);
            match (contract, player_contract) {
                (None, None) | (Some(_), None) => {}
                (None, Some(player_contract)) | (Some(_), Some(player_contract)) => {
                    taker_index = Some(current_player_index);
                    if !self.options.quiet {
                        println!(
                            "Player {} has chosen contract {}",
                            current_player.name(),
                            player_contract
                        );
                    }
                    contracts.retain(|other_contract| {
                        other_contract.multiplier() > player_contract.multiplier()
                    });
                    if current_player_in_game.announce_slam()? {
                        if !self.options.quiet {
                            println!(
                                "player {} announced a slam, {}",
                                current_player, current_player_index
                            );
                        }
                        slam_index = Some(current_player_index);
                    }
                    contract = Some(player_contract);
                }
            };
        }
        let Some(contract) = contract else {
            return Ok(None);
        };
        let Some(taker_index) = taker_index else {
            return Ok(None);
        };

        // RULE: player who slammed must start
        if let Some(slammer) = slam_index {
            if !self.options.quiet {
                println!("Chelem announced so {} must start.", slammer);
            }
            self.game.players.rotate_left(slammer);
            self.players_in_game.rotate_left(slammer);
        }

        let mut callee: Option<Card> = None;
        if let Mode::Five = self.game.mode {
            callee = Some(self.players_in_game[taker_index].call()?);
        }

        for (current_player_index, current_player) in self.players_in_game.iter_mut().enumerate() {
            current_player.set_callee(callee);
            current_player.set_team(Team::Defense);
            current_player.set_role(Role::Defenser);
            if current_player_index == taker_index {
                current_player.set_team(Team::Attack);
                current_player.set_role(Role::Taker);
            } else if let Some(ref card) = callee {
                if current_player.has(card) {
                    current_player.set_team(Team::Attack);
                    current_player.set_role(Role::Ally);
                }
            }
        }

        let (attackers, defensers): (Vec<_>, Vec<_>) = self
            .players_in_game
            .iter()
            .enumerate()
            .partition_map(|(i, player)| {
                if player.team() == Some(Team::Attack) {
                    Either::Left(i)
                } else {
                    Either::Right(i)
                }
            });

        for attacker_index in attackers {
            if self.players_in_game[attacker_index].role() != Some(Role::Taker) {
                continue;
            }

            match contract {
                Contract::GardeSans => {
                    self.players_in_game[attacker_index].set_discard(&self.dog);
                }
                Contract::GardeContre => {
                    if let Some(first_defenser_index) = defensers.first() {
                        self.players_in_game[*first_defenser_index].set_discard(&self.dog);
                    }
                }
                _ => {
                    if !self.options.quiet {
                        println!("In the dog, there was : {}", self.dog);
                    }
                    self.players_in_game[attacker_index].append_hand(&self.dog);
                    self.players_in_game[attacker_index].discard();
                }
            }
        }
        let game_started = GameStarted {
            taker_index,
            options: self.options,
            game_distributed: self,
            contract,
            petit_au_bout: None,
            defense_cards: 0,
            attack_cards: 0,
        };
        Ok(Some(game_started))
    }
}

impl<const MODE: usize> Game<MODE> {
    pub fn default() -> Result<Self, TarotErrorKind> {
        let mode: Mode = MODE.try_into()?;
        let players: [Player; MODE] =
            array_init(|i| Player::new(mode.player_name(i).to_string(), mode, Options::default()));
        Ok(Self {
            options: Options::default(),
            players,
            mode,
        })
    }
    pub fn new(options: Options) -> Result<Self, TarotErrorKind> {
        let mode: Mode = MODE.try_into()?;
        let players: [Player; MODE] =
            array_init(|i| Player::new(mode.player_name(i).to_string(), mode, options));
        Ok(Self {
            players,
            mode,
            options,
        })
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
            player.append_hand(&buffer)
        }

        for player in players_in_game.iter() {
            if player.petit_sec() {
                // RULE: PetitSec cancel the game
                return Err(TarotErrorKind::PetitSec);
            }
        }

        let game_distributed = GameDistributed {
            players_in_game,
            dog,
            options: self.options,
            game: self,
        };
        Ok(game_distributed)
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
