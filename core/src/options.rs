use std::fmt;
use strum::EnumIter;

/// The direction in which the deal, the bidding and the play go around the table.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, EnumIter)]
pub enum Direction {
    /// Seat indices go up: 0, 1, 2...
    #[default]
    Clockwise,
    /// Seat indices go down, the way a real tarot table turns.
    CounterClockwise,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Clockwise => write!(f, "clockwise"),
            Self::CounterClockwise => write!(f, "counter-clockwise"),
        }
    }
}

impl Direction {
    /// The seat after `seat`, around a table of `players`.
    #[must_use]
    pub const fn next(self, seat: usize, players: usize) -> usize {
        match self {
            Self::Clockwise => (seat + 1) % players,
            Self::CounterClockwise => (seat + players - 1) % players,
        }
    }
    /// The seat before `seat`, around a table of `players`.
    #[must_use]
    pub const fn previous(self, seat: usize, players: usize) -> usize {
        match self {
            Self::Clockwise => (seat + players - 1) % players,
            Self::CounterClockwise => (seat + 1) % players,
        }
    }
    /// The seat `steps` seats after `seat`.
    #[must_use]
    pub const fn step(self, seat: usize, steps: usize, players: usize) -> usize {
        match self {
            Self::Clockwise => (seat + steps) % players,
            Self::CounterClockwise => (seat + players - (steps % players)) % players,
        }
    }
}

/// How the cards reach the hands at the start of a deal.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, EnumIter)]
pub enum Dealing {
    /// RULE: the pack is shuffled once at the start of the game, never again.
    /// Before each deal the previous dealer cuts it, then the dealer gives the
    /// cards three by three, dropping the dog one card at a time in between,
    /// never the first nor the last card of the pack.
    #[default]
    Traditional,
    /// A fresh shuffle before every deal, each hand taken as one block.
    Random,
}

impl fmt::Display for Dealing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Traditional => write!(f, "cut and dealt three by three"),
            Self::Random => write!(f, "shuffled before every deal"),
        }
    }
}

/// The table rules a game is played with.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Options {
    pub direction: Direction,
    pub dealing: Dealing,
}

#[cfg(test)]
mod tests {
    use super::Direction;

    #[test]
    fn clockwise_goes_up_and_wraps() {
        let d = Direction::Clockwise;
        assert_eq!(d.next(3, 4), 0);
        assert_eq!(d.previous(0, 4), 3);
        assert_eq!(d.step(2, 3, 4), 1);
        assert_eq!(d.step(2, 0, 4), 2);
    }

    #[test]
    fn counter_clockwise_goes_down_and_wraps() {
        let d = Direction::CounterClockwise;
        assert_eq!(d.next(0, 4), 3);
        assert_eq!(d.previous(3, 4), 0);
        assert_eq!(d.step(2, 3, 4), 3);
        assert_eq!(d.step(2, 4, 4), 2);
        assert_eq!(d.step(1, 7, 5), 4);
    }

    #[test]
    fn next_and_previous_undo_each_other() {
        for direction in [Direction::Clockwise, Direction::CounterClockwise] {
            for players in 3..=5 {
                for seat in 0..players {
                    assert_eq!(
                        direction.previous(direction.next(seat, players), players),
                        seat
                    );
                    assert_eq!(
                        direction.step(seat, 1, players),
                        direction.next(seat, players)
                    );
                }
            }
        }
    }
}
