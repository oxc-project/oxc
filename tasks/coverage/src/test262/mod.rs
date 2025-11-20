mod meta;

use std::path::Path;

pub use self::meta::{MetaData, Negative, Phase, TestFlag};
use crate::suite::{MetadataParser, TestFilter, TestMetadata};

fn read_metadata(code: &str) -> MetaData {
    let Some(start) = code.find("/*---") else {
        return MetaData::default();
    };
    let Some(end) = code.find("---*/") else {
        return MetaData::default();
    };
    let s = &code[start + 5..end].replace('\r', "\n");
    MetaData::from_str(s)
}

pub struct Test262MetadataParser;

impl MetadataParser for Test262MetadataParser {
    fn parse(&self, _path: &Path, code: &str) -> TestMetadata {
        let meta = read_metadata(code);

        TestMetadata::Test262 {
            esid: meta.esid,
            features: meta.features,
            includes: meta.includes,
            flags: meta.flags,
            negative: meta.negative,
        }
    }
}

/// Test262 test filter
/// Filters staging tests, markdown files, and fixtures
pub struct Test262Filter {
    path_filter: crate::suite::PathBasedFilter,
}

impl Test262Filter {
    pub const fn new() -> Self {
        const EXCLUDED_DIRS: &[&str] = &["test262/test/staging"];
        const EXCLUDED_PATHS: &[&str] = &["_FIXTURE"];
        const EXCLUDED_EXTENSIONS: &[&str] = &["md"];

        Self {
            path_filter: crate::suite::PathBasedFilter::new(
                EXCLUDED_DIRS,
                EXCLUDED_PATHS,
                EXCLUDED_EXTENSIONS,
            ),
        }
    }
}

impl TestFilter for Test262Filter {
    fn skip_path(&self, path: &Path) -> bool {
        self.path_filter.should_skip(path)
    }

    fn skip_test(&self, _test: &crate::suite::ParsedTest) -> bool {
        // Parser tool runs on all tests (no additional filtering)
        false
    }
}
