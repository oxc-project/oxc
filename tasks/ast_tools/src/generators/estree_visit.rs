//! Generator of ESTree visitor keys.

use std::{
    cmp::Ordering,
    fmt::{self, Display},
    process::{Command, Stdio},
};

use serde::Deserialize;

use oxc_index::{IndexVec, define_index_type};

use crate::{
    Codegen, Generator, NAPI_PARSER_PACKAGE_PATH,
    output::Output,
    schema::Schema,
    utils::{string, write_it},
};

use super::define_generator;

define_index_type! {
    pub struct NodeId = u32;
}

impl Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw().fmt(f)
    }
}

pub struct ESTreeVisitGenerator;

define_generator!(ESTreeVisitGenerator);

impl Generator for ESTreeVisitGenerator {
    fn generate_many(&self, _schema: &Schema, codegen: &Codegen) -> Vec<Output> {
        let visitor_keys = generate(codegen);

        vec![Output::Javascript {
            path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/keys.mjs"),
            code: visitor_keys,
        }]
    }
}

/// Details of a node's name and visitor keys.
#[derive(Deserialize, Debug)]
struct NodeKeys {
    name: String,
    keys: Vec<String>,
}

/// Generate visitor keys.
fn generate(codegen: &Codegen) -> String {
    // Run `napi/parser/scripts/visitor-keys.mjs` to get visitor keys from TS-ESLint
    let script_path = codegen.root_path().join("napi/parser/scripts/visitor-keys.mjs");

    let output = Command::new("node")
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(output.status.success() && output.stderr.is_empty());
    let json = String::from_utf8(output.stdout).unwrap();
    let mut nodes: IndexVec<NodeId, NodeKeys> = serde_json::from_str(&json).unwrap();

    // Remove types which do not exist in Oxc AST
    // TODO: Why don't they exist?
    let remove = [
        "TSAbstractKeyword",
        "TSAsyncKeyword",
        "TSDeclareKeyword",
        "TSExportKeyword",
        "TSPrivateKeyword",
        "TSProtectedKeyword",
        "TSPublicKeyword",
        "TSReadonlyKeyword",
        "TSStaticKeyword",
        "ExperimentalRestProperty",
        "ExperimentalSpreadProperty",
    ];
    nodes.retain(|node| !remove.contains(&node.name.as_str()));

    // Add types which don't exist in TS-ESTree AST
    let extra: &[(&str, &[&str])] = &[
        ("ParenthesizedExpression", &["expression"]),
        ("V8IntrinsicExpression", &["name", "arguments"]),
        ("TSParenthesizedType", &["typeAnnotation"]),
        ("TSJSDocNonNullableType", &["typeAnnotation"]),
        ("TSJSDocNullableType", &["typeAnnotation"]),
        ("TSJSDocUnknownType", &[]),
    ];
    nodes.extend(extra.iter().map(|&(name, keys)| NodeKeys {
        name: name.to_string(),
        keys: keys.iter().map(|&key| key.to_string()).collect(),
    }));

    // Sort by:
    // * Leaf nodes before non-leaf nodes.
    // * JS first, then JSX, then TS.
    // * Alphabetical order.
    nodes.sort_by(|v1, v2| match (v1.keys.is_empty(), v2.keys.is_empty()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => {
            let name1 = v1.name.as_str();
            let name2 = v2.name.as_str();
            let is_jsx1 = name1.starts_with("JSX");
            let is_ts1 = name1.starts_with("TS");
            let is_jsx2 = name2.starts_with("JSX");
            let is_ts2 = name2.starts_with("TS");

            match (is_jsx1 || is_ts1, is_jsx2 || is_ts2) {
                (false, true) => Ordering::Less,
                (true, false) => Ordering::Greater,
                (true, true) if is_ts1 != is_ts2 => is_ts1.cmp(&is_ts2),
                _ => name1.cmp(name2),
            }
        }
    });

    #[rustfmt::skip]
    let mut visitor_keys = string!("
        export default {
            // Leaf nodes
    ");

    let mut leaf_nodes_count = None;
    for (node_id, node) in nodes.iter_enumerated() {
        let is_leaf = node.keys.is_empty();
        if leaf_nodes_count.is_none() && !is_leaf {
            leaf_nodes_count = Some(node_id.raw());
            visitor_keys.push_str("// Non-leaf nodes\n");
        }

        let node_name = node.name.as_str();
        let keys = &node.keys;
        write_it!(visitor_keys, "{node_name}: {keys:?},\n");
    }

    visitor_keys.push_str("};");

    visitor_keys
}
