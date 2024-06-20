#[allow(unused)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
pub enum EcmaVersion {
    #[default]
    V5,
    V2015,
    V2016,
    V2017,
    V2018,
    V2019,
    V2020,
    V2021,
    V2022,
    V2023,
    V2024,
}
#[allow(unused)]
impl EcmaVersion {
    pub fn latest_ecma_version() -> Self {
        Self::V2024
    }
}
