use std::{str::FromStr, sync::OnceLock};

use browserslist::Version;
use cow_utils::CowUtils;
use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Chrome,
    Deno,
    Edge,
    Firefox,
    Hermes,
    Ie,
    Ios,
    Node,
    Opera,
    Rhino,
    Safari,
    Samsung,
    // TODO: electron to chromium
    Electron,
    // TODO: how to handle? There is a `op_mob` key below.
    OperaMobile,
    // TODO:
    Android,
    // Special Value for ESXXXX target.
    Es,
}

impl Engine {
    /// Parse format `chrome42`.
    ///
    /// # Errors
    ///
    /// * No matching target
    /// * Invalid version
    pub fn parse_name_and_version(s: &str) -> Result<(Engine, Version), String> {
        let s = s.cow_to_ascii_lowercase();
        for (name, engine) in engines() {
            if let Some(v) = s.strip_prefix(name) {
                return Version::from_str(v).map(|version| (*engine,version))
                    .map_err(|_|
                        String::from(r#"All version numbers must be in the format "X", "X.Y", or "X.Y.Z" where X, Y, and Z are non-negative integers."#),
                    );
            }
        }
        Err(format!("Invalid target '{s}'."))
    }
}

impl FromStr for Engine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "chrome" | "and_chr" => Ok(Self::Chrome),
            "deno" => Ok(Self::Deno),
            "edge" => Ok(Self::Edge),
            "firefox" | "and_ff" => Ok(Self::Firefox),
            "hermes" => Ok(Self::Hermes),
            "ie" | "ie_mob" => Ok(Self::Ie),
            "ios" | "ios_saf" => Ok(Self::Ios),
            "node" => Ok(Self::Node),
            "opera" | "op_mob" => Ok(Self::Opera),
            "rhino" => Ok(Self::Rhino),
            "safari" => Ok(Self::Safari),
            "samsung" => Ok(Self::Samsung),
            "electron" => Ok(Self::Electron),
            "opera_mobile" => Ok(Self::OperaMobile),
            "android" => Ok(Self::Android),
            _ => Err(()),
        }
    }
}

fn engines() -> &'static FxHashMap<&'static str, Engine> {
    static ENGINES: OnceLock<FxHashMap<&'static str, Engine>> = OnceLock::new();
    ENGINES.get_or_init(|| {
        FxHashMap::from_iter([
            ("chrome", Engine::Chrome),
            ("deno", Engine::Deno),
            ("edge", Engine::Edge),
            ("firefox", Engine::Firefox),
            ("hermes", Engine::Hermes),
            ("ie", Engine::Ie),
            ("ios", Engine::Ios),
            ("node", Engine::Node),
            ("opera", Engine::Opera),
            ("rhino", Engine::Rhino),
            ("safari", Engine::Safari),
            ("samsung", Engine::Samsung),
            ("electron", Engine::Electron),
            ("opera_mobile", Engine::OperaMobile),
            ("android", Engine::Android),
        ])
    })
}
