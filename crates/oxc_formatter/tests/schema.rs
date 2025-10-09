use std::fs;

use oxc_formatter::Oxfmtrc;
use project_root::get_project_root;

// NOTE: This test generates the JSON schema for the `.oxfmtrc.json` configuration file

#[test]
fn test_schema_json() {
    let path = get_project_root().unwrap().join("npm/oxfmt/configuration_schema.json");
    let schema = schemars::schema_for!(Oxfmtrc);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    let existing_json = fs::read_to_string(&path).unwrap_or_default();
    if existing_json.trim() != json.trim() {
        std::fs::write(&path, &json).unwrap();
    }
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(json);
    });
}
