use ordered_float::OrderedFloat;

pub const MAX_CARDS: usize = 78;
pub const BASE_CONTRACT_POINTS: OrderedFloat<f64> = OrderedFloat(25.0);
pub const MAX_POINTS: OrderedFloat<f64> = OrderedFloat(91.0);
/// RULE: a cut leaves at least this many cards on each side.
pub const MIN_CUT: usize = 3;
