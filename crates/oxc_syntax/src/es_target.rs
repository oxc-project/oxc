//! ECMAScript Target
use std::{fmt, str::FromStr};

use cow_utils::CowUtils;

/// ECMAScript Target
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[allow(missing_docs)]
pub enum ESTarget {
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ES2024,
    ES2025,
    #[default]
    ESNext,
}

impl FromStr for ESTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.cow_to_ascii_lowercase().as_ref() {
            "es5" => Ok(Self::ES5),
            "es6" | "es2015" => Ok(Self::ES2015),
            "es2016" => Ok(Self::ES2016),
            "es2017" => Ok(Self::ES2017),
            "es2018" => Ok(Self::ES2018),
            "es2019" => Ok(Self::ES2019),
            "es2020" => Ok(Self::ES2020),
            "es2021" => Ok(Self::ES2021),
            "es2022" => Ok(Self::ES2022),
            "es2023" => Ok(Self::ES2023),
            "es2024" => Ok(Self::ES2024),
            "es2025" => Ok(Self::ES2025),
            "esnext" => Ok(Self::ESNext),
            _ => Err(format!("Invalid target \"{s}\".")),
        }
    }
}

impl fmt::Display for ESTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ES5 => "es5",
            Self::ES2015 => "es2015",
            Self::ES2016 => "es2016",
            Self::ES2017 => "es2017",
            Self::ES2018 => "es2018",
            Self::ES2019 => "es2019",
            Self::ES2020 => "es2020",
            Self::ES2021 => "es2021",
            Self::ES2022 => "es2022",
            Self::ES2023 => "es2023",
            Self::ES2024 => "es2024",
            Self::ES2025 => "es2025",
            Self::ESNext => "esnext",
        };
        write!(f, "{s}",)
    }
}
