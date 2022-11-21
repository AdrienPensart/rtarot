use crate::points::HasPoints;
use crate::traits::{Discardable, Representation};
use colored::{ColoredString, Colorize};
use indoc::indoc;
use ordered_float::OrderedFloat;
use std::fmt;
use strum::EnumIter;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum SuitValue {
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    Jack = 11,
    Knight = 12,
    Queen = 13,
    King = 14,
}

impl fmt::Display for SuitValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.colored_symbol())
    }
}

impl Discardable for SuitValue {
    fn discardable(&self) -> bool {
        // RULE: cant discard kings
        self != &Self::King
    }
    fn discardable_forced(&self) -> bool {
        // RULE: cant discard kings
        self != &Self::King
    }
}

impl HasPoints for SuitValue {
    fn points(&self) -> OrderedFloat<f64> {
        let points = match self {
            Self::Jack => 1.5,
            Self::Knight => 2.5,
            Self::Queen => 3.5,
            Self::King => 4.5,
            _ => 0.5,
        };
        OrderedFloat(points)
    }
}

impl Representation for SuitValue {
    fn color(&self) -> &'static str {
        "normal"
    }
    fn symbol(&self) -> &'static str {
        match self {
            Self::Jack => "V",
            Self::Knight => "C",
            Self::Queen => "Q",
            Self::King => "K",
            Self::_1 => "1",
            Self::_2 => "2",
            Self::_3 => "3",
            Self::_4 => "4",
            Self::_5 => "5",
            Self::_6 => "6",
            Self::_7 => "7",
            Self::_8 => "8",
            Self::_9 => "9",
            Self::_10 => "10",
        }
    }
    fn colored_symbol(&self) -> ColoredString {
        self.symbol().color(self.color())
    }
    fn repr(&self) -> ColoredString {
        self.symbol().normal()
    }
    fn full_repr(&self) -> ColoredString {
        let base = match self {
            Self::_1 => indoc! {r#"
            ****************
            *     __       *
            *    /  |      *
            *    `| |      *
            *     | |      *
            *    _| |_     *
            *   |_____|    *
            *              *
            ****************"#},
            Self::_2 => indoc! {r#"
            ****************
            *    _____     *
            *   / ___ `.   *
            *  |_/___) |   *
            *   .'____.'   *
            *  / /____     *
            *  |_______|   *
            *              *
            ****************"#},
            Self::_3 => indoc! {r#"
            ****************
            *    ______    *
            *   / ____ `.  *
            *   `'  __) |  *
            *   _  |__ '.  *
            *  | \____) |  *
            *   \______.'  *
            *              *
            ****************"#},
            Self::_4 => indoc! {r#"
            ****************
            *   _    _     *
            *  | |  | |    *
            *  | |__| |_   *
            *  |____   _|  *
            *      _| |_   *
            *     |_____|  *
            *              *
            ****************"#},
            Self::_5 => indoc! {r#"
            ****************
            *   _______    *
            *  |  _____|   *
            *  | |____     *
            *  '_.____''.  *
            *  | \____) |  *
            *   \______.'  *
            *              *
            ****************"#},
            Self::_6 => indoc! {r#"
            ****************
            *    ______    *
            *  .' ____ \   *
            *  | |____\_|  *
            *  | '____`'.  *
            *  | (____) |  *
            *  '.______.'  *
            *              *
            ****************"#},
            Self::_7 => indoc! {r#"
            ****************
            *    _______   *
            *   |  ___  |  *
            *   |_/  / /   *
            *       / /    *
            *      / /     *
            *     /_/      *
            *              *
            ****************"#},
            Self::_8 => indoc! {r#"
            ****************
            *     ____     *
            *   .' __ '.   *
            *   | (__) |   *
            *   .`____'.   *
            *  | (____) |  *
            *  `.______.'  *
            *              *
            ****************"#},
            Self::_9 => indoc! {r#"
            ****************
            *    ______    *
            *  .' ____ '.  *
            *  | (____) |  *
            *  '_.____. |  *
            *  | \____| |  *
            *   \______,'  *
            *              *
            ****************"#},
            Self::_10 => indoc! {r#"
            ****************
            * __      __   *
            */  |   .'  '. *
            *`| |  |  ..  |*
            * | |  | |  | |*
            *_| |_ |  `'  |*
            *_____| '.__.' *
            *              *
            ****************"#},
            Self::Jack => indoc! {r#"
            ****************
            *     _____    *
            *    |_   _|   *
            *      | |     *
            *   _  | |     *
            *  | |_' |     *
            *  `.___.'     *
            *              *
            ****************"#},
            Self::Knight => indoc! {r#"
            ****************
            *     ______   *
            *   .' ___  |  *
            *  / .'   \_|  *
            *  | |         *
            *  \ `.___.'\  *
            *   `._____.'  *
            *              *
            ****************"#},
            Self::Queen => indoc! {r#"
            ****************
            *    ___       *
            *  .'   '.     *
            * /  .-.  \    *
            * | |   | |    *
            * \  `-'  \_   *
            *  `.___.\__|  *
            *              *
            ****************"#},
            Self::King => indoc! {r#"
            ****************
            *  ___  ____   *
            * |_  ||_  _|  *
            *   | |_/ /    *
            *   |  __'.    *
            *  _| |  \ \_  *
            * |____||____| *
            *              *
            ****************"#},
        };
        base.normal()
    }
}
