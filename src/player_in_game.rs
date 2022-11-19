use derive_new::new;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, WeightedAliasIndex, WeightedError};
use std::fmt;
use strum::IntoEnumIterator;

use crate::card::Card;
use crate::contract::Contract;
use crate::deck::Deck;
use crate::errors::TarotErrorKind;
use crate::handle::Handle;
use crate::helpers::*;
use crate::mode::Mode;
use crate::options::Options;
use crate::player::Player;
use crate::points::HasPoints;
use crate::role::Role;
use crate::suit::Suit;
use crate::suit_value::SuitValue;
use crate::team::Team;
use crate::traits::Symbol;
use crate::trump::Trump;
use crate::turn::Turn;

#[derive(new, Eq, PartialEq, Clone, Debug)]
pub struct PlayerInGame {
    mode: Mode,
    options: Options,
    #[new(default)]
    slam: bool,
    #[new(default)]
    team: Option<Team>,
    #[new(default)]
    role: Option<Role>,
    #[new(default)]
    discard: Deck,
    #[new(default)]
    hand: Deck,
    #[new(default)]
    owned: Deck,
    #[new(default)]
    callee: Option<Card>,
    #[new(default)]
    handle: Option<Handle>,
}

impl fmt::Display for PlayerInGame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Slam: {}", self.slam)?;
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

impl HasPoints for PlayerInGame {
    fn points(&self) -> OrderedFloat<f64> {
        self.owned.points()
    }
}

impl PlayerInGame {
    pub fn petit_sec(&self) -> bool {
        self.hand.petit_sec()
    }
    pub fn set_callee(&mut self, callee: Option<Card>) {
        self.callee = callee
    }
    pub fn set_team(&mut self, team: Team) {
        self.team = Some(team)
    }
    pub fn set_role(&mut self, role: Role) {
        self.role = Some(role)
    }
    pub fn callee(&self) -> Option<Card> {
        self.callee
    }
    pub fn has(&self, card: &Card) -> bool {
        self.hand.has(card)
    }
    pub fn set_discard(&mut self, discard: &Deck) {
        self.discard = discard.clone()
    }
    pub fn all_cards(&self) -> Deck {
        let mut all_cards = self.owned.clone();
        all_cards.extend(&self.discard);
        all_cards
    }
    pub fn push_owned(&mut self, card: Card) {
        self.owned.push(card);
    }
    pub fn extend_hand(&mut self, deck: &Deck) {
        self.hand.extend(deck);
        self.hand.sort();
    }
    pub fn extend_owned(&mut self, deck: &Deck) {
        self.owned.extend(deck);
    }
    pub fn role(&self) -> Option<Role> {
        self.role
    }
    pub fn team(&self) -> Option<Team> {
        self.team
    }
    pub fn handle(&self) -> Option<Handle> {
        self.handle
    }
    pub fn points_for_oudlers(&self) -> Result<OrderedFloat<f64>, TarotErrorKind> {
        self.owned.points_for_oudlers()
    }
    pub fn play_card(&mut self, player: &Player, turn: &mut Turn) -> Result<Card, TarotErrorKind> {
        let Some(_) = self.role else {
            return Err(TarotErrorKind::NoRoleForPlayer(player.name().to_string()));
        };

        let Some(_) = self.team else {
            return Err(TarotErrorKind::NoTeamForPlayer(player.name().to_string()));
        };

        if (!self.owned.has_fool() && (self.owned.len() % self.mode.players() != 0))
            || (self.owned.has_fool() && (self.owned.len() % self.mode.players() != 1))
        {
            eprintln!("{}", self.owned.len() % self.mode.players());
            return Err(TarotErrorKind::InvalidDeck(self.owned.clone()));
        }

        if self.is_first_turn() {
            self.announce_handle();
        }
        if !self.options.quiet {
            println!("Hand of {} : {}", player.name(), self.hand);
            println!("Choices :");
        }
        let possible_choices = &self.choices(turn)?;
        if possible_choices.is_empty() {
            eprintln!("No possible choices available, invalid case.");
            return Err(TarotErrorKind::InvalidCase);
        }

        if !self.options.quiet {
            for &possible_choice in possible_choices {
                println!(
                    "\t{0: <4} : press {1}",
                    self.hand[possible_choice], possible_choice
                );
            }
            if let Some(called) = turn.called() {
                println!("{} must play color {}", player.name(), called.symbol())
            } else {
                println!("{} is first to play:", player.name())
            }
        }

        let final_choice = if self.options.auto && possible_choices.len() == 1 {
            possible_choices[0]
        } else if self.options.random {
            possible_choices[rand::thread_rng().gen_range(0..possible_choices.len())]
        } else {
            loop {
                let choice_index = read_index();
                if possible_choices.contains(&choice_index) {
                    break choice_index;
                } else if !self.options.quiet {
                    println!("Error, please retry")
                }
            }
        };
        Ok(self.hand.remove(final_choice))
    }
    pub fn choose_contract_among(
        &mut self,
        player: &Player,
        contracts: &Vec<Contract>,
    ) -> Option<Contract> {
        if self.options.auto && contracts.len() == 1 {
            if !self.options.quiet {
                println!("{} : auto pass", player.name());
            }
            return None;
        }

        if self.options.random {
            let random_choice_index = rand::thread_rng().gen_range(0..contracts.len() + 1);
            if random_choice_index == 0 {
                return None;
            }
            Some(contracts[random_choice_index - 1])
        } else {
            loop {
                if !self.options.quiet {
                    println!("{} must play : {}", &self, &self.hand);
                    println!("Choose a contract, possibilities :");
                    println!("\tPass : press 0");
                    for (contract_index, contract) in contracts.iter().enumerate() {
                        println!(
                            "\t{} (x{}) : press {}",
                            contract,
                            contract.multiplier(),
                            contract_index + 1
                        );
                    }
                }
                let contract_index = read_index();
                if contract_index == 0 {
                    return None;
                }
                if contract_index < contracts.len() + 1 {
                    break Some(contracts[contract_index - 1]);
                } else if !self.options.quiet {
                    println!("Error, please retry");
                }
            }
        }
    }
    pub fn slam_bonus(&self) -> f64 {
        if self.slam {
            if self.owned.is_chelem() {
                if !self.options.quiet {
                    println!("{} : chelem announced and realized !", self);
                }
                400.0
            } else {
                -200.0
            }
        } else if self.owned.is_chelem() {
            if !self.options.quiet {
                println!("{} : chelem not announced but realized !", self);
            }
            200.0
        } else if self.owned.is_empty() || self.owned.only_fool() {
            -200.0
        } else {
            0.0
        }
    }
    pub fn announce_slam(&mut self) -> Result<bool, WeightedError> {
        if self.options.no_slam {
            return Ok(false);
        }
        let slams = vec![false, true];
        self.slam = if self.options.random {
            let weights = vec![99, 1];
            let dist = WeightedAliasIndex::new(weights)?;
            let mut rng = thread_rng();
            slams[dist.sample(&mut rng)]
        } else {
            loop {
                if !self.options.quiet {
                    println!("Hand of {} : {}", &self, &self.hand);
                    println!("Slam ? : ");
                    for (i, s) in slams.iter().enumerate() {
                        println!("{} : press {}", s, i);
                    }
                }
                let slam_index = read_index();
                if slam_index < slams.len() {
                    break slams[slam_index];
                } else if !self.options.quiet {
                    println!("Error, please retry")
                }
            }
        };
        Ok(self.slam)
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
                    Handle::Simple => vec![Handle::Refused, Handle::Simple],
                    Handle::Double => vec![Handle::Refused, Handle::Simple, Handle::Double],
                    Handle::Triple => vec![
                        Handle::Refused,
                        Handle::Simple,
                        Handle::Double,
                        Handle::Triple,
                    ],
                    Handle::Refused => vec![],
                };
                handle = if self.options.random {
                    handles[rand::thread_rng().gen_range(0..handles.len())]
                } else {
                    loop {
                        if !self.options.quiet {
                            for &a in trumps.iter() {
                                println!("\t{}", &a);
                            }
                            println!(
                                "You have {} trumps, you can declare a handle : ",
                                trumps.len()
                            );
                            for (handle_index, handle) in handles.iter().enumerate() {
                                println!(
                                    "{} handle (needs: {} trumps, points: {}) : press {}",
                                    handle,
                                    self.mode.handle_limit(handle),
                                    handle,
                                    handle_index
                                );
                            }
                        }
                        let handle_index = read_index();
                        if handle_index < handles.len() {
                            break handles[handle_index];
                        } else if !self.options.quiet {
                            println!("Error, please retry")
                        }
                    }
                };
                if handle != Handle::Refused {
                    trumps.retain(|&c| !c.is_fool());
                    // RULE: cant use fool as trump when you have too much trumps for the handle
                    if total_trumps != trumps.len() + discarded_trumps.len() && !self.options.quiet
                    {
                        println!("You can't use fool as trumps in a handle");
                    }
                    trumps.extend(discarded_trumps.iter());
                    total_trumps = trumps.len();

                    let limit = self.mode.handle_limit(&handle);
                    if total_trumps > limit {
                        let mut to_discard = total_trumps - limit;
                        while to_discard > 0 {
                            loop {
                                if !self.options.quiet {
                                    for (i, a) in trumps.iter().enumerate() {
                                        println!("\t{0} : {1}", &i, &a);
                                    }
                                    println!(
                                        "You must discards {} trumps to present only {}",
                                        &to_discard, &limit
                                    );
                                }
                                if self.options.random {
                                    let index_to_remove =
                                        rand::thread_rng().gen_range(0..trumps.len());
                                    trumps.remove(index_to_remove);
                                    break;
                                } else {
                                    let trump_index = read_index();
                                    if trump_index < trumps.len() {
                                        trumps.remove(trump_index);
                                    } else if !self.options.quiet {
                                        println!("Error, please retry")
                                    }
                                }
                            }
                            to_discard -= 1;
                        }
                    } else if !self.options.quiet {
                        println!("You have exactly the good number of trumps");
                    }
                    if !self.options.quiet {
                        println!("Final handle : ");
                        for a in trumps.iter() {
                            println!("\t{}", &a);
                        }
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
        self.owned.has_fool()
            && self.owned.len() > 1
            && (self.owned.len() % self.mode.players()) == 1
    }
    pub fn missing_card(&self) -> bool {
        !self.owned.has_fool()
            && self.owned.len() > 1
            && (self.owned.len() % self.mode.players()) == (self.mode.players() - 1)
    }
    pub fn give_low(&mut self) -> Option<Card> {
        self.owned.give_low()
    }
    pub fn count_oudlers(&self) -> usize {
        self.owned.count_oudlers()
    }
    pub fn is_first_turn(&self) -> bool {
        match self.mode {
            Mode::Three => self.hand.len() == 24,
            Mode::Four => self.hand.len() == 18,
            Mode::Five => self.hand.len() == 15,
        }
    }
    pub fn call(&self) -> Result<Card, TarotErrorKind> {
        if self.mode != Mode::Five {
            return Err(TarotErrorKind::InvalidMode);
        }
        let mut value_callable: Vec<SuitValue> = vec![SuitValue::King];
        if self.hand.count_tete(SuitValue::King) == 4 {
            value_callable.push(SuitValue::Queen);
            if self.hand.count_tete(SuitValue::Queen) == 4 {
                value_callable.push(SuitValue::Knight);
                if self.hand.count_tete(SuitValue::Knight) == 4 {
                    value_callable.push(SuitValue::Jack);
                    if self.hand.count_tete(SuitValue::Jack) == 4 {
                        eprintln!("Case too rare, taker has all kings, all queens and all knights");
                        return Err(TarotErrorKind::InvalidCase);
                    }
                }
            }
        }
        let choices: Vec<Card> = Suit::iter()
            .cartesian_product(value_callable.iter())
            .map(|(c, cv)| Card::normal(c, *cv))
            .collect();
        if self.options.random {
            Ok(choices[rand::thread_rng().gen_range(0..choices.len())])
        } else {
            loop {
                if !self.options.quiet {
                    println!("Hand of taker {}", &self.hand);
                    println!("Taker must choose a card to call his partner :");
                    println!("Possibilities:");
                    for (i, c) in choices.iter().enumerate() {
                        println!("\t{0: <3} : press {1}", c, i);
                    }
                }
                let choice_index = read_index();
                if choice_index < choices.len() {
                    break Ok(choices[choice_index]);
                } else if !self.options.quiet {
                    println!("Error, please retry")
                }
            }
        }
    }
    pub fn discard(&mut self) {
        if !self.options.quiet {
            println!("{}", self);
        }
        let dog_size = self.mode.dog_size();
        for current in 0..dog_size {
            if !self.options.quiet {
                println!("You must discard {} cards", dog_size - current);
            }
            let discardables_indexes = self.hand.discardables(dog_size);
            let discard_index = if self.options.random {
                discardables_indexes[rand::thread_rng().gen_range(0..discardables_indexes.len())]
            } else {
                loop {
                    if !self.options.quiet {
                        println!("Hand of taker: {}", self.hand);
                        println!("Possibilities:");
                        for &i in &discardables_indexes {
                            println!("\t{0: <4} : press {1}", self.hand[i], i);
                        }
                        println!("Currently discarded: {}", self.owned);
                    }
                    let discard_index = read_index();
                    if discardables_indexes.contains(&discard_index) {
                        break discard_index;
                    } else if !self.options.quiet {
                        println!("Error, please retry")
                    }
                }
            };
            if !self.options.quiet {
                println!("Discarded : {}", self.hand[discard_index]);
            }
            self.discard.push(self.hand.remove(discard_index));
        }

        if !self.options.quiet {
            for c in self.owned.trumps() {
                println!("This trump was discarded: {}", &c);
            }
        }
        self.hand.sort();
    }
    pub fn choices(&self, turn: &Turn) -> Result<Vec<usize>, TarotErrorKind> {
        let mut and_fool: Option<usize> = None;
        let mut trumps = Vec::new();
        let mut trumps_less = Vec::new();
        let mut trumps_more = Vec::new();
        let mut other_colors = Vec::new();
        let mut same_color = Vec::new();
        let mut compatibles = match (turn.called(), turn.master_card()) {
            (Some(Card::Normal(called_normal)), Some(Card::Normal(_))) => {
                for (i, card) in self.hand.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &Trump::Fool {
                                and_fool = Some(i);
                            } else {
                                trumps.push(i);
                            }
                        }
                        Card::Normal(card_normal) => {
                            if card_normal.suit() == called_normal.suit() {
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
            }
            (Some(Card::Normal(called_normal)), Some(Card::Trump(master_trump_value))) => {
                for (i, card) in self.hand.iter().enumerate() {
                    match card {
                        Card::Trump(card_trump_value) => {
                            if card_trump_value == &Trump::Fool {
                                and_fool = Some(i);
                            } else if card_trump_value > master_trump_value {
                                trumps_more.push(i);
                            } else {
                                trumps_less.push(i);
                            }
                        }
                        Card::Normal(card_normal) => {
                            if card_normal.suit() == called_normal.suit() {
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
            }
            (Some(Card::Trump(_)), Some(Card::Trump(master_trump_value))) => {
                for (i, card) in self.hand.iter().enumerate() {
                    if let Card::Trump(card_trump_value) = card {
                        if card_trump_value == &Trump::Fool {
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
            }
            (Some(Card::Normal(_)), None) => {
                eprintln!("There cannot be a called color and no master card, impossible case!");
                return Err(TarotErrorKind::InvalidCase);
            }
            (Some(Card::Trump(_)), Some(Card::Normal(_))) => {
                eprintln!("There cannot be a called trump and a master color, impossible case!");
                return Err(TarotErrorKind::InvalidCase);
            }
            (Some(Card::Trump(_)), None) => {
                eprintln!("There cannot be a called trump and not master, impossible case!");
                return Err(TarotErrorKind::InvalidCase);
            }
            (None, Some(_)) => {
                eprintln!("There cannot be no called color and a master, impossible case!");
                return Err(TarotErrorKind::InvalidCase);
            }
            // RULE: first player can put the callee but no any other card in the same color
            (None, None) => match (self.is_first_turn(), self.mode) {
                (true, Mode::Five) => self
                    .hand
                    .iter()
                    .enumerate()
                    .filter(|(_, &card)| match (card, self.callee) {
                        (Card::Normal(normal), Some(Card::Normal(callee_normal))) => {
                            callee_normal.suit() != normal.suit() || normal.value() == callee_normal.value()
                        }
                        _ => true,
                    })
                    .map(|(i, _)| i)
                    .collect(),
                _ => (0..self.hand.len()).collect(),
            },
        };
        if let Some(fool_index) = and_fool {
            compatibles.push(fool_index);
        }
        Ok(compatibles)
    }
}
