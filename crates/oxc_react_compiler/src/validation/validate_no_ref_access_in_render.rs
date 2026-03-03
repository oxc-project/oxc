/// Validate no ref access during render.
///
/// Port of `Validation/ValidateNoRefAccessInRender.ts` from the React Compiler.
///
/// Validates that a function does not access a ref value during render.
/// This includes a partial check for ref values which are accessed indirectly
/// via function expressions.
use std::sync::atomic::{AtomicU32, Ordering};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
    },
    hir::{
        HIRFunction, IdentifierId, InstructionValue, Place, ReactiveParam,
        environment::get_hook_kind_for_type,
        object_shape::{BUILT_IN_REF_VALUE_ID, BUILT_IN_USE_REF_ID, HookKind},
        types::{ObjectType, Type},
        visitors::{
            each_instruction_operand, each_instruction_value_operand, each_pattern_operand,
            each_terminal_operand,
        },
    },
};

const ERROR_DESCRIPTION: &str = "React refs are values that are not needed for rendering. \
    Refs should only be accessed outside of render, such as in event handlers or effects. \
    Accessing a ref value (the `current` property) during render can cause your component \
    not to update as expected (https://react.dev/reference/react/useRef)";

// =====================================================================================
// RefId — opaque ID for tracking individual ref instances
// =====================================================================================

static REF_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RefId(u32);

fn next_ref_id() -> RefId {
    RefId(REF_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

// =====================================================================================
// RefAccessType — the type lattice for ref tracking
// =====================================================================================

#[derive(Debug, Clone)]
struct RefFnType {
    read_ref_effect: bool,
    return_type: RefAccessType,
}

#[derive(Debug, Clone)]
enum RefAccessType {
    None,
    Nullable,
    Guard { ref_id: RefId },
    Ref { ref_id: RefId },
    RefValue { loc: Option<SourceLocation>, ref_id: Option<RefId> },
    Structure { value: Option<Box<RefAccessRefType>>, fn_type: Option<Box<RefFnType>> },
}

/// The subset of RefAccessType that is a "ref type" (Ref, RefValue, Structure).
#[derive(Debug, Clone)]
enum RefAccessRefType {
    Ref { ref_id: RefId },
    RefValue { loc: Option<SourceLocation>, ref_id: Option<RefId> },
    Structure { value: Option<Box<RefAccessRefType>>, fn_type: Option<Box<RefFnType>> },
}

impl RefAccessRefType {
    fn into_access_type(self) -> RefAccessType {
        match self {
            RefAccessRefType::Ref { ref_id } => RefAccessType::Ref { ref_id },
            RefAccessRefType::RefValue { loc, ref_id } => RefAccessType::RefValue { loc, ref_id },
            RefAccessRefType::Structure { value, fn_type } => {
                RefAccessType::Structure { value, fn_type }
            }
        }
    }
}

impl RefAccessType {
    fn into_ref_type(self) -> Option<RefAccessRefType> {
        match self {
            RefAccessType::Ref { ref_id } => Some(RefAccessRefType::Ref { ref_id }),
            RefAccessType::RefValue { loc, ref_id } => {
                Some(RefAccessRefType::RefValue { loc, ref_id })
            }
            RefAccessType::Structure { value, fn_type } => {
                Some(RefAccessRefType::Structure { value, fn_type })
            }
            _ => Option::None,
        }
    }
}

// =====================================================================================
// Type equality
// =====================================================================================

fn ty_equal(a: &RefAccessType, b: &RefAccessType) -> bool {
    match (a, b) {
        (RefAccessType::None, RefAccessType::None)
        | (RefAccessType::Nullable, RefAccessType::Nullable)
        | (RefAccessType::Ref { .. }, RefAccessType::Ref { .. }) => true,
        (RefAccessType::Guard { ref_id: a_id }, RefAccessType::Guard { ref_id: b_id }) => {
            a_id == b_id
        }
        (
            RefAccessType::RefValue { loc: a_loc, .. },
            RefAccessType::RefValue { loc: b_loc, .. },
        ) => a_loc == b_loc,
        (
            RefAccessType::Structure { value: a_val, fn_type: a_fn },
            RefAccessType::Structure { value: b_val, fn_type: b_fn },
        ) => {
            let fn_types_equal = match (a_fn, b_fn) {
                (Option::None, Option::None) => true,
                (Some(a_f), Some(b_f)) => {
                    a_f.read_ref_effect == b_f.read_ref_effect
                        && ty_equal(&a_f.return_type, &b_f.return_type)
                }
                _ => false,
            };
            let val_equal = match (a_val, b_val) {
                (Option::None, Option::None) => true,
                (Some(a_v), Some(b_v)) => {
                    ty_equal(&a_v.clone().into_access_type(), &b_v.clone().into_access_type())
                }
                _ => false,
            };
            fn_types_equal && val_equal
        }
        _ => false,
    }
}

// =====================================================================================
// Join — type lattice widening for control flow merges
// =====================================================================================

fn join_ref_access_ref_types(a: RefAccessRefType, b: RefAccessRefType) -> RefAccessRefType {
    match (a, b) {
        (RefAccessRefType::RefValue { ref_id: a_id, .. }, RefAccessRefType::RefValue { ref_id: b_id, .. })
            if a_id == b_id =>
        {
            RefAccessRefType::RefValue { loc: Option::None, ref_id: a_id }
        }
        (a @ RefAccessRefType::RefValue { .. }, _) => a,
        (_, b @ RefAccessRefType::RefValue { .. }) => b,
        (RefAccessRefType::Ref { ref_id: a_id }, RefAccessRefType::Ref { ref_id: b_id })
            if a_id == b_id =>
        {
            RefAccessRefType::Ref { ref_id: a_id }
        }
        (RefAccessRefType::Ref { .. }, _) | (_, RefAccessRefType::Ref { .. }) => {
            RefAccessRefType::Ref { ref_id: next_ref_id() }
        }
        (
            RefAccessRefType::Structure { value: a_val, fn_type: a_fn },
            RefAccessRefType::Structure { value: b_val, fn_type: b_fn },
        ) => {
            let fn_type = match (a_fn, b_fn) {
                (Option::None, b_fn) => b_fn,
                (a_fn, Option::None) => a_fn,
                (Some(a_f), Some(b_f)) => Some(Box::new(RefFnType {
                    read_ref_effect: a_f.read_ref_effect || b_f.read_ref_effect,
                    return_type: join_ref_access_types(a_f.return_type, b_f.return_type),
                })),
            };
            let value = match (a_val, b_val) {
                (Option::None, b_val) => b_val,
                (a_val, Option::None) => a_val,
                (Some(a_v), Some(b_v)) => Some(Box::new(join_ref_access_ref_types(*a_v, *b_v))),
            };
            RefAccessRefType::Structure { value, fn_type }
        }
    }
}

fn join_two(a: RefAccessType, b: RefAccessType) -> RefAccessType {
    match (a, b) {
        (RefAccessType::None, b) => b,
        (a, RefAccessType::None) => a,
        (RefAccessType::Guard { ref_id: a_id }, RefAccessType::Guard { ref_id: b_id })
            if a_id == b_id =>
        {
            RefAccessType::Guard { ref_id: a_id }
        }
        (RefAccessType::Guard { .. }, RefAccessType::Nullable | RefAccessType::Guard { .. })
        | (RefAccessType::Nullable, RefAccessType::Guard { .. }) => RefAccessType::None,
        (RefAccessType::Guard { .. }, b) => b,
        (a, RefAccessType::Guard { .. }) => a,
        (RefAccessType::Nullable, b) => b,
        (a, RefAccessType::Nullable) => a,
        (a, b) => {
            // Both are ref types (Ref, RefValue, Structure)
            let a_ref = a.into_ref_type();
            let b_ref = b.into_ref_type();
            match (a_ref, b_ref) {
                (Some(a_r), Some(b_r)) => join_ref_access_ref_types(a_r, b_r).into_access_type(),
                // Fallback — shouldn't happen if TS port is correct
                _ => RefAccessType::None,
            }
        }
    }
}

fn join_ref_access_types_vec(types: Vec<RefAccessType>) -> RefAccessType {
    types.into_iter().fold(RefAccessType::None, join_two)
}

fn join_ref_access_types(a: RefAccessType, b: RefAccessType) -> RefAccessType {
    join_two(a, b)
}

// =====================================================================================
// Env — tracks identifier → RefAccessType mappings
// =====================================================================================

struct Env {
    changed: bool,
    data: FxHashMap<IdentifierId, RefAccessType>,
    temporaries: FxHashMap<IdentifierId, Place>,
}

impl Env {
    fn new() -> Self {
        Self {
            changed: false,
            data: FxHashMap::default(),
            temporaries: FxHashMap::default(),
        }
    }

    fn lookup<'a>(&'a self, place: &'a Place) -> &'a Place {
        self.temporaries.get(&place.identifier.id).unwrap_or(place)
    }

    fn define(&mut self, place: &Place, value: Place) {
        self.temporaries.insert(place.identifier.id, value);
    }

    fn reset_changed(&mut self) {
        self.changed = false;
    }

    fn has_changed(&self) -> bool {
        self.changed
    }

    fn get(&self, key: IdentifierId) -> Option<&RefAccessType> {
        let operand_id = self
            .temporaries
            .get(&key)
            .map(|p| p.identifier.id)
            .unwrap_or(key);
        self.data.get(&operand_id)
    }

    fn set(&mut self, key: IdentifierId, value: RefAccessType) {
        let operand_id = self
            .temporaries
            .get(&key)
            .map(|p| p.identifier.id)
            .unwrap_or(key);
        let cur = self.data.get(&operand_id);
        let cur_none = cur.is_none();
        let widened_value = join_ref_access_types(
            value,
            cur.cloned().unwrap_or(RefAccessType::None),
        );
        if !(cur_none && matches!(widened_value, RefAccessType::None))
            && (cur_none || !ty_equal(cur.map_or(&RefAccessType::None, |v| v), &widened_value))
        {
            self.changed = true;
        }
        self.data.insert(operand_id, widened_value);
    }
}

// =====================================================================================
// Helper: type checks on identifiers
// =====================================================================================

fn is_use_ref_type(identifier: &crate::hir::Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == BUILT_IN_USE_REF_ID
    )
}

fn is_ref_value_type(identifier: &crate::hir::Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == BUILT_IN_REF_VALUE_ID
    )
}

fn ref_type_of_type(place: &Place) -> RefAccessType {
    if is_ref_value_type(&place.identifier) {
        RefAccessType::RefValue { loc: Option::None, ref_id: Option::None }
    } else if is_use_ref_type(&place.identifier) {
        RefAccessType::Ref { ref_id: next_ref_id() }
    } else {
        RefAccessType::None
    }
}

// =====================================================================================
// collectTemporariesSidemap — maps temporaries through LoadLocal/StoreLocal/PropertyLoad
// =====================================================================================

fn collect_temporaries_sidemap(func: &HIRFunction, env: &mut Env) {
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let lvalue = &instr.lvalue;
            match &instr.value {
                InstructionValue::LoadLocal(v) => {
                    let temp = env.lookup(&v.place).clone();
                    env.define(lvalue, temp);
                }
                InstructionValue::StoreLocal(v) => {
                    let temp = env.lookup(&v.value).clone();
                    env.define(lvalue, temp.clone());
                    env.define(&v.lvalue.place, temp);
                }
                InstructionValue::PropertyLoad(v) => {
                    if is_use_ref_type(&v.object.identifier)
                        && v.property.to_string() == "current"
                    {
                        continue;
                    }
                    let temp = env.lookup(&v.object).clone();
                    env.define(lvalue, temp);
                }
                _ => {}
            }
        }
    }
}

// =====================================================================================
// Destructure helper — recursively unwrap Structure values
// =====================================================================================

fn destructure(ty: Option<RefAccessType>) -> Option<RefAccessType> {
    match ty {
        Some(RefAccessType::Structure { value: Some(val), .. }) => {
            destructure(Some(val.into_access_type()))
        }
        other => other,
    }
}

// =====================================================================================
// Validation helpers
// =====================================================================================

fn validate_no_direct_ref_value_access(
    errors: &mut CompilerError,
    operand: &Place,
    env: &Env,
) {
    let ty = destructure(env.get(operand.identifier.id).cloned());
    if let Some(RefAccessType::RefValue { loc, .. }) = &ty {
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::Refs,
                "Cannot access refs during render".to_string(),
                Some(ERROR_DESCRIPTION.to_string()),
                Option::None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(loc.unwrap_or(operand.loc)),
                message: Some("Cannot access ref value during render".to_string()),
            }),
        );
    }
}

fn validate_no_ref_value_access(
    errors: &mut CompilerError,
    env: &Env,
    operand: &Place,
) {
    let ty = destructure(env.get(operand.identifier.id).cloned());
    match &ty {
        Some(RefAccessType::RefValue { loc, .. }) => {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Refs,
                    "Cannot access refs during render".to_string(),
                    Some(ERROR_DESCRIPTION.to_string()),
                    Option::None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(loc.unwrap_or(operand.loc)),
                    message: Some("Cannot access ref value during render".to_string()),
                }),
            );
        }
        Some(RefAccessType::Structure { fn_type: Some(fn_ty), .. })
            if fn_ty.read_ref_effect =>
        {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Refs,
                    "Cannot access refs during render".to_string(),
                    Some(ERROR_DESCRIPTION.to_string()),
                    Option::None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(operand.loc),
                    message: Some("Cannot access ref value during render".to_string()),
                }),
            );
        }
        _ => {}
    }
}

fn validate_no_ref_passed_to_function(
    errors: &mut CompilerError,
    env: &Env,
    operand: &Place,
    loc: SourceLocation,
) {
    let ty = destructure(env.get(operand.identifier.id).cloned());
    match &ty {
        Some(RefAccessType::Ref { .. }) => {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Refs,
                    "Cannot access refs during render".to_string(),
                    Some(ERROR_DESCRIPTION.to_string()),
                    Option::None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(loc),
                    message: Some(
                        "Passing a ref to a function may read its value during render".to_string(),
                    ),
                }),
            );
        }
        Some(RefAccessType::RefValue { loc: ref_loc, .. }) => {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Refs,
                    "Cannot access refs during render".to_string(),
                    Some(ERROR_DESCRIPTION.to_string()),
                    Option::None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(ref_loc.unwrap_or(loc)),
                    message: Some(
                        "Passing a ref to a function may read its value during render".to_string(),
                    ),
                }),
            );
        }
        Some(RefAccessType::Structure { fn_type: Some(fn_ty), .. })
            if fn_ty.read_ref_effect =>
        {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Refs,
                    "Cannot access refs during render".to_string(),
                    Some(ERROR_DESCRIPTION.to_string()),
                    Option::None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(loc),
                    message: Some(
                        "Passing a ref to a function may read its value during render".to_string(),
                    ),
                }),
            );
        }
        _ => {}
    }
}

fn validate_no_ref_update(
    errors: &mut CompilerError,
    env: &Env,
    operand: &Place,
    loc: SourceLocation,
) {
    let ty = destructure(env.get(operand.identifier.id).cloned());
    match &ty {
        Some(RefAccessType::Ref { .. }) => {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Refs,
                    "Cannot access refs during render".to_string(),
                    Some(ERROR_DESCRIPTION.to_string()),
                    Option::None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(loc),
                    message: Some("Cannot update ref during render".to_string()),
                }),
            );
        }
        Some(RefAccessType::RefValue { loc: ref_loc, .. }) => {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Refs,
                    "Cannot access refs during render".to_string(),
                    Some(ERROR_DESCRIPTION.to_string()),
                    Option::None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(ref_loc.unwrap_or(loc)),
                    message: Some("Cannot update ref during render".to_string()),
                }),
            );
        }
        _ => {}
    }
}

fn guard_check(errors: &mut CompilerError, operand: &Place, env: &Env) {
    if matches!(env.get(operand.identifier.id), Some(RefAccessType::Guard { .. })) {
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::Refs,
                "Cannot access refs during render".to_string(),
                Some(ERROR_DESCRIPTION.to_string()),
                Option::None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(operand.loc),
                message: Some("Cannot access ref value during render".to_string()),
            }),
        );
    }
}

// =====================================================================================
// Main entry point
// =====================================================================================

/// Validate no ref access during render.
///
/// # Errors
/// Returns a `CompilerError` if ref values are accessed during render.
pub fn validate_no_ref_access_in_render(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut env = Env::new();
    collect_temporaries_sidemap(func, &mut env);
    validate_no_ref_access_in_render_impl(func, &mut env).map(|_| ())
}

fn validate_no_ref_access_in_render_impl(
    func: &HIRFunction,
    env: &mut Env,
) -> Result<RefAccessType, CompilerError> {
    let mut return_values: Vec<Option<RefAccessType>> = Vec::new();

    // Set types for params
    for param in &func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &s.place,
        };
        let ty = ref_type_of_type(place);
        env.set(place.identifier.id, ty);
    }

    // Collect identifiers that are interpolated as JSX children
    let mut interpolated_as_jsx: FxHashSet<IdentifierId> = FxHashSet::default();
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::JsxExpression(jsx) => {
                    if let Some(children) = &jsx.children {
                        for child in children {
                            interpolated_as_jsx.insert(child.identifier.id);
                        }
                    }
                }
                InstructionValue::JsxFragment(jsx) => {
                    for child in &jsx.children {
                        interpolated_as_jsx.insert(child.identifier.id);
                    }
                }
                _ => {}
            }
        }
    }

    // Fixed-point iteration (up to 10 iterations)
    for i in 0..10 {
        if i > 0 && !env.has_changed() {
            break;
        }
        env.reset_changed();
        return_values = Vec::new();
        let mut safe_blocks: Vec<(crate::hir::BlockId, RefId)> = Vec::new();
        let mut errors = CompilerError::new();

        for (_, block) in &func.body.blocks {
            // Remove safe blocks for this block
            safe_blocks.retain(|(block_id, _)| *block_id != block.id);

            // Process phi nodes
            for phi in &block.phis {
                let types: Vec<RefAccessType> = phi
                    .operands
                    .values()
                    .map(|operand| {
                        env.get(operand.identifier.id)
                            .cloned()
                            .unwrap_or(RefAccessType::None)
                    })
                    .collect();
                env.set(phi.place.identifier.id, join_ref_access_types_vec(types));
            }

            // Process instructions
            for instr in &block.instructions {
                match &instr.value {
                    InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_) => {
                        for operand in each_instruction_value_operand(&instr.value) {
                            validate_no_direct_ref_value_access(&mut errors, operand, env);
                        }
                    }
                    InstructionValue::ComputedLoad(_) | InstructionValue::PropertyLoad(_) => {
                        if let InstructionValue::ComputedLoad(v) = &instr.value {
                            validate_no_direct_ref_value_access(&mut errors, &v.property, env);
                        }
                        let object = match &instr.value {
                            InstructionValue::ComputedLoad(v) => &v.object,
                            InstructionValue::PropertyLoad(v) => &v.object,
                            _ => unreachable!(),
                        };
                        let obj_type = env.get(object.identifier.id).cloned();
                        let lookup_type: Option<RefAccessType> = match &obj_type {
                            Some(RefAccessType::Structure { value: Some(val), .. }) => {
                                Some(val.clone().into_access_type())
                            }
                            Some(RefAccessType::Ref { ref_id }) => {
                                Some(RefAccessType::RefValue {
                                    loc: Some(instr.loc),
                                    ref_id: Some(*ref_id),
                                })
                            }
                            _ => Option::None,
                        };
                        env.set(
                            instr.lvalue.identifier.id,
                            lookup_type.unwrap_or_else(|| ref_type_of_type(&instr.lvalue)),
                        );
                    }
                    InstructionValue::TypeCastExpression(v) => {
                        env.set(
                            instr.lvalue.identifier.id,
                            env.get(v.value.identifier.id)
                                .cloned()
                                .unwrap_or_else(|| ref_type_of_type(&instr.lvalue)),
                        );
                    }
                    InstructionValue::LoadContext(v) => {
                        env.set(
                            instr.lvalue.identifier.id,
                            env.get(v.place.identifier.id)
                                .cloned()
                                .unwrap_or_else(|| ref_type_of_type(&instr.lvalue)),
                        );
                    }
                    InstructionValue::LoadLocal(v) => {
                        env.set(
                            instr.lvalue.identifier.id,
                            env.get(v.place.identifier.id)
                                .cloned()
                                .unwrap_or_else(|| ref_type_of_type(&instr.lvalue)),
                        );
                    }
                    InstructionValue::StoreContext(v) => {
                        let val_type = env
                            .get(v.value.identifier.id)
                            .cloned()
                            .unwrap_or_else(|| ref_type_of_type(&v.lvalue_place));
                        env.set(v.lvalue_place.identifier.id, val_type.clone());
                        env.set(
                            instr.lvalue.identifier.id,
                            env.get(v.value.identifier.id)
                                .cloned()
                                .unwrap_or_else(|| ref_type_of_type(&instr.lvalue)),
                        );
                    }
                    InstructionValue::StoreLocal(v) => {
                        let val_type = env
                            .get(v.value.identifier.id)
                            .cloned()
                            .unwrap_or_else(|| ref_type_of_type(&v.lvalue.place));
                        env.set(v.lvalue.place.identifier.id, val_type.clone());
                        env.set(
                            instr.lvalue.identifier.id,
                            env.get(v.value.identifier.id)
                                .cloned()
                                .unwrap_or_else(|| ref_type_of_type(&instr.lvalue)),
                        );
                    }
                    InstructionValue::Destructure(v) => {
                        let obj_type = env.get(v.value.identifier.id).cloned();
                        let lookup_type: Option<RefAccessType> = match &obj_type {
                            Some(RefAccessType::Structure { value: Some(val), .. }) => {
                                Some(val.clone().into_access_type())
                            }
                            _ => Option::None,
                        };
                        env.set(
                            instr.lvalue.identifier.id,
                            lookup_type
                                .clone()
                                .unwrap_or_else(|| ref_type_of_type(&instr.lvalue)),
                        );
                        for lval in each_pattern_operand(&v.lvalue.pattern) {
                            env.set(
                                lval.identifier.id,
                                lookup_type
                                    .clone()
                                    .unwrap_or_else(|| ref_type_of_type(lval)),
                            );
                        }
                    }
                    InstructionValue::ObjectMethod(v) => {
                        let mut return_type = RefAccessType::None;
                        let mut read_ref_effect = false;
                        match validate_no_ref_access_in_render_impl(&v.lowered_func.func, env) {
                            Ok(rt) => {
                                return_type = rt;
                            }
                            Err(_) => {
                                read_ref_effect = true;
                            }
                        }
                        env.set(
                            instr.lvalue.identifier.id,
                            RefAccessType::Structure {
                                fn_type: Some(Box::new(RefFnType {
                                    read_ref_effect,
                                    return_type,
                                })),
                                value: Option::None,
                            },
                        );
                    }
                    InstructionValue::FunctionExpression(v) => {
                        let mut return_type = RefAccessType::None;
                        let mut read_ref_effect = false;
                        match validate_no_ref_access_in_render_impl(&v.lowered_func.func, env) {
                            Ok(rt) => {
                                return_type = rt;
                            }
                            Err(_) => {
                                read_ref_effect = true;
                            }
                        }
                        env.set(
                            instr.lvalue.identifier.id,
                            RefAccessType::Structure {
                                fn_type: Some(Box::new(RefFnType {
                                    read_ref_effect,
                                    return_type,
                                })),
                                value: Option::None,
                            },
                        );
                    }
                    InstructionValue::MethodCall(_) | InstructionValue::CallExpression(_) => {
                        let callee = match &instr.value {
                            InstructionValue::CallExpression(v) => &v.callee,
                            InstructionValue::MethodCall(v) => &v.property,
                            _ => unreachable!(),
                        };
                        let hook_kind =
                            get_hook_kind_for_type(&func.env, &callee.identifier.type_);
                        let mut return_type = RefAccessType::None;
                        let fn_type = env.get(callee.identifier.id).cloned();
                        let mut did_error = false;
                        if let Some(RefAccessType::Structure {
                            fn_type: Some(fn_ty),
                            ..
                        }) = &fn_type
                        {
                            return_type = fn_ty.return_type.clone();
                            if fn_ty.read_ref_effect {
                                did_error = true;
                                errors.push_diagnostic(
                                    CompilerDiagnostic::create(
                                        ErrorCategory::Refs,
                                        "Cannot access refs during render".to_string(),
                                        Some(ERROR_DESCRIPTION.to_string()),
                                        Option::None,
                                    )
                                    .with_detail(CompilerDiagnosticDetail::Error {
                                        loc: Some(callee.loc),
                                        message: Some(
                                            "This function accesses a ref value".to_string(),
                                        ),
                                    }),
                                );
                            }
                        }

                        if !did_error {
                            let is_ref_lvalue = is_use_ref_type(&instr.lvalue.identifier);
                            for operand in each_instruction_value_operand(&instr.value) {
                                if is_ref_lvalue
                                    || matches!(
                                        hook_kind,
                                        Some(hk) if !matches!(hk, HookKind::UseState | HookKind::UseReducer)
                                    )
                                {
                                    validate_no_direct_ref_value_access(
                                        &mut errors, operand, env,
                                    );
                                } else if interpolated_as_jsx
                                    .contains(&instr.lvalue.identifier.id)
                                {
                                    validate_no_ref_value_access(&mut errors, env, operand);
                                } else {
                                    validate_no_ref_passed_to_function(
                                        &mut errors,
                                        env,
                                        operand,
                                        operand.loc,
                                    );
                                }
                            }
                        }
                        env.set(instr.lvalue.identifier.id, return_type);
                    }
                    InstructionValue::ObjectExpression(_)
                    | InstructionValue::ArrayExpression(_) => {
                        let mut types: Vec<RefAccessType> = Vec::new();
                        for operand in each_instruction_value_operand(&instr.value) {
                            validate_no_direct_ref_value_access(&mut errors, operand, env);
                            types.push(
                                env.get(operand.identifier.id)
                                    .cloned()
                                    .unwrap_or(RefAccessType::None),
                            );
                        }
                        let value = join_ref_access_types_vec(types);
                        if matches!(
                            value,
                            RefAccessType::None
                                | RefAccessType::Guard { .. }
                                | RefAccessType::Nullable
                        ) {
                            env.set(instr.lvalue.identifier.id, RefAccessType::None);
                        } else {
                            let ref_val = value.into_ref_type();
                            env.set(
                                instr.lvalue.identifier.id,
                                RefAccessType::Structure {
                                    value: ref_val.map(Box::new),
                                    fn_type: Option::None,
                                },
                            );
                        }
                    }
                    InstructionValue::PropertyDelete(_)
                    | InstructionValue::PropertyStore(_)
                    | InstructionValue::ComputedDelete(_)
                    | InstructionValue::ComputedStore(_) => {
                        let object = match &instr.value {
                            InstructionValue::PropertyDelete(v) => &v.object,
                            InstructionValue::PropertyStore(v) => &v.object,
                            InstructionValue::ComputedDelete(v) => &v.object,
                            InstructionValue::ComputedStore(v) => &v.object,
                            _ => unreachable!(),
                        };
                        let target = env.get(object.identifier.id).cloned();
                        let mut safe_found: Option<usize> = Option::None;
                        if let InstructionValue::PropertyStore(_) = &instr.value {
                            if let Some(RefAccessType::Ref { ref_id }) = &target {
                                safe_found = safe_blocks
                                    .iter()
                                    .position(|(_, r)| r == ref_id);
                            }
                        }
                        if let Some(idx) = safe_found {
                            safe_blocks.remove(idx);
                        } else {
                            validate_no_ref_update(&mut errors, env, object, instr.loc);
                        }
                        // Validate computed property
                        match &instr.value {
                            InstructionValue::ComputedDelete(v) => {
                                validate_no_ref_value_access(&mut errors, env, &v.property);
                            }
                            InstructionValue::ComputedStore(v) => {
                                validate_no_ref_value_access(&mut errors, env, &v.property);
                            }
                            _ => {}
                        }
                        // Validate stored value and update object type
                        match &instr.value {
                            InstructionValue::ComputedStore(v) => {
                                validate_no_direct_ref_value_access(&mut errors, &v.value, env);
                                let val_type = env.get(v.value.identifier.id).cloned();
                                if let Some(RefAccessType::Structure { .. }) = &val_type {
                                    let mut object_type = val_type.clone().unwrap_or(RefAccessType::None);
                                    if let Some(t) = &target {
                                        object_type =
                                            join_ref_access_types(object_type, t.clone());
                                    }
                                    env.set(object.identifier.id, object_type);
                                }
                            }
                            InstructionValue::PropertyStore(v) => {
                                validate_no_direct_ref_value_access(&mut errors, &v.value, env);
                                let val_type = env.get(v.value.identifier.id).cloned();
                                if let Some(RefAccessType::Structure { .. }) = &val_type {
                                    let mut object_type = val_type.clone().unwrap_or(RefAccessType::None);
                                    if let Some(t) = &target {
                                        object_type =
                                            join_ref_access_types(object_type, t.clone());
                                    }
                                    env.set(object.identifier.id, object_type);
                                }
                            }
                            _ => {}
                        }
                    }
                    InstructionValue::StartMemoize(_) | InstructionValue::FinishMemoize(_) => {}
                    InstructionValue::LoadGlobal(v) => {
                        if v.binding.name() == "undefined" {
                            env.set(instr.lvalue.identifier.id, RefAccessType::Nullable);
                        }
                    }
                    InstructionValue::Primitive(v) => {
                        use crate::hir::PrimitiveValueKind;
                        if matches!(v.value, PrimitiveValueKind::Null | PrimitiveValueKind::Undefined)
                        {
                            env.set(instr.lvalue.identifier.id, RefAccessType::Nullable);
                        }
                    }
                    InstructionValue::UnaryExpression(v) => {
                        if v.operator == oxc_syntax::operator::UnaryOperator::LogicalNot {
                            let value = env.get(v.value.identifier.id).cloned();
                            let ref_id = match &value {
                                Some(RefAccessType::RefValue {
                                    ref_id: Some(rid), ..
                                }) => Some(*rid),
                                _ => Option::None,
                            };
                            if let Some(rid) = ref_id {
                                env.set(
                                    instr.lvalue.identifier.id,
                                    RefAccessType::Guard { ref_id: rid },
                                );
                                errors.push_diagnostic(
                                    CompilerDiagnostic::create(
                                        ErrorCategory::Refs,
                                        "Cannot access refs during render".to_string(),
                                        Some(ERROR_DESCRIPTION.to_string()),
                                        Option::None,
                                    )
                                    .with_detail(CompilerDiagnosticDetail::Error {
                                        loc: Some(v.value.loc),
                                        message: Some(
                                            "Cannot access ref value during render".to_string(),
                                        ),
                                    })
                                    .with_detail(CompilerDiagnosticDetail::Hint {
                                        message: "To initialize a ref only once, check that the ref is null with the pattern `if (ref.current == null) { ref.current = ... }`".to_string(),
                                    }),
                                );
                                // Skip the default operand check (matches TS `break`)
                                // Fall through to guard_check + type override below
                                // (handled after the match)
                                // We need to continue to the guard check, so we don't use `continue` here.
                            } else {
                                validate_no_ref_value_access(&mut errors, env, &v.value);
                            }
                        } else {
                            validate_no_ref_value_access(&mut errors, env, &v.value);
                        }
                    }
                    InstructionValue::BinaryExpression(v) => {
                        let left = env.get(v.left.identifier.id).cloned();
                        let right = env.get(v.right.identifier.id).cloned();
                        let mut nullish = false;
                        let mut ref_id: Option<RefId> = Option::None;
                        if let Some(RefAccessType::RefValue {
                            ref_id: Some(rid), ..
                        }) = &left
                        {
                            ref_id = Some(*rid);
                        } else if let Some(RefAccessType::RefValue {
                            ref_id: Some(rid), ..
                        }) = &right
                        {
                            ref_id = Some(*rid);
                        }

                        if matches!(&left, Some(RefAccessType::Nullable)) {
                            nullish = true;
                        } else if matches!(&right, Some(RefAccessType::Nullable)) {
                            nullish = true;
                        }

                        if ref_id.is_some() && nullish {
                            env.set(
                                instr.lvalue.identifier.id,
                                RefAccessType::Guard { ref_id: ref_id.unwrap() },
                            );
                        } else {
                            for operand in each_instruction_value_operand(&instr.value) {
                                validate_no_ref_value_access(&mut errors, env, operand);
                            }
                        }
                    }
                    _ => {
                        for operand in each_instruction_value_operand(&instr.value) {
                            validate_no_ref_value_access(&mut errors, env, operand);
                        }
                    }
                }

                // Guard values are derived from ref.current, so they can only be used
                // in if statement targets
                for operand in each_instruction_operand(instr) {
                    guard_check(&mut errors, operand, env);
                }

                // Ensure useRef-typed lvalues are tracked as Ref
                if is_use_ref_type(&instr.lvalue.identifier)
                    && !matches!(env.get(instr.lvalue.identifier.id), Some(RefAccessType::Ref { .. }))
                {
                    let cur = env
                        .get(instr.lvalue.identifier.id)
                        .cloned()
                        .unwrap_or(RefAccessType::None);
                    env.set(
                        instr.lvalue.identifier.id,
                        join_ref_access_types(cur, RefAccessType::Ref { ref_id: next_ref_id() }),
                    );
                }
                // Ensure refValue-typed lvalues are tracked as RefValue
                if is_ref_value_type(&instr.lvalue.identifier)
                    && !matches!(
                        env.get(instr.lvalue.identifier.id),
                        Some(RefAccessType::RefValue { .. })
                    )
                {
                    let cur = env
                        .get(instr.lvalue.identifier.id)
                        .cloned()
                        .unwrap_or(RefAccessType::None);
                    env.set(
                        instr.lvalue.identifier.id,
                        join_ref_access_types(
                            cur,
                            RefAccessType::RefValue {
                                loc: Some(instr.loc),
                                ref_id: Option::None,
                            },
                        ),
                    );
                }
            }

            // Terminal processing
            if let crate::hir::Terminal::If(if_terminal) = &block.terminal {
                let test = env.get(if_terminal.test.identifier.id).cloned();
                if let Some(RefAccessType::Guard { ref_id }) = &test {
                    if !safe_blocks.iter().any(|(_, r)| r == ref_id) {
                        safe_blocks.push((if_terminal.fallthrough, *ref_id));
                    }
                }
            }

            for operand in each_terminal_operand(&block.terminal) {
                if !matches!(block.terminal, crate::hir::Terminal::Return(_)) {
                    validate_no_ref_value_access(&mut errors, env, operand);
                    if !matches!(block.terminal, crate::hir::Terminal::If(_)) {
                        guard_check(&mut errors, operand, env);
                    }
                } else {
                    // Allow functions containing refs to be returned, but not direct ref values
                    validate_no_direct_ref_value_access(&mut errors, operand, env);
                    guard_check(&mut errors, operand, env);
                    return_values.push(env.get(operand.identifier.id).cloned());
                }
            }
        }

        if errors.has_any_errors() {
            return Err(errors);
        }
    }

    // Build the return type from all return values
    let filtered: Vec<RefAccessType> = return_values.into_iter().flatten().collect();
    Ok(join_ref_access_types_vec(filtered))
}
