use std::{fmt, str::FromStr};

use crate::project_root;

pub struct TestFiles {
    files: Vec<TestFile>,
}

impl Default for TestFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFiles {
    /// # Panics
    /// Fails to read file
    #[must_use]
    pub fn new() -> Self {
        let root = project_root();
        let files = std::fs::read_to_string(root.join("./tasks/libs.txt"))
            .unwrap()
            .lines()
            .map(|file| TestFile::new(file).unwrap())
            .collect::<Vec<_>>();
        Self { files }
    }

    #[must_use]
    pub fn files(&self) -> &Vec<TestFile> {
        &self.files
    }
}

pub struct TestFile {
    pub url: String,
    pub file_name: String,
    pub source_text: String,
}

impl TestFile {
    /// # Errors
    pub fn new(url: &str) -> Result<Self, String> {
        let (file_name, source_text) = Self::get_source_text(url)?;
        Ok(Self { url: url.to_string(), file_name, source_text })
    }

    /// # Errors
    /// # Panics
    pub fn get_source_text(lib: &str) -> Result<(String, String), String> {
        let url = url::Url::from_str(lib).map_err(err_to_string)?;

        let segments = url.path_segments().ok_or_else(|| "lib url has no segments".to_string())?;

        let filename = segments.last().ok_or_else(|| "lib url has no segments".to_string())?;

        let file = project_root().join("target").join(filename);

        if let Ok(code) = std::fs::read_to_string(&file) {
            println!("[{filename}] - using [{}]", file.display());
            Ok((filename.to_string(), code))
        } else {
            println!("[{filename}] - Downloading [{lib}] to [{}]", file.display());
            match ureq::get(lib).call() {
                Ok(response) => {
                    let mut reader = response.into_reader();

                    let _drop = std::fs::remove_file(&file);
                    let mut writer = std::fs::File::create(&file).map_err(err_to_string)?;
                    let _drop = std::io::copy(&mut reader, &mut writer);

                    std::fs::read_to_string(&file)
                        .map_err(err_to_string)
                        .map(|code| (filename.to_string(), code))
                }
                Err(e) => Err(format!("{e:?}")),
            }
        }
    }
}

fn err_to_string<E: fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}
