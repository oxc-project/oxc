/// HIR visitor infrastructure.
///
/// Port of `HIR/visitors.ts` from the React Compiler.
///
/// Provides functions for iterating over and mapping the operands and lvalues
/// of instructions and terminals. These are the building blocks for all
/// compiler passes.
use super::hir_types::{
    ArrayExpressionElement, BlockId, CallArg, Instruction, InstructionValue, JsxAttribute, JsxTag,
    ObjectPatternProperty, ObjectPropertyKey, Pattern, Place, Terminal,
};

// =====================================================================================
// Instruction operand iteration
// =====================================================================================

/// Iterate over all operand Places of an instruction.
pub fn each_instruction_operand(instr: &Instruction) -> Vec<&Place> {
    each_instruction_value_operand(&instr.value)
}

/// Iterate over all operand Places of an instruction value.
pub fn each_instruction_value_operand(value: &InstructionValue) -> Vec<&Place> {
    let mut operands = Vec::new();
    collect_instruction_value_operands(value, &mut operands);
    operands
}

fn collect_instruction_value_operands<'a>(
    value: &'a InstructionValue,
    out: &mut Vec<&'a Place>,
) {
    match value {
        InstructionValue::CallExpression(v) => {
            out.push(&v.callee);
            collect_call_args(&v.args, out);
        }
        InstructionValue::NewExpression(v) => {
            out.push(&v.callee);
            collect_call_args(&v.args, out);
        }
        InstructionValue::MethodCall(v) => {
            out.push(&v.receiver);
            out.push(&v.property);
            collect_call_args(&v.args, out);
        }
        InstructionValue::BinaryExpression(v) => {
            out.push(&v.left);
            out.push(&v.right);
        }
        InstructionValue::UnaryExpression(v) => {
            out.push(&v.value);
        }
        InstructionValue::LoadLocal(v) => {
            out.push(&v.place);
        }
        InstructionValue::LoadContext(v) => {
            out.push(&v.place);
        }
        InstructionValue::StoreLocal(v) => {
            out.push(&v.value);
        }
        InstructionValue::StoreContext(v) => {
            out.push(&v.lvalue_place);
            out.push(&v.value);
        }
        InstructionValue::StoreGlobal(v) => {
            out.push(&v.value);
        }
        InstructionValue::Destructure(v) => {
            out.push(&v.value);
        }
        InstructionValue::PropertyLoad(v) => {
            out.push(&v.object);
        }
        InstructionValue::PropertyStore(v) => {
            out.push(&v.object);
            out.push(&v.value);
        }
        InstructionValue::PropertyDelete(v) => {
            out.push(&v.object);
        }
        InstructionValue::ComputedLoad(v) => {
            out.push(&v.object);
            out.push(&v.property);
        }
        InstructionValue::ComputedStore(v) => {
            out.push(&v.object);
            out.push(&v.property);
            out.push(&v.value);
        }
        InstructionValue::ComputedDelete(v) => {
            out.push(&v.object);
            out.push(&v.property);
        }
        InstructionValue::TypeCastExpression(v) => {
            out.push(&v.value);
        }
        InstructionValue::JsxExpression(v) => {
            if let JsxTag::Place(p) = &v.tag {
                out.push(p);
            }
            for attr in &v.props {
                match attr {
                    JsxAttribute::Attribute { place, .. } => out.push(place),
                    JsxAttribute::Spread { argument } => out.push(argument),
                }
            }
            if let Some(children) = &v.children {
                for child in children {
                    out.push(child);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &v.children {
                out.push(child);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &v.properties {
                match prop {
                    ObjectPatternProperty::Property(p) => {
                        if let ObjectPropertyKey::Computed(place) = &p.key {
                            out.push(place);
                        }
                        out.push(&p.place);
                    }
                    ObjectPatternProperty::Spread(s) => out.push(&s.place),
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &v.elements {
                match elem {
                    ArrayExpressionElement::Place(p) => out.push(p),
                    ArrayExpressionElement::Spread(s) => out.push(&s.place),
                    ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &v.lowered_func.func.context {
                out.push(ctx);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &v.lowered_func.func.context {
                out.push(ctx);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            out.push(&v.tag);
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &v.subexprs {
                out.push(subexpr);
            }
        }
        InstructionValue::Await(v) => {
            out.push(&v.value);
        }
        InstructionValue::GetIterator(v) => {
            out.push(&v.collection);
        }
        InstructionValue::IteratorNext(v) => {
            out.push(&v.iterator);
            out.push(&v.collection);
        }
        InstructionValue::NextPropertyOf(v) => {
            out.push(&v.value);
        }
        InstructionValue::PrefixUpdate(v) => {
            out.push(&v.value);
        }
        InstructionValue::PostfixUpdate(v) => {
            out.push(&v.value);
        }
        InstructionValue::LoadGlobal(_) => {}
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &v.deps {
                for dep in deps {
                    if let super::hir_types::ManualMemoDependencyRoot::NamedLocal { value, .. } =
                        &dep.root
                    {
                        out.push(value);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            out.push(&v.decl);
        }
        InstructionValue::MetaProperty(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

fn collect_call_args<'a>(args: &'a [CallArg], out: &mut Vec<&'a Place>) {
    for arg in args {
        match arg {
            CallArg::Place(p) => out.push(p),
            CallArg::Spread(s) => out.push(&s.place),
        }
    }
}

// =====================================================================================
// Instruction lvalue iteration
// =====================================================================================

/// Iterate over all lvalue Places of an instruction.
pub fn each_instruction_lvalue(instr: &Instruction) -> Vec<&Place> {
    let mut lvalues = Vec::new();
    lvalues.push(&instr.lvalue);
    collect_instruction_value_lvalues(&instr.value, &mut lvalues);
    lvalues
}

fn collect_instruction_value_lvalues<'a>(
    value: &'a InstructionValue,
    out: &mut Vec<&'a Place>,
) {
    match value {
        InstructionValue::DeclareLocal(v) => {
            out.push(&v.lvalue.place);
        }
        InstructionValue::DeclareContext(v) => {
            out.push(&v.lvalue_place);
        }
        InstructionValue::StoreLocal(v) => {
            out.push(&v.lvalue.place);
        }
        InstructionValue::StoreContext(v) => {
            out.push(&v.lvalue_place);
        }
        InstructionValue::Destructure(v) => {
            collect_pattern_operands(&v.lvalue.pattern, out);
        }
        InstructionValue::PrefixUpdate(v) => {
            out.push(&v.lvalue);
        }
        InstructionValue::PostfixUpdate(v) => {
            out.push(&v.lvalue);
        }
        _ => {}
    }
}

// =====================================================================================
// Pattern operand iteration
// =====================================================================================

/// Iterate over all Places in a destructuring pattern.
pub fn each_pattern_operand(pattern: &Pattern) -> Vec<&Place> {
    let mut places = Vec::new();
    collect_pattern_operands(pattern, &mut places);
    places
}

fn collect_pattern_operands<'a>(pattern: &'a Pattern, out: &mut Vec<&'a Place>) {
    match pattern {
        Pattern::Array(arr) => {
            for item in &arr.items {
                match item {
                    super::hir_types::ArrayPatternElement::Place(p) => out.push(p),
                    super::hir_types::ArrayPatternElement::Spread(s) => out.push(&s.place),
                    super::hir_types::ArrayPatternElement::Hole => {}
                }
            }
        }
        Pattern::Object(obj) => {
            for prop in &obj.properties {
                match prop {
                    ObjectPatternProperty::Property(p) => out.push(&p.place),
                    ObjectPatternProperty::Spread(s) => out.push(&s.place),
                }
            }
        }
    }
}

// =====================================================================================
// Terminal operand/successor iteration
// =====================================================================================

/// Iterate over all operand Places of a terminal.
pub fn each_terminal_operand(terminal: &Terminal) -> Vec<&Place> {
    let mut operands = Vec::new();
    match terminal {
        Terminal::Throw(t) => operands.push(&t.value),
        Terminal::Return(t) => operands.push(&t.value),
        Terminal::If(t) => operands.push(&t.test),
        Terminal::Branch(t) => operands.push(&t.test),
        Terminal::Switch(t) => {
            operands.push(&t.test);
            for case in &t.cases {
                if let Some(test) = &case.test {
                    operands.push(test);
                }
            }
        }
        Terminal::Try(t) => {
            if let Some(binding) = &t.handler_binding {
                operands.push(binding);
            }
        }
        Terminal::Unsupported(_)
        | Terminal::Unreachable(_)
        | Terminal::Goto(_)
        | Terminal::For(_)
        | Terminal::ForOf(_)
        | Terminal::ForIn(_)
        | Terminal::DoWhile(_)
        | Terminal::While(_)
        | Terminal::Logical(_)
        | Terminal::Ternary(_)
        | Terminal::Optional(_)
        | Terminal::Label(_)
        | Terminal::Sequence(_)
        | Terminal::MaybeThrow(_)
        | Terminal::Scope(_)
        | Terminal::PrunedScope(_) => {}
    }
    operands
}

/// Iterate over all successor block IDs of a terminal.
pub fn each_terminal_successor(terminal: &Terminal) -> Vec<BlockId> {
    // Delegate to the implementation in hir_builder
    super::hir_builder::each_terminal_successor(terminal)
}

/// Get the fallthrough block of a terminal, if any.
pub fn terminal_fallthrough(terminal: &Terminal) -> Option<BlockId> {
    terminal.fallthrough()
}
