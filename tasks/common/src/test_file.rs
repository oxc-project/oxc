use std::fmt;

use crate::{project_root, request::agent};

pub struct TestFiles {
    files: Vec<TestFile>,
}

impl TestFiles {
    pub fn files(&self) -> &Vec<TestFile> {
        &self.files
    }

    pub fn react() -> Self {
        Self {
            files: vec![TestFile::new(
                "https://cdn.jsdelivr.net/npm/react@17.0.2/cjs/react.development.js",
            )],
        }
    }

    /// These are kept in sync with <https://github.com/privatenumber/minification-benchmarks/tree/d8d54ceeb206d318fa288b152904adf715b076b2>
    /// for checking against minification size in `tasks/minsize/minsize.snap`.
    pub fn minifier() -> Self {
        Self {
            files: vec![
                TestFile::new("https://cdn.jsdelivr.net/npm/react@17.0.2/cjs/react.development.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/moment@2.29.1/moment.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/jquery@3.5.1/dist/jquery.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/vue@2.6.12/dist/vue.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/d3@6.3.1/dist/d3.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/terser@5.30.3/dist/bundle.min.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/three@0.124.0/build/three.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/victory@35.8.4/dist/victory.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/echarts@5.1.1/dist/echarts.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/typescript@4.9.5/lib/typescript.js"),
            ],
        }
    }

    pub fn minimal() -> Self {
        Self {
            files: vec![
                TestFile::new("https://cdn.jsdelivr.net/npm/react@17.0.2/cjs/react.development.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js"),
                TestFile::new("https://cdn.jsdelivr.net/npm/typescript@4.9.5/lib/typescript.js"),
            ],
        }
    }

    pub fn complicated() -> Self {
        let files = Self::complicated_urls().into_iter().map(TestFile::new).collect();
        Self { files }
    }

    pub fn complicated_one(index: usize) -> Self {
        let url = Self::complicated_urls()[index];
        let file = TestFile::new(url);
        Self { files: vec![file] }
    }

    fn complicated_urls() -> [&'static str; 5] {
        [
            // TypeScript syntax (2.81MB)
            "https://raw.githubusercontent.com/microsoft/TypeScript/v5.3.3/src/compiler/checker.ts",
            // Real world app tsx (1.0M)
            "https://raw.githubusercontent.com/oxc-project/benchmark-files/main/cal.com.tsx",
            // Real world content-heavy app jsx (3K)
            "https://raw.githubusercontent.com/oxc-project/benchmark-files/main/RadixUIAdoptionSection.jsx",
            // Heavy with classes (554K)
            "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs",
            // ES5 (3.9M)
            "https://cdn.jsdelivr.net/npm/antd@5.12.5/dist/antd.js",
        ]
    }
}

pub struct TestFile {
    pub url: String,
    pub file_name: String,
    pub source_text: String,
}

impl TestFile {
    /// # Errors
    /// # Panics
    pub fn new(url: &str) -> Self {
        let (file_name, source_text) = Self::get_source_text(url).unwrap();
        Self { url: url.to_string(), file_name, source_text }
    }

    /// # Errors
    /// # Panics
    pub fn get_source_text(lib: &str) -> Result<(String, String), String> {
        if !lib.starts_with("https://") {
            return Err(format!("Not an https url: {lib:?}"));
        }
        let filename =
            lib.split('/').last().ok_or_else(|| "lib url has no segments".to_string())?;

        let file = project_root().join("target").join(filename);

        if let Ok(code) = std::fs::read_to_string(&file) {
            Ok((filename.to_string(), code))
        } else {
            println!("[{filename}] - Downloading [{lib}] to [{}]", file.display());
            match agent().get(lib).call() {
                Ok(mut response) => {
                    let mut reader = response.body_mut().as_reader();

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
