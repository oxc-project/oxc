use cow_utils::CowUtils;

use crate::ir::{Doc, Fill, Group, IfBreak, IndentIfBreak, Line};

impl std::fmt::Display for Doc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_doc_ast(self))
    }
}

/// Print as Prettier's Doc AST format.
/// You can pass this to `Prettier.__debug.formatDoc(docAst)` to inspect them as Doc commands.
fn print_doc_ast(doc: &Doc<'_>) -> String {
    let mut json = String::new();

    match doc {
        Doc::Str(s) => {
            let escaped = s.cow_replace('\\', "\\\\");
            let escaped = escaped.cow_replace('"', "\\\"");
            let escaped = escaped.cow_replace('\n', "\\n");
            let escaped = escaped.cow_replace('\r', "\\r");
            let escaped = escaped.cow_replace('\t', "\\t");
            let escaped = escaped.cow_replace('\u{000C}', "\\f");
            let escaped = escaped.cow_replace('\u{0008}', "\\b");
            json.push_str(&format!("\"{escaped}\""));
        }
        Doc::Array(docs) => {
            json.push_str(&format!(
                "[{}]",
                docs.iter().map(|doc| print_doc_ast(doc)).collect::<Vec<_>>().join(",")
            ));
        }
        Doc::Indent(contents) => {
            json.push('{');
            json.push_str(r#""type":"indent""#);
            json.push(',');
            json.push_str(&format!(
                r#""contents":[{}]"#,
                contents.iter().map(|doc| print_doc_ast(doc)).collect::<Vec<_>>().join(",")
            ));
            json.push('}');
        }
        Doc::IndentIfBreak(IndentIfBreak { contents, group_id }) => {
            json.push('{');
            json.push_str(r#""type":"indent-if-break""#);
            json.push(',');
            json.push_str(&format!(r#""contents":{}"#, print_doc_ast(contents)));
            json.push(',');
            json.push_str(&format!(r#""groupId":"{group_id}""#));
            json.push('}');
        }
        Doc::Group(Group { contents, should_break, expanded_states, group_id }) => {
            json.push('{');
            json.push_str(r#""type":"group""#);
            if let Some(group_id) = group_id {
                json.push(',');
                json.push_str(&format!(r#""id":"{group_id}""#));
            }
            json.push(',');
            json.push_str(&format!(
                r#""contents":[{}]"#,
                contents.iter().map(|doc| print_doc_ast(doc)).collect::<Vec<_>>().join(",")
            ));
            json.push(',');
            json.push_str(&format!(r#""break":{should_break}"#));
            if let Some(expanded_states) = expanded_states {
                json.push(',');
                json.push_str(&format!(
                    r#""expandStates":[{}]"#,
                    expanded_states
                        .iter()
                        .map(|doc| print_doc_ast(doc))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            json.push('}');
        }
        Doc::Line(Line { soft, hard, literal }) => {
            if *literal {
                json.push_str(r#"{"type":"line","literal":true,"hard":true}"#);
            } else if *hard {
                json.push_str(r#"{"type":"line","hard":true}"#);
            } else if *soft {
                json.push_str(r#"{"type":"line","soft":true}"#);
            } else {
                json.push_str(r#"{"type":"line"}"#);
            }
        }
        Doc::IfBreak(IfBreak { break_contents, flat_contents, group_id }) => {
            json.push('{');
            json.push_str(r#""type":"if-break""#);
            json.push(',');
            json.push_str(&format!(r#""breakContents":{}"#, print_doc_ast(break_contents)));
            json.push(',');
            json.push_str(&format!(r#""flatContents":{}"#, print_doc_ast(flat_contents)));
            if let Some(group_id) = group_id {
                json.push(',');
                json.push_str(&format!(r#""groupId":"{group_id}""#));
            }
            json.push('}');
        }
        Doc::Fill(Fill { parts }) => {
            json.push('{');
            json.push_str(r#""type":"fill""#);
            json.push(',');
            json.push_str(&format!(
                r#""parts":[{}]"#,
                parts.iter().map(|doc| print_doc_ast(doc)).collect::<Vec<_>>().join(",")
            ));
            json.push('}');
        }
        Doc::LineSuffix(docs) => {
            json.push('{');
            json.push_str(r#""type":"line-suffix""#);
            json.push(',');
            json.push_str(&format!(
                r#""contents":[{}]"#,
                docs.iter().map(|doc| print_doc_ast(doc)).collect::<Vec<_>>().join(",")
            ));
            json.push('}');
        }
        Doc::LineSuffixBoundary => {
            json.push_str(r#"{"type":"line-suffix-boundary"}"#);
        }
        Doc::BreakParent => {
            json.push_str(r#"{"type":"break-parent"}"#);
        }
    }

    json
}
