use colored::ColoredString;

pub trait Power {
    fn power(&self) -> usize;
}

pub trait Discardable {
    fn discardable(&self) -> bool;
    fn discardable_forced(&self) -> bool;
}

pub trait Representation {
    fn color(&self) -> &'static str;
    fn colored_symbol(&self) -> ColoredString;
    fn symbol(&self) -> &'static str;
    fn repr(&self) -> ColoredString;
    fn full_repr(&self) -> ColoredString;
}
