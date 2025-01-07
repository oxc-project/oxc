use crate::ir::{Doc, Line};

impl std::fmt::Display for Doc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", print_doc_to_debug(self))
    }
}

// https://github.com/prettier/prettier/blob/3.3.3/src/document/debug.js
fn print_doc_to_debug(doc: &Doc<'_>) -> std::string::String {
    use std::string::String;
    let mut string = String::new();
    match doc {
        Doc::Str(s) => {
            string.push('"');
            string.push_str(s);
            string.push('"');
        }
        Doc::Array(docs) => {
            string.push_str("[\n");
            for (idx, doc) in docs.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != docs.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("]\n");
        }
        Doc::Indent(contents) => {
            string.push_str("indent([");
            for (idx, doc) in contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != contents.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("])");
        }
        Doc::IndentIfBreak(indent_if_break) => {
            string.push_str("indentIfBreak(");
            string.push_str("[\n");
            string.push_str(&print_doc_to_debug(&indent_if_break.contents));
            string.push_str(&format!(", {{id: {}}}", indent_if_break.group_id));
            string.push_str("])");
        }
        Doc::Group(group) => {
            if group.expanded_states.is_some() {
                string.push_str("conditionalGroup([\n");
            }

            string.push_str("group([\n");
            for (idx, doc) in group.contents.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != group.contents.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("], { shouldBreak: ");
            string.push_str(&group.should_break.to_string());
            if let Some(id) = group.group_id {
                string.push_str(&format!(", id: {id}"));
            }
            string.push_str(" })");

            if let Some(expanded_states) = &group.expanded_states {
                string.push_str(",\n");
                for (idx, doc) in expanded_states.iter().enumerate() {
                    string.push_str(&print_doc_to_debug(doc));
                    if idx != expanded_states.len() - 1 {
                        string.push_str(", ");
                    }
                }
                string.push_str("])");
            }
        }
        Doc::Line(Line { soft, hard, .. }) => {
            if *soft {
                string.push_str("softline");
            } else if *hard {
                string.push_str("hardline");
            } else {
                string.push_str("line");
            }
        }
        Doc::IfBreak(if_break) => {
            string.push_str(&format!(
                "ifBreak({}, {}",
                print_doc_to_debug(&if_break.break_contents),
                print_doc_to_debug(&if_break.flat_contents)
            ));
            if let Some(group_id) = if_break.group_id {
                string.push_str(&format!(", {{ groupId: {group_id} }}"));
            }
            string.push(')');
        }
        Doc::Fill(fill) => {
            string.push_str("fill([\n");
            let parts = fill.parts();
            for (idx, doc) in parts.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != parts.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push_str("])");
        }
        Doc::LineSuffix(docs) => {
            string.push_str("lineSuffix(");
            for (idx, doc) in docs.iter().enumerate() {
                string.push_str(&print_doc_to_debug(doc));
                if idx != docs.len() - 1 {
                    string.push_str(", ");
                }
            }
            string.push(')');
        }
        Doc::LineSuffixBoundary => {
            string.push_str("lineSuffixBoundary");
        }
        Doc::BreakParent => {
            string.push_str("BreakParent");
        }
    }

    string
}
