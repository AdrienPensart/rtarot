/// A color name, so that a user interface can paint a card without the
/// engine knowing anything about terminals, HTML or canvases.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    Blue,
    Cyan,
    Green,
    Red,
    Yellow,
    Plain,
}

pub trait Power {
    fn power(&self) -> usize;
}

pub trait Discardable {
    fn discardable(&self) -> bool;
    fn discardable_forced(&self) -> bool;
}

pub trait Representation {
    /// Suggested color for this element.
    fn color(&self) -> Color;
    /// Short textual symbol, as used in a compact hand listing.
    fn symbol(&self) -> &'static str;
    /// Multi line ascii art. For a colored card, `*` stands for its suit symbol.
    fn art(&self) -> &'static str;
}
