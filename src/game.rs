use std::fmt;
use rand::Rng;
use strum::IntoEnumIterator;
use array_init::array_init;
use log::debug;
use crate::deck::Deck;
use crate::mode::Mode;
use crate::contract::Contract;
use crate::player::Player;
use crate::errors::TarotErrorKind;
use crate::turn::Turn;
use crate::card::Card;
use crate::role::Role;
use crate::team::Team;
use crate::traits::{Symbol, Points};
use crate::helpers::*;
use crate::constants::MAX_CARDS;

#[derive(Debug)]
pub struct Game<const MODE: usize> {
    dog: Deck,
    players: [Player; MODE],
    mode: Mode,
    auto: bool,
    petit_au_bout: Option<Team>,
    defense_cards: usize,
    attack_cards: usize,
}

impl<const MODE: usize> fmt::Display for Game<MODE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Game with dog {} and players : ", self.dog)?;
        for p in &self.players {
            writeln!(f, "\t{}", p)?;
        }
        Ok(())
    }
}

impl<const MODE: usize> Game<MODE>
{
    pub fn default() -> Result<Self, TarotErrorKind>
    {
        let mode: Mode = MODE.try_into()?;
        let players: [Player; MODE] = array_init(|i| Player::new(mode.player_name(i).to_string(), mode, false));
        Ok(Self {
            auto: false,
            petit_au_bout: None,
            defense_cards: 0,
            attack_cards: 0,
            dog: Deck::default(),
            players,
            mode,
        })
    }
    pub fn new(random: bool, auto: bool) -> Result<Self, TarotErrorKind> {
        let mode: Mode = MODE.try_into()?;
        let players: [Player; MODE] = array_init(|i| Player::new(mode.player_name(i).to_string(), mode, random));
        Ok(Self {
            auto,
            players,
            mode,
            ..Self::default()?
        })
    }
    pub fn start(mut self) -> Result<(), TarotErrorKind> {
        loop {
            self.distribute()?;
            self.bidding()?;
            if self.passed() {
                println!("Everyone passed !");
                continue
            }
            self.discard()?;
            while !self.finished() {
                self.play()?;
            }
            self.count_points()?;
            break
        }
        println!("GAME ENDED");
        println!("{}", self);
        Ok(())
    }
    fn is_consistent(&self) {
        let sum = self.players.iter().map(|p| p.total).sum::<f64>();
        debug!("Current points sum : {}", sum);
        assert!(sum == 0.0)
    }
    pub fn distribute(&mut self) -> Result<(), TarotErrorKind> {
        let mut new_deck = Deck::random();
        self.dog = Deck::empty();
        for player in self.players.iter_mut() {
            player.prepare();
        }
        self.petit_au_bout = None;
        self.defense_cards = 0;
        self.attack_cards = 0;

        self.dog = new_deck.give(self.mode.dog_size());
        self.dog.sort();
        for player in self.players.iter_mut() {
            player.hand.append(&mut new_deck.give(self.mode.cards_per_player()))
        }
        for player in self.players.iter_mut() {
            if player.hand.petit_sec() {
                // RULE: PetitSec cancel the game
                return Err(TarotErrorKind::PetitSec)
            }
            player.hand.sort();
        }
        Ok(())
    }
    pub fn bidding(&mut self) -> Result<(), TarotErrorKind> {
        let mut contracts: Vec<Contract> = Contract::iter().collect();
        let mut slam_index : Option<usize> = None;
        for (i, p) in self.players.iter_mut().enumerate() {
            if self.auto && contracts.len() == 1 {
                p.contract = Some(Contract::Pass);
                println!("Auto pass");
                continue
            }

            p.contract = if p.random {
                Some(contracts[rand::thread_rng().gen_range(0..contracts.len())])
            } else {
                loop {
                    println!("{} must play : {}", &p, &p.hand);
                    println!("Choose a contract, possibilities :");
                    for (contract_index, contract) in contracts.iter().enumerate() {
                        println!("\t{} (x{}) : press {}", contract, contract.multiplier(), contract_index)
                    }
                    let contract_index = read_index();
                    if contract_index < contracts.len() {
                        break Some(contracts[contract_index])
                    } else {
                        println!("Error, please retry");
                    }
                }
            };
            contracts = match p.contract {
                Some(contract) => {
                    println!("Player {} has chosen contract {}", p.name, contract);
                    contracts
                        .into_iter()
                        .filter(|other_contract| other_contract == &Contract::Pass || other_contract.multiplier() > contract.multiplier())
                        .collect()
                },
                _ => {
                    println!("A contract must be available for everyone!");
                    return Err(TarotErrorKind::InvalidCase)
                }
            };
            if p.contract != Some(Contract::Pass) && p.announce_slam() {
                slam_index = Some(i);
            }
        }
        // RULE: player who slammed must start
        if let Some(slammer) = slam_index {
            self.players.rotate_left(slammer);
        }
        Ok(())
    }
    pub fn passed(&self) -> bool {
        self.players.iter().all(|p| p.contract == Some(Contract::Pass))
    }
    pub fn finished(&self) -> bool {
        self.players.iter().all(|p| p.hand.is_empty())
    }
    pub fn discard(&mut self) -> Result<(), TarotErrorKind> {
        if self.passed() {
            return Err(TarotErrorKind::NoTaker);
        }

        let mut callee: Option<Card> = None;
        let contract: Option<Contract> = if let Some(taker) = self.players.iter_mut().max_by(|c1, c2| c1.contract.unwrap().cmp(&c2.contract.unwrap())) {
            println!("{} has taken", taker);
            if let Mode::Five = self.mode {
                callee = Some(taker.call()?);
            }
            taker.contract
        } else {
            return Err(TarotErrorKind::NoTaker);
        };

        for p in &mut self.players {
            println!("Player before : {}", p);
            p.callee = callee;
            p.team = Some(Team::Defense);
            p.role = Some(Role::Defenser);
            if p.contract == contract {
                p.team = Some(Team::Attack);
                p.role = Some(Role::Taker);
            } else if let Some(ref card) = callee {
                if p.hand.0.contains(card) {
                    p.team = Some(Team::Attack);
                    p.role = Some(Role::Ally);
                }
            }
            p.contract = contract;
            println!("Player after : {}", p);
        }

        let team_partitioner = |p: &'_ &mut Player| -> bool {
            match &p.team {
                Some(team) => team == &Team::Attack,
                _ => false
            }
        };

        let (takers, others): (Vec<_>, Vec<_>) = self.players.iter_mut().partition(team_partitioner);
        for taker in takers {
            if taker.role != Some(Role::Taker) {
                continue
            }

            match taker.contract {
                Some(Contract::Pass) => continue,
                Some(Contract::GardeSans) => {
                    taker.owned.append(&mut self.dog.give_all());
                    return Ok(())
                }
                Some(Contract::GardeContre) => {
                    for other in others {
                        other.owned.append(&mut self.dog.give_all());
                    }
                    return Ok(())
                },
                _ => {
                    let discard = self.dog.len();
                    println!("In the dog, there was : {}", &self.dog);
                    taker.hand.append(&mut self.dog.give_all());
                    taker.hand.sort();
                    taker.discard(discard);
                },
            }
        }
        Ok(())
    }
    pub fn play(&mut self) -> Result<(), TarotErrorKind> {
        let mut turn = Turn::default();
        let mut master_player: usize = 0;
        let mut master_player_name: String = self.players[master_player].name.clone();
        for (current_player_index, current_player) in self.players.iter_mut().enumerate() {
            if current_player.is_first_turn() {
                current_player.announce_handle();
            }
            println!("Hand of {} : {}", &current_player, &current_player.hand);
            println!("Choices :");
            let possible_choices = &current_player.choices(&turn)?;
            if possible_choices.is_empty() {
                println!("No possible choices available, invalid case.");
                return Err(TarotErrorKind::InvalidCase)
            }
            for &possible_choice in possible_choices {
                println!("\t{0: <4} : press {1}", current_player.hand.0[possible_choice], possible_choice);
            }

            if let Some(called) = turn.called() {
                println!("{} must play color {}", current_player.name, called.symbol())
            } else {
                println!("{} is first to play:", current_player.name)
            }

            let final_choice = if self.auto && possible_choices.len() == 1 {
                possible_choices[0]
            } else if current_player.random {
                possible_choices[rand::thread_rng().gen_range(0..possible_choices.len())]
            } else {
                loop {
                    let choice_index = read_index();
                    if possible_choices.contains(&choice_index) {
                        break choice_index
                    } else {
                        println!("Error, please retry")
                    }
                }
            };

            let card = current_player.give_one(final_choice);
            if card.is_fool() {
                if !current_player.last_turn() {
                    // RULE: the fool is always preserved to his owner
                    current_player.owned.push(card);
                    // we must mark as the fool was played
                    turn.fool_played = true;
                } else {
                    // RULE: exception in the last turn, the fool is in game and can be lost
                    turn.put(card);
                    match current_player.team {
                        Some(Team::Attack)  => {
                            if self.attack_cards == MAX_CARDS - self.mode.dog_size() {
                                turn.master_index = Some(turn.len()-1);
                                master_player = current_player_index;
                            }
                        },
                        Some(Team::Defense) => {
                            if self.defense_cards == MAX_CARDS - self.mode.dog_size() {
                                turn.master_index = Some(turn.len()-1);
                                master_player = current_player_index;
                            }
                        },
                        _ => {
                            return Err(TarotErrorKind::NoTeam)
                        }
                    }
                }
            } else {
                turn.put(card);
                if let Some(master) = turn.master_card() {
                    if master.master(card) {
                        println!("Master card is {}, so player {} stays master", master, master_player_name);
                    } else {
                        println!("Master card is {}, so player {} becomes master", card, current_player.name);
                        master_player = current_player_index;
                        master_player_name = current_player.name.clone();
                        turn.master_index = Some(turn.len()-1);
                    }
                } else {
                    println!("First card is {}, so player {} becomes master", card, current_player.name);
                    master_player = current_player_index;
                    master_player_name = current_player.name.clone();
                    turn.master_index = Some(turn.len()-1);
                }
            }
            println!("{}", &turn);
        }

        let mut cards = turn.take();
        println!("Winner is player {}", self.players[master_player]);
        // RULE: petit au bout works for last turn, or before last turn if a slam is occuring
        if cards.has_petit() &&
            (self.players[master_player].last_turn() ||
             (self.players[master_player].before_last_turn() &&
              ((self.attack_cards == MAX_CARDS - self.mode.dog_size() - self.mode.players() ) || (self.defense_cards == MAX_CARDS - self.mode.dog_size() - self.mode.players())))) {
            println!("{} has Petit in last turn (Petit au bout) : +10 points", self.players[master_player]);
            self.petit_au_bout = self.players[master_player].team.clone();
        }
        match self.players[master_player].team {
            Some(Team::Attack) => self.attack_cards += cards.len(),
            Some(Team::Defense) => self.defense_cards += cards.len(),
            _ => return Err(TarotErrorKind::NoTeam)
        }
        self.players[master_player].owned.append(&mut cards);
        self.players.rotate_left(master_player);
        Ok(())
    }
    pub fn count_points(&mut self) -> Result<(), TarotErrorKind> {
        if self.passed() {
            return Err(TarotErrorKind::NoTaker);
        }
        let mut taker_index : Option<usize> = None;
        let mut ally_index : Option<usize> = None;
        let mut defense : Vec<usize> = Vec::new();
        let mut owning_card_player_index : Option<usize> = None;
        let mut missing_card_player_index : Option<usize> = None;
        let mut handle_bonus = 0.0;
        for (i, p) in self.players.iter().enumerate() {
            if p.owe_card() {
                owning_card_player_index = Some(i);
            }
            if p.missing_card() {
                missing_card_player_index = Some(i);
            }
            if let Some(handle) = &p.handle {
                handle_bonus = handle.points();
                println!("Handle bonus: {}", handle_bonus);
            }
            match p.role {
                Some(Role::Taker) => {
                    assert!(taker_index.is_none());
                    taker_index = Some(i)
                }
                Some(Role::Ally) => {
                    assert!(ally_index.is_none());
                    ally_index = Some(i)
                }
                Some(Role::Defenser) => {
                    defense.push(i)
                }
                None => return Err(TarotErrorKind::InvalidCase),
            }
        }
        match self.mode {
            Mode::Three => assert!(defense.len() == 2),
            Mode::Four => assert!(defense.len() == 3),
            Mode::Five => {
                if ally_index.is_some() {
                    assert!(defense.len() == 3)
                } else {
                    assert!(defense.len() == 4)
                }
            }
        };

        // give a low card if someone one a card to someone else
        if let Some(owning_index) = owning_card_player_index {
            let low_card = self.players[owning_index].give_low();
            if let Some(low) = low_card {
                if let Some(missing_index) = missing_card_player_index {
                    self.players[missing_index].owned.push(low);
                }
            }
        }
        if let Some(ally_index) = ally_index {
            let mut ally_cards = self.players[ally_index].owned.give_all();
            if let Some(taker_index) = taker_index {
                self.players[taker_index].owned.append(&mut ally_cards)
            } else {
                println!("Cant merge cards of ally if no taker");
                return Err(TarotErrorKind::NoTaker);
            }
        }

        if let Some(taker_index) = taker_index {
            let slam_bonus = self.players[taker_index].slam_bonus();
            println!("Taker slam bonus: {}", &slam_bonus);
            let contract_points = self.players[taker_index].contract_points()?;
            println!("Taker contract points: {}", &contract_points);

            let petit_au_bout_bonus = if let Some(contract) = self.players[taker_index].contract {
                let points_petit_au_bout = 10.0 * contract.multiplier();
                match self.petit_au_bout {
                    Some(Team::Defense) => {
                        println!("Petit au bout for defense: {}", -points_petit_au_bout);
                        -points_petit_au_bout
                    },
                    Some(Team::Attack) => {
                        println!("Petit au bout for attack: {}", points_petit_au_bout);
                        points_petit_au_bout
                    },
                    None => 0.0
                }
            } else {
                return Err(TarotErrorKind::NoContract)
            };

            let ratio = match self.mode {
                Mode::Three => 2.0,
                Mode::Four => 3.0,
                Mode::Five => if ally_index.is_none() { 4.0 } else { 2.0 },
            };

            if contract_points < 0.0 {
                handle_bonus *= -1.0;
            }
            println!("Attack handle bonus: {}", &handle_bonus);

            let points = contract_points + petit_au_bout_bonus + handle_bonus + slam_bonus;
            println!("Taker points: {}", &points);

            self.players[taker_index].total = ratio * points;
            println!("Taker total points: {}", &self.players[taker_index].total);

            if let Some(ally_index) = ally_index {
                self.players[ally_index].total = points;
                println!("Ally total points: {}", &self.players[ally_index].total);
            }

            println!("Defense indexes : {:?}", defense);
            for defenser_index in defense {
                self.players[defenser_index].total = -1.0 * points;
                println!("Defenser {} total points: {}", defenser_index, &self.players[defenser_index].total);
            }
            //if handle_bonus != 0.0  && petit_au_bout_bonus != 0.0 && slam_bonus != 0.0 && ratio == 4.0 {
            //    helpers::wait_input();
            //}
        } else {
            println!("Cant count points if no taker");
            return Err(TarotErrorKind::NoTaker);
        }
        self.is_consistent();
        Ok(())
    }
}

#[test]
fn game_tests() {
    use crate::mode::*;
    test_game::<{Mode::Three.players()}>().unwrap();
    test_game::<{Mode::Four.players()}>().unwrap();
    test_game::<{Mode::Five.players()}>().unwrap();
}
