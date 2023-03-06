use std::path::Path;

use git2::Repository;
use miette::Diagnostic;
use oxc_diagnostics::{thiserror::Error, Error};

#[derive(Debug, Error, Diagnostic)]
#[error("No repository found")]
#[diagnostic(severity(warning), help("Ensure target path(s) belong to a Git repository"))]
struct NoRepositoryFound;

#[derive(Debug, Error, Diagnostic)]
#[error("Multiple repositories found")]
#[diagnostic(severity(warning), help("Ensure all paths belong to a single repository"))]
struct MultipleRepositoriesFound;

#[derive(Debug, Error, Diagnostic)]
#[error("Uncommitted changes")]
#[diagnostic(severity(warning), help("Commit any changes before linting"))]
struct UncommittedChanges;

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
            return Err(NoRepositoryFound.into());
        }
        match self.is_same_repo() {
            Ok(repo) => {
                for path in self.paths {
                    if Self::is_uncommitted(repo, path) {
                        return Err(UncommittedChanges.into());
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
                return Err(MultipleRepositoriesFound.into());
            }
        }
        Ok(first_repo)
    }

    fn is_uncommitted(repo: &Repository, path: &Path) -> bool {
        repo.status_file(path).map_or(true, |status| !matches!(status, git2::Status::CURRENT))
    }
}
