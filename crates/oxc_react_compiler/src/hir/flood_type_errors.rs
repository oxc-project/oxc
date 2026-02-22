/// Type error reporting for the Flood type system.
///
/// Port of `Flood/TypeErrors.ts` from the React Compiler.
///
/// Provides error constructors for type checking failures in the
/// experimental Flow type inference system.
use crate::compiler_error::{CompilerError, SourceLocation};

use super::flood_types::{ConcreteType, VariableId};

/// Error when a language feature is not supported by the type checker.
pub fn unsupported_language_feature(desc: &str, loc: SourceLocation) -> CompilerError {
    CompilerError::invalid_js(
        &format!("Type checker does not currently support language feature: {desc}"),
        None,
        loc,
    )
}

/// A unification error — two types that cannot be unified.
#[derive(Debug, Clone)]
pub enum UnificationError {
    TypeUnification { left: ConcreteType, right: ConcreteType },
}

/// Report unification errors.
///
/// # Errors
/// Returns a `CompilerError` if there are any unification errors.
pub fn raise_unification_errors(
    errors: Option<&[UnificationError]>,
    loc: SourceLocation,
) -> Result<(), CompilerError> {
    match errors {
        None => Ok(()),
        Some([]) => {
            Err(CompilerError::invariant("Should not have array of zero errors", None, loc))
        }
        Some(errs) if errs.len() == 1 => Err(CompilerError::invalid_js(
            &format!("Unable to unify types: {:?}", errs[0]),
            None,
            loc,
        )),
        Some(errs) => Err(CompilerError::invalid_js(
            &format!("Unable to unify types ({} errors)", errs.len()),
            None,
            loc,
        )),
    }
}

/// Error for unresolvable type variables.
pub fn unresolvable_type_variable(id: VariableId, loc: SourceLocation) -> CompilerError {
    CompilerError::invalid_js(
        &format!("Unable to resolve free variable {id:?} to a concrete type"),
        None,
        loc,
    )
}

/// Error for void in addition.
pub fn cannot_add_void(explicit: bool, loc: SourceLocation) -> CompilerError {
    if explicit {
        CompilerError::invalid_js("Undefined is not a valid operand of `+`", None, loc)
    } else {
        CompilerError::invalid_js(
            "Value may be undefined, which is not a valid operand of `+`",
            None,
            loc,
        )
    }
}

/// Error for unsupported type annotations.
pub fn unsupported_type_annotation(desc: &str, loc: SourceLocation) -> CompilerError {
    CompilerError::invalid_js(
        &format!("Type checker does not currently support type annotation: {desc}"),
        None,
        loc,
    )
}

/// Check type argument arity.
///
/// # Errors
/// Returns a `CompilerError` if the expected and actual arity don't match.
pub fn check_type_argument_arity(
    desc: &str,
    expected: usize,
    actual: usize,
    loc: SourceLocation,
) -> Result<(), CompilerError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CompilerError::invalid_js(
            &format!("Expected {desc} to have {expected} type parameters, got {actual}"),
            None,
            loc,
        ))
    }
}

/// Error when calling a non-function.
pub fn not_a_function(desc: &str, loc: SourceLocation) -> CompilerError {
    CompilerError::invalid_js(
        &format!("Cannot call {desc} because it is not a function"),
        None,
        loc,
    )
}

/// Error when calling a non-polymorphic function with type arguments.
pub fn not_a_polymorphic_function(desc: &str, loc: SourceLocation) -> CompilerError {
    CompilerError::invalid_js(
        &format!("Cannot call {desc} with type arguments because it is not a polymorphic function"),
        None,
        loc,
    )
}
