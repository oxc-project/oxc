/// Tri state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    True,
    False,
    Unknown,
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
        Self::for_int(-self.value() * other.value())
    }

    pub fn for_int(int: i8) -> Self {
        match int {
            -1 => Self::False,
            1 => Self::True,
            _ => Self::Unknown,
        }
    }

    pub fn for_boolean(boolean: bool) -> Self {
        if boolean {
            Self::True
        } else {
            Self::False
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
