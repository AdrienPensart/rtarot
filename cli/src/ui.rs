//! Everything the terminal asks a human, and everything it shows.

use crate::render;
use colored::Colorize;
use rtarot_core::{Answer, Card, Engine, Event, Points, Question, TarotErrorKind};
use std::io;

/// Read a number from the terminal, forever if need be.
#[must_use]
pub fn read_index() -> usize {
    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok()
            && let Ok(output) = input.trim().parse::<usize>()
        {
            return output;
        }
    }
}

fn pick(prompt: &str, choices: &[String]) -> usize {
    loop {
        println!("{prompt}");
        for (index, choice) in choices.iter().enumerate() {
            println!("\t{choice} : press {index}");
        }
        let index = read_index();
        if index < choices.len() {
            return index;
        }
        println!("Error, please retry");
    }
}

/// Ask a human to answer `question`, showing him his own hand first.
pub fn ask(engine: &Engine, question: &Question) -> Result<Answer, TarotErrorKind> {
    let player = question.player();
    let name = engine.player(player)?.name().to_string();
    if let Some(deal) = engine.deal() {
        println!(
            "{name} hand : {}",
            render::hand(deal.player(player)?.hand())
        );
    }
    match question {
        Question::Contract { allowed, .. } => {
            let mut choices = vec!["Pass".to_string()];
            choices.extend(allowed.iter().map(|c| format!("{c} (x{})", c.multiplier())));
            let index = pick(&format!("{name} must choose a contract :"), &choices);
            Ok(Answer::Contract(
                index.checked_sub(1).and_then(|i| allowed.get(i).copied()),
            ))
        }
        Question::Slam { .. } => {
            let index = pick(
                &format!("{name}, announce a slam ?"),
                &["No".into(), "Yes".into()],
            );
            Ok(Answer::Slam(index == 1))
        }
        Question::Handle { allowed, .. } => {
            let choices: Vec<String> = allowed
                .iter()
                .map(|handle| format!("{handle} handle ({} points)", handle.points()))
                .collect();
            let index = pick(&format!("{name} may announce a handle :"), &choices);
            allowed
                .get(index)
                .copied()
                .map(Answer::Handle)
                .ok_or(TarotErrorKind::NoHandle(index))
        }
        Question::Call { allowed, .. } => {
            card_question(&format!("{name} must call a partner :"), allowed)
        }
        Question::Discard {
            allowed, remaining, ..
        } => card_question(
            &format!("{name} must discard {remaining} more card(s) :"),
            allowed,
        ),
        Question::TrimHandle {
            shown, to_remove, ..
        } => card_question(
            &format!("{name} must drop {to_remove} trump(s) from his handle :"),
            shown,
        ),
        Question::Play { allowed, .. } => {
            if let Some(deal) = engine.deal()
                && let Some(called) = deal.turn().called()
            {
                println!("{name} must follow {}", render::card(*called));
            }
            card_question(&format!("{name} must play a card :"), allowed)
        }
    }
}

fn card_question(prompt: &str, allowed: &[rtarot_core::Card]) -> Result<Answer, TarotErrorKind> {
    let choices: Vec<String> = allowed
        .iter()
        .map(|c| render::card(*c).to_string())
        .collect();
    let index = pick(prompt, &choices);
    allowed
        .get(index)
        .copied()
        .map(Answer::Card)
        .ok_or(TarotErrorKind::NoCard(index))
}

/// The king `seat` called, when `seat` is the taker of the running deal.
fn called_king(engine: &Engine, seat: usize) -> Option<Card> {
    let deal = engine.deal()?;
    if deal.taker() != Some(seat) {
        return None;
    }
    deal.player(seat).ok()?.callee()
}

/// Show what just happened.
#[allow(clippy::too_many_lines)]
pub fn show(engine: &Engine, event: &Event) {
    let name = |index: usize| -> String {
        engine
            .player(index)
            .map_or_else(|_| format!("player {index}"), |p| p.name().to_string())
    };
    match event {
        Event::DealStarted { deals_left, dealer } => {
            println!(
                "\n=== Deals left : {deals_left} (dealer : {}) ===",
                name(*dealer)
            );
        }
        Event::DeckCut { player } => println!("{} cuts the deck", name(*player)),
        Event::PetitSec { player } => {
            println!("{} has a petit sec, cancel the game", name(*player));
        }
        Event::Bid { player, contract } => match contract {
            Some(contract) => println!("{} has chosen contract {contract}", name(*player)),
            None => println!("{} passed", name(*player)),
        },
        Event::EveryonePassed => println!("Everyone passed !"),
        Event::KingCalled { player, card } => {
            println!(
                "{} called {} to find an ally",
                name(*player),
                render::card(*card)
            );
        }
        Event::SlamAnnounced { player } => println!("{} announced a slam", name(*player)),
        Event::DogGivenToTaker { player, dog } => {
            println!("In the dog, there was :{}", render::hand(dog));
            println!("{}", render::art(dog));
            println!("Taker {} received the dog", name(*player));
            if called_king(engine, *player).is_some_and(|king| dog.has(&king)) {
                println!(
                    "The called king was in the dog: {} plays alone",
                    name(*player)
                );
            }
        }
        Event::DogKeptByAttack { player } => {
            println!("{} keeps the dog because Garde Sans", name(*player));
        }
        Event::DogGivenToDefense { player } => {
            println!(
                "Dog goes to defenser {} because Garde Contre",
                name(*player)
            );
        }
        Event::Discarded { player, card } => {
            println!("{} discarded {}", name(*player), render::card(*card));
        }
        Event::HandleAnnounced {
            player,
            handle,
            trumps,
        } => {
            let shown = trumps
                .iter()
                .map(|c| render::card(*c).to_string())
                .collect::<Vec<_>>()
                .join(" ");
            println!(
                "{} announced a {handle} handle ({} points) : {shown}",
                name(*player),
                handle.points()
            );
        }
        Event::CardPlayed { player, card } if called_king(engine, *player) == Some(*card) => {
            println!("{} plays {}", name(*player), render::card(*card));
            println!(
                "{} called a king of their own and plays alone",
                name(*player)
            );
        }
        Event::CardPlayed { player, card } => {
            println!("{} played {}", name(*player), render::card(*card));
        }
        Event::FoolKept { player } => println!("{} keeps the fool", name(*player)),
        Event::FoolLost { player } => {
            println!(
                "{} played the fool in the last turn, it is lost",
                name(*player)
            );
        }
        Event::MasterChanged { player, card } => {
            println!(
                "{} becomes master with {}",
                name(*player),
                render::card(*card)
            );
        }
        Event::TurnWon { player, cards } => {
            println!("{} wins the turn ({cards} cards)", name(*player).bold());
        }
        Event::PetitAuBout { player, team } => {
            println!("Petit au bout for {} (team {team})", name(*player));
        }
        Event::FoolCompensation { from, to, card } => {
            println!(
                "{} owes a card to {}, giving {} in exchange",
                name(*from),
                name(*to),
                render::card(*card)
            );
        }
        Event::SlamResult {
            announced,
            realized,
            bonus,
            ..
        } => {
            println!("Slam announced: {announced}, realized: {realized}, bonus: {bonus}");
        }
        Event::DealScored {
            taker,
            ally,
            attack_points,
            needed,
            contract,
            contract_points,
            petit_au_bout,
            handles,
            slam,
            points,
        } => {
            println!("Taker : {}", name(*taker));
            if let Some(ally) = ally {
                println!("Ally : {}", name(*ally));
            }
            println!("Attack made {attack_points} points, needed {needed}");
            println!(
                "Contract {contract} (x{}) : {contract_points}",
                contract.multiplier()
            );
            println!("Petit au bout : {petit_au_bout}, handles : {handles}, slam : {slam}");
            println!("Deal is worth {points} for the attack");
            for player in engine.players() {
                println!("\t{player}");
            }
            if let Some(deal) = engine.last_deal() {
                println!("\nAs dealt :");
                for (seat, hand) in deal.dealt().iter().enumerate() {
                    println!("{} :{}", name(seat), render::hand(hand));
                }
                println!("Dog :{}", render::hand(deal.dog()));
            }
        }
        Event::GameFinished => {
            println!("\nGAME ENDED");
            for player in engine.players() {
                println!("\t{player}");
            }
        }
    }
}
