/// Type inference pass for the React Compiler HIR.
///
/// Port of `TypeInference/InferTypes.ts` from the React Compiler.
///
/// Infers types for HIR identifiers using a union-find based unification approach.
/// Generates type equations from instructions and unifies them to determine types.
use rustc_hash::FxHashMap;

use crate::hir::{
    HIRFunction, IdentifierId, IdentifierName, Instruction, InstructionKind, InstructionValue,
    ReactFunctionType, ReactiveParam,
    environment::Environment,
    globals::Global,
    object_shape::{
        BUILT_IN_ARRAY_ID, BUILT_IN_FUNCTION_ID, BUILT_IN_JSX_ID, BUILT_IN_OBJECT_ID,
        BUILT_IN_PROPS_ID, BUILT_IN_REF_VALUE_ID, BUILT_IN_SET_STATE_ID, BUILT_IN_USE_REF_ID,
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
            if is_ref_like_name(prop) {
                self.unify(
                    prop.object_type.clone(),
                    Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_REF_ID.to_string()) }),
                );
                self.unify(
                    left,
                    Type::Object(ObjectType { shape_id: Some(BUILT_IN_REF_VALUE_ID.to_string()) }),
                );
                return;
            }
            let object_type = self.get(prop.object_type.clone());
            let prop_type = match &prop.property_name {
                PropertyName::Literal { value } => {
                    self.env.get_property_type(&object_type, &value.to_string())
                }
                PropertyName::Computed { .. } => {
                    self.env.get_fallthrough_property_type(&object_type)
                }
            };
            if let Some(prop_type) = prop_type {
                self.unify(left, prop_type);
            }
            return;
        }
        if let Type::Property(prop) = &left {
            if is_ref_like_name(prop) {
                self.unify(
                    prop.object_type.clone(),
                    Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_REF_ID.to_string()) }),
                );
                self.unify(
                    Type::Object(ObjectType { shape_id: Some(BUILT_IN_REF_VALUE_ID.to_string()) }),
                    right,
                );
                return;
            }
            let object_type = self.get(prop.object_type.clone());
            let prop_type = match &prop.property_name {
                PropertyName::Literal { value } => {
                    self.env.get_property_type(&object_type, &value.to_string())
                }
                PropertyName::Computed { .. } => {
                    self.env.get_fallthrough_property_type(&object_type)
                }
            };
            if let Some(prop_type) = prop_type {
                self.unify(prop_type, right);
            }
            return;
        }

        // Handle Phi types: attempt to collapse the phi to a consensus concrete type.
        //
        // Port of TS InferTypes.ts unify() Phi handling (lines 607-633):
        //   if (type.kind === 'Phi') {
        //     let candidateType = null;
        //     for (const operand of type.operands) {
        //       const resolved = this.get(operand);
        //       if (candidateType === null) { candidateType = resolved; }
        //       else if (!typeEquals(resolved, candidateType)) {
        //         const unionType = tryUnionTypes(resolved, candidateType);
        //         if (unionType === null) { candidateType = null; break; }
        //         else { candidateType = unionType; }
        //       }
        //     }
        //     if (candidateType !== null) { this.unify(v, candidateType); return; }
        //   }
        //
        // Key insight: ONLY unify the phi result (left) with the consensus type if ALL
        // phi operands resolve to the SAME concrete type. If operands disagree (e.g.,
        // one is a TypeVar, another is Primitive), leave the result as unresolved.
        // This prevents incorrect backpropagation where one branch's known type
        // incorrectly sets the type of variables in the other branch.
        //
        // Important: we handle (Type::Var, Phi) here before the (Type::Var, _) match arm
        // to prevent storing Phi types in substitutions. If we stored `x → Phi([...])`,
        // subsequent resolves of TypeVars chaining to x would return a Phi, causing
        // (Type::Phi, _) match arm recursion that can overflow the stack.
        if let (Type::Var(_), Type::Phi(phi_b)) = (&left, right.clone()) {
            let consensus = self.phi_consensus_type(&phi_b.operands);
            if let Some(consensus_type) = consensus {
                self.unify(left, consensus_type);
            }
            // If no consensus, leave the TypeVar unresolved (don't store Phi).
            return;
        }
        // Symmetrical case: (Phi, Var) - each phi operand unifies with the var.
        if let (Type::Phi(phi_a), Type::Var(_)) = (left.clone(), &right) {
            let consensus = self.phi_consensus_type(&phi_a.operands);
            if let Some(consensus_type) = consensus {
                self.unify(consensus_type, right);
            }
            return;
        }
        // (Phi, non-Var): each phi operand is unified with the right type.
        if let Type::Phi(phi_a) = left.clone() {
            for operand in &phi_a.operands {
                self.unify(operand.clone(), right.clone());
            }
            return;
        }
        // (non-Var, Phi): unify left with consensus, or with each operand.
        if let Type::Phi(phi_b) = right.clone() {
            let consensus = self.phi_consensus_type(&phi_b.operands);
            if let Some(consensus_type) = consensus {
                self.unify(left, consensus_type);
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
            (Type::Function(fa), Type::Function(fb)) => {
                self.unify(*fa.return_type.clone(), *fb.return_type.clone());
            }
            _ => {
                // Types don't unify — this is fine, the type system is optimistic
            }
        }
    }

    /// Try to find a consensus type from a list of phi operands.
    ///
    /// Port of the TS InferTypes.ts phi consensus logic (lines 613-627):
    ///   let candidateType = null;
    ///   for (const operand of type.operands) {
    ///     const resolved = this.get(operand);
    ///     if (candidateType === null) { candidateType = resolved; }
    ///     else if (!typeEquals(resolved, candidateType)) {
    ///       const unionType = tryUnionTypes(resolved, candidateType);
    ///       if (unionType === null) { candidateType = null; break; }
    ///       else { candidateType = unionType; }
    ///     }
    ///   }
    ///   if (candidateType !== null) { this.unify(v, candidateType); return; }
    ///
    /// Key behavior: DOES NOT skip TypeVars. If operand A resolves to TypeVar(x)
    /// and operand B resolves to `Primitive`, they disagree → no consensus.
    /// Only returns a type if ALL operands resolve to the SAME type
    /// (including possibly the same TypeVar, which happens for self-referential phis).
    fn phi_consensus_type(&self, operands: &[Type]) -> Option<Type> {
        let mut candidate: Option<Type> = None;
        for operand in operands {
            let resolved = self.resolve(operand.clone());
            match &candidate {
                None => {
                    candidate = Some(resolved);
                }
                Some(prev) => {
                    if !type_equals(&resolved, prev) {
                        // Disagreement: no consensus possible.
                        // (We don't implement tryUnionTypes for simplicity, since
                        // it only handles BuiltInMixedReadonlyId which is rarely
                        // relevant for the ref-type propagation use case.)
                        return None;
                    }
                }
            }
        }
        candidate
    }

    /// Resolve a type by following substitutions (iterative to avoid stack overflow).
    fn resolve(&self, mut ty: Type) -> Type {
        // Iteratively follow Var → substitution chains to avoid stack overflow
        // for deep chains (e.g. many phi equations creating long Var→Var chains).
        loop {
            match ty {
                Type::Var(id) => match self.substitutions.get(&id) {
                    Some(resolved) => {
                        ty = resolved.clone();
                    }
                    None => return Type::Var(id),
                },
                other => return other,
            }
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
    /// Resolve a type by following substitutions (iterative to avoid stack overflow).
    fn resolve(&self, mut ty: Type) -> Type {
        // Iteratively follow Var → substitution chains to avoid stack overflow
        // for deep chains that can arise from phi node type equations.
        loop {
            match ty {
                Type::Var(id) => match self.substitutions.get(&id) {
                    Some(resolved) => {
                        ty = resolved.clone();
                    }
                    None => return Type::Var(id),
                },
                other => return other,
            }
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
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(p) = &mut v.tag {
                resolve_place(p, unifier);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        resolve_place(place, unifier);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        resolve_place(argument, unifier);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children {
                    resolve_place(child, unifier);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                resolve_place(child, unifier);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(key_place) = &mut p.key {
                            resolve_place(key_place, unifier);
                        }
                        resolve_place(&mut p.place, unifier);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        resolve_place(&mut s.place, unifier);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        resolve_place(p, unifier);
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        resolve_place(&mut s.place, unifier);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::BinaryExpression(v) => {
            resolve_place(&mut v.left, unifier);
            resolve_place(&mut v.right, unifier);
        }
        InstructionValue::UnaryExpression(v) => {
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::TemplateLiteral(v) => {
            for expr in &mut v.subexprs {
                resolve_place(expr, unifier);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            resolve_place(&mut v.tag, unifier);
        }
        InstructionValue::TypeCastExpression(v) => {
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::PropertyDelete(v) => {
            resolve_place(&mut v.object, unifier);
        }
        InstructionValue::ComputedDelete(v) => {
            resolve_place(&mut v.object, unifier);
            resolve_place(&mut v.property, unifier);
        }
        InstructionValue::GetIterator(v) => {
            resolve_place(&mut v.collection, unifier);
        }
        InstructionValue::IteratorNext(v) => {
            resolve_place(&mut v.iterator, unifier);
            resolve_place(&mut v.collection, unifier);
        }
        InstructionValue::NextPropertyOf(v) => {
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::PrefixUpdate(v) => {
            resolve_place(&mut v.lvalue, unifier);
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::PostfixUpdate(v) => {
            resolve_place(&mut v.lvalue, unifier);
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::Await(v) => {
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::StoreGlobal(v) => {
            resolve_place(&mut v.value, unifier);
        }
        InstructionValue::DeclareLocal(v) => {
            resolve_place(&mut v.lvalue.place, unifier);
        }
        InstructionValue::DeclareContext(v) => {
            resolve_place(&mut v.lvalue_place, unifier);
        }
        InstructionValue::FinishMemoize(v) => {
            resolve_place(&mut v.decl, unifier);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps.iter_mut() {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal { value, .. } =
                        &mut dep.root
                    {
                        resolve_place(value, unifier);
                    }
                }
            }
        }
        // These variants have no Place fields that need type resolution.
        InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::LoadGlobal(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::UnsupportedNode(_) => {}
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
        if let Some(first_param) = params_iter.next()
            && let ReactiveParam::Place(p) = first_param
        {
            equations.push(TypeEquation {
                left: p.identifier.type_.clone(),
                right: Type::Object(ObjectType { shape_id: Some(BUILT_IN_PROPS_ID.to_string()) }),
            });
        }
        // Second param → BuiltInUseRefId (for forwardRef)
        if let Some(second_param) = params_iter.next()
            && let ReactiveParam::Place(p) = second_param
        {
            equations.push(TypeEquation {
                left: p.identifier.type_.clone(),
                right: Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_REF_ID.to_string()) }),
            });
        }
    }

    // Port of TS InferTypes.ts: `const names = new Map();`
    // Maps temporary identifier IDs to their original source names. When
    // `LoadLocal(maybeRef) → $tmp`, we store `names[$tmp.id] = "maybeRef"`.
    // Later, `PropertyLoad($tmp, "current")` uses `names[$tmp.id]` to get
    // "maybeRef", allowing `is_ref_like_name` to fire on the property access.
    let mut names: FxHashMap<IdentifierId, String> = FxHashMap::default();

    // Port of TS InferTypes.ts: collect return value types to connect func.returns.
    let mut return_types: Vec<Type> = Vec::new();
    for block in func.body.blocks.values() {
        if let crate::hir::Terminal::Return(ret) = &block.terminal {
            return_types.push(ret.value.identifier.type_.clone());
        }
    }
    match return_types.len() {
        0 => {}
        1 => {
            equations.push(TypeEquation {
                left: func.returns.identifier.type_.clone(),
                right: return_types.remove(0),
            });
        }
        _ => {
            equations.push(TypeEquation {
                left: func.returns.identifier.type_.clone(),
                right: Type::Phi(crate::hir::types::PhiType { operands: return_types }),
            });
        }
    }

    for block in func.body.blocks.values() {
        for phi in &block.phis {
            let operand_types: Vec<Type> =
                phi.operands.values().map(|p| p.identifier.type_.clone()).collect();
            if !operand_types.is_empty() {
                equations.push(TypeEquation {
                    left: phi.place.identifier.type_.clone(),
                    right: Type::Phi(crate::hir::types::PhiType { operands: operand_types }),
                });
            }
        }

        for instr in &block.instructions {
            generate_instruction_equations(instr, &func.env, &mut names, &mut equations);
        }
    }

    equations
}

/// Port of TS `setName()`: store the original name of an identifier in the names map.
/// Only stores if the identifier has a `Named` name (not a promoted temporary).
fn set_name(
    names: &mut FxHashMap<IdentifierId, String>,
    id: IdentifierId,
    name: &Option<IdentifierName>,
) {
    if let Some(IdentifierName::Named(value)) = name {
        names.insert(id, value.clone());
    }
}

/// Port of TS `getName()`: look up the original name for an identifier ID.
/// Returns the stored name or empty string if not found.
fn get_name(names: &FxHashMap<IdentifierId, String>, id: IdentifierId) -> String {
    names.get(&id).cloned().unwrap_or_default()
}

fn generate_instruction_equations(
    instr: &Instruction,
    env: &Environment,
    names: &mut FxHashMap<IdentifierId, String>,
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
                // Constrain operands to Primitive for primitive binary ops (matches TS InferTypes.ts)
                equations.push(TypeEquation {
                    left: v.left.identifier.type_.clone(),
                    right: Type::Primitive,
                });
                equations.push(TypeEquation {
                    left: v.right.identifier.type_.clone(),
                    right: Type::Primitive,
                });
            }
            // All binary expression results are Primitive (matches TS InferTypes.ts)
            equations.push(TypeEquation { left: lvalue_type, right: Type::Primitive });
        }
        InstructionValue::LoadLocal(v) => {
            set_name(names, instr.lvalue.identifier.id, &v.place.identifier.name);
            equations
                .push(TypeEquation { left: lvalue_type, right: v.place.identifier.type_.clone() });
        }
        InstructionValue::LoadContext(_) => {
            // We intentionally do not infer types for context variable loads.
            // Port of TS InferTypes.ts lines 199-202:
            //   case 'DeclareContext':
            //   case 'LoadContext': { break; }
            // This leaves the lvalue as an unresolved TypeVar, preventing
            // over-propagation of return types through context variables
            // (e.g. a recursive function whose return type resolves to Primitive
            // would incorrectly cause `may_allocate` to return false for calls
            // through context, preventing scope creation).
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
        InstructionValue::ObjectExpression(obj) => {
            // TS InferTypes.ts: for each property with a computed key, emit
            // a type equation constraining the key to Primitive.
            for prop in &obj.properties {
                if let crate::hir::ObjectPatternProperty::Property(p) = prop
                    && let crate::hir::ObjectPropertyKey::Computed(key_place) = &p.key
                {
                    equations.push(TypeEquation {
                        left: key_place.identifier.type_.clone(),
                        right: Type::Primitive,
                    });
                }
            }
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
        InstructionValue::JsxExpression(v) => {
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Object(ObjectType { shape_id: Some(BUILT_IN_JSX_ID.to_string()) }),
            });
            for attr in &v.props {
                if let crate::hir::JsxAttribute::Attribute { name, place } = attr
                    && name == "ref"
                {
                    equations.push(TypeEquation {
                        left: place.identifier.type_.clone(),
                        right: Type::Object(ObjectType {
                            shape_id: Some(BUILT_IN_USE_REF_ID.to_string()),
                        }),
                    });
                }
            }
        }
        InstructionValue::JsxFragment(_) => {
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Object(ObjectType { shape_id: Some(BUILT_IN_JSX_ID.to_string()) }),
            });
        }
        InstructionValue::FunctionExpression(v) => {
            // Port of TS InferTypes.ts `case 'FunctionExpression': yield* generate(value.loweredFunc.func)`.
            // Recursively generate type equations for the inner function's instructions.
            // This is critical for propagating types through LoadContext instructions
            // inside inner functions (e.g. LoadContext setState → lvalue gets the
            // TFunction<BuiltInSetState> type, enabling the correct aliasing signature
            // to be used in InferMutationAliasingEffects instead of the conservative fallback).
            let inner_eqs = generate(&v.lowered_func.func);
            equations.extend(inner_eqs);
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Function(FunctionType {
                    shape_id: Some(BUILT_IN_FUNCTION_ID.to_string()),
                    return_type: Box::new(v.lowered_func.func.returns.identifier.type_.clone()),
                    is_constructor: false,
                }),
            });
        }
        InstructionValue::ObjectMethod(v) => {
            let inner_eqs = generate(&v.lowered_func.func);
            equations.extend(inner_eqs);
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
            // Use getName() to resolve the object name through temporary aliases
            // (e.g. LoadLocal(maybeRef) → $tmp, then PropertyLoad($tmp, "current")
            // needs to see "maybeRef" not "" to trigger is_ref_like_name).
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Property(Box::new(PropType {
                    object_type: load.object.identifier.type_.clone(),
                    object_name: get_name(names, load.object.identifier.id),
                    property_name: PropertyName::Literal {
                        value: PropertyLiteral::String(load.property.to_string()),
                    },
                })),
            });
        }
        InstructionValue::ComputedLoad(v) => {
            // Port of TS InferTypes.ts `ComputedLoad` case:
            // Generates a Property equation with a computed property name so that the
            // unifier can look up the wildcard `*` property on the receiver's shape.
            equations.push(TypeEquation {
                left: lvalue_type,
                right: Type::Property(Box::new(PropType {
                    object_type: v.object.identifier.type_.clone(),
                    object_name: get_name(names, v.object.identifier.id),
                    property_name: PropertyName::Computed {
                        value: Box::new(v.property.identifier.type_.clone()),
                    },
                })),
            });
        }
        InstructionValue::NewExpression(v) => {
            // Port of TypeScript InferTypes.ts `NewExpression` case:
            // ```ts
            // const returnType = makeType();
            // yield equation(value.callee.identifier.type, { kind: 'Function', return: returnType, isConstructor: true });
            // yield equation(left, returnType);
            // ```
            // This allows the constructor's return type (e.g. `new Map()` → Object(BuiltInMap))
            // to propagate to the lvalue, enabling precise method signature resolution
            // (e.g. `s.set(...)` on a Map correctly uses Effect.Store + Effect.Capture
            // rather than the conservative MutateTransitiveConditionally fallback).
            let return_type = make_type();
            equations.push(TypeEquation {
                left: v.callee.identifier.type_.clone(),
                right: Type::Function(FunctionType {
                    shape_id: None,
                    return_type: Box::new(return_type.clone()),
                    is_constructor: true,
                }),
            });
            equations.push(TypeEquation { left: lvalue_type, right: return_type });
        }
        InstructionValue::CallExpression(v) => {
            // The callee must be a function; its return type equals the lvalue type.
            let return_type = make_type();
            // Port of TS InferTypes.ts enableTreatSetIdentifiersAsStateSetters:
            // If enabled and the callee name starts with "set", treat it as a setState function.
            let mut shape_id = None;
            if env.config.enable_treat_set_identifiers_as_state_setters {
                let name = get_name(names, v.callee.identifier.id);
                if name.starts_with("set") {
                    shape_id = Some(BUILT_IN_SET_STATE_ID.to_string());
                }
            }
            equations.push(TypeEquation {
                left: v.callee.identifier.type_.clone(),
                right: Type::Function(FunctionType {
                    shape_id,
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
            let source_name = get_name(names, v.value.identifier.id);
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
                                        object_name: source_name.clone(),
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
                                            object_name: source_name.clone(),
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
        InstructionValue::PropertyStore(v) => {
            // Port of TS InferTypes.ts `PropertyStore` case (lines 473-501):
            // Infer types based on assignments to known object properties.
            // Important for refs, where assignment to `<maybeRef>.current`
            // can help infer that an object itself is a ref.
            // Uses a dummy type for the lvalue since we only want to trigger
            // ref inference from the Property type, not infer rvalue types.
            equations.push(TypeEquation {
                left: make_type(),
                right: Type::Property(Box::new(PropType {
                    object_type: v.object.identifier.type_.clone(),
                    object_name: get_name(names, v.object.identifier.id),
                    property_name: PropertyName::Literal {
                        value: PropertyLiteral::String(v.property.to_string()),
                    },
                })),
            });
        }
        InstructionValue::StoreContext(v) => {
            // TS InferTypes.ts: only emit type equation when lvalue.kind === InstructionKind.Const
            if v.lvalue_kind == InstructionKind::Const {
                equations.push(TypeEquation {
                    left: v.lvalue_place.identifier.type_.clone(),
                    right: v.value.identifier.type_.clone(),
                });
            }
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

/// Check if a property type refers to a ref-like name.
///
/// Port of TS InferTypes.ts `isRefLikeName()`:
/// Returns true if the property is `.current` on an identifier whose name
/// ends with `Ref` (e.g. `myRef.current`) or is exactly `ref`.
///
/// This is used by the `enableTreatRefLikeIdentifiersAsRefs` behavior to
/// infer that `someRef.current` accesses produce a BuiltInRefValue type,
/// which prevents mutations to `.current` from extending mutable ranges.
fn is_ref_like_name(prop: &PropType) -> bool {
    let is_current = match &prop.property_name {
        PropertyName::Literal { value } => match value {
            PropertyLiteral::String(s) => s == "current",
            _ => false,
        },
        _ => false,
    };
    if !is_current {
        return false;
    }
    let name = &prop.object_name;
    if name == "ref" {
        return true;
    }
    if name.ends_with("Ref") && name.len() > 3 {
        let prefix = &name[..name.len() - 3];
        let mut chars = prefix.chars();
        if let Some(first) = chars.next()
            && (first.is_ascii_alphabetic() || first == '$' || first == '_')
        {
            return chars.all(|c| c.is_ascii_alphanumeric() || c == '$' || c == '_');
        }
    }
    false
}
