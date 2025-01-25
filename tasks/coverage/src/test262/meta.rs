use saphyr::Yaml;

#[derive(Debug, Clone, Default)]
pub struct MetaData {
    // pub description: Box<str>,
    pub esid: Option<Box<str>>,
    // pub es5id: Option<Box<str>>,
    // pub es6id: Option<Box<str>>,
    // pub info: Box<str>,
    pub features: Box<[Box<str>]>,
    pub includes: Box<[Box<str>]>,
    pub flags: Box<[TestFlag]>,
    pub negative: Option<Negative>,
    // pub locale: Box<[Box<str>]>,
}

/// Individual test flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestFlag {
    OnlyStrict,
    NoStrict,
    Module,
    Raw,
    Async,
    Generated,
    CanBlockIsFalse,
    CanBlockIsTrue,
    NonDeterministic,
}

impl TestFlag {
    fn from_str(s: &str) -> Self {
        match s {
            "onlyStrict" => Self::OnlyStrict,
            "noStrict" => Self::NoStrict,
            "module" => Self::Module,
            "raw" => Self::Raw,
            "async" => Self::Async,
            "generated" => Self::Generated,
            "CanBlockIsFalse" => Self::CanBlockIsFalse,
            "CanBlockIsTrue" => Self::CanBlockIsTrue,
            "non-deterministic" => Self::NonDeterministic,
            _ => panic!("{s} not supported for TestFlag"),
        }
    }
}

/// Negative test information structure.
#[derive(Debug, Clone)]
pub struct Negative {
    pub phase: Phase,
    pub error_type: Box<str>,
}

impl Negative {
    fn from_yaml(yaml: &Yaml) -> Self {
        Self {
            phase: Phase::from_str(yaml["phase"].as_str().unwrap()),
            error_type: yaml["type"].as_str().unwrap().into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Parse,
    Early,
    Resolution,
    Runtime,
}

impl Phase {
    pub fn is_runtime(self) -> bool {
        matches!(self, Self::Runtime)
    }

    fn from_str(s: &str) -> Self {
        match s {
            "parse" => Self::Parse,
            "early" => Self::Early,
            "resolution" => Self::Resolution,
            "runtime" => Self::Runtime,
            _ => panic!("{s} not support for Phase"),
        }
    }
}

impl MetaData {
    pub fn from_str(s: &str) -> Self {
        let yamls = Yaml::load_from_str(s).unwrap_or_default();
        let Some(yaml) = yamls.first() else { return Self::default() };
        Self {
            // description: yaml["description"].as_str().unwrap_or_default().into(),
            esid: yaml["esid"].as_str().map(Into::into),
            // es5id: yaml["es5id"].as_str().map(Into::into),
            // es6id: yaml["es6id"].as_str().map(Into::into),
            // info: yaml["info"].as_str().unwrap_or_default().into(),
            features: Self::get_vec_of_string(&yaml["features"]),
            includes: Self::get_vec_of_string(&yaml["includes"]),
            flags: yaml["flags"]
                .as_vec()
                .map_or_else(Vec::new, |a| {
                    a.iter()
                        .map(|v| v.as_str().map(TestFlag::from_str).unwrap())
                        .collect::<Vec<_>>()
                })
                .into(),
            negative: {
                let yaml = &yaml["negative"];
                (!yaml.is_null() && !yaml.is_badvalue()).then(|| Negative::from_yaml(yaml))
            },
            // locale: Self::get_vec_of_string(&yaml["locale"]),
        }
    }

    fn get_vec_of_string(yaml: &Yaml) -> Box<[Box<str>]> {
        yaml.as_vec()
            .map_or_else(Vec::new, |a| {
                a.iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string().into_boxed_str())
                    .collect::<Vec<_>>()
            })
            .into()
    }
}
