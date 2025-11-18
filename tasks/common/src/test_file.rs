use std::fmt;

use oxc_span::SourceType;

use crate::{project_root, request::agent};

pub struct TestFiles {
    files: Vec<TestFile>,
}

impl TestFiles {
    pub fn files(&self) -> &Vec<TestFile> {
        &self.files
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

    pub fn formatter() -> Self {
        Self {
            files: [
                // Small JSX (61L / 2.46KB)
                "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx",
                // Small TS (191L / 8.23KB)
                "https://cdn.jsdelivr.net/gh/vuejs/core@v3.5.24/packages/compiler-core/src/errors.ts",
                // Medium TSX (389L / 11.7KB)
                "https://cdn.jsdelivr.net/gh/outline/outline@v1.0.1/app/scenes/Search/Search.tsx",
                // Medium JS (426L / 12.3KB)
                "https://cdn.jsdelivr.net/gh/prettier/prettier@3.6.2/src/main/core.js",
                // Medium TS (619L / 17.3KB)
                "https://cdn.jsdelivr.net/gh/vercel/next.js@v16.0.2/packages/next/src/server/next.ts",
                // Large TSX (995L / 30.5KB)
                "https://cdn.jsdelivr.net/gh/vercel/next.js@v16.0.2/packages/next/src/client/index.tsx",
                // Large JS (1092L / 27.2KB)
                "https://cdn.jsdelivr.net/gh/prettier/prettier@3.6.2/src/language-js/comments/handle-comments.js",
                // Large TS (2370L / 76.1KB)
                "https://cdn.jsdelivr.net/gh/honojs/hono@v4.10.5/src/types.ts",
                // Extra large TSX (11180L / 346KB)
                "https://cdn.jsdelivr.net/gh/excalidraw/excalidraw@v0.18.0/packages/excalidraw/components/App.tsx",
            ].into_iter().map(TestFile::new).collect(),
        }
    }

    pub fn minimal() -> Self {
        Self {
            files: [
                "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx",
                "https://cdn.jsdelivr.net/npm/react@17.0.2/cjs/react.development.js",
                "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/cal.com.tsx",
                "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/binder.ts",
            ].into_iter().map(TestFile::new).collect(),
        }
    }

    pub fn complicated() -> Self {
        Self {
            files: [
                // TypeScript syntax (2.81MB)
                "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts",
                // Real world app tsx (1.0M)
                "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/cal.com.tsx",
                // Real world content-heavy app jsx (3K)
                "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx",
                // Heavy with classes (554K)
                "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs",
                // ES5 (6.7M)
                "https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js",
                // TypeScript syntax (189K)
                "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/binder.ts",
            ].into_iter().map(TestFile::new).collect(),
        }
    }
}

pub struct TestFile {
    pub url: String,
    pub file_name: String,
    pub source_text: String,
    pub source_type: SourceType,
}

impl TestFile {
    /// # Errors
    /// # Panics
    pub fn new(url: &str) -> Self {
        let (file_name, source_text) = Self::get_source_text(url).unwrap();
        let source_type = SourceType::from_path(&file_name).unwrap();
        Self { url: url.to_string(), file_name, source_text, source_type }
    }

    /// # Errors
    /// # Panics
    pub fn get_source_text(lib: &str) -> Result<(String, String), String> {
        if !lib.starts_with("https://") {
            return Err(format!("Not an https url: {lib:?}"));
        }
        let filename =
            lib.split('/').next_back().ok_or_else(|| "lib url has no segments".to_string())?;

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
                    std::io::copy(&mut reader, &mut writer).map_err(err_to_string)?;

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
