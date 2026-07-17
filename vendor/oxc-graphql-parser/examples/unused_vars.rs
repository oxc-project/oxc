//! Example usage of `oxc-graphql-parser` to check for unused vars in a given GraphQL
//! query.

use oxc_graphql_parser::Allocator;
use oxc_graphql_parser::Parser;
use oxc_graphql_parser::ast;
use std::fs;
use std::path::Path;

fn are_variables_unused() {
    // Example mutation with variables.
    let file = Path::new("crates/oxc_graphql_parser/examples/graph_check_mutation.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, &src);
    let ast = parser.parse();

    assert_eq!(0, ast.errors().len());

    let doc = ast.document();

    for def in &doc.definitions {
        if let ast::Definition::Operation(op_def) = def {
            assert_eq!(op_def.name.as_ref().unwrap().as_str(), "GraphCheckMutation");

            // We grab all the variables defined in the mutation
            let variables: Vec<String> = op_def
                .variable_definitions
                .iter()
                .map(|definition| definition.variable.name.to_string())
                .collect();

            if let Some(selection_set) = &op_def.selection_set {
                let mut vec = Vec::default();
                // Get the variables defined in the mutation's selection set.
                let used_vars = get_variables_from_selection(&mut vec, selection_set);
                // Compare the two sets of variables.
                assert!(do_variables_match(&variables, used_vars));
            }
        }
    }
}

fn get_variables_from_selection<'a>(
    used_vars: &'a mut Vec<String>,
    selection_set: &ast::SelectionSet<'_>,
) -> &'a Vec<String> {
    for selection in &selection_set.selections {
        match selection {
            ast::Selection::Field(field) => {
                let mut vars = Vec::new();
                for argument in &field.arguments {
                    collect_variable_value(argument.value.as_ref(), &mut vars);
                }
                used_vars.append(&mut vars);
                if let Some(selection_set) = &field.selection_set {
                    get_variables_from_selection(used_vars, selection_set);
                }
            }
            _ => unimplemented!(),
        }
    }
    used_vars
}

fn collect_variable_value(value: Option<&ast::Value<'_>>, variables: &mut Vec<String>) {
    match value {
        Some(ast::Value::Variable(variable)) => variables.push(variable.name.to_string()),
        Some(ast::Value::List(list)) => {
            for value in &list.values {
                collect_variable_value(Some(value), variables);
            }
        }
        Some(ast::Value::Object(object)) => {
            for field in &object.fields {
                collect_variable_value(field.value.as_ref(), variables);
            }
        }
        _ => {}
    }
}

fn do_variables_match(a: &[String], b: &[String]) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

fn main() {
    are_variables_unused();
}
