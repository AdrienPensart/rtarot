use colored::ColoredString;

pub trait Points {
    fn points(&self) -> f64;
}

pub trait Power {
    fn power(&self) -> usize;
}

pub trait Discardable {
    fn discardable(&self) -> bool;
    fn discardable_forced(&self) -> bool;
}

pub trait Representation {
    fn repr(&self) -> ColoredString;
}

pub trait Colored {
    fn color(&self) -> &'static str;
}
