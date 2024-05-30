//! Module for browser versions
//!
//! This file is copied from <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/preset_env_base/src/version.rs>

use std::{cmp, cmp::Ordering, fmt, str::FromStr};

use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize};

/// A version of a browser.
///
/// This is similar to semver, but this assumes a production build. (No tag like
/// `alpha`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Version {
    /// `a` in `a.b.c`
    pub major: u32,
    /// `b` in `a.b.c`
    pub minor: u32,
    /// `c` in `a.b.c`
    pub patch: u32,
}

#[allow(clippy::cast_possible_truncation)]
impl FromStr for Version {
    type Err = ();

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        let mut parts = v.split('.');
        // safari tp
        let major = parts.next().unwrap().parse::<u32>().map_err(|_| ())?;
        let minor = parts.next().unwrap_or("0").parse::<u32>().unwrap();
        let patch = parts.next().unwrap_or("0").parse::<u32>().unwrap();
        Ok(Version { major, minor, patch })
    }
}

impl cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            r => return r,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            r => return r,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            r => return r,
        }

        Ordering::Equal
    }
}

struct SerdeVisitor;

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl<'de> Visitor<'de> for SerdeVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a browser version")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Version { major: v as _, minor: 0, patch: 0 })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Version { major: v as _, minor: 0, patch: 0 })
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Version { major: v.floor() as _, minor: v.fract() as _, patch: 0 })
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(|()| de::Error::invalid_type(de::Unexpected::Str(v), &self))
    }

    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&v)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SerdeVisitor)
    }
}
