use std::fmt::Display;

use located_yaml::{YamlElt, YamlLoader};
use miette::SourceSpan;

/// Whether a rule is under the pass or the fail column of the plugin file.
pub enum PassOrFail {
    Pass,
    Fail,
}

impl Display for PassOrFail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "pass"),
            Self::Fail => write!(f, "fail"),
        }
    }
}

/// Finds the entire span of the test (including the `relative_path` and the `code`)
pub fn span_of_test_n(
    yaml_text: &str,
    test_ix: usize,
    test_code: &str,
    pass_or_fail: &PassOrFail,
) -> SourceSpan {
    let yaml = YamlLoader::load_from_str(
        // TODO: Should we just save the string after we read it originally?
        yaml_text,
    )
    .expect("to be able to parse yaml for error reporting");

    // access the first hash map which will be the top-level hashmap (the one that contains `query`, `rules`, etc)
    let YamlElt::Hash(hash) = &yaml.docs[0].yaml else {
        unreachable!("must be a top level hashmap in the yaml")
    };
    // find the `tests` hashmap key
    let tests_hash_key = hash
        .keys()
        .find(|x| {
            let YamlElt::String(str) = &x.yaml else { return false };
            str == "tests"
        })
        .expect("to be able to find tests hash in yaml file");
    // access the `tests` hashmap
    let YamlElt::Hash(tests_hash) = &hash[tests_hash_key].yaml else {
        unreachable!("there must be a tests hashmap in the yaml")
    };
    // find the `pass` or `fail` hashmap key
    let pass_or_fail_hash_key = tests_hash
        .keys()
        .find(|x| {
            let YamlElt::String(str) = &x.yaml else { return false };
            *str == pass_or_fail.to_string()
        })
        .expect("to be able to find pass hash in yaml file");
    // access the `pass` or `fail` hashmap
    let YamlElt::Array(test_arr) = &tests_hash[pass_or_fail_hash_key].yaml else {
        unreachable!("there must be a pass array in the yaml")
    };
    // access the specific test we're looking for
    let test_hash_span = &test_arr[test_ix].lines_range();
    let start = yaml_text
        .char_indices()
        .filter(|a| a.1 == '\n')
        .nth(test_hash_span.0 - 1) // subtract one because span is 1-based
        .map(|a| a.0)
        .expect("to find start of span of test");
    let start_of_end = yaml_text[start..]
        .find(&test_code[0..test_code.find('\n').unwrap_or(test_code.len())])
        .expect("to find start of end")
        + start;

    let nl = test_code.chars().filter(|a| *a == '\n').count();
    let end_of_end = yaml_text[start_of_end..]
        .char_indices()
        .filter(|a| a.1 == '\n')
        .nth(nl - 1)
        .map(|a| a.0)
        .expect("to find end of end of span of test")
        + start_of_end;

    SourceSpan::new(start.into(), (end_of_end - start).into())
}
