/// Type inference pass for the React Compiler HIR.
///
/// Port of `TypeInference/InferTypes.ts` from the React Compiler.
///
/// Infers types for HIR identifiers using a union-find based unification approach.
/// Generates type equations from instructions and unifies them to determine types.
use rustc_hash::FxHashMap;

use crate::hir::{
    HIRFunction, Instruction, InstructionKind, InstructionValue,
    object_shape::{BUILT_IN_ARRAY_ID, BUILT_IN_FUNCTION_ID, BUILT_IN_JSX_ID, BUILT_IN_OBJECT_ID},
    types::{FunctionType, ObjectType, Type, TypeId, make_type, type_equals},
};
use oxc_syntax::operator::BinaryOperator;

/// A type equation: left = right.
struct TypeEquation {
    left: Type,
    right: Type,
}

/// Union-find based type unifier.
struct Unifier {
    substitutions: FxHashMap<TypeId, Type>,
}

impl Unifier {
    fn new() -> Self {
        Self { substitutions: FxHashMap::default() }
    }

    /// Unify two types, recording any substitutions needed.
    fn unify(&mut self, left: Type, right: Type) {
        let left = self.resolve(left);
        let right = self.resolve(right);

        if type_equals(&left, &right) {
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

/// Run type inference on the given function.
pub fn infer_types(func: &mut HIRFunction) {
    let mut unifier = Unifier::new();

    // Generate type equations
    let equations = generate(func);
    for eq in equations {
        unifier.unify(eq.left, eq.right);
    }

    // Apply resolved types back to the function
    apply(func, &unifier);
}

fn apply(func: &mut HIRFunction, unifier: &Unifier) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        if let Some(block) = func.body.blocks.get_mut(&block_id) {
            for instr in &mut block.instructions {
                // Apply to lvalue
                instr.lvalue.identifier.type_ = unifier.get(instr.lvalue.identifier.type_.clone());

                // Apply to nested functions
                match &mut instr.value {
                    InstructionValue::FunctionExpression(v) => {
                        apply(&mut v.lowered_func.func, unifier);
                    }
                    InstructionValue::ObjectMethod(v) => {
                        apply(&mut v.lowered_func.func, unifier);
                    }
                    _ => {}
                }
            }
        }
    }

    // Apply to returns
    func.returns.identifier.type_ = unifier.get(func.returns.identifier.type_.clone());
}

fn generate(func: &HIRFunction) -> Vec<TypeEquation> {
    let mut equations = Vec::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            generate_instruction_equations(instr, &mut equations);
        }
    }

    equations
}

fn generate_instruction_equations(instr: &Instruction, equations: &mut Vec<TypeEquation>) {
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
