/// Hook declaration detection utilities.
///
/// Port of `Utils/HookDeclaration.ts` from the React Compiler.
///
/// In the TS version, hook declarations are tagged Babel AST nodes.
/// In Rust, we detect hooks by name convention.
/// Check if a function name looks like a React hook (starts with "use" + uppercase).
pub fn is_hook_name(name: &str) -> bool {
    if !name.starts_with("use") {
        return false;
    }
    if name.len() == 3 {
        return true; // "use" alone
    }
    // The character after "use" must be uppercase
    name[3..].starts_with(|c: char| c.is_ascii_uppercase())
}
