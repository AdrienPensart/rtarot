use crate::card::Card;
use crate::contract::Contract;
use crate::deck::Deck;
use crate::handle::Handle;
use crate::team::Team;
use ordered_float::OrderedFloat;

/// What just happened. A user interface renders these however it wants;
/// the engine never prints anything itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    DealStarted {
        deals_left: u64,
        dealer: usize,
    },
    PetitSec {
        player: usize,
    },
    Bid {
        player: usize,
        contract: Option<Contract>,
    },
    EveryonePassed,
    KingCalled {
        player: usize,
        card: Card,
    },
    SlamAnnounced {
        player: usize,
    },
    DogGivenToTaker {
        player: usize,
        dog: Deck,
    },
    DogKeptByAttack {
        player: usize,
    },
    DogGivenToDefense {
        player: usize,
    },
    Discarded {
        player: usize,
        card: Card,
    },
    HandleAnnounced {
        player: usize,
        handle: Handle,
        trumps: Vec<Card>,
    },
    CardPlayed {
        player: usize,
        card: Card,
    },
    FoolKept {
        player: usize,
    },
    FoolLost {
        player: usize,
    },
    MasterChanged {
        player: usize,
        card: Card,
    },
    TurnWon {
        player: usize,
        cards: usize,
    },
    PetitAuBout {
        player: usize,
        team: Team,
    },
    FoolCompensation {
        from: usize,
        to: usize,
        card: Card,
    },
    SlamResult {
        attack: bool,
        announced: bool,
        realized: bool,
        bonus: OrderedFloat<f64>,
    },
    DealScored {
        taker: usize,
        ally: Option<usize>,
        attack_points: OrderedFloat<f64>,
        needed: OrderedFloat<f64>,
        contract: Contract,
        contract_points: OrderedFloat<f64>,
        petit_au_bout: OrderedFloat<f64>,
        handles: OrderedFloat<f64>,
        slam: OrderedFloat<f64>,
        /// Signed from the attack point of view, before the taker's ratio.
        points: OrderedFloat<f64>,
    },
    GameFinished,
}
