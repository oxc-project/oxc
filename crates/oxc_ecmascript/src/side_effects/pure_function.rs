use oxc_ast::ast::{ChainElement, Expression};

/// Check if the callee is a pure function based on a list of pure function names.
///
/// This handles:
/// - Simple identifiers: `foo()` matches `["foo"]`
/// - Member expressions: `console.log()` matches `["console"]` or `["console.log"]`
/// - Chained calls: `styled()()` or `styled().div()` matches `["styled"]`
/// - Optional chaining: `styled?.div()` or `console?.log()` matches `["styled"]` or `["console.log"]`
///
/// Besides any functions matching that name, any properties on a pure function
/// and any functions returned from a pure function will also be considered pure.
/// For example, if `["console.log"]` is specified:
/// - `console.log()` is pure
/// - `console.log.foo()` is pure (property on pure function)
/// - `console.log()()` is pure (function returned from pure function)
/// - `console.log().foo()` is pure (property on returned function)
pub fn is_pure_function(callee: &Expression, pure_functions: &[String]) -> bool {
    if pure_functions.is_empty() {
        return false;
    }
    let Some(path_parts) = extract_callee_path(callee) else {
        return false;
    };
    pure_functions.iter().any(|pure_fn| is_path_match(&path_parts, pure_fn))
}

/// Extract the path parts from a callee expression.
/// Returns None if the callee cannot be matched (e.g., contains non-string computed properties).
/// The returned Vec contains path parts in reverse order (from outermost to root).
fn extract_callee_path<'a>(callee: &'a Expression<'a>) -> Option<Vec<&'a str>> {
    let mut path_parts: Vec<&str> = Vec::new();
    let mut current = callee;

    loop {
        match current {
            Expression::Identifier(ident) => {
                path_parts.push(ident.name.as_str());
                break;
            }
            Expression::StaticMemberExpression(member) => {
                path_parts.push(member.property.name.as_str());
                current = &member.object;
            }
            Expression::ComputedMemberExpression(member) => {
                let Expression::StringLiteral(lit) = &member.expression else {
                    return None;
                };
                path_parts.push(lit.value.as_str());
                current = &member.object;
            }
            Expression::CallExpression(call) => {
                // Call expressions don't add to the path, they just pass through
                // But they do "seal" the previous path - anything before a call is an extension
                path_parts.clear();
                current = &call.callee;
            }
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::StaticMemberExpression(member) => {
                    path_parts.push(member.property.name.as_str());
                    current = &member.object;
                }
                ChainElement::ComputedMemberExpression(member) => {
                    let Expression::StringLiteral(lit) = &member.expression else {
                        return None;
                    };
                    path_parts.push(lit.value.as_str());
                    current = &member.object;
                }
                ChainElement::CallExpression(call) => {
                    // Call expressions don't add to the path, they just pass through
                    // But they do "seal" the previous path - anything before a call is an extension
                    path_parts.clear();
                    current = &call.callee;
                }
                ChainElement::TSNonNullExpression(ts) => {
                    current = &ts.expression;
                }
                ChainElement::PrivateFieldExpression(_) => {
                    return None;
                }
            },
            Expression::ParenthesizedExpression(paren) => {
                current = &paren.expression;
            }
            _ => {
                return None;
            }
        }
    }

    Some(path_parts)
}

/// Check if the extracted path matches the given pure function name.
/// The pure function name can be a dotted path like "console.log".
/// Returns true if the pure function name is a prefix of the callee's path.
fn is_path_match(path_parts: &[&str], pure_fn: &str) -> bool {
    let pure_parts_count = pure_fn.bytes().filter(|&b| b == b'.').count() + 1;
    if pure_parts_count > path_parts.len() {
        return false;
    }
    pure_fn.split('.').zip(path_parts.iter().rev()).all(|(a, b)| a == *b)
}
