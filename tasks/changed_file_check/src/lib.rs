#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{self, BufRead},
        path::{Path, PathBuf},
    };

    use git2::Repository;

    fn repo_path() -> PathBuf {
        // The test file is executed {root}/tasks/changed_file_check
        let current_path = std::env::current_dir().unwrap();

        // So the repo path is {root}/tasks/changed_file_check/../..
        current_path.parent().unwrap().parent().unwrap().to_path_buf()
    }

    fn git_diff_new_files<P: AsRef<Path>>(repo_path: P) -> Vec<PathBuf> {
        let repo = match Repository::init(repo_path.as_ref()) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to init: {}", e),
        };

        let main_branch_tree = repo
            .find_branch("main", git2::BranchType::Local)
            .unwrap()
            .get()
            .peel_to_tree()
            .unwrap();

        let diff = repo.diff_tree_to_workdir(Some(&main_branch_tree), None).unwrap();

        let mut result = vec![];

        for delta in diff.deltas() {
            let tmp = delta.new_file().path().unwrap();
            result.push(repo_path.as_ref().join(tmp));
        }

        result
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
    use regex::Regex;

    #[test]
    fn no_master_or_main_branch_for_git_url() {
        let github_main_like_url_pattern =
            Regex::new(r"https://github.com/.*?/.*?/(tree|blob)/(master|main)").unwrap();

        let files = git_diff_new_files(repo_path());

        // Read file content and check whether it contains tabs
        for ref file in files {
            let lines = read_lines(file).unwrap();

            lines.enumerate().for_each(|(line_number, line)| {
                assert!(
                    !github_main_like_url_pattern.is_match(&line.unwrap()),
                    "A github url associated with main/master branch is found in file: '{}:{}', please use a tag or branch to ensure the link is stable",
                    file.display(),
                    line_number
                );
            })
        }
    }

    #[test]
    fn no_tab_used_in_repo() {
        let files = git_diff_new_files(repo_path());
        // Read file content and check whether it contains tabs
        for ref file in files {
            let lines = read_lines(file).unwrap();

            lines.enumerate().for_each(|(line_number, line)| {
                assert!(
                    !line.unwrap().contains('\t'),
                    "there is a tab found in file: '{}: {}'",
                    file.display(),
                    line_number
                );
            })
        }
    }
}
