/// Optimize props method calls into regular calls.
///
/// Port of `Optimization/OptimizePropsMethodCalls.ts` from the React Compiler.
///
/// Converts method calls into regular calls where the receiver is the props object:
/// ```js
/// // INPUT: props.foo();
/// // OUTPUT: const t0 = props.foo; t0();
/// ```
use crate::hir::{
    HIRFunction, InstructionValue,
    types::{ObjectType, Type},
};

/// Run the optimization on the given function.
pub fn optimize_props_method_calls(func: &mut HIRFunction) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let block = match func.body.blocks.get_mut(&block_id) {
            Some(b) => b,
            None => continue,
        };
        for instr in &mut block.instructions {
            if let InstructionValue::MethodCall(method) = &instr.value
                && is_props_type(&method.receiver.identifier.type_) {
                    let callee = method.property.clone();
                    let args = method.args.clone();
                    let loc = method.loc;
                    instr.value = InstructionValue::CallExpression(
                        crate::hir::CallExpression { callee, args, loc },
                    );
                }
        }
    }
}

fn is_props_type(ty: &Type) -> bool {
    matches!(ty, Type::Object(ObjectType { shape_id: Some(id) }) if id == "BuiltInProps")
}
