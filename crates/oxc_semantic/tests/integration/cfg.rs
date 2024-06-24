use std::fs;

use oxc_span::SourceType;

use crate::util::SemanticTester;

#[test]
fn test_cfg_files() {
    insta::glob!("cfg_fixtures/*.js", |path| {
        let code = fs::read_to_string(path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let output =
            SemanticTester::new(&code, SourceType::from_path(path).unwrap()).with_cfg(true);
        let snapshot = format!("{}\n\n{}", output.basic_blocks_printed(), output.cfg_dot_diagram());
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "" }, {
            insta::assert_snapshot!(name, snapshot);
        });
    });
}
