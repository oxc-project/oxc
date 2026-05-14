/// Validate no capitalized function calls (they should be JSX components).
///
/// Port of `Validation/ValidateNoCapitalizedCalls.ts` from the React Compiler.
///
/// Capitalized functions are reserved for components, which must be invoked with JSX.
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{HIRFunction, IdentifierId, Instruction, InstructionValue, environment::Environment},
    validation::dispatcher::InstructionVisitor,
};

/// Validate that no capitalized functions are called directly.
///
/// # Errors
/// Returns a `CompilerError` if capitalized call violations are found.
pub fn validate_no_capitalized_calls(func: &HIRFunction) -> Result<(), CompilerError> {
    crate::validation::dispatcher::dispatch_instruction_visitors(
        func,
        vec![Box::new(ValidateNoCapitalizedCalls::default())],
    )
}

/// `InstructionVisitor` impl for `validate_no_capitalized_calls`.
///
/// Maintains two function-local maps populated as the dispatcher walks the
/// HIR: identifier ids of capitalized globals (from `LoadGlobal`) and of
/// capitalized property loads (from `PropertyLoad`). The dispatcher walks
/// every block in iteration order, and the original validator likewise
/// retained both maps across block boundaries — no per-block reset.
#[derive(Default)]
pub struct ValidateNoCapitalizedCalls {
    errors: CompilerError,
    capital_load_globals: FxHashMap<IdentifierId, String>,
    capitalized_properties: FxHashMap<IdentifierId, String>,
}

const REASON: &str =
    "Capitalized functions are reserved for components, which must be invoked with JSX";

impl InstructionVisitor for ValidateNoCapitalizedCalls {
    fn visit_instruction(&mut self, env: &Environment, instr: &Instruction) {
        match &instr.value {
            InstructionValue::LoadGlobal(v) => {
                let name = v.binding.name().to_string();
                if !name.is_empty()
                    && name.starts_with(|c: char| c.is_ascii_uppercase())
                    && !name.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
                    && !is_allowed(env, &name)
                {
                    self.capital_load_globals.insert(instr.lvalue.identifier.id, name);
                }
            }
            InstructionValue::CallExpression(v) => {
                if let Some(callee_name) = self.capital_load_globals.get(&v.callee.identifier.id) {
                    self.errors.push_diagnostic(
                        CompilerDiagnostic::create(
                            ErrorCategory::CapitalizedCalls,
                            REASON.to_string(),
                            Some(format!("{callee_name} may be a component")),
                            None,
                        )
                        .with_detail(CompilerDiagnosticDetail::Error {
                            loc: Some(v.loc),
                            message: Some(format!("{callee_name} called as function")),
                        }),
                    );
                }
            }
            InstructionValue::PropertyLoad(v) => {
                let prop_str = v.property.to_string();
                if prop_str.starts_with(|c: char| c.is_ascii_uppercase()) {
                    self.capitalized_properties.insert(instr.lvalue.identifier.id, prop_str);
                }
            }
            InstructionValue::MethodCall(v) => {
                if let Some(prop_name) = self.capitalized_properties.get(&v.property.identifier.id)
                {
                    self.errors.push_diagnostic(
                        CompilerDiagnostic::create(
                            ErrorCategory::CapitalizedCalls,
                            REASON.to_string(),
                            Some(format!("{prop_name} may be a component")),
                            None,
                        )
                        .with_detail(CompilerDiagnosticDetail::Error {
                            loc: Some(v.loc),
                            message: Some(format!("{prop_name} called as method")),
                        }),
                    );
                }
            }
            _ => {}
        }
    }

    fn finish(self: Box<Self>, _env: &Environment) -> Result<(), CompilerError> {
        self.errors.into_result()
    }
}

/// Mirrors the TS allowlist:
/// `new Set([...DEFAULT_GLOBALS.keys(), ...(envConfig.validateNoCapitalizedCalls ?? [])])`.
fn is_allowed(env: &Environment, name: &str) -> bool {
    if env.globals().contains_key(name) {
        return true;
    }
    let user_allowlist = env.config().validate_no_capitalized_calls.as_deref().unwrap_or(&[]);
    user_allowlist.iter().any(|allowed| allowed == name)
}
