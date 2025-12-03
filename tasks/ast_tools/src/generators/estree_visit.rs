//! Generator of ESTree visitor.

use std::{
    borrow::Cow,
    cmp::Ordering,
    fmt::{self, Display},
    process::{Command, Stdio},
};

use convert_case::{Case, Casing};
use indexmap::map::Entry;
use itertools::Itertools;
use serde::Deserialize;

use oxc_index::{IndexVec, define_index_type};

use crate::{
    Codegen, Generator, NAPI_PARSER_PACKAGE_PATH, OXLINT_APP_PATH,
    output::{Output, javascript::VariantGenerator},
    schema::Schema,
    utils::{FxIndexMap, string, write_it},
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

/// Names of ESTree function types.
const FUNCTION_TYPE_NAMES: &[&str] =
    &["ArrowFunctionExpression", "FunctionDeclaration", "FunctionExpression"];

pub struct ESTreeVisitGenerator;

define_generator!(ESTreeVisitGenerator);

impl Generator for ESTreeVisitGenerator {
    fn generate_many(&self, _schema: &Schema, codegen: &Codegen) -> Vec<Output> {
        let Codes {
            walk_parser,
            walk_oxlint,
            visitor_keys,
            type_ids_map_parser,
            type_ids_map_oxlint,
            visitor_type_parser,
            visitor_type_oxlint,
        } = generate(codegen);

        vec![
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/walk.js"),
                code: walk_parser,
            },
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/keys.js"),
                code: visitor_keys.clone(),
            },
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/type_ids.js"),
                code: type_ids_map_parser,
            },
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/visit/visitor.d.ts"),
                code: visitor_type_parser,
            },
            Output::Javascript {
                path: format!("{OXLINT_APP_PATH}/src-js/generated/walk.js"),
                code: walk_oxlint,
            },
            Output::Javascript {
                // This file is also valid as TS
                path: format!("{OXLINT_APP_PATH}/src-js/generated/keys.ts"),
                code: visitor_keys,
            },
            Output::Javascript {
                // This file is also valid as TS
                path: format!("{OXLINT_APP_PATH}/src-js/generated/type_ids.ts"),
                code: type_ids_map_oxlint,
            },
            Output::Javascript {
                path: format!("{OXLINT_APP_PATH}/src-js/generated/visitor.ts"),
                code: visitor_type_oxlint,
            },
        ]
    }
}

/// Output code.
struct Codes {
    walk_parser: String,
    walk_oxlint: String,
    visitor_keys: String,
    type_ids_map_parser: String,
    type_ids_map_oxlint: String,
    visitor_type_parser: String,
    visitor_type_oxlint: String,
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
#[expect(clippy::items_after_statements)]
fn generate(codegen: &Codegen) -> Codes {
    // Run `napi/parser/scripts/visitor-keys.js` to get visitor keys from TS-ESLint
    let script_path = codegen.root_path().join("napi/parser/scripts/visitor-keys.js");

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

        /* IF ANCESTORS */
        export const ancestors = [];
        /* END_IF */

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
    let mut type_ids_map = string!("
        // Mapping from node type name to node type ID
        export const NODE_TYPE_IDS_MAP = new Map([
            // Leaf nodes
    ");

    // `visitor_keys_entries` contains `(node_name, keys_str)` pairs, where `keys_str` formatted keys array
    // e.g. `["decorators", "key", "typeAnnotation"]`
    // `visitor_keys_vars` contains `(keys_str, var_name)` pairs, where `keys_str` is formatted keys array,
    // and `var_name` is name of variable for that array of keys.
    // `Some` if this array is used more than once (and so needs a variable), `None` otherwise.
    let mut visitor_keys_entries = vec![];
    let mut visitor_keys_vars = FxIndexMap::default();
    visitor_keys_vars.insert("[]".to_string(), Some("$EMPTY".to_string()));

    let mut visitor_type = string!("");

    let mut leaf_nodes_count = None;
    let mut function_node_ids = vec![];
    for (node_id, node) in nodes.iter_enumerated() {
        let is_leaf = node.keys.is_empty();
        if leaf_nodes_count.is_none() && !is_leaf {
            leaf_nodes_count = Some(node_id.raw() as usize);
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
                if (ANCESTORS) ancestors.unshift(node);
            ");

            for key in &node.keys {
                write_it!(walk_fn_body, "walkNode(node.{key}, visitors);\n");
            }

            walk_fn_body.push_str("
                if (ANCESTORS) ancestors.shift();
                if (exit !== null) exit(node);
            ");

            walk_fn_body
        };

        #[rustfmt::skip]
        write_it!(walk_fns, "
            function walk{node_name}(node, visitors) {{
                {walk_fn_body}
            }}
        ");

        let keys = &node.keys;
        let keys_str = format!("{keys:?}");

        visitor_keys_entries.push((node_name, keys_str.clone()));

        match visitor_keys_vars.entry(keys_str) {
            Entry::Occupied(mut entry) => {
                let var_name = entry.get_mut();
                if var_name.is_none() {
                    let name = keys.iter().map(|key| key.to_case(Case::Constant)).join("__");
                    *var_name = Some(name);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(None);
            }
        }

        write_it!(type_ids_map, "[\"{node_name}\", {node_id}],\n");

        if FUNCTION_TYPE_NAMES.contains(&node_name) {
            function_node_ids.push(node_id);
        }

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
            "TSModuleDeclaration" => Some(&["TSModuleDeclaration", "TSGlobalDeclaration"]),
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

    // Create 2 walker variants for parser and oxlint, by setting `ANCESTORS` const,
    // and running through minifier to shake out irrelevant code
    struct WalkVariantGenerator;
    impl VariantGenerator<1> for WalkVariantGenerator {
        const FLAG_NAMES: [&str; 1] = ["ANCESTORS"];
    }

    let mut walk_variants = WalkVariantGenerator.generate(&walk).into_iter();
    assert!(walk_variants.len() == 2);
    let walk_parser = walk_variants.next().unwrap();
    let walk_oxlint = walk_variants.next().unwrap();

    let leaf_nodes_count = leaf_nodes_count.unwrap();

    let visitor_keys_vars_str = visitor_keys_vars
        .iter()
        .filter_map(|(keys_str, var_name)| {
            var_name.as_ref().map(|name| format!("{name} = freeze({keys_str})"))
        })
        .join(",\n");

    let visitor_keys_entries_str = visitor_keys_entries
        .iter()
        .enumerate()
        .map(|(index, (name, keys_str))| {
            let entry = if let Some(var_name) = &visitor_keys_vars[keys_str] {
                format!("{name}: {var_name},\n")
            } else {
                format!("{name}: freeze({keys_str}),\n")
            };
            if index == leaf_nodes_count { format!("// Non-leaf nodes\n{entry}") } else { entry }
        })
        .join("");

    #[rustfmt::skip]
    let visitor_keys = format!("
        const {{ freeze }} = Object;

        const {visitor_keys_vars_str};

        export default freeze({{
            // Leaf nodes
            {visitor_keys_entries_str}
        }});
    ");

    let nodes_count = nodes.len();
    #[rustfmt::skip]
    write_it!(type_ids_map, "]);

        export const NODE_TYPES_COUNT = {nodes_count};
        export const LEAF_NODE_TYPES_COUNT = {leaf_nodes_count};");

    function_node_ids.sort_unstable();
    #[rustfmt::skip]
    let type_ids_map_oxlint = format!("
        {type_ids_map}
        export const FUNCTION_NODE_TYPE_IDS = {function_node_ids:?};
    ");

    // Versions of `visitor.d.ts` for parser and Oxlint import ESTree types from different places.
    // Oxlint version also allows any arbitrary properties (selectors).
    #[rustfmt::skip]
    let visitor_type_parser = format!("
        import * as ESTree from '@oxc-project/types';

        export interface VisitorObject {{
            {visitor_type}
        }}
    ");

    #[rustfmt::skip]
    let visitor_type_oxlint = format!("
        import type * as ESTree from './types.d.ts';

        type VisitorObjectBase = {{
            {visitor_type}
        }};

        export type VisitorObject = VisitorObjectBase & {{
            [key: string]: (node: ESTree.Node) => void;
        }}
    ");

    Codes {
        walk_parser,
        walk_oxlint,
        visitor_keys,
        type_ids_map_parser: type_ids_map,
        type_ids_map_oxlint,
        visitor_type_parser,
        visitor_type_oxlint,
    }
}
