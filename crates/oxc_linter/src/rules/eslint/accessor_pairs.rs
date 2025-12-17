use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, ClassBody, ClassElement, Expression, MethodDefinitionKind,
        ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind, TSMethodSignatureKind,
        TSSignature, TSTypeLiteral,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{ContentEq, GetSpan, Span};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn setter_without_getter_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Setter is defined without a getter")
        .with_help("Define a getter for this property")
        .with_label(span)
}

fn getter_without_setter_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Getter is defined without a setter")
        .with_help("Define a setter for this property")
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct AccessorPairsConfig {
    /// Report a setter without a getter.
    set_without_get: bool,
    /// Report a getter without a setter.
    get_without_set: bool,
    /// Enforce the rule for class members.
    enforce_for_class_members: bool,
    /// Enforce the rule for TypeScript interfaces and types.
    #[serde(rename = "enforceForTSTypes")]
    enforce_for_ts_types: bool,
}

impl Default for AccessorPairsConfig {
    fn default() -> Self {
        Self {
            set_without_get: true,
            get_without_set: false,
            enforce_for_class_members: true,
            enforce_for_ts_types: false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AccessorPairs(Box<AccessorPairsConfig>);

impl std::ops::Deref for AccessorPairs {
    type Target = AccessorPairsConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces getter/setter pairs in objects and classes.
    ///
    /// ### Why is this bad?
    ///
    /// It's a common mistake in JavaScript to create an object with just a setter
    /// for a property but never have a corresponding getter defined for it.
    /// Without a getter, you cannot read the property, so it ends up not being used.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var o = {
    ///     set a(value) {
    ///         this.val = value;
    ///     }
    /// };
    ///
    /// class C {
    ///     set a(value) {
    ///         this.val = value;
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var o = {
    ///     set a(value) {
    ///         this.val = value;
    ///     },
    ///     get a() {
    ///         return this.val;
    ///     }
    /// };
    ///
    /// class C {
    ///     set a(value) {
    ///         this.val = value;
    ///     }
    ///     get a() {
    ///         return this.val;
    ///     }
    /// }
    /// ```
    AccessorPairs,
    eslint,
    pedantic,
    config = AccessorPairsConfig,
);

impl Rule for AccessorPairs {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = serde_json::from_value::<DefaultRuleConfig<AccessorPairsConfig>>(value)
            .unwrap_or_default()
            .into_inner();
        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(obj) => {
                self.check_object_expression(obj, ctx);
            }
            AstKind::ClassBody(class_body) => {
                if self.enforce_for_class_members {
                    self.check_class_body(class_body, ctx);
                }
            }
            AstKind::CallExpression(call) => {
                self.check_call_expression(call, node, ctx);
            }
            AstKind::TSInterfaceBody(interface_body) => {
                if self.enforce_for_ts_types {
                    self.check_ts_signatures(&interface_body.body, ctx);
                }
            }
            AstKind::TSTypeLiteral(type_literal) => {
                if self.enforce_for_ts_types {
                    self.check_ts_type_literal(type_literal, ctx);
                }
            }
            _ => {}
        }
    }
}

/// Represents accessor information for a property
#[derive(Default)]
struct AccessorInfo {
    getter: Option<Span>,
    setter: Option<Span>,
}

impl AccessorPairs {
    /// Check if two property keys are equivalent
    fn are_keys_equivalent(key1: &PropertyKey, key2: &PropertyKey) -> bool {
        // For expression keys (computed properties), strip parentheses before comparing
        // so that `[a]` and `[(a)]` are considered equivalent
        match (key1.as_expression(), key2.as_expression()) {
            (Some(expr1), Some(expr2)) => {
                expr1.get_inner_expression().content_eq(expr2.get_inner_expression())
            }
            _ => key1.content_eq(key2),
        }
    }

    fn check_object_expression(&self, obj: &ObjectExpression, ctx: &LintContext) {
        let mut accessors: FxHashMap<String, AccessorInfo> = FxHashMap::default();
        let mut computed_accessors: Vec<(&PropertyKey, PropertyKind, Span)> = vec![];

        for prop in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                continue;
            };

            let kind = prop.kind;
            if kind != PropertyKind::Get && kind != PropertyKind::Set {
                continue;
            }

            if let Some(name) = prop.key.static_name() {
                let info = accessors.entry(name.into_owned()).or_default();
                if kind == PropertyKind::Get {
                    info.getter = Some(prop.key.span());
                } else {
                    info.setter = Some(prop.key.span());
                }
            } else {
                computed_accessors.push((&prop.key, kind, prop.key.span()));
            }
        }

        self.report_accessor_issues(&accessors, ctx);

        self.check_computed_accessors(&computed_accessors, ctx);
    }

    fn check_class_body(&self, class_body: &ClassBody, ctx: &LintContext) {
        // Track static and instance accessors separately
        let mut instance_accessors: FxHashMap<String, AccessorInfo> = FxHashMap::default();
        let mut static_accessors: FxHashMap<String, AccessorInfo> = FxHashMap::default();
        let mut computed_instance: Vec<(&PropertyKey, MethodDefinitionKind, Span)> = vec![];
        let mut computed_static: Vec<(&PropertyKey, MethodDefinitionKind, Span)> = vec![];

        for element in &class_body.body {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };

            let kind = method.kind;
            if kind != MethodDefinitionKind::Get && kind != MethodDefinitionKind::Set {
                continue;
            }

            let accessors =
                if method.r#static { &mut static_accessors } else { &mut instance_accessors };

            let computed =
                if method.r#static { &mut computed_static } else { &mut computed_instance };

            if let Some(name) = method.key.static_name() {
                let info = accessors.entry(name.into_owned()).or_default();
                if kind == MethodDefinitionKind::Get {
                    info.getter = Some(method.key.span());
                } else {
                    info.setter = Some(method.key.span());
                }
            } else {
                computed.push((&method.key, kind, method.key.span()));
            }
        }

        self.report_accessor_issues(&instance_accessors, ctx);
        self.report_accessor_issues(&static_accessors, ctx);
        self.check_computed_class_accessors(&computed_instance, ctx);
        self.check_computed_class_accessors(&computed_static, ctx);
    }

    fn check_computed_accessors(
        &self,
        accessors: &[(&PropertyKey, PropertyKind, Span)],
        ctx: &LintContext,
    ) {
        for (i, (key1, kind1, span1)) in accessors.iter().enumerate() {
            let has_pair = accessors.iter().enumerate().any(|(j, (key2, kind2, _))| {
                i != j && kind1 != kind2 && Self::are_keys_equivalent(key1, key2)
            });

            if !has_pair {
                if *kind1 == PropertyKind::Set && self.set_without_get {
                    ctx.diagnostic(setter_without_getter_diagnostic(*span1));
                } else if *kind1 == PropertyKind::Get && self.get_without_set {
                    ctx.diagnostic(getter_without_setter_diagnostic(*span1));
                }
            }
        }
    }

    fn check_computed_class_accessors(
        &self,
        accessors: &[(&PropertyKey, MethodDefinitionKind, Span)],
        ctx: &LintContext,
    ) {
        for (i, (key1, kind1, span1)) in accessors.iter().enumerate() {
            let has_pair = accessors.iter().enumerate().any(|(j, (key2, kind2, _))| {
                i != j && kind1 != kind2 && Self::are_keys_equivalent(key1, key2)
            });

            if !has_pair {
                if *kind1 == MethodDefinitionKind::Set && self.set_without_get {
                    ctx.diagnostic(setter_without_getter_diagnostic(*span1));
                } else if *kind1 == MethodDefinitionKind::Get && self.get_without_set {
                    ctx.diagnostic(getter_without_setter_diagnostic(*span1));
                }
            }
        }
    }

    fn report_accessor_issues(
        &self,
        accessors: &FxHashMap<String, AccessorInfo>,
        ctx: &LintContext,
    ) {
        for info in accessors.values() {
            match (info.getter, info.setter) {
                (None, Some(setter_span)) if self.set_without_get => {
                    ctx.diagnostic(setter_without_getter_diagnostic(setter_span));
                }
                (Some(getter_span), None) if self.get_without_set => {
                    ctx.diagnostic(getter_without_setter_diagnostic(getter_span));
                }
                _ => {}
            }
        }
    }

    /// Check calls to Object.defineProperty, Object.defineProperties, Object.create, Reflect.defineProperty
    fn check_call_expression<'a>(
        &self,
        call: &CallExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(callee) = Self::get_define_property_callee(call, node, ctx) else {
            return;
        };

        match callee {
            DefinePropertyCallee::DefineProperty => {
                // Object.defineProperty(obj, 'prop', descriptor)
                // Reflect.defineProperty(obj, 'prop', descriptor)
                if call.arguments.len() >= 3
                    && let Some(Argument::ObjectExpression(descriptor)) = call.arguments.get(2)
                {
                    self.check_property_descriptor(descriptor, ctx);
                }
            }
            DefinePropertyCallee::DefineProperties | DefinePropertyCallee::Create => {
                // Object.defineProperties(obj, props)
                // Object.create(proto, props)
                if let Some(Argument::ObjectExpression(props)) = call.arguments.get(1) {
                    for prop in &props.properties {
                        if let ObjectPropertyKind::ObjectProperty(prop) = prop
                            && let Expression::ObjectExpression(descriptor) = &prop.value
                        {
                            self.check_property_descriptor(descriptor, ctx);
                        }
                    }
                }
            }
        }
    }

    fn get_define_property_callee<'a>(
        call: &CallExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> Option<DefinePropertyCallee> {
        let callee = call.callee.without_parentheses();

        // Handle optional chaining: Object?.defineProperty
        let member = match callee {
            Expression::StaticMemberExpression(m) => m,
            Expression::ChainExpression(chain) => match &chain.expression {
                oxc_ast::ast::ChainElement::StaticMemberExpression(m) => m,
                _ => return None,
            },
            _ => return None,
        };

        let object_name = match member.object.without_parentheses() {
            Expression::Identifier(id) => &id.name,
            _ => return None,
        };

        // Check if Object/Reflect is the global object (not shadowed by a local variable)
        let is_global = ctx.scoping().find_binding(node.scope_id(), object_name).is_none();
        if !is_global {
            return None;
        }

        let property_name = &member.property.name;

        match (object_name.as_str(), property_name.as_str()) {
            ("Object" | "Reflect", "defineProperty") => Some(DefinePropertyCallee::DefineProperty),
            ("Object", "defineProperties") => Some(DefinePropertyCallee::DefineProperties),
            ("Object", "create") => Some(DefinePropertyCallee::Create),
            _ => None,
        }
    }

    fn check_property_descriptor(&self, descriptor: &ObjectExpression, ctx: &LintContext) {
        let mut has_get = false;
        let mut has_set = false;
        let mut set_span = None;

        for prop in &descriptor.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                continue;
            };

            let Some(name) = prop.key.static_name() else {
                continue;
            };

            match &*name {
                "get" => has_get = true,
                "set" => {
                    has_set = true;
                    set_span = Some(prop.key.span());
                }
                _ => {}
            }
        }

        // Only report setWithoutGet for property descriptors
        // (ESLint doesn't check getWithoutSet for Object.defineProperty)
        if has_set
            && !has_get
            && self.set_without_get
            && let Some(span) = set_span
        {
            ctx.diagnostic(setter_without_getter_diagnostic(span));
        }
    }

    fn check_ts_signatures(&self, signatures: &[TSSignature], ctx: &LintContext) {
        let mut accessors: FxHashMap<String, AccessorInfo> = FxHashMap::default();
        let mut computed_accessors: Vec<(&PropertyKey, TSMethodSignatureKind, Span)> = vec![];

        for sig in signatures {
            let TSSignature::TSMethodSignature(method) = sig else {
                continue;
            };

            let kind = method.kind;
            if kind != TSMethodSignatureKind::Get && kind != TSMethodSignatureKind::Set {
                continue;
            }

            if let Some(name) = method.key.static_name() {
                let info = accessors.entry(name.into_owned()).or_default();
                if kind == TSMethodSignatureKind::Get {
                    info.getter = Some(method.key.span());
                } else {
                    info.setter = Some(method.key.span());
                }
            } else {
                computed_accessors.push((&method.key, kind, method.key.span()));
            }
        }

        self.report_accessor_issues(&accessors, ctx);
        self.report_computed_ts_accessor_issues(&computed_accessors, ctx);
    }

    fn check_ts_type_literal(&self, type_literal: &TSTypeLiteral, ctx: &LintContext) {
        let mut accessors: FxHashMap<String, AccessorInfo> = FxHashMap::default();
        let mut computed_accessors: Vec<(&PropertyKey, TSMethodSignatureKind, Span)> = vec![];

        for member in &type_literal.members {
            let TSSignature::TSMethodSignature(method) = member else {
                continue;
            };

            let kind = method.kind;
            if kind != TSMethodSignatureKind::Get && kind != TSMethodSignatureKind::Set {
                continue;
            }

            if let Some(name) = method.key.static_name() {
                let info = accessors.entry(name.into_owned()).or_default();
                if kind == TSMethodSignatureKind::Get {
                    info.getter = Some(method.key.span());
                } else {
                    info.setter = Some(method.key.span());
                }
            } else {
                computed_accessors.push((&method.key, kind, method.key.span()));
            }
        }

        self.report_accessor_issues(&accessors, ctx);
        self.report_computed_ts_accessor_issues(&computed_accessors, ctx);
    }

    fn report_computed_ts_accessor_issues(
        &self,
        computed_accessors: &[(&PropertyKey, TSMethodSignatureKind, Span)],
        ctx: &LintContext,
    ) {
        for (key, kind, span) in computed_accessors {
            let has_pair = computed_accessors.iter().any(|(other_key, other_kind, _)| {
                other_kind != kind && Self::are_keys_equivalent(key, other_key)
            });

            if has_pair {
                continue;
            }

            match kind {
                TSMethodSignatureKind::Set if self.set_without_get => {
                    ctx.diagnostic(setter_without_getter_diagnostic(*span));
                }
                TSMethodSignatureKind::Get if self.get_without_set => {
                    ctx.diagnostic(getter_without_setter_diagnostic(*span));
                }
                _ => {}
            }
        }
    }
}

enum DefinePropertyCallee {
    DefineProperty,
    DefineProperties,
    Create,
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "var { get: foo } = bar; ({ set: foo } = bar);",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var { set } = foo; ({ get } = foo);",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        ("var o = { get a() {} }", None),
        ("var o = { get a() {} }", Some(serde_json::json!([{}]))),
        (
            "var o = {};",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a: 1 };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a: get };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a: set };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get: function(){} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set: function(foo){} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { [get]: function() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { [set]: function(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": false }])),
        ),
        (
            "var o = { get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false }])),
        ),
        (
            "var o = { set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": false }])),
        ),
        (
            "var o = { set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true }])),
        ),
        ("var o = { set a(foo) {} };", Some(serde_json::json!([{ "setWithoutGet": false }]))),
        (
            "var o = { get a() {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(foo) {}, get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get 'a'() {}, set 'a'(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set 'a'(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get ['abc']() {}, set ['abc'](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [1e2]() {}, set 100(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get abc() {}, set [`abc`](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get ['123']() {}, set 123(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a]() {}, set [a](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a]() {}, set [(a)](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [(a)]() {}, set [a](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a]() {}, set [ a ](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [/*comment*/a/*comment*/]() {}, set [a](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [f()]() {}, set [f()](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [f(a)]() {}, set [f(a)](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a + b]() {}, set [a + b](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [`${a}`]() {}, set [`${a}`](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {}, get b() {}, set b(bar) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set c(foo) {}, set a(bar) {}, get b() {}, get c() {}, set b(baz) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {}, b: bar };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, b, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, ...b, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {}, ...a };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, get a() {}, set a(foo) {}, };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {}, get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(bar) {}, get a() {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false }])),
        ),
        (
            "var o = { set a(foo) {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set a(foo) {}, a };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a, get a() {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, a:1, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = {a: 1};
			 Object.defineProperty(o, 'b',
			{set: function(value) {
			 val = value;
			},
			 get: function() {
			 return val;
			}
			});",
            None,
        ),
        ("var o = {set: function() {}}", None),
        ("Object.defineProperties(obj, {set: {value: function() {}}});", None),
        ("Object.create(null, {set: {value: function() {}}});", None),
        // Shadowed Object should not be flagged
        ("var Object = {}; Object.defineProperty(obj, 'foo', {set: function(value) {}});", None),
        (
            "function f(Object) { Object.defineProperty(obj, 'foo', {set: function(value) {}}); }",
            None,
        ),
        ("var o = {get: function() {}}", Some(serde_json::json!([{ "getWithoutSet": true }]))),
        ("var o = {[set]: function() {}}", None),
        (
            "var set = 'value'; Object.defineProperty(obj, 'foo', {[set]: function(value) {}});",
            None,
        ),
        ("class A { get a() {} }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        ("class A { get #a() {} }", Some(serde_json::json!([{ "enforceForClassMembers": true }]))),
        (
            "class A { set a(foo) {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": false }])),
        ),
        (
            "class A { get a() {} set b(foo) {} static get c() {} static set d(bar) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": false, }]),
            ),
        ),
        (
            "(class A { get a() {} set b(foo) {} static get c() {} static set d(bar) {} });",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": false, }]),
            ),
        ),
        (
            "class A { get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { set a(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set b(foo) {} static get c() {} static set d(bar) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": false, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A {}",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "(class {})",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { constructor () {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static a() {} 'b'() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { [a]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { a() {} static a() {} b() {} static c() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set a(foo) {} get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get a() {} static set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set a(foo) {} static get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "(class { set a(foo) {} get a() {} });",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get 'a'() {} set ['a'](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set [`a`](foo) {} get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get 'a'() {} set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static get 1e2() {} static set [100](foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [a]() {} set [a](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { set [(f())](foo) {} get [(f())]() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set [f(a)](foo) {} static get [f(a)]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set b(foo) {} set a(bar) {} get b() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set a(bar) {} b() {} set c(foo) {} get c() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "(class { get a() {} static set a(foo) {} set a(bar) {} static get a() {} });",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} b() {} set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set a(foo) {} get a() {} b() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { a() {} get b() {} c() {} set b(foo) {} d() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set a(foo) {} static a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static get a() {} static b() {} static set a(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static set a(foo) {} static get a() {} a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} get a() {} set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [a]() {} set [a](foo) {} set [a](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set 'a'(foo) {} get [`a`]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { get a() {} set a(foo) {} a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { a() {} get a() {} set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set a(foo) {} static set a(foo) {} static get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get a() {} static set a(foo) {} static get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set a(foo) {} static get a() {} static a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} a() {} set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set a(foo) {} static a() {} static get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        ("interface I { get prop(): any }", None),
        ("type T = { set prop(value: any) }", None),
        (
            "interface I { get prop(): any, set prop(value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "type T = { get prop(): any, set prop(value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { get prop(): any, between: true, set prop(value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { set prop(value: any), get prop(): any }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { set prop(value: any), get 'prop'(): any }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        ("interface I {}", Some(serde_json::json!([{ "enforceForTSTypes": true }]))),
        (
            "interface I { (...args): void }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { new(...args): unknown }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { prop: () => any }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        ("interface I { method(): any }", Some(serde_json::json!([{ "enforceForTSTypes": true }]))),
        ("type T = { get prop(): any }", Some(serde_json::json!([{ "enforceForTSTypes": true }]))),
    ];

    let fail = vec![
        ("var o = { set a(value) {} };", None),
        ("var o = { set a(value) {} };", Some(serde_json::json!([{}]))),
        (
            "var o = { set a(value) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false }])),
        ),
        (
            "var o = { set a(value) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        ("var o = { get a() {} };", Some(serde_json::json!([{ "getWithoutSet": true }]))),
        (
            "var o = { get abc() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get 'abc'() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get 123() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get 1e2() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get ['abc']() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [`abc`]() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [123]() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [abc]() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [f(abc)]() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a + b]() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set abc(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set 'abc'(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set 123(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set 1e2(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set ['abc'](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set [`abc`](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set [123](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set [abc](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set [f(abc)](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set [a + b](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set b(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(foo) {}, get b() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get 1() {}, set b(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set 1(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set 'a '(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get ' a'() {}, set 'a'(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get ''() {}, set ' '(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get ''() {}, set null(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [`a`]() {}, set b(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a]() {}, set [b](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a]() {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set [a](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [a + b]() {}, set [a - b](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get [`${0} `]() {}, set [`${0}`](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, get b() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(foo) {}, set b(bar) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, set b(foo) {}, set c(foo) {}, get d() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o1 = { get a() {} }, o2 = { set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o1 = { set a(foo) {} }, o2 = { get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, get b() {}, set b(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get b() {}, get a() {}, set b(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get b() {}, set b(foo) {}, get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(foo) {}, get b() {}, set b(bar) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get b() {}, set a(foo) {}, set b(bar) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get b() {}, set b(bar) {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get v1() {}, set i1(foo) {}, get v2() {}, set v2(bar) {}, get i2() {}, set v1(baz) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(foo) {}, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a, get b() {}, c };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a, get b() {}, c, set d(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, a:1 };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a, get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(foo) {}, a:1 };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { a, set a(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, ...b };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get a() {}, ...a };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { set a(foo) {}, ...a };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = { get b() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = {
			  set [
			 a](foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "var o = {d: 1};
			 Object.defineProperty(o, 'c',
			{set: function(value) {
			 val = value;
			}
			});",
            None,
        ),
        ("Reflect.defineProperty(obj, 'foo', {set: function(value) {}});", None),
        ("Object.defineProperties(obj, {foo: {set: function(value) {}}});", None),
        ("Object.create(null, {foo: {set: function(value) {}}});", None),
        (
            "var o = {d: 1};
			 Object?.defineProperty(o, 'c',
			{set: function(value) {
			 val = value;
			}
			});",
            None,
        ),
        ("Reflect?.defineProperty(obj, 'foo', {set: function(value) {}});", None),
        ("Object?.defineProperties(obj, {foo: {set: function(value) {}}});", None),
        ("Object?.create(null, {foo: {set: function(value) {}}});", None),
        (
            "var o = {d: 1};
			 (Object?.defineProperty)(o, 'c',
			{set: function(value) {
			 val = value;
			}
			});",
            None,
        ),
        ("(Reflect?.defineProperty)(obj, 'foo', {set: function(value) {}});", None),
        ("(Object?.defineProperties)(obj, {foo: {set: function(value) {}}});", None),
        ("(Object?.create)(null, {foo: {set: function(value) {}}});", None),
        ("class A { set a(foo) {} }", None),
        ("class A { get a() {} set b(foo) {} }", Some(serde_json::json!([{}]))),
        (
            "class A { get a() {} }",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "class A { set a(foo) {} }",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "class A { static get a() {} }",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "class A { static set a(foo) {} }",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "A = class { get a() {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "A = class { get a() {} set b(foo) {} };",
            Some(serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true }])),
        ),
        (
            "class A { set a(value) {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        (
            "class A { static set a(value) {} }",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        (
            "A = class { set a(value) {} };",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        (
            "(class A { static set a(value) {} });",
            Some(serde_json::json!([{ "enforceForClassMembers": true }])),
        ),
        ("class A { set '#a'(foo) {} }", None),
        ("class A { set #a(foo) {} }", None),
        ("class A { static set '#a'(foo) {} }", None),
        ("class A { static set #a(foo) {} }", None),
        (
            "class A { set a(value) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static set a(value) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "let foo = class A { get a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "(class { get a() {} });",
            Some(serde_json::json!([{ "getWithoutSet": true, "enforceForClassMembers": true }])),
        ),
        (
            "class A { get '#a'() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get #a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get '#a'() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get #a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get abc() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static set 'abc'(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "(class { get 123() {} });",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get 1e2() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { get ['abc']() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set [`abc`](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get [123]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [abc]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get [f(abc)]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { set [a + b](foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get ['constructor']() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set b(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { set a(foo) {} get b() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static get a() {} static set b(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set b(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": false, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set b(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": false, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get 'a '() {} set 'a'(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get 'a'() {} set 1(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get 1() {} set 2(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get ''() {} set null(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} set [a](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [a]() {} set [b](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [a]() {} set [a++](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [a + b]() {} set [a - b](foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get #a() {} set '#a'(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get '#a'() {} set #a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} static set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static get a() {} set a(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set [a](foo) {} static get [a]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set [a](foo) {} get [a]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} get b() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { get a() {} get [b]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [a]() {} get [b]() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { set a(foo) {} set b(bar) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get a() {} static get b() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static set a(foo) {} static set b(bar) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get a() {} set b(foo) {} static set c(bar) {} get d() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} } class B { set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { set a(foo) {} }, class { get a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { get a() {} }, { set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = { get a() {} }, class { set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} get b() {} set b(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { get b() {} get a() {} set b(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set b(foo) {} get b() {} set a(bar) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static get b() {} set a(foo) {} static set b(bar) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set a(foo) {} get b() {} set b(bar) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get b() {} static get a() {} set b(bar) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set b(foo) {} static get a() {} static get b() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get [v1](){} static set i1(foo){} static set v2(bar){} get [i2](){} static get i3(){} set [v1](baz){} static get v2(){} set i4(quux){} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { set a(foo) {} set a(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static get a() {} static get a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { set a(foo) {} set a(foo) {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { a() {} get b() {} c() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { a() {} get b() {} c() {} set d(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static a() {} get b() {} static c() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { a() {} get a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static a() {} set a(foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { a() {} static get b() {} c() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class { static a() {} static set b(foo) {} static c() {} d() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { a() {} static get a() {} a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static set a(foo) {} static a() {} }",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { get a() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "A = class {
			  set [
			 a](foo) {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        (
            "class A { static get b() {} };",
            Some(
                serde_json::json!([{ "setWithoutGet": true, "getWithoutSet": true, "enforceForClassMembers": true, }]),
            ),
        ),
        ("({ set prop(value) {} });", None),
        (
            "interface I { set prop(value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { set prop(value: any), get other(): any }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { set prop(value: any), prop(): any }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { set [prop](value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "interface I { get prop(): any } interface J { set prop(value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "type T = { set prop(value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "function fn(): { set prop(value: any) }",
            Some(serde_json::json!([{ "enforceForTSTypes": true }])),
        ),
        (
            "type T = { get prop(): any }",
            Some(serde_json::json!([{ "enforceForTSTypes": true, "getWithoutSet": true }])),
        ),
    ];

    Tester::new(AccessorPairs::NAME, AccessorPairs::PLUGIN, pass, fail).test_and_snapshot();
}
