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
use itertools::{Either, Itertools};
use std::fmt;
use strum::IntoEnumIterator;

pub struct GameDistributed<'a, const MODE: usize> {
    game: &'a mut Game<MODE>,
    options: Options,
    dog: Deck,
    players_in_game: [PlayerInGame; MODE],
}

impl<const MODE: usize> fmt::Display for GameDistributed<'_, MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Game {} with dog {}", self.game.mode(), self.dog)
    }
}

impl<'a, const MODE: usize> GameDistributed<'a, MODE> {
    pub fn new(
        game: &'a mut Game<MODE>,
        dog: Deck,
        players_in_game: [PlayerInGame; MODE],
        options: Options,
    ) -> Self {
        Self {
            game,
            players_in_game,
            dog,
            options,
        }
    }
    pub fn game(&mut self) -> &mut Game<MODE> {
        self.game
    }
    pub fn players_and_their_game_mut(&mut self) -> (&[Player; MODE], &mut [PlayerInGame; MODE]) {
        (self.game.players(), &mut self.players_in_game)
    }
    pub fn player(&self, index: usize) -> &Player {
        self.game.player(index)
    }
    pub fn player_and_his_game(&self, index: usize) -> (&Player, &PlayerInGame) {
        (self.game.player(index), &self.players_in_game[index])
    }
    pub fn player_and_his_game_mut(&mut self, index: usize) -> (&Player, &mut PlayerInGame) {
        (self.game.player(index), &mut self.players_in_game[index])
    }
    pub fn finished(&self) -> bool {
        self.players_in_game.iter().all(|player| player.last_turn())
    }
    pub fn rotate_at(&mut self, index: usize) {
        self.players_in_game.rotate_left(index);
        self.game.rotate_at(index);
    }
    pub fn bidding_and_discard(
        &'a mut self,
    ) -> Result<Option<GameStarted<'a, MODE>>, TarotErrorKind> {
        let mut contracts: Vec<Contract> = Contract::iter().collect();
        let mut slam_index: Option<usize> = None;
        let mut taker_index: Option<usize> = None;
        let mut contract: Option<Contract> = None;

        for (current_player_index, current_player_in_game) in
            self.players_in_game.iter_mut().enumerate()
        {
            let current_player = &self.game.player(current_player_index);
            let player_contract =
                current_player_in_game.choose_contract_among(current_player, &contracts);
            match (contract, player_contract) {
                (None, None) | (Some(_), None) => {}
                (None, Some(player_contract)) | (Some(_), Some(player_contract)) => {
                    taker_index = Some(current_player_index);
                    if !self.options.quiet {
                        println!(
                            "Player {} has chosen contract {player_contract}",
                            current_player.name()
                        );
                    }
                    contracts.retain(|other_contract| {
                        other_contract.multiplier() > player_contract.multiplier()
                    });
                    if current_player_in_game.announce_slam()? {
                        if !self.options.quiet {
                            println!("Player {current_player} announced a slam");
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
                println!("Chelem announced so {slammer} must start.");
            }
            self.rotate_at(slammer);
        }

        let callee = self.players_in_game[taker_index].call();
        for (current_player_index, current_player) in self.players_in_game.iter_mut().enumerate() {
            current_player.set_callee(callee);
            current_player.set_team(Team::Defense);
            current_player.set_role(Role::Defenser);
            if current_player_index == taker_index {
                current_player.set_team(Team::Attack);
                current_player.set_role(Role::Taker);
            } else if let Some(ref card) = callee {
                if current_player_index != taker_index && current_player.has(card) {
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
                if player.is_attack() {
                    Either::Left(i)
                } else {
                    Either::Right(i)
                }
            });

        for attacker_index in attackers {
            if !self.players_in_game[attacker_index].is_taker() {
                continue;
            }

            match contract {
                Contract::GardeSans => {
                    if !self.options.quiet {
                        println!("Attacker keeps dog because GardeSans");
                    }
                    self.players_in_game[attacker_index].set_discard(&self.dog);
                }
                Contract::GardeContre => {
                    if let Some(first_defenser_index) = defensers.first() {
                        if !self.options.quiet {
                            println!("Attacker gives dog to first defenser because GardeContre");
                        }
                        self.players_in_game[*first_defenser_index].set_discard(&self.dog);
                    }
                }
                _ => {
                    if !self.options.quiet {
                        let taker_name = self.player(attacker_index).name();
                        println!("In the dog, there was : {}", self.dog);
                        println!("Taker {taker_name} received the dog");
                    }
                    self.players_in_game[attacker_index].extend_hand(&self.dog);
                    self.players_in_game[attacker_index].discard();
                }
            }
        }
        let game_started = GameStarted::new(self, taker_index, contract, self.options);
        Ok(Some(game_started))
    }
}
