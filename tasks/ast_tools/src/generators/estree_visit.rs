//! Generator of ESTree visitor.

use std::{
    borrow::Cow,
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
        let Codes { walk, visitor_keys, type_ids_map, visitor_type } = generate(codegen);

        vec![
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/walk.mjs"),
                code: walk,
            },
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/keys.mjs"),
                code: visitor_keys,
            },
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/types.mjs"),
                code: type_ids_map,
            },
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/visitor.d.ts"),
                code: visitor_type,
            },
        ]
    }
}

/// Output code.
struct Codes {
    walk: String,
    visitor_keys: String,
    type_ids_map: String,
    visitor_type: String,
}

/// Details of a node's name and visitor keys.
#[derive(Deserialize, Debug)]
struct NodeKeys {
    name: String,
    keys: Vec<String>,
}

/// Generate:
/// * Walk functions.
/// * Visitor keys.
/// * `Map` from node type name to node type ID.
/// * Visitor type definition.
fn generate(codegen: &Codegen) -> Codes {
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

    // Generate code
    #[rustfmt::skip]
    let mut walk = string!("
        export { walkProgram }

        const { isArray } = Array;

        function walkNode(node, visitors) {
            if (node == null) return;
            if (isArray(node)) {
                const len = node.length;
                for (let i = 0; i < len; i++) {
                    walkNode(node[i], visitors);
                }
            } else {
                switch (node.type) {
    ");

    let mut walk_fns = string!("");

    #[rustfmt::skip]
    let mut visitor_keys = string!("
        export default Object.freeze({
            // Leaf nodes
    ");

    #[rustfmt::skip]
    let mut type_ids_map = string!("
        // Mapping from node type name to node type ID
        export const NODE_TYPE_IDS_MAP = new Map([
            // Leaf nodes
    ");

    #[rustfmt::skip]
    let mut visitor_type = string!("
        import * as ESTree from '@oxc-project/types';

        export interface VisitorObject {
    ");

    let mut leaf_nodes_count = None;
    for (node_id, node) in nodes.iter_enumerated() {
        let is_leaf = node.keys.is_empty();
        if leaf_nodes_count.is_none() && !is_leaf {
            leaf_nodes_count = Some(node_id.raw());
            visitor_keys.push_str("// Non-leaf nodes\n");
            type_ids_map.push_str("// Non-leaf nodes\n");
        }

        let node_name = node.name.as_str();
        write_it!(walk, "case \"{node_name}\": walk{node_name}(node, visitors); break;\n");

        #[rustfmt::skip]
        let walk_fn_body = if is_leaf {
            format!("
                const visit = visitors[{node_id}];
                if (visit !== null) visit(node);
            ")
        } else {
            let mut walk_fn_body = format!("
                const enterExit = visitors[{node_id}];
                let exit = null;
                if (enterExit !== null) {{
                    let enter;
                    ({{ enter, exit }} = enterExit);
                    if (enter !== null) enter(node);
                }}
            ");

            for key in &node.keys {
                write_it!(walk_fn_body, "walkNode(node.{key}, visitors);\n");
            }

            walk_fn_body.push_str("if (exit !== null) exit(node);\n");

            walk_fn_body
        };

        #[rustfmt::skip]
        write_it!(walk_fns, "
            function walk{node_name}(node, visitors) {{
                {walk_fn_body}
            }}
        ");

        let keys = &node.keys;
        write_it!(visitor_keys, "{node_name}: {keys:?},\n");
        write_it!(type_ids_map, "[\"{node_name}\", {node_id}],\n");

        // Convert ESTree type name to Oxc type names where they diverge
        let type_names: Option<&[&str]> = match node_name {
            "Literal" => Some(&[
                "BooleanLiteral",
                "NullLiteral",
                "NumericLiteral",
                "StringLiteral",
                "BigIntLiteral",
                "RegExpLiteral",
            ]),
            "Identifier" => Some(&[
                "IdentifierName",
                "IdentifierReference",
                "BindingIdentifier",
                "LabelIdentifier",
                "TSThisParameter",
                "TSIndexSignatureName",
            ]),
            "Property" => Some(&[
                "ObjectProperty",
                "AssignmentTargetProperty",
                "AssignmentTargetPropertyProperty",
                "BindingProperty",
            ]),
            "RestElement" => {
                Some(&["AssignmentTargetRest", "BindingRestElement", "FormalParameterRest"])
            }
            _ => None,
        };
        let type_def = if let Some(type_names) = type_names {
            Cow::Owned(type_names.join(" | ESTree."))
        } else {
            let type_name = match node_name {
                "FunctionDeclaration"
                | "FunctionExpression"
                | "TSDeclareFunction"
                | "TSEmptyBodyFunctionExpression" => "Function",
                "ClassDeclaration" | "ClassExpression" => "Class",
                _ if node_name.starts_with("TSJSDoc") => &node_name[2..],
                _ if node_name.starts_with("TSAbstract") => &node_name[10..],
                _ => node_name,
            };
            Cow::Borrowed(type_name)
        };

        write_it!(
            visitor_type,
            "{node_name}?: (node: ESTree.{type_def}) => void;
            '{node_name}:exit'?: (node: ESTree.{type_def}) => void;
            "
        );
    }

    #[rustfmt::skip]
    write_it!(walk, "
                }}
            }}
        }}

        {walk_fns}
    ");

    visitor_keys.push_str("});");

    let nodes_count = nodes.len();
    let leaf_nodes_count = leaf_nodes_count.unwrap();
    #[rustfmt::skip]
    write_it!(type_ids_map, "]);

        export const NODE_TYPES_COUNT = {nodes_count};
        export const LEAF_NODE_TYPES_COUNT = {leaf_nodes_count};
    ");

    visitor_type.push('}');

    Codes { walk, visitor_keys, type_ids_map, visitor_type }
}
