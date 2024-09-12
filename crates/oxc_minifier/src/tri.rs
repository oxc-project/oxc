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
    pub fn is_true(self) -> bool {
        self == Tri::True
    }

    pub fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(Tri) -> U,
    {
        match self {
            Self::True => Some(f(Tri::True)),
            Self::False => Some(f(Tri::False)),
            Self::Unknown => None,
        }
    }

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

    pub fn to_option(self) -> Option<bool> {
        match self {
            Self::True => Some(true),
            Self::False => Some(false),
            Self::Unknown => None,
        }
    }

    pub fn value(self) -> i8 {
        match self {
            Self::True => 1,
            Self::False => -1,
            Self::Unknown => 0,
        }
    }
}
