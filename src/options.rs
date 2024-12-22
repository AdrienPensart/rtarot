#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Options {
    pub random: bool,
    pub auto: bool,
    pub quiet: bool,
    pub no_slam: bool,
    pub test: bool,
    pub attack: bool,
}
