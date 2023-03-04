use std::path::Path;

use git2::Repository;

pub enum GitResult {
    NoRepo,
    MultipleRepos,
    UncommittedChanges,
}

pub struct Git<'a> {
    paths: &'a Vec<Box<Path>>,
    repos: Vec<Repository>,
}

impl<'a> Git<'a> {
    pub fn new(paths: &'a Vec<Box<Path>>) -> Self {
        let repos: Vec<Repository> =
            paths.iter().filter_map(|path| Repository::discover(path).ok()).collect();
        Self { paths, repos }
    }

    pub fn verify(&'a self) -> Result<&'a Repository, GitResult> {
        if self.repos.is_empty() {
            return Err(GitResult::NoRepo);
        }
        match self.is_same_repo() {
            Ok(repo) => {
                for path in self.paths {
                    if Self::is_uncommitted(repo, path) {
                        return Err(GitResult::UncommittedChanges);
                    }
                }
                Ok(repo)
            }
            err => err,
        }
    }

    /// Given a list of repositories, verify they're all the same repository.
    fn is_same_repo(&self) -> Result<&Repository, GitResult> {
        assert!(!self.repos.is_empty());
        let first_repo = self.repos.first().unwrap();
        for repo in &self.repos[1..] {
            if repo.path() != first_repo.path() {
                return Err(GitResult::MultipleRepos);
            }
        }
        Ok(first_repo)
    }

    fn is_uncommitted(repo: &Repository, path: &Path) -> bool {
        repo.status_file(path).map_or(true, |status| !matches!(status, git2::Status::CURRENT))
    }
}
