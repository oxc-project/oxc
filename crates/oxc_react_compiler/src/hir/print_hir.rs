/// Pretty-printer for the HIR.
///
/// Port of `HIR/PrintHIR.ts` from the React Compiler.
///
/// Provides functions for printing the HIR to a human-readable string format,
/// useful for debugging and test output.
use std::fmt::Write;

use crate::compiler_error::SourceLocation;

use super::hir_types::{
    ArrayExpressionElement, CallArg, GotoVariant, Hir, HIRFunction, Identifier,
    IdentifierName, Instruction, InstructionValue, JsxAttribute, JsxTag,
    ObjectPatternProperty, ObjectPropertyKey, Pattern, Place,
    PrimitiveValueKind, ReactiveParam, Terminal,
};
use super::types::Type;

/// Print a full HIR function.
pub fn print_function(func: &HIRFunction) -> String {
    let mut output = String::new();
    let mut definition = String::new();

    if let Some(id) = &func.id {
        definition.push_str(id);
    } else {
        definition.push_str("<<anonymous>>");
    }

    if let Some(hint) = &func.name_hint {
        write!(definition, " {hint}").ok();
    }

    if !func.params.is_empty() {
        definition.push('(');
        let params: Vec<String> = func
            .params
            .iter()
            .map(|param| match param {
                ReactiveParam::Place(p) => print_place(p),
                ReactiveParam::Spread(s) => format!("...{}", print_place(&s.place)),
            })
            .collect();
        definition.push_str(&params.join(", "));
        definition.push(')');
    } else {
        definition.push_str("()");
    }

    write!(definition, ": {}", print_place(&func.returns)).ok();
    output.push_str(&definition);
    output.push('\n');

    for directive in &func.directives {
        output.push_str(directive);
        output.push('\n');
    }

    output.push_str(&print_hir(&func.body));
    output
}

/// Print the HIR control-flow graph.
pub fn print_hir(ir: &Hir) -> String {
    let mut output = Vec::new();

    for (&block_id, block) in &ir.blocks {
        output.push(format!("bb{} ({:?}):", block_id.0, block.kind));

        if !block.preds.is_empty() {
            let preds: Vec<String> = block.preds.iter().map(|p| format!("bb{}", p.0)).collect();
            output.push(format!("  predecessor blocks: {}", preds.join(" ")));
        }

        for instr in &block.instructions {
            output.push(format!("  {}", print_instruction(instr)));
        }

        let terminal = print_terminal(&block.terminal);
        for line in &terminal {
            output.push(format!("  {line}"));
        }
    }

    output.join("\n")
}

/// Print a single instruction.
pub fn print_instruction(instr: &Instruction) -> String {
    let id = format!("[{}]", instr.id.0);
    let value = print_instruction_value(&instr.value);
    format!("{id} {} = {value}", print_place(&instr.lvalue))
}

/// Print a terminal node.
pub fn print_terminal(terminal: &Terminal) -> Vec<String> {
    match terminal {
        Terminal::If(t) => vec![format!(
            "[{}] If ({}) then:bb{} else:bb{} fallthrough=bb{}",
            t.id.0,
            print_place(&t.test),
            t.consequent.0,
            t.alternate.0,
            t.fallthrough.0
        )],
        Terminal::Branch(t) => vec![format!(
            "[{}] Branch ({}) then:bb{} else:bb{} fallthrough:bb{}",
            t.id.0,
            print_place(&t.test),
            t.consequent.0,
            t.alternate.0,
            t.fallthrough.0
        )],
        Terminal::Logical(t) => vec![format!(
            "[{}] Logical {:?} test:bb{} fallthrough=bb{}",
            t.id.0, t.operator, t.test.0, t.fallthrough.0
        )],
        Terminal::Ternary(t) => vec![format!(
            "[{}] Ternary test:bb{} fallthrough=bb{}",
            t.id.0, t.test.0, t.fallthrough.0
        )],
        Terminal::Optional(t) => vec![format!(
            "[{}] Optional (optional={}) test:bb{} fallthrough=bb{}",
            t.id.0, t.optional, t.test.0, t.fallthrough.0
        )],
        Terminal::Throw(t) => {
            vec![format!("[{}] Throw {}", t.id.0, print_place(&t.value))]
        }
        Terminal::Return(t) => {
            vec![format!("[{}] Return {:?} {}", t.id.0, t.return_variant, print_place(&t.value))]
        }
        Terminal::Goto(t) => {
            let variant = match t.variant {
                GotoVariant::Continue => "(Continue)",
                GotoVariant::Break => "",
                GotoVariant::Try => "(Try)",
            };
            vec![format!("[{}] Goto{variant} bb{}", t.id.0, t.block.0)]
        }
        Terminal::Switch(t) => {
            let mut lines = vec![format!("[{}] Switch ({})", t.id.0, print_place(&t.test))];
            for case in &t.cases {
                if let Some(test) = &case.test {
                    lines.push(format!("  Case {}: bb{}", print_place(test), case.block.0));
                } else {
                    lines.push(format!("  Default: bb{}", case.block.0));
                }
            }
            lines.push(format!("  Fallthrough: bb{}", t.fallthrough.0));
            lines
        }
        Terminal::DoWhile(t) => vec![format!(
            "[{}] DoWhile loop=bb{} test=bb{} fallthrough=bb{}",
            t.id.0, t.r#loop.0, t.test.0, t.fallthrough.0
        )],
        Terminal::While(t) => vec![format!(
            "[{}] While test=bb{} loop=bb{} fallthrough=bb{}",
            t.id.0, t.test.0, t.r#loop.0, t.fallthrough.0
        )],
        Terminal::For(t) => {
            let update = t.update.map_or("(none)".to_string(), |u| format!("bb{}", u.0));
            vec![format!(
                "[{}] For init=bb{} test=bb{} loop=bb{} update={update} fallthrough=bb{}",
                t.id.0, t.init.0, t.test.0, t.r#loop.0, t.fallthrough.0
            )]
        }
        Terminal::ForOf(t) => vec![format!(
            "[{}] ForOf init=bb{} test=bb{} loop=bb{} fallthrough=bb{}",
            t.id.0, t.init.0, t.test.0, t.r#loop.0, t.fallthrough.0
        )],
        Terminal::ForIn(t) => vec![format!(
            "[{}] ForIn init=bb{} loop=bb{} fallthrough=bb{}",
            t.id.0, t.init.0, t.r#loop.0, t.fallthrough.0
        )],
        Terminal::Label(t) => vec![format!(
            "[{}] Label block=bb{} fallthrough=bb{}",
            t.id.0, t.block.0, t.fallthrough.0
        )],
        Terminal::Sequence(t) => vec![format!(
            "[{}] Sequence block=bb{} fallthrough=bb{}",
            t.id.0, t.block.0, t.fallthrough.0
        )],
        Terminal::Unreachable(t) => vec![format!("[{}] Unreachable", t.id.0)],
        Terminal::Unsupported(t) => vec![format!("[{}] Unsupported", t.id.0)],
        Terminal::MaybeThrow(t) => {
            let handler = t.handler.map_or("(none)".to_string(), |h| format!("bb{}", h.0));
            vec![format!(
                "[{}] MaybeThrow continuation=bb{} handler={handler}",
                t.id.0, t.continuation.0
            )]
        }
        Terminal::Try(t) => {
            let binding = t
                .handler_binding
                .as_ref()
                .map_or(String::new(), |b| format!(" handlerBinding=({})", print_place(b)));
            vec![format!(
                "[{}] Try block=bb{} handler=bb{}{binding} fallthrough=bb{}",
                t.id.0, t.block.0, t.handler.0, t.fallthrough.0
            )]
        }
        Terminal::Scope(t) => vec![format!(
            "[{}] Scope scope={} block=bb{} fallthrough=bb{}",
            t.id.0, t.scope.id.0, t.block.0, t.fallthrough.0
        )],
        Terminal::PrunedScope(t) => vec![format!(
            "[{}] <pruned> Scope scope={} block=bb{} fallthrough=bb{}",
            t.id.0, t.scope.id.0, t.block.0, t.fallthrough.0
        )],
    }
}

/// Print an instruction value.
pub fn print_instruction_value(value: &InstructionValue) -> String {
    match value {
        InstructionValue::ArrayExpression(v) => {
            let elements: Vec<String> = v
                .elements
                .iter()
                .map(|e| match e {
                    ArrayExpressionElement::Place(p) => print_place(p),
                    ArrayExpressionElement::Hole => "<hole>".to_string(),
                    ArrayExpressionElement::Spread(s) => format!("...{}", print_place(&s.place)),
                })
                .collect();
            format!("Array [{}]", elements.join(", "))
        }
        InstructionValue::ObjectExpression(v) => {
            let props: Vec<String> = v
                .properties
                .iter()
                .map(|p| match p {
                    ObjectPatternProperty::Property(prop) => {
                        format!(
                            "{}: {}",
                            print_object_property_key(&prop.key),
                            print_place(&prop.place)
                        )
                    }
                    ObjectPatternProperty::Spread(s) => format!("...{}", print_place(&s.place)),
                })
                .collect();
            format!("Object {{ {} }}", props.join(", "))
        }
        InstructionValue::BinaryExpression(v) => {
            format!("Binary {} {:?} {}", print_place(&v.left), v.operator, print_place(&v.right))
        }
        InstructionValue::UnaryExpression(v) => {
            format!("Unary {:?} {}", v.operator, print_place(&v.value))
        }
        InstructionValue::CallExpression(v) => {
            let args: Vec<String> = v.args.iter().map(print_call_arg).collect();
            format!("Call {}({})", print_place(&v.callee), args.join(", "))
        }
        InstructionValue::MethodCall(v) => {
            let args: Vec<String> = v.args.iter().map(print_call_arg).collect();
            format!(
                "MethodCall {}.{}({})",
                print_place(&v.receiver),
                print_place(&v.property),
                args.join(", ")
            )
        }
        InstructionValue::NewExpression(v) => {
            let args: Vec<String> = v.args.iter().map(print_call_arg).collect();
            format!("New {}({})", print_place(&v.callee), args.join(", "))
        }
        InstructionValue::Primitive(v) => match &v.value {
            PrimitiveValueKind::Undefined => "<undefined>".to_string(),
            PrimitiveValueKind::Null => "null".to_string(),
            PrimitiveValueKind::Boolean(b) => b.to_string(),
            PrimitiveValueKind::Number(n) => n.to_string(),
            PrimitiveValueKind::String(s) => format!("\"{s}\""),
        },
        InstructionValue::JsxText(v) => format!("JSXText {:?}", v.value),
        InstructionValue::LoadLocal(v) => format!("LoadLocal {}", print_place(&v.place)),
        InstructionValue::LoadContext(v) => format!("LoadContext {}", print_place(&v.place)),
        InstructionValue::DeclareLocal(v) => {
            format!("DeclareLocal {:?} {}", v.lvalue.kind, print_place(&v.lvalue.place))
        }
        InstructionValue::DeclareContext(v) => {
            format!("DeclareContext {:?} {}", v.lvalue_kind, print_place(&v.lvalue_place))
        }
        InstructionValue::StoreLocal(v) => {
            format!(
                "StoreLocal {:?} {} = {}",
                v.lvalue.kind,
                print_place(&v.lvalue.place),
                print_place(&v.value)
            )
        }
        InstructionValue::StoreContext(v) => {
            format!(
                "StoreContext {:?} {} = {}",
                v.lvalue_kind,
                print_place(&v.lvalue_place),
                print_place(&v.value)
            )
        }
        InstructionValue::Destructure(v) => {
            format!(
                "Destructure {:?} {} = {}",
                v.lvalue.kind,
                print_pattern(&v.lvalue.pattern),
                print_place(&v.value)
            )
        }
        InstructionValue::PropertyLoad(v) => {
            format!("PropertyLoad {}.{}", print_place(&v.object), v.property)
        }
        InstructionValue::PropertyStore(v) => {
            format!(
                "PropertyStore {}.{} = {}",
                print_place(&v.object),
                v.property,
                print_place(&v.value)
            )
        }
        InstructionValue::PropertyDelete(v) => {
            format!("PropertyDelete {}.{}", print_place(&v.object), v.property)
        }
        InstructionValue::ComputedLoad(v) => {
            format!("ComputedLoad {}[{}]", print_place(&v.object), print_place(&v.property))
        }
        InstructionValue::ComputedStore(v) => {
            format!(
                "ComputedStore {}[{}] = {}",
                print_place(&v.object),
                print_place(&v.property),
                print_place(&v.value)
            )
        }
        InstructionValue::ComputedDelete(v) => {
            format!("ComputedDelete {}[{}]", print_place(&v.object), print_place(&v.property))
        }
        InstructionValue::LoadGlobal(v) => {
            let name = match &v.binding {
                super::hir_types::NonLocalBinding::Global { name } => name.clone(),
                super::hir_types::NonLocalBinding::ModuleLocal { name } => name.clone(),
                super::hir_types::NonLocalBinding::ImportDefault { name, .. } => name.clone(),
                super::hir_types::NonLocalBinding::ImportNamespace { name, .. } => name.clone(),
                super::hir_types::NonLocalBinding::ImportSpecifier { name, .. } => name.clone(),
            };
            format!("LoadGlobal {name}")
        }
        InstructionValue::StoreGlobal(v) => {
            format!("StoreGlobal {} = {}", v.name, print_place(&v.value))
        }
        InstructionValue::TypeCastExpression(v) => {
            format!("TypeCast {}: {}", print_place(&v.value), print_type(&v.type_))
        }
        InstructionValue::JsxExpression(v) => {
            let tag = match &v.tag {
                JsxTag::Place(p) => print_place(p),
                JsxTag::BuiltIn(b) => b.name.clone(),
            };
            let props: Vec<String> = v
                .props
                .iter()
                .map(|a| match a {
                    JsxAttribute::Attribute { name, place } => {
                        format!("{name}={{{}}}", print_place(place))
                    }
                    JsxAttribute::Spread { argument } => {
                        format!("...{}", print_place(argument))
                    }
                })
                .collect();
            let props_str = if props.is_empty() { String::new() } else { format!(" {}", props.join(" ")) };
            if let Some(children) = &v.children {
                let children_str: Vec<String> =
                    children.iter().map(|c| format!("{{{}}}", print_place(c))).collect();
                format!("JSX <{tag}{props_str}>{}</{tag}>", children_str.join(""))
            } else {
                format!("JSX <{tag}{props_str}/>")
            }
        }
        InstructionValue::JsxFragment(v) => {
            let children: Vec<String> = v.children.iter().map(print_place).collect();
            format!("JsxFragment [{}]", children.join(", "))
        }
        InstructionValue::FunctionExpression(v) => {
            let name = v.name.as_deref().unwrap_or("<<anonymous>>");
            format!("FunctionExpression {name}")
        }
        InstructionValue::ObjectMethod(v) => {
            format!("ObjectMethod ({})", print_place(&v.lowered_func.func.returns))
        }
        InstructionValue::TemplateLiteral(v) => {
            let exprs: Vec<String> = v.subexprs.iter().map(print_place).collect();
            format!("TemplateLiteral({} expressions)", exprs.len())
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            format!("TaggedTemplate {}", print_place(&v.tag))
        }
        InstructionValue::RegExpLiteral(v) => format!("RegExp /{}/{}", v.pattern, v.flags),
        InstructionValue::Await(v) => format!("Await {}", print_place(&v.value)),
        InstructionValue::GetIterator(v) => {
            format!("GetIterator {}", print_place(&v.collection))
        }
        InstructionValue::IteratorNext(v) => {
            format!("IteratorNext {} of {}", print_place(&v.iterator), print_place(&v.collection))
        }
        InstructionValue::NextPropertyOf(v) => {
            format!("NextPropertyOf {}", print_place(&v.value))
        }
        InstructionValue::PrefixUpdate(v) => {
            format!("PrefixUpdate {:?} {}", v.operation, print_place(&v.value))
        }
        InstructionValue::PostfixUpdate(v) => {
            format!("PostfixUpdate {:?} {}", v.operation, print_place(&v.value))
        }
        InstructionValue::StartMemoize(v) => format!("StartMemoize({})", v.manual_memo_id),
        InstructionValue::FinishMemoize(v) => {
            format!("FinishMemoize({}) decl={}", v.manual_memo_id, print_place(&v.decl))
        }
        InstructionValue::MetaProperty(v) => format!("MetaProperty {}.{}", v.meta, v.property),
        InstructionValue::Debugger(_) => "Debugger".to_string(),
        InstructionValue::UnsupportedNode(_) => "UnsupportedNode".to_string(),
    }
}

/// Print a Place.
pub fn print_place(place: &Place) -> String {
    let mut parts = Vec::new();
    parts.push(format!("{}", place.effect));
    parts.push(" ".to_string());
    parts.push(print_identifier(&place.identifier));
    parts.push(print_mutable_range(&place.identifier));
    parts.push(print_type(&place.identifier.type_));
    if place.reactive {
        parts.push("{reactive}".to_string());
    }
    parts.join("")
}

/// Print an Identifier.
pub fn print_identifier(id: &Identifier) -> String {
    let name = print_name(&id.name);
    let scope_str = id
        .scope
        .as_ref()
        .map_or(String::new(), |s| format!("_@{}", s.id.0));
    format!("{name}${}{scope_str}", id.id.0)
}

fn print_name(name: &Option<IdentifierName>) -> String {
    match name {
        None => String::new(),
        Some(IdentifierName::Named(n)) => n.clone(),
        Some(IdentifierName::Promoted(n)) => n.clone(),
    }
}

fn print_mutable_range(id: &Identifier) -> String {
    let range = &id.mutable_range;
    if range.start.0 == 0 && range.end.0 == 0 {
        return String::new();
    }
    format!("[{}:{}]", range.start.0, range.end.0)
}

/// Print a Type.
pub fn print_type(ty: &Type) -> String {
    match ty {
        Type::Var(_) => String::new(),
        Type::Object(obj) if obj.shape_id.is_some() => {
            format!(":TObject<{}>", obj.shape_id.as_ref().expect("checked"))
        }
        Type::Function(func) if func.shape_id.is_some() => {
            let return_type = print_type(&func.return_type);
            let ret = if return_type.is_empty() { String::new() } else { format!(":  {return_type}") };
            format!(":TFunction<{}>()){ret}", func.shape_id.as_ref().expect("checked"))
        }
        _ => format!(":T{}", ty.kind()),
    }
}

/// Print a source location.
pub fn print_source_location(loc: SourceLocation) -> String {
    match loc {
        SourceLocation::Generated => "generated".to_string(),
        SourceLocation::Source(span) => format!("{}:{}", span.start, span.end),
    }
}

fn print_object_property_key(key: &ObjectPropertyKey) -> String {
    match key {
        ObjectPropertyKey::Identifier(name) => name.clone(),
        ObjectPropertyKey::String(name) => format!("\"{name}\""),
        ObjectPropertyKey::Computed(place) => format!("[{}]", print_place(place)),
        ObjectPropertyKey::Number(n) => n.to_string(),
    }
}

fn print_call_arg(arg: &CallArg) -> String {
    match arg {
        CallArg::Place(p) => print_place(p),
        CallArg::Spread(s) => format!("...{}", print_place(&s.place)),
    }
}

fn print_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Array(arr) => {
            let items: Vec<String> = arr
                .items
                .iter()
                .map(|item| match item {
                    super::hir_types::ArrayPatternElement::Place(p) => print_place(p),
                    super::hir_types::ArrayPatternElement::Hole => "<hole>".to_string(),
                    super::hir_types::ArrayPatternElement::Spread(s) => {
                        format!("...{}", print_place(&s.place))
                    }
                })
                .collect();
            format!("[ {} ]", items.join(", "))
        }
        Pattern::Object(obj) => {
            let props: Vec<String> = obj
                .properties
                .iter()
                .map(|prop| match prop {
                    ObjectPatternProperty::Property(p) => {
                        format!("{}: {}", print_object_property_key(&p.key), print_place(&p.place))
                    }
                    ObjectPatternProperty::Spread(s) => format!("...{}", print_place(&s.place)),
                })
                .collect();
            format!("{{ {} }}", props.join(", "))
        }
    }
}
