use ordered_float::OrderedFloat;

pub trait Points {
    fn points(&self) -> OrderedFloat<f64>;
}
