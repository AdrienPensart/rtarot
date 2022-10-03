use std::fmt;
use rand::Rng;
use failure::Error;
use strum::IntoEnumIterator;
use crate::itertools::Itertools;

use crate::traits::*;
use crate::contract::Contract;
use crate::deck::Deck;
use crate::color::Color;
use crate::color_value::ColorValue;
use crate::trump_value::TrumpValue;
use crate::card::Card;
use crate::errors::*;
use crate::team::Team;
use crate::role::Role;
use crate::mode::Mode;
use crate::turn::Turn;
use crate::handle::Handle;
use crate::constants::BASE_CONTRACT_POINTS;
use crate::helpers::*;

#[derive(Default, Clone, Debug)]
pub struct Player {
    pub name: String,
    pub contract: Option<Contract>,
    pub slam: bool,
    pub mode: Mode,
    pub team: Option<Team>,
    pub role: Option<Role>,
    pub hand: Deck,
    pub owned: Deck,
    pub callee: Option<Card>,
    pub total: f64,
    pub handle : Option<Handle>,
    pub random: bool,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.contract {
            Some(contract) => {
                write!(f, "{}, total: {}, contract: {}, slam: {}", &self.name, &self.total, &contract, &self.slam)?;
            },
            None => {
                write!(f, "{}, total: {}, no contract yet, slam: {}", self.name, &self.total, &self.slam)?;
            },
        }
        if let Some(role) = &self.role {
            write!(f, ", Role : {}", role)?;
        }
        if let Some(team) = &self.team {
            write!(f, ", Team : {}", team)?;
        }
        if let Some(callee) = &self.callee {
            write!(f, ", Callee : {}", callee)?;
        }
        Ok(())
    }
}

impl Player
{
    pub fn new(name: String, mode: Mode, random: bool) -> Player {
        Player {
            name,
            random,
            mode,
            ..Player::default()
        }
    }
    pub fn prepare(&mut self) {
        self.contract = None;
        self.slam = false;
        self.team = None;
        self.role = None;
        self.callee = None;
        self.handle = None;
    }
    pub fn slam_bonus(&self) -> f64 {
        if self.slam {
            if self.owned.is_chelem() {
                400.0
            } else {
                -200.0
            }
        } else if self.owned.is_chelem() {
            200.0
        } else if self.owned.is_empty() || (self.owned.len() == 1 && self.owned.has_fool()) {
            -200.0
        } else {
            0.0
        }
    }
    pub fn announce_slam(&mut self) -> bool {
        let slams = vec![
            false,
            true,
        ];
        self.slam = if self.random {
            slams[rand::thread_rng().gen_range(0..slams.len())]
        } else {
            loop {
                println!("Hand of {} : {}", &self, &self.hand);
                println!("Slam ? : ");
                for (i, s) in slams.iter().enumerate() {
                    println!("{} : press {}", s, i);
                }
                let slam_index = read_index();
                if slam_index < slams.len() {
                    break slams[slam_index]
                } else {
                    println!("Error, please retry")
                }
            }
        };
        self.slam
    }
    pub fn announce_handle(&mut self) {
        let mut trumps = self.hand.trumps();
        let discarded_trumps = self.owned.trumps();
        let mut total_trumps = trumps.len() + discarded_trumps.len();
        let handle = self.mode.handle(total_trumps);
        self.handle = match handle {
            None => None,
            Some(mut handle) => {
                let handles = match handle {
                    Handle::Simple  => vec![Handle::Refused, Handle::Simple],
                    Handle::Double  => vec![Handle::Refused, Handle::Simple, Handle::Double],
                    Handle::Triple  => vec![Handle::Refused, Handle::Simple, Handle::Double, Handle::Triple],
                    Handle::Refused => vec![]
                };
                handle = if self.random {
                    handles[rand::thread_rng().gen_range(0..handles.len())].clone()
                } else {
                    loop {
                        for &a in trumps.iter() {
                            println!("\t{}", &a);
                        }
                        println!("You have {} trumps, you can declare a handle : ", trumps.len());
                        for (i, handle) in handles.iter().enumerate() {
                            println!("{} (limit: {}, base points: {}) : press {}", handle, handle.points(), self.mode.handle_limit(handle), i);
                        }
                        let handle_index = read_index();
                        if handle_index < handles.len() {
                            break handles[handle_index].clone()
                        } else {
                            println!("Error, please retry")
                        }
                    }
                };
                if handle != Handle::Refused {
                    trumps.retain(|&c| !c.is_fool());
                    // RULE: cant use fool as trump when you have too much trumps for the handle
                    if total_trumps != trumps.len() + discarded_trumps.len() {
                        println!("You can't use fool as trumps in a handle");
                    }
                    trumps.extend(discarded_trumps.iter());
                    total_trumps = trumps.len();

                    let limit = self.mode.handle_limit(&handle);
                    if total_trumps > limit {
                        let mut to_discard = total_trumps - limit;
                        while to_discard > 0 {
                            loop {
                                for (i, a) in trumps.iter().enumerate() {
                                    println!("\t{0} : {1}", &i, &a);
                                }
                                println!("You must discards {} trumps to present only {}", &to_discard, &limit);
                                if self.random {
                                    let index_to_remove = rand::thread_rng().gen_range(0..trumps.len());
                                    trumps.remove(index_to_remove);
                                    break
                                } else {
                                    let trump_index = read_index();
                                    if trump_index < trumps.len() {
                                        trumps.remove(trump_index);
                                    } else {
                                        println!("Error, please retry")
                                    }
                                }
                            }
                            to_discard -= 1;
                        }
                    } else {
                        println!("You have exactly the good number of trumps");
                    }
                    println!("Final handle : ");
                    for a in trumps.iter() {
                        println!("\t{}", &a);
                    }
                }
                Some(handle)
            }
        };
    }
    pub fn last_turn(&self) -> bool {
        self.hand.is_empty()
    }
    pub fn before_last_turn(&self) -> bool {
        self.hand.len() == 1
    }
    pub fn owe_card(&self) -> bool {
        self.owned.has_fool() && self.owned.len() != 1 && (self.owned.len() % self.mode.players()) == 1
    }
    pub fn missing_card(&self) -> bool {
        !self.owned.has_fool() && self.owned.len() != 1 && (self.owned.len() % self.mode.players()) == (self.mode.players() - 1)
    }
    pub fn give_low(&mut self) -> Option<Card> {
        self.owned.give_low()
    }
    pub fn give_one(&mut self, index: usize) -> Card {
        self.hand.0.remove(index)
    }
    pub fn points(&self) -> f64 {
        self.owned.points()
    }
    pub fn count_oudlers(&self) -> usize {
        self.owned.count_oudlers()
    }
    pub fn contract_points(&self) -> Result<f64, Error> {
        let points = self.points();
        let contract_points = self.owned.points_for_oudlers()?;
        println!("Taker owned points: {}", &points);
        println!("Contract todo: {}", &contract_points);
        println!("Contract base: {}", BASE_CONTRACT_POINTS);
        println!("Contract difference: {}", points - contract_points);

        match self.contract {
            Some(Contract::Pass) | None => Err(TarotErrorKind::NoContract.into()),
            Some(contract) => {
                println!("Taker contract: {}", &contract);
                if points >= contract_points {
                    println!("Contract total: {}", points - contract_points + BASE_CONTRACT_POINTS);
                    Ok((points - contract_points + BASE_CONTRACT_POINTS) * contract.multiplier())
                } else {
                    println!("Contract total: {}", points - contract_points - BASE_CONTRACT_POINTS);
                    Ok((points - contract_points - BASE_CONTRACT_POINTS) * contract.multiplier())
                }
            }
        }
    }
    pub fn is_first_turn(&self) -> bool {
        match self.mode {
            Mode::Three => self.hand.len() == 24,
            Mode::Four => self.hand.len() == 18,
            Mode::Five => self.hand.len() == 15,
        }
    }
    pub fn call(&self) -> Result<Card, Error> {
        if self.mode != Mode::Five {
            return Err(TarotErrorKind::InvalidMode.into());
        }
        let mut value_callable : Vec<ColorValue> = vec![ColorValue::King];
        if self.hand.count_tete(ColorValue::King) == 4 {
            value_callable.push(ColorValue::Queen);
            if self.hand.count_tete(ColorValue::Queen) == 4 {
                value_callable.push(ColorValue::Knight);
                if self.hand.count_tete(ColorValue::Knight) == 4 {
                    value_callable.push(ColorValue::Jack);
                    if self.hand.count_tete(ColorValue::Jack) == 4 {
                        println!("Case too rare, taker has all kings, all queens and all knights");
                        return Err(TarotErrorKind::InvalidCase.into())
                    }
                }
            }
        }
        let choices : Vec<Card> = Color::iter().cartesian_product(value_callable.iter()).map(|(c, cv)| Card::normal(c, *cv)).collect();
        if self.random {
            Ok(choices[rand::thread_rng().gen_range(0..choices.len())])
        } else {
            loop {
                println!("Hand of taker {}", &self.hand);
                println!("Taker must choose a card to call his partner :");
                println!("Possibilities:");
                for (i, c) in choices.iter().enumerate() {
                    println!("\t{0: <3} : press {1}", c, i);
                }
                let choice_index = read_index();
                if choice_index < choices.len() {
                    break Ok(choices[choice_index])
                } else {
                    println!("Error, please retry")
                }
            }
        }
    }
    pub fn discard (&mut self, discard: usize) {
        println!("{}", self);
        for current in 0..discard {
            println!("You must discard {} cards", discard - current);
            let discardables_indexes = self.hand.discardables(discard);
            let discard_index = if self.random {
                discardables_indexes[rand::thread_rng().gen_range(0..discardables_indexes.len())]
            } else {
                loop {
                    println!("Hand of taker: {}", self.hand);
                    println!("Possibilities:");
                    for &i in &discardables_indexes {
                        println!("\t{0: <4} : press {1}", self.hand.0[i], i);
                    }
                    println!("Currently discarded: {}", self.owned);
                    let discard_index = read_index();
                    if discardables_indexes.contains(&discard_index) {
                        break discard_index
                    } else {
                        println!("Error, please retry")
                    }
                }
            };
            self.owned.push(self.hand.0.remove(discard_index));
        }

        for c in self.owned.trumps() {
            println!("This trump was discarded: {}", &c);
        }
        self.hand.sort();
    }
    pub fn choices(&self, turn: &Turn) -> Result<Vec<usize>, Error> {
        let mut and_fool : Option<usize> = None;
        let mut trumps = Vec::new();
        let mut trumps_less = Vec::new();
        let mut trumps_more = Vec::new();
        let mut other_colors = Vec::new();
        let mut same_color  = Vec::new();
        let mut compatibles = match (turn.called(), turn.master_card()) {
            (Some(Card::Normal(called_normal)), Some(Card::Normal(_))) => {
                for (i, card) in self.hand.0.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &TrumpValue::Fool {
                                and_fool = Some(i);
                            } else {
                                trumps.push(i);
                            }
                        },
                        Card::Normal(card_normal) => {
                            if card_normal.color == called_normal.color {
                                same_color.push(i);
                            } else {
                                other_colors.push(i);
                            }
                        }
                    }
                }
                if !same_color.is_empty() {
                    same_color
                } else if !trumps.is_empty() {
                    trumps
                } else {
                    other_colors
                }
            },
            (Some(Card::Normal(called_normal)), Some(Card::Trump(master_trump_value))) => {
                for (i, card) in self.hand.0.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &TrumpValue::Fool {
                                and_fool = Some(i);
                            } else if card_trump_value > master_trump_value {
                                trumps_more.push(i);
                            } else {
                                trumps_less.push(i);
                            }
                        },
                        Card::Normal(card_normal) => {
                            if card_normal.color == called_normal.color {
                                same_color.push(i);
                            } else {
                                other_colors.push(i);
                            }
                        }
                    }
                }
                if !same_color.is_empty() {
                    same_color
                } else if !trumps_more.is_empty() {
                    trumps_more
                } else if !trumps_less.is_empty() {
                    trumps_less
                } else {
                    other_colors
                }
            },
            (Some(Card::Trump(_)), Some(Card::Trump(master_trump_value))) => {
                for (i, card) in self.hand.0.iter().enumerate() {
                    if let Card::Trump(card_trump_value) = card {
                        if card_trump_value == &TrumpValue::Fool {
                            and_fool = Some(i);
                        } else {
                            trumps.push(i);
                            if card_trump_value > master_trump_value {
                                trumps_more.push(i);
                            } else {
                                trumps_less.push(i);
                                other_colors.push(i);
                            }
                        }
                    } else {
                        other_colors.push(i)
                    }
                }
                if !trumps_more.is_empty() {
                    trumps_more
                } else if !trumps_less.is_empty() {
                    trumps_less
                } else {
                    other_colors
                }
            },
            (Some(Card::Normal(_)), None) => {
                println!("There cannot be a called color and no master card, impossible case!");
                return Err(TarotErrorKind::InvalidCase.into())
            }
            (Some(Card::Trump(_)), Some(Card::Normal(_))) => {
                println!("There cannot be a called trump and a master color, impossible case!");
                return Err(TarotErrorKind::InvalidCase.into())
            }
            (Some(Card::Trump(_)), None) => {
                println!("There cannot be a called trump and not master, impossible case!");
                return Err(TarotErrorKind::InvalidCase.into())
            }
            (None, Some(_)) => {
                println!("There cannot be no called color and a master, impossible case!");
                return Err(TarotErrorKind::InvalidCase.into())
            },
            // RULE: first player can put the callee but no any other card in the same color
            (None, None) => match (self.is_first_turn(), self.mode) {
                (true, Mode::Five) => {
                    self.hand.0.iter().enumerate().filter(|(_, &card)| {
                        match (card, self.callee) {
                            (Card::Normal(normal), Some(Card::Normal(callee_normal))) => callee_normal.color != normal.color || normal.value == callee_normal.value,
                            _ => true
                        }
                    }).map(|(i, _)| i).collect()
                },
                _ => (0..self.hand.len()).collect()
            },
        };
        if let Some(fool_index) = and_fool {
            compatibles.push(fool_index);
        }
        Ok(compatibles)
    }
}

#[test]
fn player_tests() {
    use std::f64::EPSILON;
    use crate::constants::MAX_POINTS;

    let looser = Player {
        name: "Player looser".to_string(),
        contract: Some(Contract::Petite),
        mode: Mode::Four,
        ..Player::default()
    };
    println!("looser: {}", &looser);

    assert!(looser.points() == 0.0);
    assert!(looser.count_oudlers() == 0);
    println!("looser points: {}", looser.contract_points().unwrap());
    assert!((looser.contract_points().unwrap() - -81.0).abs() < EPSILON);

    let mut winner = Player {
        name: "Player looser".to_string(),
        contract: Some(Contract::GardeContre),
        owned: Deck::build_deck(),
        mode: Mode::Five,
        random: true,
        ..Player::default()
    };
    winner.callee = Some(winner.call().unwrap());
    let turn = Turn::default();
    println!("{}", &winner.hand);
    let choices = &winner.choices(&turn).unwrap();
    println!("Choices :");
    for &i in choices {
        println!("\t{0: <2} : {1}", &i, &winner.hand.0[i]);
    }

    assert!((winner.points() - MAX_POINTS).abs() < EPSILON);
    assert!(winner.count_oudlers() == 3);
    println!("winner points: {}", winner.contract_points().unwrap());
    assert!((winner.contract_points().unwrap() - 480.0).abs() < EPSILON);

    let mut handle_owner = Player {
        name: "Player looser".to_string(),
        contract: Some(Contract::GardeContre),
        callee: Some(Card::normal(Color::Club, ColorValue::King)),
        mode: Mode::Five,
        random: true,
        ..Player::default()
    };

    handle_owner.announce_handle();
}
