use std::{
    borrow::Cow,
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Snapshot {
    git_repo_path: PathBuf,
    sha: Option<String>,
}

impl Snapshot {
    /// # Panics
    ///
    /// * Git operation fails
    pub fn new(git_repo_path: &Path, show_commit: bool) -> Self {
        let sha = show_commit.then(|| {
            let path = git_repo_path.to_str().unwrap();
            let output = Command::new("git")
                .args(["-C", path, "rev-parse", "--short=8", "HEAD"])
                .output()
                .unwrap()
                .stdout;
            String::from_utf8(output).unwrap().trim().to_string()
        });
        Self { git_repo_path: git_repo_path.to_path_buf(), sha }
    }

    /// # Panics
    ///
    /// * File operation fails
    pub fn save(&self, path: &Path, content: &str) {
        let content = if let Some(new_sha) = &self.sha {
            if path.exists() {
                let file = fs::read_to_string(path).unwrap();
                let line =
                    file.lines().next().unwrap_or_else(|| panic!("{path:?} content is empty."));
                if let Some(old_sha) = line.strip_prefix("commit: ") {
                    let outdated = new_sha != old_sha && env::var("UPDATE_SNAPSHOT").is_err();
                    assert!(
                        !outdated,
                        "\nRepository {:?} is outdated for {path:?}.\nsha from file = {old_sha}, sha from repo = {new_sha}\nPlease run `just submodules` to update it.\n",
                        self.git_repo_path
                    );
                }
            }
            Cow::Owned(format!("commit: {new_sha}\n\n{content}"))
        } else {
            Cow::Borrowed(content)
        };

        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}
