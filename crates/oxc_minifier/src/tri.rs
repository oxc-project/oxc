/// Tri state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    True,
    False,
    Unknown,
}

impl From<bool> for Tri {
    fn from(value: bool) -> Self {
        if value {
            Tri::True
        } else {
            Tri::False
        }
    }
}

impl From<Option<bool>> for Tri {
    fn from(value: Option<bool>) -> Self {
        value.map_or(Tri::Unknown, From::from)
    }
}

impl From<i8> for Tri {
    fn from(value: i8) -> Self {
        match value {
            -1 => Self::False,
            1 => Self::True,
            _ => Self::Unknown,
        }
    }
}

impl Tri {
    pub fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }

    pub fn xor(self, other: Self) -> Self {
        Self::from(-self.value() * other.value())
    }

    pub fn value(self) -> i8 {
        match self {
            Self::True => 1,
            Self::False => -1,
            Self::Unknown => 0,
        }
    }
}
