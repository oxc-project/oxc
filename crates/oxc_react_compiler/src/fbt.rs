//! fbt/fbs macro lowering.
//!
//! Ports (a subset of) Meta's `babel-plugin-fbt` + `babel-plugin-fbt-runtime`,
//! which upstream run *after* the React Compiler in the fixture test harness
//! (see `RunReactCompilerBabelPlugin.ts`: `plugins: [reactCompiler,
//! 'babel-plugin-fbt', 'babel-plugin-fbt-runtime']`). Those plugins lower
//! `fbt(...)` / `<fbt>` / `fbt.param(...)` / `<fbt:param>` macros into their
//! runtime form:
//!
//! ```js
//! fbt._("Hello {user name}", [fbt._param("user name", props.name)], { hk: "2zEDKF" })
//! ```
//!
//! This module runs on the compiled oxc `Program`, mirroring that ordering.
//!
//! ## Scope
//!
//! Implemented: the "single leaf" case — plain text with `fbt.param` /
//! `<fbt:param>` interpolations (no string variations), for both the call form
//! (`fbt(text, desc)`, incl. template-literal / string-concat / array text) and
//! the JSX form (`<fbt desc>...</fbt>`), for both the `fbt` and `fbs` modules.
//!
//! Not yet implemented (left untransformed, matching prior behaviour): string
//! variations (`fbt.plural` / `<fbt:plural>`, `fbt.enum` / `<fbt:enum>`,
//! `fbt.pronoun`, and `number`/`gender` param options — all of which build a
//! `jsfbt` *table*) and implicit-param subtrees (non-fbt JSX elements nested
//! directly inside `<fbt>`, lowered to `fbt._implicitParam`).

use oxc_allocator::{CloneIn, GetAllocator, Vec as ArenaVec};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, Expression, IdentifierName, JSXAttributeItem,
    JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement, JSXElementName, JSXExpression,
    ObjectPropertyKind, Program, PropertyKey, PropertyKind, Statement,
    TSTypeParameterInstantiation,
};
use oxc_ast::builder::AstBuilder;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_span::SPAN;
use oxc_syntax::operator::BinaryOperator;

/// Lower `fbt`/`fbs` macros in `program` to their runtime `fbt._(...)` form.
///
/// No-op when the module never imports `fbt`/`fbs` from `"fbt"`.
pub fn transform_fbt<'a>(ast: &AstBuilder<'a>, program: &mut Program<'a>) {
    let (fbt, fbs) = collect_fbt_modules(program);
    if !fbt && !fbs {
        return;
    }
    let mut visitor = FbtVisitor { ast, fbt, fbs };
    visitor.visit_program(program);
}

/// Which of the `fbt` / `fbs` names are bound to the `"fbt"` module in this file.
fn collect_fbt_modules(program: &Program) -> (bool, bool) {
    let (mut fbt, mut fbs) = (false, false);
    for stmt in &program.body {
        let Statement::ImportDeclaration(import) = stmt else { continue };
        if import.source.value != "fbt" {
            continue;
        }
        let Some(specifiers) = &import.specifiers else { continue };
        for spec in specifiers {
            match spec.local().name.as_str() {
                "fbt" => fbt = true,
                "fbs" => fbs = true,
                _ => {}
            }
        }
    }
    (fbt, fbs)
}

struct FbtVisitor<'a, 'b> {
    ast: &'b AstBuilder<'a>,
    fbt: bool,
    fbs: bool,
}

impl<'a> VisitMut<'a> for FbtVisitor<'a, '_> {
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        // Lower innermost macros first so a nested `fbt(...)` inside a param value
        // is already in runtime form when the enclosing macro clones it.
        walk_mut::walk_expression(self, expr);
        if let Some(replacement) = self.try_lower(expr) {
            *expr = replacement;
        }
    }
}

/// A piece of an fbt string: literal text, or an interpolated `fbt.param`.
enum Part<'a> {
    Text(String),
    Param { name: String, value: Expression<'a> },
}

impl<'a> FbtVisitor<'a, '_> {
    /// If `expr` is an `fbt(...)` call or `<fbt>` element (or `fbs`), return its
    /// lowered runtime form. `None` means "leave untouched" (not fbt, or a case
    /// this port does not handle).
    fn try_lower(&self, expr: &Expression<'a>) -> Option<Expression<'a>> {
        match expr {
            Expression::CallExpression(call) => {
                let module = self.callee_module(&call.callee)?;
                self.lower_call(module, &call.arguments)
            }
            Expression::JSXElement(el) => {
                let module = self.element_module(el)?;
                self.lower_jsx(module, el)
            }
            _ => None,
        }
    }

    /// Module name if `callee` is a bare `fbt` / `fbs` identifier.
    fn callee_module(&self, callee: &Expression<'a>) -> Option<&'static str> {
        let Expression::Identifier(id) = callee else { return None };
        self.module_for(id.name.as_str())
    }

    /// Module name if `el`'s tag is a bare `fbt` / `fbs` identifier.
    fn element_module(&self, el: &JSXElement<'a>) -> Option<&'static str> {
        let JSXElementName::Identifier(id) = &el.opening_element.name else { return None };
        self.module_for(id.name.as_str())
    }

    fn module_for(&self, name: &str) -> Option<&'static str> {
        match name {
            "fbt" if self.fbt => Some("fbt"),
            "fbs" if self.fbs => Some("fbs"),
            _ => None,
        }
    }

    fn lower_call(
        &self,
        module: &str,
        args: &ArenaVec<'a, Argument<'a>>,
    ) -> Option<Expression<'a>> {
        let text_arg = args.first()?.as_expression()?;
        let desc_arg = args.get(1)?.as_expression()?;
        let mut parts = Vec::new();
        self.collect_call_parts(text_arg, &mut parts)?;
        let desc = normalize_spaces(&expand_string_concat(desc_arg)?).trim().to_string();
        Some(self.build_runtime_call(module, parts, &desc))
    }

    fn lower_jsx(&self, module: &str, el: &JSXElement<'a>) -> Option<Expression<'a>> {
        let desc = self.jsx_description(el)?;
        let mut parts = Vec::new();
        self.collect_jsx_children(&el.children, &mut parts)?;
        Some(self.build_runtime_call(module, parts, &desc))
    }

    // --- call-form text extraction ------------------------------------------

    fn collect_call_parts(&self, expr: &Expression<'a>, out: &mut Vec<Part<'a>>) -> Option<()> {
        match expr {
            Expression::StringLiteral(s) => out.push(Part::Text(s.value.as_str().to_string())),
            Expression::TemplateLiteral(tpl) => {
                for (i, quasi) in tpl.quasis.iter().enumerate() {
                    if let Some(cooked) = &quasi.value.cooked {
                        out.push(Part::Text(cooked.as_str().to_string()));
                    }
                    if let Some(expr) = tpl.expressions.get(i) {
                        self.collect_construct(expr, out)?;
                    }
                }
            }
            Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
                self.collect_call_parts(&bin.left, out)?;
                self.collect_call_parts(&bin.right, out)?;
            }
            Expression::ArrayExpression(arr) => {
                for el in &arr.elements {
                    self.collect_call_parts(el.as_expression()?, out)?;
                }
            }
            Expression::ParenthesizedExpression(paren) => {
                self.collect_call_parts(&paren.expression, out)?;
            }
            Expression::CallExpression(_) => self.collect_construct(expr, out)?,
            _ => return None,
        }
        Some(())
    }

    /// Extract one `fbt.param(name, value)` construct. Anything else (plural,
    /// enum, pronoun, variation params) aborts the whole lowering.
    fn collect_construct(&self, expr: &Expression<'a>, out: &mut Vec<Part<'a>>) -> Option<()> {
        let Expression::CallExpression(call) = expr else { return None };
        let name = self.member_construct_name(&call.callee)?;
        if name != "param" {
            return None;
        }
        let arg0 = call.arguments.first()?.as_expression()?;
        let Expression::StringLiteral(name_lit) = arg0 else { return None };
        let value = call.arguments.get(1)?.as_expression()?;
        if let Some(opts) = call.arguments.get(2).and_then(Argument::as_expression) {
            if object_has_variation(opts) {
                return None;
            }
        }
        out.push(Part::Param {
            name: name_lit.value.as_str().to_string(),
            value: value.clone_in(self.ast.allocator()),
        });
        Some(())
    }

    /// Construct name for a `fbt.param` / `fbt.plural` / ... member callee.
    fn member_construct_name(&self, callee: &Expression<'a>) -> Option<&str> {
        let Expression::StaticMemberExpression(member) = callee else { return None };
        let Expression::Identifier(obj) = &member.object else { return None };
        self.module_for(obj.name.as_str())?;
        Some(member.property.name.as_str())
    }

    fn is_construct_call(&self, expr: &Expression<'a>) -> bool {
        matches!(expr, Expression::CallExpression(call)
            if self.member_construct_name(&call.callee).is_some())
    }

    // --- JSX-form text extraction -------------------------------------------

    fn collect_jsx_children(
        &self,
        children: &ArenaVec<'a, JSXChild<'a>>,
        out: &mut Vec<Part<'a>>,
    ) -> Option<()> {
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    let value = text.value.as_str();
                    // filterEmptyNodes: drop whitespace-only text nodes.
                    if !is_whitespace_only(value) {
                        out.push(Part::Text(value.to_string()));
                    }
                }
                JSXChild::ExpressionContainer(container) => match &container.expression {
                    JSXExpression::EmptyExpression(_) => {}
                    _ => {
                        let expr = container.expression.as_expression()?;
                        if self.is_construct_call(expr) {
                            self.collect_construct(expr, out)?;
                        } else {
                            out.push(Part::Text(expand_string_concat(expr)?));
                        }
                    }
                },
                JSXChild::Element(el) => self.collect_jsx_element_child(el, out)?,
                _ => return None,
            }
        }
        Some(())
    }

    fn collect_jsx_element_child(
        &self,
        el: &JSXElement<'a>,
        out: &mut Vec<Part<'a>>,
    ) -> Option<()> {
        // Only namespaced `fbt:param` is handled. Plain elements would become
        // implicit params; `fbt:plural`/`fbt:enum` build tables — both abort.
        let JSXElementName::NamespacedName(name) = &el.opening_element.name else { return None };
        self.module_for(name.namespace.name.as_str())?;
        if name.name.name.as_str() != "param" {
            return None;
        }
        let (param_name, value) = self.extract_jsx_param(el)?;
        out.push(Part::Param { name: param_name, value });
        Some(())
    }

    /// Extract the `(name, value)` of a `<fbt:param name="...">{value}</fbt:param>`.
    fn extract_jsx_param(&self, el: &JSXElement<'a>) -> Option<(String, Expression<'a>)> {
        let mut param_name = None;
        for attr in &el.opening_element.attributes {
            let JSXAttributeItem::Attribute(attr) = attr else { return None };
            let JSXAttributeName::Identifier(id) = &attr.name else { return None };
            match id.name.as_str() {
                "name" => {
                    let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value else {
                        return None;
                    };
                    let raw = lit.value.as_str();
                    // Multi-line name attributes are space-normalized upstream.
                    param_name = Some(if raw.contains('\n') {
                        normalize_spaces(raw)
                    } else {
                        raw.to_string()
                    });
                }
                // Any variation option (`number`, `gender`) builds a table: abort.
                "number" | "gender" => return None,
                _ => {}
            }
        }
        let param_name = param_name?;

        // The value is the single expression-container or element child.
        let mut value = None;
        for child in &el.children {
            match child {
                JSXChild::Text(text) => {
                    // `<fbt:param> </fbt:param>` is equivalent to `{' '}`.
                    if text.value.as_str() == " " && value.is_none() && el.children.len() == 1 {
                        value = Some(self.string_expr(" "));
                    } else if !is_whitespace_only(text.value.as_str()) {
                        return None;
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if value.is_some() {
                        return None;
                    }
                    let expr = container.expression.as_expression()?;
                    value = Some(expr.clone_in(self.ast.allocator()));
                }
                JSXChild::Element(inner) => {
                    if value.is_some() {
                        return None;
                    }
                    value = Some(Expression::JSXElement(inner.clone_in(self.ast.allocator())));
                }
                _ => return None,
            }
        }
        Some((param_name, value?))
    }

    fn jsx_description(&self, el: &JSXElement<'a>) -> Option<String> {
        for attr in &el.opening_element.attributes {
            let JSXAttributeItem::Attribute(attr) = attr else { continue };
            let JSXAttributeName::Identifier(id) = &attr.name else { continue };
            if id.name.as_str() != "desc" {
                continue;
            }
            let raw = match attr.value.as_ref()? {
                JSXAttributeValue::StringLiteral(lit) => lit.value.as_str().to_string(),
                JSXAttributeValue::ExpressionContainer(container) => {
                    expand_string_concat(container.expression.as_expression()?)?
                }
                _ => return None,
            };
            return Some(normalize_spaces(&raw).trim().to_string());
        }
        None
    }

    // --- runtime call construction ------------------------------------------

    fn build_runtime_call(&self, module: &str, parts: Vec<Part<'a>>, desc: &str) -> Expression<'a> {
        let ast = self.ast;
        let mut text = String::new();
        let mut runtime_params = ArenaVec::new_in(ast);
        for part in parts {
            match part {
                Part::Text(piece) => text.push_str(&piece),
                Part::Param { name, value } => {
                    text.push('{');
                    text.push_str(&name);
                    text.push('}');
                    let mut param_args = ArenaVec::new_in(ast);
                    param_args.push(Argument::from(self.string_expr(&name)));
                    param_args.push(Argument::from(value));
                    runtime_params.push(ArrayExpressionElement::from(
                        self.member_call(module, "_param", param_args),
                    ));
                }
            }
        }
        let text = normalize_spaces(&text).trim().to_string();
        let hk = fbt_hash_key(&text, desc);

        let mut call_args = ArenaVec::new_in(ast);
        call_args.push(Argument::from(self.string_expr(&text)));
        // 2nd arg: the params array, or `null` when there are none.
        if runtime_params.is_empty() {
            call_args.push(Argument::from(Expression::new_null_literal(SPAN, ast)));
        } else {
            call_args.push(Argument::from(Expression::new_array_expression(
                SPAN,
                runtime_params,
                ast,
            )));
        }
        // 3rd arg: `{ hk: "<hash>" }`.
        let hk_key = PropertyKey::new_static_identifier(SPAN, self.atom("hk"), ast);
        let hk_prop = ObjectPropertyKind::new_object_property(
            SPAN,
            PropertyKind::Init,
            hk_key,
            self.string_expr(&hk),
            false,
            false,
            false,
            ast,
        );
        let mut options = ArenaVec::new_in(ast);
        options.push(hk_prop);
        call_args.push(Argument::from(Expression::new_object_expression(SPAN, options, ast)));

        self.member_call(module, "_", call_args)
    }

    /// `<module>.<method>(<args>)`, e.g. `fbt._param(...)`.
    fn member_call(
        &self,
        module: &str,
        method: &str,
        args: ArenaVec<'a, Argument<'a>>,
    ) -> Expression<'a> {
        let ast = self.ast;
        let callee = Expression::new_static_member_expression(
            SPAN,
            Expression::new_identifier(SPAN, self.atom(module), ast),
            IdentifierName::new(SPAN, self.atom(method), ast),
            false,
            ast,
        );
        Expression::new_call_expression(
            SPAN,
            callee,
            None::<oxc_allocator::Box<TSTypeParameterInstantiation>>,
            args,
            false,
            ast,
        )
    }

    fn string_expr(&self, value: &str) -> Expression<'a> {
        Expression::new_string_literal(SPAN, self.atom(value), None, self.ast)
    }

    fn atom(&self, s: &str) -> &'a str {
        oxc_allocator::StringBuilder::from_str_in(s, self.ast.allocator()).into_str()
    }
}

/// `true` when `obj` is `{ number: ... }` or `{ gender: ... }` (a variation).
fn object_has_variation(obj: &Expression) -> bool {
    let Expression::ObjectExpression(obj) = obj else { return false };
    obj.properties.iter().any(|prop| {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else { return false };
        matches!(prop.key.static_name().as_deref(), Some("number" | "gender"))
    })
}

/// Concatenate a static-string expression (string / no-expression template /
/// `+` concatenation). `None` for anything dynamic.
fn expand_string_concat(expr: &Expression) -> Option<String> {
    match expr {
        Expression::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => {
            let mut out = String::new();
            for quasi in &tpl.quasis {
                if let Some(cooked) = &quasi.value.cooked {
                    out.push_str(cooked.as_str());
                }
            }
            Some(out)
        }
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            Some(expand_string_concat(&bin.left)? + &expand_string_concat(&bin.right)?)
        }
        Expression::ParenthesizedExpression(paren) => expand_string_concat(&paren.expression),
        _ => None,
    }
}

fn is_whitespace_only(s: &str) -> bool {
    !s.is_empty() && s.chars().all(char::is_whitespace)
}

/// Collapse runs of whitespace (except U+00A0) to a single space.
/// Mirrors `FbtUtil.normalizeSpaces`: `value.replace(/[^\S ]+/g, ' ')`.
fn normalize_spaces(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_ws = false;
    for c in value.chars() {
        if c.is_whitespace() && c != '\u{00A0}' {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    out
}

/// `fbtHashKey`: base62 of the Jenkins hash of `JSON.stringify(text) + '|' + desc`.
///
/// This is the single-leaf (no token aliases) path of upstream `fbtJenkinsHash`.
fn fbt_hash_key(text: &str, desc: &str) -> String {
    let mut key = json_stringify_str(text);
    key.push('|');
    key.push_str(desc);
    uint_to_base62(jenkins_hash(&key))
}

/// `jenkinsHash` over the UTF-8 bytes of `s` (matches upstream `toUtf8`).
fn jenkins_hash(s: &str) -> u32 {
    let mut hash: u32 = 0;
    for &byte in s.as_bytes() {
        hash = hash.wrapping_add(u32::from(byte));
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash
}

const BASE_N_SYMBOLS: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn uint_to_base62(mut number: u32) -> String {
    let mut out = Vec::new();
    loop {
        out.push(BASE_N_SYMBOLS[(number % 62) as usize]);
        number /= 62;
        if number == 0 {
            break;
        }
    }
    out.reverse();
    // SAFETY: all bytes come from the ASCII `BASE_N_SYMBOLS` table.
    String::from_utf8(out).unwrap()
}

/// `JSON.stringify` for a single string, matching JS semantics: escape the
/// standard control characters and `"` / `\`, keep every other char (incl.
/// non-ASCII) verbatim.
fn json_stringify_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
