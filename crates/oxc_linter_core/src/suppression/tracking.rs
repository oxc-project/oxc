use cow_utils::CowUtils;
use std::{ffi::OsStr, fs, path::Path, sync::Arc};

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::read_to_string;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DiagnosticCounts {
    pub count: usize,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(default)]
pub struct Filename(String);

impl Filename {
    pub fn new(path: &Path) -> Self {
        Self(path.as_os_str().to_string_lossy().cow_replace('\\', "/").to_string())
    }
}

impl std::fmt::Display for Filename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type FileSuppressionsMap = FxHashMap<String, DiagnosticCounts>;
type AllSuppressionsMap = Arc<FxHashMap<Filename, FileSuppressionsMap>>;

fn serialize_arc_map<S>(map: &AllSuppressionsMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use std::collections::BTreeMap;
    let sorted: BTreeMap<&Filename, BTreeMap<&String, &DiagnosticCounts>> = map
        .iter()
        .map(|(filename, rules)| (filename, rules.iter().collect::<BTreeMap<_, _>>()))
        .collect();
    sorted.serialize(serializer)
}

fn deserialize_arc_map<'de, D>(deserializer: D) -> Result<AllSuppressionsMap, D::Error>
where
    D: serde::Deserializer<'de>,
{
    FxHashMap::<Filename, FileSuppressionsMap>::deserialize(deserializer).map(Arc::new)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SuppressionTracking {
    #[serde(deserialize_with = "deserialize_arc_map", serialize_with = "serialize_arc_map")]
    suppressions: AllSuppressionsMap,
}

impl Default for SuppressionTracking {
    fn default() -> Self {
        Self { suppressions: Arc::new(FxHashMap::default()) }
    }
}

impl SuppressionTracking {
    pub fn from_file(path: &Path, cwd: &Path) -> Result<Self, OxcDiagnostic> {
        let path_to_error = if let Ok(path_error) = path.strip_prefix(cwd) {
            path_error.display()
        } else {
            path.display()
        };

        let string = read_to_string(path).map_err(|e| {
            OxcDiagnostic::error(format!(
                "Failed to parse suppression rules file {path_to_error} with error {e:?}"
            ))
        })?;

        let json = serde_json::from_str::<serde_json::Value>(&string).map_err(|error| {
            let ext = path.extension().and_then(OsStr::to_str);
            let err = match ext {
                // syntax error
                Some("json") => error.to_string(),
                Some(_) => "Only JSON suppression rules files are supported".to_string(),
                None => {
                    format!(
                        "{error}, if the configuration is not a JSON file, please use JSON instead."
                    )
                }
            };
            OxcDiagnostic::error(format!("Failed to parse oxlint config {path_to_error}.\n{err}"))
        })?;

        let config = Self::deserialize(&json).map_err(|err| {
            OxcDiagnostic::error(format!(
                "Failed to parse rules suppression file with error {err:?}"
            ))
        })?;

        Ok(config)
    }

    pub fn from_map(map: FxHashMap<Filename, FileSuppressionsMap>) -> Self {
        Self { suppressions: Arc::new(map) }
    }

    pub fn suppressions(&self) -> &AllSuppressionsMap {
        &self.suppressions
    }

    pub fn save(&self, path: &Path) -> Result<(), OxcDiagnostic> {
        let content = serde_json::to_string_pretty(&self).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to serialize suppression file: {err}"))
        })?;

        fs::write(path, content).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to write suppression file: {err}"))
        })?;

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SuppressionFileState {
    Ignored,
    New,
    Exists,
}

#[derive(Debug)]
pub struct SuppressionFile<'a> {
    state: SuppressionFileState,
    suppression_data: Option<&'a FileSuppressionsMap>,
}

impl Default for SuppressionFile<'_> {
    fn default() -> Self {
        Self { state: SuppressionFileState::Ignored, suppression_data: None }
    }
}

impl<'a> SuppressionFile<'a> {
    pub fn new<'b>(
        file_exists: bool,
        suppress_all: bool,
        suppression_data: Option<&'b FileSuppressionsMap>,
    ) -> Self
    where
        'b: 'a,
    {
        if !file_exists && !suppress_all {
            return Self { state: SuppressionFileState::Ignored, suppression_data: None };
        }

        if suppress_all {
            // All errors will be suppressed and written to the file.
            return Self { state: SuppressionFileState::New, suppression_data: None };
        }

        Self { state: SuppressionFileState::Exists, suppression_data }
    }

    pub fn suppression_state(&self) -> &SuppressionFileState {
        &self.state
    }

    pub fn suppression_data(&self) -> Option<&'a FileSuppressionsMap> {
        self.suppression_data
    }
}
