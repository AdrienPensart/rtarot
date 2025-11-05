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
use itertools::{Either, Itertools};
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
    pub fn rotate_at(&mut self, index: usize) {
        self.players_in_game.rotate_left(index);
        self.game.rotate_at(index);
    }
    pub fn bidding_and_discard(
        &'a mut self,
    ) -> Result<Option<GameStarted<'a, MODE>>, TarotErrorKind> {
        let quiet = self.options.quiet;
        let mut contracts: Vec<Contract> = Contract::iter().collect();
        let mut slam_index: Option<usize> = None;
        let mut taker_index: Option<usize> = None;
        let mut contract: Option<Contract> = None;

        for (current_player_index, current_player_in_game) in
            self.players_in_game.iter_mut().enumerate()
        {
            let current_player = self.game.player(current_player_index)?;
            let player_contract =
                current_player_in_game.choose_contract_among(current_player, &contracts)?;
            match (contract, player_contract) {
                (None | Some(_), None) => {}
                (None | Some(_), Some(player_contract)) => {
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
            }
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

        let Some(taker) = self.players_in_game.get(taker_index) else {
            return Err(TarotErrorKind::NoTaker(taker_index));
        };
        let callee = taker.call()?;
        for (current_player_index, current_player) in self.players_in_game.iter_mut().enumerate() {
            current_player.set_callee(callee);
            current_player.set_team(Team::Defense);
            current_player.set_role(Role::Defenser);
            if current_player_index == taker_index {
                current_player.set_team(Team::Attack);
                current_player.set_role(Role::Taker);
            } else if let Some(ref card) = callee
                && current_player_index != taker_index
                && current_player.has(card)
            {
                current_player.set_team(Team::Attack);
                current_player.set_role(Role::Ally);
            }
        }

        let (attacker_indices, defenser_indices): (Vec<_>, Vec<_>) = self
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

        for attacker_index in attacker_indices {
            let attacker_in_game = self
                .players_in_game
                .get_mut(attacker_index)
                .ok_or(TarotErrorKind::NoAttacker(attacker_index))?;

            if !attacker_in_game.is_taker() {
                continue;
            }

            let taker = self
                .game
                .player(attacker_index)
                .map_err(|_| TarotErrorKind::NoTaker(attacker_index))?;

            match contract {
                Contract::GardeSans => {
                    if !quiet {
                        println!("Attacker keeps dog because GardeSans");
                    }
                    attacker_in_game.set_discard(&self.dog);
                }
                Contract::GardeContre => {
                    if let Some(first_defenser_index) = defenser_indices.first() {
                        if !quiet {
                            println!("Attacker gives dog to first defenser because GardeContre");
                        }
                        let Some(defenser) = self.players_in_game.get_mut(*first_defenser_index)
                        else {
                            return Err(TarotErrorKind::NoDefenser(*first_defenser_index));
                        };
                        defenser.set_discard(&self.dog);
                    }
                }
                Contract::Petite | Contract::Garde => {
                    if !quiet {
                        println!("In the dog, there was : {}", self.dog);
                        println!("Taker {} received the dog", taker.name());
                    }
                    attacker_in_game.extend_hand(&self.dog);
                    attacker_in_game.discard()?;
                }
            }
        }
        let game_started = GameStarted::new(self, taker_index, contract, self.options);
        Ok(Some(game_started))
    }
}

impl<const MODE: usize> fmt::Display for GameDistributed<'_, MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Game {} with dog {}", self.game.mode(), self.dog)
    }
}
