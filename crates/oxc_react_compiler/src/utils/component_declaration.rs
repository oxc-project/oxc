/// Component declaration detection utilities.
///
/// Port of `Utils/ComponentDeclaration.ts` from the React Compiler.
///
/// In the TS version, component declarations are tagged Babel AST nodes.
/// In Rust, we detect components by name convention and context.

/// Check if a function name looks like a React component (starts with uppercase).
pub fn is_component_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_uppercase())
}
