use std::fmt;
use rand::Rng;
use failure::Error;

use crate::traits::*;
use crate::contract::*;
use crate::deck::*;
use crate::color::*;
use crate::trump::*;
use crate::card::*;
use crate::errors::*;
use crate::team::*;
use crate::role::*;
use crate::mode::*;
use crate::turn::*;
use crate::handle::*;
use crate::helpers::*;

//pub struct PlayerState<'a> {
//    pub contract: Option<Contract>,
//    pub team: Option<Team>,
//    pub role: Option<Role>,
//    pub callee: Option<Card>,
//    pub handle : Option<Handle>,
//    pub slam: bool,
//    pub hand: Deck<'a>,
//    pub owned: Deck<'a>,
//}
//
//#[derive(Default, Clone, Debug)]
//pub struct Player {
//    pub name: String,
//    pub total: f64,
//    pub mode: Mode,
//    random: bool,
//}

#[derive(Default, Clone, Debug)]
pub struct Player {
    pub name: String,
    pub contract: Option<Contract>,
    pub slam: bool,
    pub team: Option<Team>,
    pub role: Option<Role>,
    pub hand: Deck,
    pub owned: Deck,
    pub callee: Option<Card>,
    pub total: f64,
    pub mode: Mode,
    pub handle : Option<Handle>,
    random: bool,
}

pub fn default_name(index: usize) -> Result<String, Error> {
    match index {
        0 => Ok("East".to_string()),
        1 => Ok("North".to_string()),
        2 => Ok("South".to_string()),
        3 => Ok("West".to_string()),
        4 => Ok("Compass".to_string()),
        _ => Err(TarotErrorKind::InvalidCase)?
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.contract {
            Some(contract) => {
                write!(f, "{}, total: {}, contract: {}, slam: {}", &self.name, &self.total, &contract, &self.slam)?;
            },
            _ => {
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
    pub fn new(mode: Mode, random: bool) -> Player {
        Player {
            mode,
            random,
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
            slams[rand::thread_rng().gen_range(0, slams.len())]
        } else {
            loop {
                println!("Hand of {} : {}", &self, &self.hand);
                println!("Slam ? : ");
                for (i, s) in slams.iter().enumerate() {
                    println!("{} : press {}", s, i);
                }
                let slam_index: Result<usize, _> = read_index();
                match slam_index {
                    Ok(index) => {
                        break slams[index]
                    },
                    Err(_) => {
                        println!("Error, please retry")
                    }
                }
            }
        };
        self.slam
    }
    pub fn announce_handle(&mut self) {
        let mut trumps = self.hand.trumps();
        let discarded_trumps = self.owned.trumps();
        let mut total_trumps = trumps.len() + discarded_trumps.len();
        let handle = Handle::new(total_trumps, self.mode);
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
                    handles[rand::thread_rng().gen_range(0, handles.len())].clone()
                } else {
                    loop {
                        for &a in trumps.iter() {
                            println!("\t{}", &a);
                        }
                        println!("You have {} trumps, you can declare a handle : ", trumps.len());
                        for (i, h) in handles.iter().enumerate() {
                            println!("{} (limit: {}) : press {}", h, h.limit(self.mode), i);
                        }
                        let handle_index: Result<usize, _> = read_index();
                        match handle_index {
                            Ok(index) => {
                                break handles[index].clone()
                            },
                            Err(_) => {
                                println!("Error, please retry")
                            }
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

                    let limit = handle.limit(self.mode);
                    if total_trumps > limit {
                        let mut to_discard = total_trumps - limit;
                        while to_discard > 0 {
                            loop {
                                for (i, a) in trumps.iter().enumerate() {
                                    println!("\t{0} : {1}", &i, &a);
                                }
                                println!("You must discards {} trumps to present only {}", &to_discard, &limit);
                                if self.random {
                                    let index_to_remove = rand::thread_rng().gen_range(0, trumps.len());
                                    trumps.remove(index_to_remove);
                                    break
                                } else {
                                    let trump_index: Result<usize, _> = read_index();
                                    match trump_index {
                                        Ok(index) => {
                                            trumps.remove(index);
                                            break
                                        },
                                        Err(_) => {
                                            println!("Error, please retry")
                                        }
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
        self.owned.has_fool() && self.owned.len() != 1 && (self.owned.len() % self.mode as usize) == 1
    }
    pub fn missing_card(&self) -> bool {
        !self.owned.has_fool() && self.owned.len() != 1 && (self.owned.len() % self.mode as usize) == (self.mode as usize - 1)
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
        println!("Contract difference: {}", points - contract_points);

        match self.contract {
            Some(Contract::Pass) | None => Err(TarotErrorKind::NoContract)?,
            Some(contract) => {
                println!("Taker contract: {}", &contract);
                if points >= contract_points {
                    println!("Contract total: {}", points - contract_points + 25.0);
                    Ok((points - contract_points + 25.0) * f64::from(contract as u8))
                } else {
                    println!("Contract total: {}", points - contract_points - 25.0);
                    Ok((points - contract_points - 25.0) * f64::from(contract as u8))
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
    pub fn call(&mut self) -> Result<Option<Card>, Error> {
        if self.mode != Mode::Five {
            Err(TarotErrorKind::InvalidMode)?;
        }
        let value = if self.hand.count_tete(ColorValue::King) == 4 {
            if self.hand.count_tete(ColorValue::Queen) == 4 {
                if self.hand.count_tete(ColorValue::Knight) == 4 {
                    if self.hand.count_tete(ColorValue::Jack) == 4 {
                        debug!("Case too rare, taker has all kings, all queens and all knights");
                        Err(TarotErrorKind::InvalidCase)?
                    } else {
                        println!("You have 4 kings, 4 queens and 4 knights, you must choose a jack as partner");
                        ColorValue::Jack
                    }
                } else {
                    println!("You have 4 kings and 4 queens, you must choose a knight as partner");
                    ColorValue::Knight
                }
            } else {
                println!("You have 4 kings, you must choose a queen as partner");
                ColorValue::Queen
            }
        } else {
            println!("You must choose a king as partner");
            ColorValue::King
        };
        let colors = vec![
            Color::Spade,
            Color::Heart,
            Color::Diamond,
            Color::Club,
        ];

        let color = if self.random {
            &colors[rand::thread_rng().gen_range(0, colors.len())]
        } else {
            loop {
                println!("Hand of taker {}", &self.hand);
                println!("Taker must choose a color to call his partner :");
                println!("Possibilities:");
                for (i, c) in colors.iter().enumerate() {
                    println!("\t{0: <3} : press {1}", c, i);
                }
                let color_index: Result<usize, _> = read_index();
                match color_index {
                    Ok(index) => {
                        if index < colors.len() {
                            break &colors[index]
                        } else {
                            println!("Error, please retry");
                        }
                    },
                    Err(_) => {
                        println!("Error, please retry")
                    }
                }
            }
        };
        Ok(Some(Card::Color(*color, value)))
    }
    pub fn discard (&mut self, discard: usize) {
        println!("{}", &self);
        println!("You must discard {} cards", discard);
        for _ in 0..discard {
            let discardables_indexes = self.hand.discardables(discard);
            let discard_index = if self.random {
                discardables_indexes[rand::thread_rng().gen_range(0, discardables_indexes.len())]
            } else {
                loop {
                    println!("Hand of taker {}", &self.hand);
                    println!("Possibilities:");
                    for &i in &discardables_indexes {
                        println!("\t{0: <4} : press {1}", self.hand.0[i], i);
                    }
                    let index: Result<usize, _> = read_index();
                    match index {
                        Ok(index) => {
                            if discardables_indexes.contains(&index) {
                                break index
                            } else {
                                println!("Error, please retry");
                            }
                        },
                        Err(_) => {
                            println!("Error, please retry")
                        }
                    }
                }
            };
            self.owned.push(self.hand.0.remove(discard_index));
        }

        for c in self.owned.trumps() {
            println!("This trump was discarded: {}", &c);
        }
    }
    pub fn choices(&self, turn: &Turn) -> Vec<usize> {
        let mut and_fool : Option<usize> = None;
        let mut trumps = Vec::new();
        let mut trumps_less = Vec::new();
        let mut trumps_more = Vec::new();
        let mut other_colors = Vec::new();
        let mut same_color  = Vec::new();
        let mut compatibles = match (turn.called(), turn.master_card()) {
            (Some(Card::Color(called_color, _)), Some(Card::Color(_, _))) => {
                for (i, card) in self.hand.0.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &TrumpValue::Fool {
                                and_fool = Some(i);
                            } else {
                                trumps.push(i);
                            }
                        },
                        Card::Color(card_color, _) => {
                            if card_color == called_color {
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
            (Some(Card::Color(called_color, _)), Some(Card::Trump(master_trump_value))) => {
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
                        Card::Color(card_color, _) => {
                            if card_color == called_color {
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
            _ => (0..self.hand.len()).collect()
        };
        if let Some(fool_index) = and_fool {
            compatibles.push(fool_index);
        }
        compatibles
    }
}

#[test]
fn player_tests() {
    use std::f64::EPSILON;
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

    let winner = Player {
        name: "Player looser".to_string(),
        contract: Some(Contract::GardeContre),
        callee: Some(Card::Color(Color::Club, ColorValue::King)),
        owned: Deck::build_deck(),
        mode: Mode::Five,
        ..Player::default()
    };
    let turn = Turn::default();
    println!("{}", &winner.hand);
    let choices = &winner.choices(&turn);
    println!("Choices :");
    for &i in choices {
        println!("\t{0: <2} : {1}", &i, &winner.hand.0[i]);
    }

    assert!((winner.points() - 91.0).abs() < EPSILON);
    assert!(winner.count_oudlers() == 3);
    println!("winner points: {}", winner.contract_points().unwrap());
    assert!((winner.contract_points().unwrap() - 480.0).abs() < EPSILON);

    let mut handle_owner = Player {
        name: "Player looser".to_string(),
        contract: Some(Contract::GardeContre),
        callee: Some(Card::Color(Color::Club, ColorValue::King)),
        mode: Mode::Five,
        random: true,
        ..Player::default()
    };

    handle_owner.announce_handle();
}
