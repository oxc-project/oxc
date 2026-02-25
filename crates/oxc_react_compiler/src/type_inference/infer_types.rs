/// Type inference pass for the React Compiler HIR.
///
/// Port of `TypeInference/InferTypes.ts` from the React Compiler.
///
/// Infers types for HIR identifiers using a union-find based unification approach.
/// Generates type equations from instructions and unifies them to determine types.
use rustc_hash::FxHashMap;

use crate::hir::{
    HIRFunction, Instruction, InstructionKind, InstructionValue, ReactFunctionType, ReactiveParam,
    environment::Environment,
    globals::Global,
    object_shape::{
        BUILT_IN_ARRAY_ID, BUILT_IN_FUNCTION_ID, BUILT_IN_JSX_ID, BUILT_IN_OBJECT_ID,
        BUILT_IN_PROPS_ID, BUILT_IN_USE_REF_ID,
    },
    types::{
        FunctionType, ObjectType, PropType, PropertyLiteral, PropertyName, Type, TypeId, make_type,
        type_equals,
    },
};
use oxc_syntax::operator::BinaryOperator;

/// A type equation: left = right.
struct TypeEquation {
    left: Type,
    right: Type,
}

/// Union-find based type unifier.
struct Unifier<'a> {
    substitutions: FxHashMap<TypeId, Type>,
    env: &'a Environment,
}

impl<'a> Unifier<'a> {
    fn new(env: &'a Environment) -> Self {
        Self { substitutions: FxHashMap::default(), env }
    }

    /// Unify two types, recording any substitutions needed.
    fn unify(&mut self, left: Type, right: Type) {
        let left = self.resolve(left);
        let right = self.resolve(right);

        if type_equals(&left, &right) {
            return;
        }

        // Handle Property types: resolve the object type and look up the property
        if let Type::Property(prop) = &right {
            let object_type = self.get(prop.object_type.clone());
            if let PropertyName::Literal { value } = &prop.property_name {
                let property_str = value.to_string();
                let prop_type = self.env.get_property_type(&object_type, &property_str);
                if let Some(prop_type) = prop_type {
                    self.unify(left, prop_type);
                    return;
                }
            }
            // If we can't resolve the property, unify left with a fresh type var
            return;
        }
        if let Type::Property(prop) = &left {
            let object_type = self.get(prop.object_type.clone());
            if let PropertyName::Literal { value } = &prop.property_name {
                let property_str = value.to_string();
                if let Some(prop_type) = self.env.get_property_type(&object_type, &property_str) {
                    self.unify(prop_type, right);
                    return;
                }
            }
            return;
        }

        match (&left, &right) {
            (Type::Var(id), _) => {
                self.substitutions.insert(*id, right);
            }
            (_, Type::Var(id)) => {
                self.substitutions.insert(*id, left);
            }
            (Type::Phi(phi_a), _) => {
                for operand in &phi_a.operands {
                    self.unify(operand.clone(), right.clone());
                }
            }
            (_, Type::Phi(phi_b)) => {
                for operand in &phi_b.operands {
                    self.unify(left.clone(), operand.clone());
                }
            }
            (Type::Function(fa), Type::Function(fb)) => {
                self.unify(*fa.return_type.clone(), *fb.return_type.clone());
            }
            _ => {
                // Types don't unify — this is fine, the type system is optimistic
            }
        }
    }

    /// Resolve a type by following substitutions.
    fn resolve(&self, ty: Type) -> Type {
        match &ty {
            Type::Var(id) => match self.substitutions.get(id) {
                Some(resolved) => self.resolve(resolved.clone()),
                None => ty,
            },
            _ => ty,
        }
    }

    /// Get the fully resolved type.
    fn get(&self, ty: Type) -> Type {
        self.resolve(ty)
    }
}

/// A resolved set of type substitutions (no env reference needed).
struct ResolvedTypes {
    substitutions: FxHashMap<TypeId, Type>,
}

impl ResolvedTypes {
    fn resolve(&self, ty: Type) -> Type {
        match &ty {
            Type::Var(id) => match self.substitutions.get(id) {
                Some(resolved) => self.resolve(resolved.clone()),
                None => ty,
            },
            _ => ty,
        }
    }

    fn get(&self, ty: Type) -> Type {
        self.resolve(ty)
    }
}

/// Run type inference on the given function.
pub fn infer_types(func: &mut HIRFunction) {
    let mut unifier = Unifier::new(&func.env);

    // Generate type equations
    let equations = generate(func);
    for eq in equations {
        unifier.unify(eq.left, eq.right);
    }

    // Extract substitutions so we can release the env borrow
    let resolved = ResolvedTypes { substitutions: unifier.substitutions };

    // Apply resolved types back to the function
    apply(func, &resolved);
}

fn apply(func: &mut HIRFunction, unifier: &ResolvedTypes) {
    use crate::hir::Place;

    // Helper to resolve a place's type
    fn resolve_place(place: &mut Place, unifier: &ResolvedTypes) {
        place.identifier.type_ = unifier.get(place.identifier.type_.clone());
    }

    // Apply to params (these are not in block instructions, so we need
    // to resolve them explicitly).
    for param in &mut func.params {
        if let ReactiveParam::Place(p) = param {
            resolve_place(p, unifier);
        }
    }

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        if let Some(block) = func.body.blocks.get_mut(&block_id) {
            // Apply to phi nodes (TS: phi.place.identifier.type = unifier.get(...))
            for phi in &mut block.phis {
                resolve_place(&mut phi.place, unifier);
            }

            for instr in &mut block.instructions {
                // Apply to lvalue
                resolve_place(&mut instr.lvalue, unifier);

                // Apply to all operand places within the instruction value
                apply_to_instruction_value(&mut instr.value, unifier);
            }
        }
    }

    // Apply to context operands.
    // In TS, context Place objects share identity with instruction lvalue/operand
    // Place objects, so type updates propagate automatically. In Rust, these are
    // separate clones, so we must resolve them explicitly.
    for ctx_place in &mut func.context {
        resolve_place(ctx_place, unifier);
    }

    // Apply to returns
    resolve_place(&mut func.returns, unifier);
}

/// Apply resolved types to all places within an instruction value.
fn apply_to_instruction_value(value: &mut InstructionValue, unifier: &ResolvedTypes) {
    use crate::hir::{CallArg, Place};

    fn resolve_place(place: &mut Place, unifier: &ResolvedTypes) {
        place.identifier.type_ = unifier.get(place.identifier.type_.clone());
    }

    fn resolve_args(args: &mut [CallArg], unifier: &ResolvedTypes) {
        for arg in args.iter_mut() {
            match arg {
                CallArg::Place(p) => resolve_place(p, unifier),
                CallArg::Spread(s) => resolve_place(&mut s.place, unifier),
            }
        }
    }

    match value {
        InstructionValue::CallExpression(v) => {
            resolve_place(&mut v.callee, unifier);
            resolve_args(&mut v.args, unifier);
        }
        InstructionValue::MethodCall(v) => {
            resolve_place(&mut v.receiver, unifier);
            resolve_place(&mut v.property, unifier);
            resolve_args(&mut v.args, unifier);
        }
        InstructionValue::NewExpression(v) => {
            resolve_place(&mut v.callee, unifier);
            resolve_args(&mut v.args, unifier);
        }
        InstructionValue::PropertyLoad(v) => {
            resolve_place(&mut v.object, unifier);
        }
        InstructionValue::PropertyStore(v) => {
            resolve_place(&mut v.object, unifier);
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::ComputedLoad(v) => {
            resolve_place(&mut v.object, unifier);
        }
        InstructionValue::ComputedStore(v) => {
            resolve_place(&mut v.object, unifier);
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::LoadLocal(v) => {
            resolve_place(&mut v.place, unifier);
        }
        InstructionValue::LoadContext(v) => {
            resolve_place(&mut v.place, unifier);
        }
        InstructionValue::StoreLocal(v) => {
            resolve_place(&mut v.lvalue.place, unifier);
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::StoreContext(v) => {
            resolve_place(&mut v.lvalue_place, unifier);
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::FunctionExpression(v) => {
            apply(&mut v.lowered_func.func, unifier);
        }
        InstructionValue::ObjectMethod(v) => {
            apply(&mut v.lowered_func.func, unifier);
        }
        InstructionValue::Destructure(v) => {
            resolve_place(&mut v.value, unifier);
            match &mut v.lvalue.pattern {
                crate::hir::Pattern::Array(arr) => {
                    for item in &mut arr.items {
                        match item {
                            crate::hir::ArrayPatternElement::Place(p) => {
                                resolve_place(p, unifier);
                            }
                            crate::hir::ArrayPatternElement::Spread(s) => {
                                resolve_place(&mut s.place, unifier);
                            }
                            crate::hir::ArrayPatternElement::Hole => {}
                        }
                    }
                }
                crate::hir::Pattern::Object(obj) => {
                    for prop in &mut obj.properties {
                        match prop {
                            crate::hir::ObjectPatternProperty::Property(p) => {
                                resolve_place(&mut p.place, unifier);
                            }
                            crate::hir::ObjectPatternProperty::Spread(s) => {
                                resolve_place(&mut s.place, unifier);
                            }
                        }
                    }
                }
            }
        }
        // Other instruction values don't have nested places that need resolution,
        // or their places don't participate in function signature resolution.
        _ => {}
    }
}

fn generate(func: &HIRFunction) -> Vec<TypeEquation> {
    let mut equations = Vec::new();

    // Match TS InferTypes.ts: for Component functions, type the first param as
    // Props and the second param as BuiltInUseRefId (for forwardRef components).
    // This ensures that `ref` parameters in components like `function Foo(props, ref) {}`
    // are recognized as ref-like mutable types, allowing mutation in effects.
    if func.fn_type == ReactFunctionType::Component {
        let mut params_iter = func.params.iter();
        // First param → BuiltInPropsId
        if let Some(first_param) = params_iter.next() {
            if let ReactiveParam::Place(p) = first_param {
                equations.push(TypeEquation {
                    left: p.identifier.type_.clone(),
                    right: Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_PROPS_ID.to_string()),
                    }),
                });
            }
        }
        // Second param → BuiltInUseRefId (for forwardRef)
        if let Some(second_param) = params_iter.next() {
            if let ReactiveParam::Place(p) = second_param {
                equations.push(TypeEquation {
                    left: p.identifier.type_.clone(),
                    right: Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_USE_REF_ID.to_string()),
                    }),
                });
            }
        }
    }

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            generate_instruction_equations(instr, &func.env, &mut equations);
        }
    }

    equations
}

fn generate_instruction_equations(
    instr: &Instruction,
    env: &Environment,
    equations: &mut Vec<TypeEquation>,
) {
    let lvalue_type = instr.lvalue.identifier.type_.clone();

    match &instr.value {
        InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::UnaryExpression(_)
        | InstructionValue::TemplateLiteral(_) => {
            equations.push(TypeEquation { left: lvalue_type, right: Type::Primitive });
        }
        InstructionValue::BinaryExpression(v) => {
            if is_primitive_binary_op(v.operator) {
                equations.push(TypeEquation { left: lvalue_type, right: Type::Primitive });
            }
        }
        InstructionValue::LoadLocal(v) => {
            equations
                .push(TypeEquation { left: lvalue_type, right: v.place.identifier.type_.clone() });
        }
        InstructionValue::LoadContext(v) => {
            equations
                .push(TypeEquation { left: lvalue_type, right: v.place.identifier.type_.clone() });
        }
        InstructionValue::StoreLocal(v) => {
            equations.push(TypeEquation {
                left: v.lvalue.place.identifier.type_.clone(),
                right: v.value.identifier.type_.clone(),
            });
        }
        InstructionValue::DeclareLocal(v) => {
            // For parameters, the type is set by the function declaration
            if v.lvalue.kind == InstructionKind::Const || v.lvalue.kind == InstructionKind::Let {
                // DeclareLocal introduces a new variable, its type is unconstrained
            }
        }
        InstructionValue::ObjectExpression(_) => {
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Object(ObjectType { shape_id: Some(BUILT_IN_OBJECT_ID.to_string()) }),
            });
        }
        InstructionValue::ArrayExpression(_) => {
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) }),
            });
        }
        InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_) => {
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Object(ObjectType { shape_id: Some(BUILT_IN_JSX_ID.to_string()) }),
            });
        }
        InstructionValue::FunctionExpression(_) | InstructionValue::ObjectMethod(_) => {
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Function(FunctionType {
                    shape_id: Some(BUILT_IN_FUNCTION_ID.to_string()),
                    return_type: Box::new(make_type()),
                    is_constructor: false,
                }),
            });
        }
        InstructionValue::RegExpLiteral(_) => {
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Object(ObjectType { shape_id: None }),
            });
        }
        InstructionValue::TypeCastExpression(v) => {
            equations.push(TypeEquation { left: lvalue_type, right: v.type_.clone() });
        }
        InstructionValue::LoadGlobal(load) => {
            // Use the environment to resolve the type of this global
            if let Some(global) = env.get_global_declaration(&load.binding) {
                let global_type = Global::to_type(&global);
                equations.push(TypeEquation { left: lvalue_type, right: global_type });
            }
        }
        InstructionValue::PropertyLoad(load) => {
            // Create a lazy Property type that will be resolved during unification,
            // allowing type propagation through property accesses like React.useState.
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Property(Box::new(PropType {
                    object_type: load.object.identifier.type_.clone(),
                    object_name: String::new(),
                    property_name: PropertyName::Literal {
                        value: PropertyLiteral::String(load.property.to_string()),
                    },
                })),
            });
        }
        InstructionValue::CallExpression(v) => {
            // The callee must be a function; its return type equals the lvalue type.
            let return_type = make_type();
            equations.push(TypeEquation {
                left: v.callee.identifier.type_.clone(),
                right: Type::Function(FunctionType {
                    shape_id: None,
                    return_type: Box::new(return_type.clone()),
                    is_constructor: false,
                }),
            });
            equations.push(TypeEquation { left: lvalue_type, right: return_type });
        }
        InstructionValue::MethodCall(v) => {
            // The method (property) must be a function; its return type equals the lvalue type.
            let return_type = make_type();
            equations.push(TypeEquation {
                left: v.property.identifier.type_.clone(),
                right: Type::Function(FunctionType {
                    shape_id: None,
                    return_type: Box::new(return_type.clone()),
                    is_constructor: false,
                }),
            });
            equations.push(TypeEquation { left: lvalue_type, right: return_type });
        }
        InstructionValue::Destructure(v) => {
            let source_type = v.value.identifier.type_.clone();
            match &v.lvalue.pattern {
                crate::hir::Pattern::Array(arr) => {
                    for (i, item) in arr.items.iter().enumerate() {
                        match item {
                            crate::hir::ArrayPatternElement::Place(place) => {
                                // Array element type = Property(source, "i")
                                equations.push(TypeEquation {
                                    left: place.identifier.type_.clone(),
                                    right: Type::Property(Box::new(PropType {
                                        object_type: source_type.clone(),
                                        object_name: String::new(),
                                        property_name: PropertyName::Literal {
                                            value: PropertyLiteral::String(i.to_string()),
                                        },
                                    })),
                                });
                            }
                            crate::hir::ArrayPatternElement::Spread(spread) => {
                                // Spread element always produces an array
                                equations.push(TypeEquation {
                                    left: spread.place.identifier.type_.clone(),
                                    right: Type::Object(ObjectType {
                                        shape_id: Some(BUILT_IN_ARRAY_ID.to_string()),
                                    }),
                                });
                            }
                            crate::hir::ArrayPatternElement::Hole => {}
                        }
                    }
                }
                crate::hir::Pattern::Object(obj) => {
                    for prop in &obj.properties {
                        match prop {
                            crate::hir::ObjectPatternProperty::Property(p) => {
                                let key_name = match &p.key {
                                    crate::hir::ObjectPropertyKey::Identifier(name)
                                    | crate::hir::ObjectPropertyKey::String(name) => {
                                        Some(name.clone())
                                    }
                                    _ => None,
                                };
                                if let Some(name) = key_name {
                                    equations.push(TypeEquation {
                                        left: p.place.identifier.type_.clone(),
                                        right: Type::Property(Box::new(PropType {
                                            object_type: source_type.clone(),
                                            object_name: String::new(),
                                            property_name: PropertyName::Literal {
                                                value: PropertyLiteral::String(name),
                                            },
                                        })),
                                    });
                                }
                            }
                            crate::hir::ObjectPatternProperty::Spread(_) => {
                                // Rest element in object pattern - type is unconstrained
                            }
                        }
                    }
                }
            }
        }
        InstructionValue::StoreContext(v) => {
            equations.push(TypeEquation {
                left: v.lvalue_place.identifier.type_.clone(),
                right: v.value.identifier.type_.clone(),
            });
        }
        _ => {
            // Many instruction values don't produce enough info for type equations
        }
    }
}

fn is_primitive_binary_op(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Multiplication
            | BinaryOperator::Exponential
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftLeft
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::LessEqualThan
    )
}
