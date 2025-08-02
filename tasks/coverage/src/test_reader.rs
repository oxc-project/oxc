use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{AppArgs, suite::Case, workspace_root};

/// Handles reading and filtering test cases from the filesystem
pub struct TestReader {
    test_root: PathBuf,
}

impl TestReader {
    pub fn new(test_root: PathBuf) -> Self {
        Self { test_root }
    }

    /// Read test cases from filesystem with filtering and submodule initialization
    pub fn read_test_cases<T: Case>(
        &self,
        name: &str,
        args: &AppArgs,
        skip_test_path: impl Fn(&Path) -> bool + Sync,
    ) -> Vec<T> {
        let test_path = workspace_root();
        let cases_path = test_path.join(&self.test_root);

        let get_paths = || self.collect_paths(&cases_path, args, &skip_test_path);

        let mut paths = get_paths();

        // Initialize git submodule if it is empty and no filter is provided
        if paths.is_empty() && args.filter.is_none() {
            self.initialize_submodules(name);
            paths = get_paths();
        }

        self.read_files_parallel(paths, &test_path)
    }

    fn collect_paths<F>(
        &self,
        cases_path: &Path,
        args: &AppArgs,
        skip_test_path: &F,
    ) -> Vec<PathBuf>
    where
        F: Fn(&Path) -> bool,
    {
        let filter = args.filter.as_ref();
        WalkDir::new(cases_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .filter(|e| e.file_name() != ".DS_Store")
            .map(|e| e.path().to_owned())
            .filter(|path| !skip_test_path(path))
            .filter(|path| filter.is_none_or(|query| path.to_string_lossy().contains(query)))
            .collect::<Vec<_>>()
    }

    fn initialize_submodules(&self, name: &str) {
        println!("-------------------------------------------------------");
        println!("git submodule is empty for {name}");
        println!("Running `just submodules` to clone the submodules");
        println!("This may take a while.");
        println!("-------------------------------------------------------");
        Command::new("just")
            .args(["submodules"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute `just submodules`");
    }

    fn read_files_parallel<T: Case>(&self, paths: Vec<PathBuf>, test_path: &Path) -> Vec<T> {
        paths
            .into_par_iter()
            .map(|path| {
                let mut code = self.read_file_content(&path);
                let path = path.strip_prefix(test_path).unwrap().to_owned();

                // Remove the Byte Order Mark in some of the TypeScript files
                if code.starts_with('\u{feff}') {
                    code.remove(0);
                }
                T::new(path, code)
            })
            .filter(|case| !case.skip_test_case())
            .collect::<Vec<_>>()
    }

    fn read_file_content(&self, path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|_| {
            // TypeScript tests may contain utf_16 encoding files
            let file = fs::File::open(path).unwrap();
            let mut content = String::new();
            DecodeReaderBytesBuilder::new()
                .encoding(Some(UTF_16LE))
                .build(file)
                .read_to_string(&mut content)
                .unwrap();
            content
        })
    }
}
