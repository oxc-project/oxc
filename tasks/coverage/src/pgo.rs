use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use oxc::span::SourceType;
use oxc_tasks_common::write_pgo_case;

use crate::{
    TestData,
    test262::{Phase, TestFlag},
};

impl TestData {
    /// Export the same inputs as conformance, without executing tools or saving snapshots.
    ///
    /// # Errors
    /// Returns an error if the output cannot be written.
    pub fn export_pgo(&self, output: &Path) -> io::Result<()> {
        let mut writer = BufWriter::new(File::create(output)?);
        self.write_pgo(&mut writer)?;
        writer.flush()
    }

    fn write_pgo(&self, writer: &mut impl Write) -> io::Result<()> {
        // Loader order is parallel; sort each suite for reproducible corpus shards.
        let mut files: Vec<_> = self.test262.iter().collect();
        files.sort_unstable_by_key(|file| &file.path);
        for file in files {
            let flags = &file.meta.flags;
            let expected_failure =
                file.meta.negative.as_ref().is_some_and(|n| n.phase == Phase::Parse);
            let source_type = SourceType::script();
            if flags.contains(&TestFlag::Module) {
                write_pgo_case(
                    writer,
                    "test262",
                    &file.path,
                    &file.code,
                    source_type.with_module(true),
                    expected_failure,
                )?;
            } else {
                if !flags.contains(&TestFlag::OnlyStrict) {
                    write_pgo_case(
                        writer,
                        "test262",
                        &file.path,
                        &file.code,
                        source_type,
                        expected_failure,
                    )?;
                }
                if !flags.contains(&TestFlag::NoStrict) && !flags.contains(&TestFlag::Raw) {
                    write_pgo_case(
                        writer,
                        "test262",
                        &file.path,
                        &format!("'use strict';\n{}", file.code),
                        source_type,
                        expected_failure,
                    )?;
                }
            }
        }
        let mut files: Vec<_> = self.babel.iter().collect();
        files.sort_unstable_by_key(|file| &file.path);
        for file in files {
            let source_type = if file.options.allow_return_outside_function {
                file.source_type.with_commonjs(true)
            } else {
                file.source_type
            };
            write_pgo_case(
                writer,
                "babel-parser",
                &file.path,
                &file.code,
                source_type,
                file.should_fail,
            )?;
        }
        let mut files: Vec<_> = self.typescript.iter().collect();
        files.sort_unstable_by_key(|file| &file.path);
        for file in files {
            for unit in &file.units {
                for &always_strict in &file.settings.always_strict {
                    let code = if always_strict {
                        format!("'use strict';\n{}", unit.content)
                    } else {
                        unit.content.clone()
                    };
                    write_pgo_case(
                        writer,
                        "typescript",
                        &file.path.join(&unit.name),
                        &code,
                        unit.source_type,
                        file.should_fail,
                    )?;
                }
            }
        }
        let mut files: Vec<_> = self.misc.iter().collect();
        files.sort_unstable_by_key(|file| &file.path);
        for file in files {
            write_pgo_case(
                writer,
                "misc",
                &file.path,
                &file.code,
                file.source_type,
                file.should_fail,
            )?;
        }
        let mut files: Vec<_> = self.acorn_jsx.iter().collect();
        files.sort_unstable_by_key(|file| &file.path);
        for file in files {
            write_pgo_case(
                writer,
                "acorn-jsx",
                &file.path,
                &file.code,
                SourceType::jsx(),
                file.should_fail,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Test262File, test262::parse_meta};

    #[test]
    fn strict_variants_and_negative_metadata() {
        let data = TestData {
            test262: vec![Test262File {
                path: "negative.js".into(),
                code: "/*---\nnegative:\n  phase: parse\n  type: SyntaxError\n---*/\nlet =".into(),
                meta: parse_meta("/*---\nnegative:\n  phase: parse\n  type: SyntaxError\n---*/"),
            }],
            babel: vec![],
            typescript: vec![],
            misc: vec![],
            acorn_jsx: vec![],
        };
        let mut output = Vec::new();
        data.write_pgo(&mut output).unwrap();
        let records: Vec<serde_json::Value> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["options"]["sourceType"], "script");
        assert_eq!(records[0]["expectedFailure"], true);
        assert!(records[1]["code"].as_str().unwrap().starts_with("'use strict';\n"));
    }

    #[test]
    fn typescript_virtual_units_preserve_language_and_strict_variants() {
        use crate::{
            TypeScriptFile,
            typescript::meta::{CompilerSettings, TestUnitData},
        };

        let mut settings = CompilerSettings::new(&rustc_hash::FxHashMap::default());
        settings.always_strict = vec![false, true];
        let data = TestData {
            test262: vec![],
            babel: vec![],
            misc: vec![],
            acorn_jsx: vec![],
            typescript: vec![TypeScriptFile {
                path: "virtual.ts".into(),
                code: String::new(),
                settings,
                should_fail: false,
                error_codes: vec![],
                units: vec![TestUnitData {
                    name: "view.tsx".into(),
                    content: "export const el = <div />;".into(),
                    source_type: SourceType::tsx().with_module(true),
                    ts_ignore_spans: vec![],
                }],
            }],
        };
        let mut output = Vec::new();
        data.write_pgo(&mut output).unwrap();
        let records: Vec<serde_json::Value> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["filename"], "virtual.ts/view.tsx");
        assert_eq!(records[0]["options"]["lang"], "tsx");
        assert_eq!(records[0]["options"]["sourceType"], "module");
        assert!(records[1]["code"].as_str().unwrap().starts_with("'use strict';\n"));
    }
}
