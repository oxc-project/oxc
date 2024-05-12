use std::path::Path;

use git2::Repository;

use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};

pub struct Git<'a> {
    // paths: &'a Vec<Box<Path>>,
    repos: Vec<Repository>,
}

impl<'a> Git<'a> {
    pub fn new() -> Self {
        // let repos: Vec<Repository> =
        // paths.iter().filter_map(|path| Repository::discover(path).ok()).collect();
        Self { repos }
    }

    pub fn verify(&'a self) -> Result<&'a Repository, Error> {
        if self.repos.is_empty() {
            return Err(OxcDiagnostic::warning("No repository found"))
                .with_help("Ensure target path(s) belong to a Git repository");
        }
        match self.is_same_repo() {
            Ok(repo) => {
                for path in self.paths {
                    if Self::is_uncommitted(repo, path) {
                        return Err(OxcDiagnostic::warning("Uncommitted changes")
                            .with_help("Commit any changes before linting"));
                    }
                }
                Ok(repo)
            }
            err => err,
        }
    }

    /// Given a list of repositories, verify they're all the same repository.
    fn is_same_repo(&self) -> Result<&Repository, Error> {
        assert!(!self.repos.is_empty());
        let first_repo = self.repos.first().unwrap();
        for repo in &self.repos[1..] {
            if repo.path() != first_repo.path() {
                return Err(OxcDiagnostic::warning("Multiple repositories found")
                    .with_help("Ensure all paths belong to a single repository"));
            }
        }
        Ok(first_repo)
    }

    fn is_uncommitted(repo: &Repository, path: &Path) -> bool {
        repo.status_file(path).map_or(true, |status| !matches!(status, git2::Status::CURRENT))
    }
}
