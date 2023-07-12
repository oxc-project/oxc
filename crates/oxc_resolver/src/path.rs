use std::path::{Path, PathBuf};

use normalize_path::NormalizePath;

use crate::request::Request;

pub struct ResolvePath<'a>(&'a Path);

impl<'a> From<&'a Path> for ResolvePath<'a> {
    fn from(path: &'a Path) -> Self {
        Self(path)
    }
}

impl<'a> ResolvePath<'a> {
    pub fn join(&self, request: &Request) -> PathBuf {
        self.0.join(request.as_str()).normalize()
    }
}
