use crate::traits::Points;

#[derive(Default, Display, Eq, PartialEq, Debug, Clone, EnumIter)]
pub enum Handle {
    #[default]
    Refused,
    Simple,
    Double,
    Triple,
}

impl Points for Handle {
    fn points(&self) -> f64 {
        match self {
            Self::Refused => 0.0,
            Self::Simple  => 20.0,
            Self::Double  => 30.0,
            Self::Triple  => 40.0,
        }
    }
}
