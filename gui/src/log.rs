//! Turning engine events into sentences for the table talk panel.

use crate::game::Summary;
use rtarot_core::{Card, Engine, Event, Points};

fn cards(cards: &[Card]) -> String {
    cards
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

/// One sentence per event, the way a kibitzer would say it.
pub fn describe(engine: &Engine, event: &Event) -> String {
    let name = |index: usize| -> String {
        engine
            .player(index)
            .map_or_else(|_| format!("player {index}"), |p| p.name().to_string())
    };
    match event {
        Event::DealStarted { deals_left, dealer } => {
            format!("— New deal, {deals_left} left, {} deals —", name(*dealer))
        }
        Event::PetitSec { player } => {
            format!("{} has a petit sec, the deal is cancelled", name(*player))
        }
        Event::Bid { player, contract } => contract.map_or_else(
            || format!("{} passes", name(*player)),
            |contract| format!("{} bids {contract}", name(*player)),
        ),
        Event::EveryonePassed => "Everyone passed".to_string(),
        Event::KingCalled { player, card } => format!("{} calls {card}", name(*player)),
        Event::SlamAnnounced { player } => format!("{} announces a slam!", name(*player)),
        Event::DogGivenToTaker { player, dog } => {
            format!("The dog held {}, {} takes it", cards(dog), name(*player))
        }
        Event::DogKeptByAttack { player } => {
            format!("{} keeps the dog unseen (Garde Sans)", name(*player))
        }
        Event::DogGivenToDefense { player } => {
            format!(
                "The dog goes to the defense, to {} (Garde Contre)",
                name(*player)
            )
        }
        Event::Discarded { player, card } => format!("{} discards {card}", name(*player)),
        Event::HandleAnnounced {
            player,
            handle,
            trumps,
        } => format!(
            "{} shows a {handle} handle ({} points): {}",
            name(*player),
            handle.points(),
            cards(trumps)
        ),
        Event::CardPlayed { player, card } => format!("{} plays {card}", name(*player)),
        Event::FoolKept { player } => format!("{} keeps the fool", name(*player)),
        Event::FoolLost { player } => {
            format!(
                "{} played the fool in the last trick, it is lost",
                name(*player)
            )
        }
        Event::MasterChanged { player, card } => format!("{} leads with {card}", name(*player)),
        Event::TurnWon { player, cards } => {
            format!("{} wins the trick ({cards} cards)", name(*player))
        }
        Event::PetitAuBout { player, team } => {
            format!("Petit au bout by {} for the {team}", name(*player))
        }
        Event::FoolCompensation { from, to, card } => {
            format!(
                "{} owes a card to {} and gives {card}",
                name(*from),
                name(*to)
            )
        }
        Event::SlamResult {
            attack,
            announced,
            realized,
            bonus,
        } => {
            let side = if *attack { "attack" } else { "defense" };
            format!(
                "Slam for the {side}: announced {announced}, realized {realized}, bonus {bonus}"
            )
        }
        Event::DealScored {
            taker,
            ally,
            attack_points,
            needed,
            contract,
            points,
            ..
        } => {
            let ally = ally.map_or_else(String::new, |ally| format!(" with {}", name(ally)));
            format!(
                "{}{ally} made {attack_points} points on a {contract}, needed {needed}: {points} for the attack",
                name(*taker)
            )
        }
        Event::GameFinished => "— Game over —".to_string(),
    }
}

/// The score sheet of a deal, from its [`Event::DealScored`].
pub fn summary(engine: &Engine, event: &Event) -> Summary {
    let name = |index: usize| -> String {
        engine
            .player(index)
            .map_or_else(|_| format!("player {index}"), |p| p.name().to_string())
    };
    let mut rows = Vec::new();
    if let Event::DealScored {
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
    } = event
    {
        rows.push(("Taker".to_string(), name(*taker)));
        if let Some(ally) = ally {
            rows.push(("Ally".to_string(), name(*ally)));
        }
        rows.push((
            "Contract".to_string(),
            format!("{contract} (x{})", contract.multiplier()),
        ));
        rows.push((
            "Attack made".to_string(),
            format!("{attack_points} points, needed {needed}"),
        ));
        rows.push(("Contract points".to_string(), contract_points.to_string()));
        rows.push(("Petit au bout".to_string(), petit_au_bout.to_string()));
        rows.push(("Handles".to_string(), handles.to_string()));
        rows.push(("Slam".to_string(), slam.to_string()));
        rows.push(("Deal worth".to_string(), format!("{points} for the attack")));
        for player in engine.players() {
            rows.push((player.name().to_string(), player.score().to_string()));
        }
    }
    Summary { rows }
}
