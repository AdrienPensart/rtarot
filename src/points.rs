use ordered_float::OrderedFloat;

pub const MAX_CARDS: usize = 78;
pub const BASE_CONTRACT_POINTS: OrderedFloat<f64> = OrderedFloat(25.0);
pub const MAX_POINTS: OrderedFloat<f64> = OrderedFloat(91.0);
pub const MAX_POINTS_WITHOUT_FOOL: OrderedFloat<f64> = OrderedFloat(87.0);

pub trait HasPoints {
    fn points(&self) -> OrderedFloat<f64>;
}
