use browserslist::Version;

pub use oxc_syntax::es_target::ESTarget;

pub trait ESVersion {
    fn version(&self) -> Version;
}

impl ESVersion for ESTarget {
    fn version(&self) -> Version {
        match self {
            Self::ES5 => Version(5, 0, 0),
            Self::ES2015 => Version(2015, 0, 0),
            Self::ES2016 => Version(2016, 0, 0),
            Self::ES2017 => Version(2017, 0, 0),
            Self::ES2018 => Version(2018, 0, 0),
            Self::ES2019 => Version(2019, 0, 0),
            Self::ES2020 => Version(2020, 0, 0),
            Self::ES2021 => Version(2021, 0, 0),
            Self::ES2022 => Version(2022, 0, 0),
            Self::ES2023 => Version(2023, 0, 0),
            Self::ES2024 => Version(2024, 0, 0),
            Self::ES2025 => Version(2025, 0, 0),
            Self::ESNext => Version(9999, 0, 0),
        }
    }
}
