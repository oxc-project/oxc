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

fn collect_instruction_value_operands<'a>(value: &'a InstructionValue, out: &mut Vec<&'a Place>) {
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
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &v.deps {
                for dep in deps {
                    if let super::hir_types::ManualMemoDependencyRoot::NamedLocal {
                        value, ..
                    } = &dep.root
                    {
                        out.push(value);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            out.push(&v.decl);
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::MetaProperty(_)
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

/// Iterate over all lvalue Places from an InstructionValue (excluding the instruction's own lvalue).
///
/// This covers DeclareLocal, StoreLocal, DeclareContext, StoreContext,
/// Destructure, PrefixUpdate, PostfixUpdate.
pub fn each_instruction_value_lvalue(value: &InstructionValue) -> Vec<&Place> {
    let mut lvalues = Vec::new();
    collect_instruction_value_lvalues(value, &mut lvalues);
    lvalues
}

fn collect_instruction_value_lvalues<'a>(value: &'a InstructionValue, out: &mut Vec<&'a Place>) {
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

// =====================================================================================
// Mutable mapping functions (replace Places in-place)
// =====================================================================================

/// Map all operand Places of an instruction in-place using the provided function.
pub fn map_instruction_operands(instr: &mut Instruction, f: &mut impl FnMut(Place) -> Place) {
    map_instruction_value_operands(&mut instr.value, f);
}

/// Map all operand Places of an instruction value in-place.
pub fn map_instruction_value_operands(
    value: &mut InstructionValue,
    f: &mut impl FnMut(Place) -> Place,
) {
    match value {
        InstructionValue::BinaryExpression(v) => {
            v.left = f(v.left.clone());
            v.right = f(v.right.clone());
        }
        InstructionValue::UnaryExpression(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::LoadLocal(v) => {
            v.place = f(v.place.clone());
        }
        InstructionValue::LoadContext(v) => {
            v.place = f(v.place.clone());
        }
        InstructionValue::StoreLocal(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::StoreContext(v) => {
            v.lvalue_place = f(v.lvalue_place.clone());
            v.value = f(v.value.clone());
        }
        InstructionValue::StoreGlobal(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::Destructure(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::PropertyLoad(v) => {
            v.object = f(v.object.clone());
        }
        InstructionValue::PropertyStore(v) => {
            v.object = f(v.object.clone());
            v.value = f(v.value.clone());
        }
        InstructionValue::PropertyDelete(v) => {
            v.object = f(v.object.clone());
        }
        InstructionValue::ComputedLoad(v) => {
            v.object = f(v.object.clone());
            v.property = f(v.property.clone());
        }
        InstructionValue::ComputedStore(v) => {
            v.object = f(v.object.clone());
            v.property = f(v.property.clone());
            v.value = f(v.value.clone());
        }
        InstructionValue::ComputedDelete(v) => {
            v.object = f(v.object.clone());
            v.property = f(v.property.clone());
        }
        InstructionValue::CallExpression(v) => {
            v.callee = f(v.callee.clone());
            map_call_args(&mut v.args, f);
        }
        InstructionValue::NewExpression(v) => {
            v.callee = f(v.callee.clone());
            map_call_args(&mut v.args, f);
        }
        InstructionValue::MethodCall(v) => {
            v.receiver = f(v.receiver.clone());
            v.property = f(v.property.clone());
            map_call_args(&mut v.args, f);
        }
        InstructionValue::TypeCastExpression(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::JsxExpression(v) => {
            if let JsxTag::Place(ref mut p) = v.tag {
                *p = f(p.clone());
            }
            for attr in &mut v.props {
                match attr {
                    JsxAttribute::Attribute { place, .. } => *place = f(place.clone()),
                    JsxAttribute::Spread { argument } => *argument = f(argument.clone()),
                }
            }
            if let Some(children) = &mut v.children {
                for child in children.iter_mut() {
                    *child = f(child.clone());
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                *child = f(child.clone());
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    ObjectPatternProperty::Property(p) => {
                        if let ObjectPropertyKey::Computed(ref mut place) = p.key {
                            *place = f(place.clone());
                        }
                        p.place = f(p.place.clone());
                    }
                    ObjectPatternProperty::Spread(s) => s.place = f(s.place.clone()),
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    ArrayExpressionElement::Place(p) => *p = f(p.clone()),
                    ArrayExpressionElement::Spread(s) => s.place = f(s.place.clone()),
                    ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                *ctx = f(ctx.clone());
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                *ctx = f(ctx.clone());
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            v.tag = f(v.tag.clone());
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                *subexpr = f(subexpr.clone());
            }
        }
        InstructionValue::Await(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::GetIterator(v) => {
            v.collection = f(v.collection.clone());
        }
        InstructionValue::IteratorNext(v) => {
            v.iterator = f(v.iterator.clone());
            v.collection = f(v.collection.clone());
        }
        InstructionValue::NextPropertyOf(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::PrefixUpdate(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::PostfixUpdate(v) => {
            v.value = f(v.value.clone());
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let super::hir_types::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value,
                        ..
                    } = dep.root
                    {
                        *value = f(value.clone());
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            v.decl = f(v.decl.clone());
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

/// Map all lvalue Places of an instruction in-place using the provided function.
pub fn map_instruction_lvalues(instr: &mut Instruction, f: &mut impl FnMut(Place) -> Place) {
    match &mut instr.value {
        InstructionValue::DeclareLocal(v) => {
            v.lvalue.place = f(v.lvalue.place.clone());
        }
        InstructionValue::StoreLocal(v) => {
            v.lvalue.place = f(v.lvalue.place.clone());
        }
        InstructionValue::Destructure(v) => {
            map_pattern_operands(&mut v.lvalue.pattern, f);
        }
        InstructionValue::PrefixUpdate(v) => {
            v.lvalue = f(v.lvalue.clone());
        }
        InstructionValue::PostfixUpdate(v) => {
            v.lvalue = f(v.lvalue.clone());
        }
        _ => {}
    }
    instr.lvalue = f(instr.lvalue.clone());
}

/// Map all Places in a pattern in-place.
pub fn map_pattern_operands(pattern: &mut Pattern, f: &mut impl FnMut(Place) -> Place) {
    match pattern {
        Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    super::hir_types::ArrayPatternElement::Place(p) => *p = f(p.clone()),
                    super::hir_types::ArrayPatternElement::Spread(s) => {
                        s.place = f(s.place.clone());
                    }
                    super::hir_types::ArrayPatternElement::Hole => {}
                }
            }
        }
        Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    ObjectPatternProperty::Property(p) => p.place = f(p.place.clone()),
                    ObjectPatternProperty::Spread(s) => s.place = f(s.place.clone()),
                }
            }
        }
    }
}

/// Map all operand Places of a terminal in-place.
pub fn map_terminal_operands(terminal: &mut Terminal, f: &mut impl FnMut(Place) -> Place) {
    match terminal {
        Terminal::Throw(t) => t.value = f(t.value.clone()),
        Terminal::Return(t) => t.value = f(t.value.clone()),
        Terminal::If(t) => t.test = f(t.test.clone()),
        Terminal::Branch(t) => t.test = f(t.test.clone()),
        Terminal::Switch(t) => {
            t.test = f(t.test.clone());
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    *test = f(test.clone());
                }
            }
        }
        Terminal::Try(t) => {
            if let Some(ref mut binding) = t.handler_binding {
                *binding = f(binding.clone());
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
}

fn map_call_args(args: &mut [CallArg], f: &mut impl FnMut(Place) -> Place) {
    for arg in args.iter_mut() {
        match arg {
            CallArg::Place(p) => *p = f(p.clone()),
            CallArg::Spread(s) => s.place = f(s.place.clone()),
        }
    }
}
