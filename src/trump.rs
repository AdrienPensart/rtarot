use crate::points::HasPoints;
use crate::traits::{Discardable, Representation};
use colored::{ColoredString, Colorize};
use indoc::indoc;
use ordered_float::OrderedFloat;
use std::fmt;
use strum::EnumIter;

#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, EnumIter)]
pub enum Trump {
    Fool = 0,
    Petit = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
    _16 = 16,
    _17 = 17,
    _18 = 18,
    _19 = 19,
    _20 = 20,
    _21 = 21,
}

impl fmt::Display for Trump {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

impl Trump {
    #[must_use]
    pub const fn is_oudler(&self) -> bool {
        matches!(self, Self::Fool | Self::Petit | Self::_21)
    }
}

impl Discardable for Trump {
    fn discardable(&self) -> bool {
        // RULE: cant discard trumps
        false
    }
    fn discardable_forced(&self) -> bool {
        // RULE: if we have 4 kings and x trumps, we must discard some trumps, except oudlers
        !self.is_oudler()
    }
}

impl HasPoints for Trump {
    fn points(&self) -> OrderedFloat<f64> {
        let points = if self.is_oudler() { 4.5 } else { 0.5 };
        OrderedFloat(points)
    }
}

impl Representation for Trump {
    fn color(&self) -> &'static str {
        "cyan"
    }
    fn symbol(&self) -> &'static str {
        match self {
            Self::Fool => "🃏",
            Self::Petit => "1",
            Self::_2 => "2",
            Self::_3 => "3",
            Self::_4 => "4",
            Self::_5 => "5",
            Self::_6 => "6",
            Self::_7 => "7",
            Self::_8 => "8",
            Self::_9 => "9",
            Self::_10 => "10",
            Self::_11 => "11",
            Self::_12 => "12",
            Self::_13 => "13",
            Self::_14 => "14",
            Self::_15 => "15",
            Self::_16 => "16",
            Self::_17 => "17",
            Self::_18 => "18",
            Self::_19 => "19",
            Self::_20 => "20",
            Self::_21 => "21",
        }
    }
    fn colored_symbol(&self) -> ColoredString {
        match self {
            Self::Fool => self.symbol().normal(),
            _ => "#".color(self.color()),
        }
    }
    fn repr(&self) -> ColoredString {
        match self {
            Self::Fool => self.symbol().normal(),
            _ => format!("#{}", self.symbol()).color(self.color()),
        }
    }
    fn full_repr(&self) -> ColoredString {
        let trump = match self {
            Self::Fool => indoc! {r#"
            🃏🃏🃏🃏🃏🃏🃏🃏
            🃏  _________ 🃏
            🃏 |_   ___  |🃏
            🃏   | |_  \_|🃏
            🃏   |  _|    🃏
            🃏  _| |_     🃏
            🃏 |_____|    🃏
            🃏            🃏
            🃏🃏🃏🃏🃏🃏🃏🃏"#},
            Self::Petit => indoc! {r#"
            ################
            #     __       #
            #    /  |      #
            #    `| |      #
            #     | |      #
            #    _| |_     #
            #   |_____|    #
            #              #
            ################"#},
            Self::_2 => indoc! {r#"
            ################
            #    _____     #
            #   / ___ `.   #
            #  |_/___) |   #
            #   .'____.'   #
            #  / /____     #
            #  |_______|   #
            #              #
            ################"#},
            Self::_3 => indoc! {r#"
            ################
            #    ______    #
            #   / ____ `.  #
            #   `'  __) |  #
            #   _  |__ '.  #
            #  | \____) |  #
            #   \______.'  #
            #              #
            ################"#},
            Self::_4 => indoc! {r#"
            ################
            #   _    _     #
            #  | |  | |    #
            #  | |__| |_   #
            #  |____   _|  #
            #      _| |_   #
            #     |_____|  #
            #              #
            ################"#},
            Self::_5 => indoc! {r#"
            ################
            #   _______    #
            #  |  _____|   #
            #  | |____     #
            #  '_.____''.  #
            #  | \____) |  #
            #   \______.'  #
            #              #
            ################"#},
            Self::_6 => indoc! {r#"
            ################
            #    ______    #
            #  .' ____ \   #
            #  | |____\_|  #
            #  | '____`'.  #
            #  | (____) |  #
            #  '.______.'  #
            #              #
            ################"#},
            Self::_7 => indoc! {r#"
            ################
            #    _______   #
            #   |  ___  |  #
            #   |_/  / /   #
            #       / /    #
            #      / /     #
            #     /_/      #
            #              #
            ################"#},
            Self::_8 => indoc! {r#"
            ################
            #     ____     #
            #   .' __ '.   #
            #   | (__) |   #
            #   .`____'.   #
            #  | (____) |  #
            #  `.______.'  #
            #              #
            ################"#},
            Self::_9 => indoc! {r#"
            ################
            #    ______    #
            #  .' ____ '.  #
            #  | (____) |  #
            #  '_.____. |  #
            #  | \____| |  #
            #   \______,'  #
            #              #
            ################"#},
            Self::_10 => indoc! {r#"
            ################
            # __      __   #
            #/  |   .'  '. #
            #`| |  |  ..  |#
            # | |  | |  | |#
            #_| |_ |  `'  |#
            #_____| '.__.' #
            #              #
            ################"#},
            Self::_11 => indoc! {r#"
            ################
            #  __     __   #
            # /  |   /  |  #
            # `| |   `| |  #
            #  | |    | |  #
            # _| |_  _| |_ #
            #|_____||_____|#
            #              #
            ################"#},
            Self::_12 => indoc! {r#"
            ################
            #  __     ___  #
            # /  |   / _ `.#
            # `| |  |_/_) |#
            #  | |   .'__.'#
            # _| |_ / /__  #
            #|_____||_____|#
            #              #
            ################"#},
            Self::_13 => indoc! {r#"
            ################
            #  __    ____  #
            # /  |  / __ `.#
            # `| |  `' _) |#
            #  | |  _ |_ '.#
            # _| |_| \__) |#
            #|_____ \____.'#
            #              #
            ################"#},
            Self::_14 => indoc! {r#"
            ################
            #  __   _   _  #
            # /  | | | | | #
            # `| | | |_| |_#
            #  | | |___   _#
            # _| |_  _| |_ #
            #|_____| |____|#
            #              #
            ################"#},
            Self::_15 => indoc! {r#"
            ################
            #  __   _____  #
            # /  | |  ___| #
            # `| | | |__   #
            #  | | '_.__''.#
            # _| |_| \__) |#
            #|_____ \____.'#
            #              #
            ################"#},
            Self::_16 => indoc! {r#"
            ################
            #  __    ____  #
            # /  | .' __ \ #
            # `| | | |__\_|#
            #  | | | '__`'.#
            # _| |_| (__) |#
            #|_____'.____.'#
            #              #
            ################"#},
            Self::_17 => indoc! {r#"
            ################
            #  __   _______#
            # /  | |  ___  #
            # `| | |_/  / /#
            #  | |     / / #
            # _| |_   / /  #
            #|_____| /_/   #
            #              #
            ################"#},
            Self::_18 => indoc! {r#"
            ################
            #  __     ___  #
            # /  |  .' _ '.#
            # `| |  | (_) |#
            #  | |  .`___'.#
            # _| |_| (___) #
            #|_____`._____.#
            #              #
            ################"#},
            Self::_19 => indoc! {r#"
            ################
            #  __    ____  #
            # /  | .' __ '.#
            # `| | | (__) |#
            #  | | '_.__. |#
            # _| |_| \__| |#
            #|_____ \____,'#
            #              #
            ################"#},
            Self::_20 => indoc! {r#"
            ################
            #   __     _   #
            #  /  `. .' '. #
            # |_/) ||  .  |#
            #  .'_.'| | | |#
            # / /_  |  `  |#
            # |____| '._.' #
            #              #
            ################"#},
            Self::_21 => indoc! {r#"
            ################
            #   ___   __   #
            #  / _ `./  |  #
            # |_/_) |`| |  #
            #  .'__.' | |  #
            # / /__  _| |_ #
            # |_____|_____|#
            #              #
            ################"#},
        };
        trump.color(self.color())
    }
}
