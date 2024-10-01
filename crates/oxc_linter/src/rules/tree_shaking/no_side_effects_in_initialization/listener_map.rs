use oxc_ast::{
    ast::{
        match_declaration, match_expression, match_member_expression,
        match_simple_assignment_target, Argument, ArrayExpressionElement, ArrowFunctionExpression,
        AssignmentTarget, BinaryExpression, BindingIdentifier, BindingPattern, BindingPatternKind,
        CallExpression, Class, ClassBody, ClassElement, ComputedMemberExpression,
        ConditionalExpression, Declaration, ExportSpecifier, Expression, ForStatementInit,
        FormalParameter, Function, IdentifierReference, JSXAttribute, JSXAttributeItem,
        JSXAttributeValue, JSXChild, JSXElement, JSXElementName, JSXExpression,
        JSXExpressionContainer, JSXFragment, JSXMemberExpression, JSXOpeningElement,
        LogicalExpression, MemberExpression, ModuleExportName, NewExpression, ObjectExpression,
        ObjectPropertyKind, ParenthesizedExpression, PrivateFieldExpression, Program, PropertyKey,
        SequenceExpression, SimpleAssignmentTarget, Statement, StaticMemberExpression, SwitchCase,
        ThisExpression, UnaryExpression, VariableDeclarator,
    },
    AstKind,
};
use oxc_semantic::{AstNode, NodeId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{LogicalOperator, UnaryOperator};

use crate::{
    ast_util::{get_declaration_of_variable, get_symbol_id_of_variable},
    utils::{
        calculate_binary_operation, calculate_logical_operation, calculate_unary_operation,
        get_write_expr, has_comment_about_side_effect_check, has_pure_notation,
        is_function_side_effect_free, is_local_variable_a_whitelisted_module, is_pure_function,
        no_effects, FunctionName, NodeListenerOptions, Value,
    },
};

pub trait ListenerMap {
    fn report_effects(&self, _options: &NodeListenerOptions) {}
    fn report_effects_when_assigned(&self, _options: &NodeListenerOptions) {}
    fn report_effects_when_called(&self, _options: &NodeListenerOptions) {}
    fn report_effects_when_mutated(&self, _options: &NodeListenerOptions) {}
    fn get_value_and_report_effects(&self, _options: &NodeListenerOptions) -> Value {
        Value::Unknown
    }
}

impl<'a> ListenerMap for Program<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.body.iter().for_each(|stmt| stmt.report_effects(options));
    }
}

impl<'a> ListenerMap for Statement<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::ExpressionStatement(expr_stmt) => {
                expr_stmt.expression.report_effects(options);
            }
            #[allow(clippy::match_same_arms)]
            Self::BreakStatement(_) | Self::ContinueStatement(_) | Self::EmptyStatement(_) => {
                no_effects();
            }
            match_declaration!(Self) => self.to_declaration().report_effects(options),
            Self::ReturnStatement(stmt) => {
                if let Some(arg) = &stmt.argument {
                    arg.report_effects(options);
                }
            }
            Self::ExportAllDeclaration(_) | Self::ImportDeclaration(_) => {
                no_effects();
            }
            Self::ExportDefaultDeclaration(stmt) => {
                if let Some(expr) = &stmt.declaration.as_expression() {
                    if has_comment_about_side_effect_check(expr.span(), options.ctx) {
                        expr.report_effects_when_called(options);
                    }
                    expr.report_effects(options);
                }
            }
            Self::ExportNamedDeclaration(stmt) => {
                stmt.specifiers.iter().for_each(|specifier| {
                    specifier.report_effects(options);
                });

                if let Some(decl) = &stmt.declaration {
                    decl.report_effects(options);
                }
            }
            Self::TryStatement(stmt) => {
                stmt.block.body.iter().for_each(|stmt| stmt.report_effects(options));
                stmt.handler.iter().for_each(|handler| {
                    handler.body.body.iter().for_each(|stmt| stmt.report_effects(options));
                });
                stmt.finalizer.iter().for_each(|finalizer| {
                    finalizer.body.iter().for_each(|stmt| stmt.report_effects(options));
                });
            }
            Self::ThrowStatement(stmt) => {
                options.ctx.diagnostic(super::throw(stmt.span));
            }
            Self::BlockStatement(stmt) => {
                stmt.body.iter().for_each(|stmt| stmt.report_effects(options));
            }
            Self::IfStatement(stmt) => {
                let test_result = stmt.test.get_value_and_report_effects(options);

                if let Some(is_falsy) = test_result.get_falsy_value() {
                    if is_falsy {
                        if let Some(alternate) = &stmt.alternate {
                            alternate.report_effects(options);
                        }
                    } else {
                        stmt.consequent.report_effects(options);
                    }
                } else {
                    stmt.consequent.report_effects(options);
                    if let Some(alternate) = &stmt.alternate {
                        alternate.report_effects(options);
                    }
                }
            }
            Self::DoWhileStatement(stmt) => {
                if stmt
                    .test
                    .get_value_and_report_effects(options)
                    .get_falsy_value()
                    .is_some_and(|is_falsy| is_falsy)
                {
                    return;
                }
                stmt.body.report_effects(options);
            }
            Self::DebuggerStatement(stmt) => {
                options.ctx.diagnostic(super::debugger(stmt.span));
            }
            Self::ForStatement(stmt) => {
                if let Some(init) = &stmt.init {
                    init.report_effects(options);
                }
                if let Some(test) = &stmt.test {
                    test.report_effects(options);
                }
                if let Some(update) = &stmt.update {
                    update.report_effects(options);
                }
                stmt.body.report_effects(options);
            }
            Self::ForInStatement(stmt) => {
                if let Some(assign) = stmt.left.as_assignment_target() {
                    assign.report_effects_when_assigned(options);
                }
                stmt.right.report_effects(options);
                stmt.body.report_effects(options);
            }
            Self::ForOfStatement(stmt) => {
                if let Some(assign) = stmt.left.as_assignment_target() {
                    assign.report_effects_when_assigned(options);
                }
                stmt.right.report_effects(options);
                stmt.body.report_effects(options);
            }
            Self::LabeledStatement(stmt) => {
                stmt.body.report_effects(options);
            }
            Self::WhileStatement(stmt) => {
                if stmt
                    .test
                    .get_value_and_report_effects(options)
                    .get_falsy_value()
                    .is_some_and(|is_falsy| is_falsy)
                {
                    return;
                }
                stmt.body.report_effects(options);
            }
            Self::SwitchStatement(stmt) => {
                stmt.discriminant.report_effects(options);
                stmt.cases.iter().for_each(|case| {
                    case.report_effects(options);
                });
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for ForStatementInit<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            match_expression!(Self) => self.to_expression().report_effects(options),
            Self::VariableDeclaration(decl) => {
                decl.declarations.iter().for_each(|decl| decl.report_effects(options));
            }
        }
    }
}

impl<'a> ListenerMap for ExportSpecifier<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        let ctx = options.ctx;
        let symbol_table = ctx.symbols();
        if has_comment_about_side_effect_check(self.exported.span(), ctx) {
            let ModuleExportName::IdentifierReference(ident) = &self.local else {
                return;
            };
            let Some(symbol_id) = get_symbol_id_of_variable(ident, ctx) else {
                return;
            };

            for reference in symbol_table.get_resolved_references(symbol_id) {
                if reference.is_write() {
                    let node_id = reference.node_id();
                    if let Some(expr) = get_write_expr(node_id, ctx) {
                        expr.report_effects_when_called(options);
                    }
                }
            }
            let symbol_table = ctx.semantic().symbols();
            let node = ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));
            node.report_effects_when_called(options);
        }
    }
}

// we don't need implement all AstNode
// it's same as `reportSideEffectsInDefinitionWhenCalled` in eslint-plugin-tree-shaking
// <https://github.com/lukastaegert/eslint-plugin-tree-shaking/blob/463fa1f0bef7caa2b231a38b9c3557051f506c92/src/rules/no-side-effects-in-initialization.ts#L1070-L1080>
impl<'a> ListenerMap for AstNode<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        match self.kind() {
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    init.report_effects_when_called(options);
                }
            }
            AstKind::FormalParameter(param) => {
                options.ctx.diagnostic(super::call_parameter(param.span));
            }
            AstKind::BindingRestElement(rest) => {
                let start = rest.span.start + 3;
                let end = rest.span.end;
                options.ctx.diagnostic(super::call_parameter(Span::new(start, end)));
            }
            AstKind::Function(function) => {
                let old_val = options.has_valid_this.get();
                options.has_valid_this.set(options.called_with_new.get());
                function.report_effects_when_called(options);
                options.has_valid_this.set(old_val);
            }
            AstKind::Class(class) => {
                class.report_effects_when_called(options);
            }
            AstKind::ImportDefaultSpecifier(specifier) => {
                report_on_imported_call(
                    specifier.local.span,
                    &specifier.local.name,
                    self.id(),
                    options,
                );
            }
            AstKind::ImportSpecifier(specifier) => {
                report_on_imported_call(
                    specifier.local.span,
                    &specifier.local.name,
                    self.id(),
                    options,
                );
            }
            AstKind::ImportNamespaceSpecifier(specifier) => {
                report_on_imported_call(
                    specifier.local.span,
                    &specifier.local.name,
                    self.id(),
                    options,
                );
            }
            _ => {}
        }
    }

    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        match self.kind() {
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    init.report_effects_when_mutated(options);
                }
            }
            AstKind::FormalParameter(param) => {
                options.ctx.diagnostic(super::mutate_parameter(param.span));
            }
            AstKind::BindingRestElement(rest) => {
                let start = rest.span.start + 3;
                let end = rest.span.end;
                options.ctx.diagnostic(super::mutate_parameter(Span::new(start, end)));
            }
            AstKind::ImportDefaultSpecifier(specifier) => {
                options.ctx.diagnostic(super::mutate_import(specifier.span));
            }
            AstKind::ImportSpecifier(specifier) => {
                options.ctx.diagnostic(super::mutate_import(specifier.local.span));
            }
            AstKind::ImportNamespaceSpecifier(specifier) => {
                options.ctx.diagnostic(super::mutate_import(specifier.local.span));
            }
            _ => {}
        }
    }
}

fn report_on_imported_call(span: Span, name: &str, node_id: NodeId, options: &NodeListenerOptions) {
    if has_comment_about_side_effect_check(span, options.ctx) {
        return;
    }
    let Some(AstKind::ImportDeclaration(decl)) = options.ctx.nodes().parent_kind(node_id) else {
        return;
    };
    if is_function_side_effect_free(name, &decl.source.value, options) {
        return;
    }
    options.ctx.diagnostic(super::call_import(span));
}

impl<'a> ListenerMap for Declaration<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::VariableDeclaration(decl) => {
                decl.declarations.iter().for_each(|decl| decl.report_effects(options));
            }
            Self::ClassDeclaration(decl) => {
                decl.report_effects(options);
            }
            Self::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    if has_comment_about_side_effect_check(id.span, options.ctx) {
                        id.report_effects_when_called(options);
                    }
                }
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for Class<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        if let Some(super_class) = &self.super_class {
            super_class.report_effects(options);
        }
        self.body.report_effects(options);
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        if let Some(super_class) = &self.super_class {
            super_class.report_effects_when_called(options);
        }
        self.body.report_effects_when_called(options);
    }
}

impl<'a> ListenerMap for ClassBody<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.body.iter().for_each(|class_element| {
            class_element.report_effects(options);
        });
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        let constructor = self.body.iter().find(|class_element| {
            if let ClassElement::MethodDefinition(definition) = class_element {
                return definition.kind.is_constructor();
            }
            false
        });

        if let Some(constructor) = constructor {
            let old_val = options.has_valid_this.get();
            options.has_valid_this.set(options.called_with_new.get());
            constructor.report_effects_when_called(options);
            options.has_valid_this.set(old_val);
        }

        self.body
            .iter()
            .filter(|class_element| matches!(class_element, ClassElement::PropertyDefinition(_)))
            .for_each(|property_definition| {
                property_definition.report_effects_when_called(options);
            });
    }
}

impl<'a> ListenerMap for ClassElement<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::MethodDefinition(method) => {
                method.key.report_effects(options);
            }
            Self::PropertyDefinition(prop) => {
                prop.key.report_effects(options);
            }
            _ => {}
        }
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        match self {
            Self::MethodDefinition(method) => {
                method.value.report_effects_when_called(options);
            }
            Self::PropertyDefinition(prop) => {
                if let Some(value) = &prop.value {
                    value.report_effects_when_called(options);
                }
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for PropertyKey<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self.as_expression() {
            Some(expr) => expr.report_effects(options),
            None => no_effects(),
        }
    }
}

impl<'a> ListenerMap for VariableDeclarator<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.id.report_effects(options);
        if has_comment_about_side_effect_check(self.id.span(), options.ctx) {
            self.id.report_effects_when_called(options);
        }

        if let Some(init) = &self.init {
            init.report_effects(options);
        }
    }
}

impl<'a> ListenerMap for BindingPattern<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(_) => {}
            BindingPatternKind::ArrayPattern(array) => {
                array.elements.iter().for_each(|el| {
                    if let Some(el) = el {
                        el.report_effects(options);
                    }
                });
            }
            BindingPatternKind::ObjectPattern(object) => {
                object.properties.iter().for_each(|prop| {
                    prop.key.report_effects(options);
                    prop.value.report_effects(options);
                });
            }
            BindingPatternKind::AssignmentPattern(assign_p) => {
                assign_p.left.report_effects(options);
                assign_p.right.report_effects(options);
            }
        }
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        if let BindingPatternKind::BindingIdentifier(ident) = &self.kind {
            ident.report_effects_when_called(options);
        }
    }
}

impl<'a> ListenerMap for BindingIdentifier<'a> {
    fn report_effects(&self, _options: &NodeListenerOptions) {
        no_effects();
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        let ctx = options.ctx;
        if let Some(symbol_id) = self.symbol_id.get() {
            let symbol_table = ctx.semantic().symbols();
            for reference in symbol_table.get_resolved_references(symbol_id) {
                if reference.is_write() {
                    let node_id = reference.node_id();
                    if let Some(expr) = get_write_expr(node_id, ctx) {
                        expr.report_effects_when_called(options);
                    }
                }
            }
            let node = ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));
            node.report_effects_when_called(options);
        }
    }
}

impl<'a> ListenerMap for Expression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::ArrayExpression(array_expr) => {
                array_expr.elements.iter().for_each(|el| el.report_effects(options));
            }
            Self::AssignmentExpression(assign_expr) => {
                assign_expr.left.report_effects_when_assigned(options);
                assign_expr.right.report_effects(options);
            }
            Self::CallExpression(call_expr) => {
                call_expr.report_effects(options);
            }
            Self::ParenthesizedExpression(expr) => {
                expr.report_effects(options);
            }
            Self::NewExpression(expr) => {
                expr.report_effects(options);
            }
            Self::AwaitExpression(expr) => {
                expr.argument.report_effects(options);
            }
            Self::BinaryExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::ClassExpression(expr) => {
                expr.report_effects(options);
            }
            Self::ConditionalExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::JSXElement(expr) => {
                expr.report_effects(options);
            }
            Self::ObjectExpression(expr) => {
                expr.report_effects(options);
            }
            Self::LogicalExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects(options);
            }
            Self::UnaryExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::UpdateExpression(expr) => {
                expr.argument.report_effects_when_assigned(options);
            }
            Self::SequenceExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::YieldExpression(expr) => {
                expr.argument.iter().for_each(|arg| arg.report_effects(options));
            }
            Self::TaggedTemplateExpression(expr) => {
                expr.tag.report_effects_when_called(options);
                expr.quasi.expressions.iter().for_each(|expr| {
                    expr.report_effects(options);
                });
            }
            Self::TemplateLiteral(expr) => {
                expr.expressions.iter().for_each(|expr| {
                    expr.report_effects(options);
                });
            }
            Self::ArrowFunctionExpression(_)
            | Self::FunctionExpression(_)
            | Self::Identifier(_)
            | Self::MetaProperty(_)
            | Self::Super(_)
            | Self::ThisExpression(_) => no_effects(),
            _ => {}
        }
    }

    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        match self {
            Self::Identifier(ident) => {
                ident.report_effects_when_mutated(options);
            }
            Self::ArrowFunctionExpression(_) | Self::ObjectExpression(_) => no_effects(),
            Self::ParenthesizedExpression(expr) => {
                expr.report_effects_when_mutated(options);
            }
            Self::CallExpression(expr) => {
                expr.report_effects_when_mutated(options);
            }
            Self::ThisExpression(expr) => {
                expr.report_effects_when_mutated(options);
            }
            _ => {
                // Default behavior
                options.ctx.diagnostic(super::mutate(self.span()));
            }
        }
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        match self {
            Self::CallExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::Identifier(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::FunctionExpression(expr) => {
                let old_val = options.has_valid_this.get();
                options.has_valid_this.set(options.called_with_new.get());
                expr.report_effects_when_called(options);
                options.has_valid_this.set(old_val);
            }
            Self::ArrowFunctionExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::ParenthesizedExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::ClassExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::ConditionalExpression(expr) => expr.report_effects_when_called(options),
            Self::StaticMemberExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            _ => {
                // Default behavior
                options.ctx.diagnostic(super::call(self.span()));
            }
        }
    }

    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Value {
        match self {
            Self::BooleanLiteral(_)
            | Self::StringLiteral(_)
            | Self::NumericLiteral(_)
            | Self::TemplateLiteral(_) => Value::new(self),
            Self::BinaryExpression(expr) => expr.get_value_and_report_effects(options),
            Self::ConditionalExpression(expr) => expr.get_value_and_report_effects(options),
            Self::LogicalExpression(expr) => expr.get_value_and_report_effects(options),
            Self::SequenceExpression(expr) => expr.get_value_and_report_effects(options),
            _ => {
                self.report_effects(options);
                Value::Unknown
            }
        }
    }
}

// which kind of Expression defines `report_effects_when_called` method.
fn defined_custom_report_effects_when_called(expr: &Expression) -> bool {
    matches!(
        expr.get_inner_expression(),
        Expression::ArrowFunctionExpression(_)
            | Expression::CallExpression(_)
            | Expression::ClassExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::Identifier(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
    )
}

impl<'a> ListenerMap for SwitchCase<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        if let Some(test) = &self.test {
            test.report_effects(options);
        }
        self.consequent.iter().for_each(|stmt| {
            stmt.report_effects(options);
        });
    }
}

impl<'a> ListenerMap for SequenceExpression<'a> {
    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Value {
        let mut val = Value::Unknown;
        for expr in &self.expressions {
            val = expr.get_value_and_report_effects(options);
        }
        val
    }
}

impl<'a> ListenerMap for UnaryExpression<'a> {
    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Value {
        if self.operator == UnaryOperator::Delete {
            match &self.argument {
                Expression::StaticMemberExpression(expr) => {
                    expr.object.report_effects_when_mutated(options);
                }
                Expression::ComputedMemberExpression(expr) => {
                    expr.object.report_effects_when_mutated(options);
                }
                Expression::PrivateFieldExpression(expr) => {
                    expr.object.report_effects_when_mutated(options);
                }
                _ => options.ctx.diagnostic(super::delete(self.argument.span())),
            }
            return Value::Unknown;
        }

        let value = self.argument.get_value_and_report_effects(options);
        calculate_unary_operation(self.operator, value)
    }
}

impl<'a> ListenerMap for LogicalExpression<'a> {
    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Value {
        let left = self.left.get_value_and_report_effects(options);
        // `false && foo`
        if self.operator == LogicalOperator::And
            && left.get_falsy_value().is_some_and(|is_falsy| is_falsy)
        {
            return left;
        }
        // `true || foo`
        if self.operator == LogicalOperator::Or
            && left.get_falsy_value().is_some_and(|is_falsy| !is_falsy)
        {
            return left;
        }
        let right = self.right.get_value_and_report_effects(options);
        calculate_logical_operation(self.operator, left, right)
    }
}

impl<'a> ListenerMap for ObjectExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.properties.iter().for_each(|property| match property {
            ObjectPropertyKind::ObjectProperty(p) => {
                p.key.report_effects(options);
                p.value.report_effects(options);
            }
            ObjectPropertyKind::SpreadProperty(spreed) => {
                spreed.argument.report_effects(options);
            }
        });
    }
}

impl<'a> ListenerMap for JSXElement<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.opening_element.report_effects(options);
        self.children.iter().for_each(|child| {
            child.report_effects(options);
        });
    }
}

impl<'a> ListenerMap for JSXChild<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            JSXChild::Element(element) => {
                element.report_effects(options);
            }
            JSXChild::Spread(spread) => {
                spread.expression.report_effects(options);
            }
            JSXChild::Fragment(fragment) => {
                fragment.report_effects(options);
            }
            JSXChild::ExpressionContainer(container) => {
                container.report_effects(options);
            }
            JSXChild::Text(_) => {
                no_effects();
            }
        }
    }
}

impl<'a> ListenerMap for JSXOpeningElement<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.name.report_effects_when_called(options);
        self.attributes.iter().for_each(|attr| attr.report_effects(options));
    }
}

impl<'a> ListenerMap for JSXElementName<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        match self {
            Self::Identifier(_) | Self::NamespacedName(_) => {}
            Self::IdentifierReference(ident) => ident.report_effects_when_called(options),
            Self::MemberExpression(member) => member.report_effects_when_called(options),
            Self::ThisExpression(expr) => expr.report_effects_when_called(options),
        }
    }
}

impl<'a> ListenerMap for JSXMemberExpression<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        options.ctx.diagnostic(super::call_member(self.property.span()));
    }
}

impl<'a> ListenerMap for JSXAttributeItem<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::Attribute(attribute) => {
                attribute.report_effects(options);
            }
            Self::SpreadAttribute(attribute) => {
                attribute.argument.report_effects(options);
            }
        }
    }
}

impl<'a> ListenerMap for JSXAttribute<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        if let Some(value) = &self.value {
            match value {
                JSXAttributeValue::ExpressionContainer(container) => {
                    container.report_effects(options);
                }
                JSXAttributeValue::Element(element) => {
                    element.report_effects(options);
                }
                JSXAttributeValue::Fragment(fragment) => {
                    fragment.report_effects(options);
                }
                JSXAttributeValue::StringLiteral(_) => {
                    no_effects();
                }
            }
        }
    }
}

impl<'a> ListenerMap for JSXExpressionContainer<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.expression.report_effects(options);
    }
}

impl<'a> ListenerMap for JSXExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::ArrayExpression(array_expr) => {
                array_expr.elements.iter().for_each(|el| el.report_effects(options));
            }
            Self::AssignmentExpression(assign_expr) => {
                assign_expr.left.report_effects_when_assigned(options);
                assign_expr.right.report_effects(options);
            }
            Self::CallExpression(call_expr) => {
                call_expr.report_effects(options);
            }
            Self::ParenthesizedExpression(expr) => {
                expr.report_effects(options);
            }
            Self::NewExpression(expr) => {
                expr.report_effects(options);
            }
            Self::AwaitExpression(expr) => {
                expr.argument.report_effects(options);
            }
            Self::BinaryExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::ClassExpression(expr) => {
                expr.report_effects(options);
            }
            Self::ConditionalExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::JSXElement(expr) => {
                expr.report_effects(options);
            }
            Self::ObjectExpression(expr) => {
                expr.report_effects(options);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects(options);
            }
            Self::UnaryExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::SequenceExpression(expr) => {
                expr.get_value_and_report_effects(options);
            }
            Self::ArrowFunctionExpression(_)
            | Self::EmptyExpression(_)
            | Self::FunctionExpression(_)
            | Self::Identifier(_)
            | Self::MetaProperty(_)
            | Self::Super(_)
            | Self::ThisExpression(_) => no_effects(),
            _ => {}
        }
    }
}

impl<'a> ListenerMap for JSXFragment<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.children.iter().for_each(|child| child.report_effects(options));
    }
}

impl<'a> ListenerMap for ConditionalExpression<'a> {
    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Value {
        let test_result = self.test.get_value_and_report_effects(options);

        if let Some(is_falsy) = test_result.get_falsy_value() {
            if is_falsy {
                self.alternate.get_value_and_report_effects(options)
            } else {
                self.consequent.get_value_and_report_effects(options)
            }
        } else {
            self.consequent.report_effects(options);
            self.alternate.report_effects(options);
            test_result
        }
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        let test_result = self.test.get_value_and_report_effects(options);

        if let Some(falsy) = test_result.get_falsy_value() {
            if falsy {
                self.alternate.report_effects_when_called(options);
            } else {
                self.consequent.report_effects_when_called(options);
            }
        } else {
            self.consequent.report_effects_when_called(options);
            self.alternate.report_effects_when_called(options);
        }
    }
}

impl<'a> ListenerMap for BinaryExpression<'a> {
    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Value {
        let left = self.left.get_value_and_report_effects(options);
        let right = self.right.get_value_and_report_effects(options);
        calculate_binary_operation(self.operator, left, right)
    }
}

impl ListenerMap for ThisExpression {
    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        if !options.has_valid_this.get() {
            options.ctx.diagnostic(super::mutate_of_this(self.span));
        }
    }
}

impl<'a> ListenerMap for NewExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        if has_pure_notation(self.span, options.ctx) {
            return;
        }
        self.arguments.iter().for_each(|arg| arg.report_effects(options));
        let old_val = options.called_with_new.get();
        options.called_with_new.set(true);
        self.callee.report_effects_when_called(options);
        options.called_with_new.set(old_val);
    }
}

impl<'a> ListenerMap for ParenthesizedExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.expression.report_effects(options);
    }

    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        self.expression.report_effects_when_assigned(options);
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.expression.report_effects_when_called(options);
    }

    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        self.expression.report_effects_when_mutated(options);
    }

    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Value {
        self.expression.get_value_and_report_effects(options)
    }
}

impl<'a> ListenerMap for ArrowFunctionExpression<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.params.items.iter().for_each(|param| param.report_effects(options));
        self.body.statements.iter().for_each(|stmt| stmt.report_effects(options));
    }
}

impl<'a> ListenerMap for Function<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.params.items.iter().for_each(|param| param.report_effects(options));

        if let Some(body) = &self.body {
            body.statements.iter().for_each(|stmt| stmt.report_effects(options));
        }
    }
}

impl<'a> ListenerMap for FormalParameter<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.pattern.report_effects(options);
    }
}

impl<'a> ListenerMap for CallExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.arguments.iter().for_each(|arg| arg.report_effects(options));
        if defined_custom_report_effects_when_called(&self.callee) {
            let old_value = options.called_with_new.get();
            options.called_with_new.set(false);
            self.callee.report_effects_when_called(options);
            options.called_with_new.set(old_value);
        } else {
            options.ctx.diagnostic(super::call(self.callee.span()));
        }
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        let ctx = options.ctx;
        if let Expression::Identifier(ident) = &self.callee {
            if let Some(node) = get_declaration_of_variable(ident, ctx) {
                if is_local_variable_a_whitelisted_module(node, ident.name.as_str(), options) {
                    return;
                }
                options.ctx.diagnostic(super::call_return_value(self.span));
            } else {
                options.ctx.diagnostic(super::call_return_value(self.span));
            }
        }
    }

    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        options.ctx.diagnostic(super::mutate_function_return_value(self.span));
    }
}

impl<'a> ListenerMap for Argument<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            match_expression!(Self) => self.to_expression().report_effects(options),
            Self::SpreadElement(spread) => {
                spread.argument.report_effects(options);
            }
        }
    }
}

impl<'a> ListenerMap for AssignmentTarget<'a> {
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        match self {
            match_simple_assignment_target!(Self) => {
                self.to_simple_assignment_target().report_effects_when_assigned(options);
            }
            Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_) => {}
        }
    }
}

impl<'a> ListenerMap for SimpleAssignmentTarget<'a> {
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => {
                ident.report_effects_when_assigned(options);
            }
            match_member_expression!(Self) => {
                self.to_member_expression().report_effects_when_assigned(options);
            }
            _ => {
                // For remain TypeScript AST, just visit its expression
                if let Some(expr) = self.get_expression() {
                    expr.report_effects_when_assigned(options);
                }
            }
        }
    }
}

impl<'a> ListenerMap for IdentifierReference<'a> {
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        if get_symbol_id_of_variable(self, options.ctx).is_none() {
            options.ctx.diagnostic(super::assignment(self.name.as_str(), self.span));
        }
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        if is_pure_function(&FunctionName::Identifier(self), options) {
            return;
        }

        let ctx = options.ctx;

        if let Some(symbol_id) = get_symbol_id_of_variable(self, ctx) {
            let is_used_in_jsx = matches!(
                ctx.nodes().parent_kind(
                    ctx.symbols().get_reference(self.reference_id.get().unwrap()).node_id()
                ),
                Some(AstKind::JSXElementName(_) | AstKind::JSXMemberExpressionObject(_))
            );

            if is_used_in_jsx {
                for reference in options.ctx.symbols().get_resolved_references(symbol_id) {
                    if reference.is_write() {
                        let node_id = reference.node_id();
                        if let Some(expr) = get_write_expr(node_id, options.ctx) {
                            let old_val = options.called_with_new.get();
                            options.called_with_new.set(true);
                            expr.report_effects_when_called(options);
                            options.called_with_new.set(old_val);
                        }
                    }
                }
                let symbol_table = options.ctx.semantic().symbols();
                let node = options.ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));
                let old_val = options.called_with_new.get();
                options.called_with_new.set(true);
                node.report_effects_when_called(options);
                options.called_with_new.set(old_val);
                return;
            }

            if options.insert_called_node(symbol_id) {
                let symbol_table = ctx.semantic().symbols();
                for reference in symbol_table.get_resolved_references(symbol_id) {
                    if reference.is_write() {
                        let node_id = reference.node_id();
                        if let Some(expr) = get_write_expr(node_id, ctx) {
                            expr.report_effects_when_called(options);
                        }
                    }
                }
                let symbol_table = ctx.semantic().symbols();
                let node = ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));
                node.report_effects_when_called(options);
            }
        } else {
            ctx.diagnostic(super::call_global(self.name.as_str(), self.span));
        }
    }

    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        let ctx = options.ctx;
        if let Some(symbol_id) = get_symbol_id_of_variable(self, ctx) {
            if options.insert_mutated_node(symbol_id) {
                for reference in ctx.symbols().get_resolved_references(symbol_id) {
                    if reference.is_write() {
                        let node_id = reference.node_id();
                        if let Some(expr) = get_write_expr(node_id, ctx) {
                            expr.report_effects_when_mutated(options);
                        }
                    }
                }

                let symbol_table = ctx.semantic().symbols();
                let node = ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));
                node.report_effects_when_mutated(options);
            }
        } else {
            ctx.diagnostic(super::mutate_with_name(self.name.as_str(), self.span));
        }
    }
}

impl<'a> ListenerMap for MemberExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects(options);
            }
        }
    }

    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        match self {
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects_when_assigned(options);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects_when_assigned(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects_when_assigned(options);
            }
        }
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        match self {
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects_when_called(options);
            }
        }
    }
}

impl<'a> ListenerMap for ComputedMemberExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.expression.report_effects(options);
        self.object.report_effects(options);
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.report_effects(options);

        let mut node = &self.object;
        loop {
            match node {
                Expression::ComputedMemberExpression(expr) => {
                    node = &expr.object;
                }
                Expression::StaticMemberExpression(expr) => node = &expr.object,
                Expression::PrivateInExpression(expr) => node = &expr.right,
                _ => {
                    break;
                }
            }
        }

        if let Expression::Identifier(ident) = node {
            ident.report_effects_when_called(options);
        } else {
            options.ctx.diagnostic(super::call_member(node.span()));
        }
    }

    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        self.report_effects(options);
        self.object.report_effects_when_mutated(options);
    }
}

impl<'a> ListenerMap for StaticMemberExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.object.report_effects(options);
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.report_effects(options);

        let mut root_member_expr = &self.object;
        loop {
            match root_member_expr {
                Expression::ComputedMemberExpression(expr) => {
                    root_member_expr = &expr.object;
                }
                Expression::StaticMemberExpression(expr) => root_member_expr = &expr.object,
                Expression::PrivateInExpression(expr) => root_member_expr = &expr.right,
                _ => {
                    break;
                }
            }
        }

        let Expression::Identifier(ident) = root_member_expr else {
            options.ctx.diagnostic(super::call_member(root_member_expr.span()));
            return;
        };

        let Some(node) = get_declaration_of_variable(ident, options.ctx) else {
            // If the variable is not declared, it is a global variable.
            // `ext.x()`
            if !is_pure_function(&FunctionName::StaticMemberExpr(self), options) {
                options.ctx.diagnostic(super::call_member(self.span));
            }
            return;
        };

        if is_local_variable_a_whitelisted_module(node, &ident.name, options) {
            return;
        };

        if has_pure_notation(self.span, options.ctx) {
            return;
        }

        options.ctx.diagnostic(super::call_member(self.span));
    }

    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        self.report_effects(options);
        self.object.report_effects_when_mutated(options);
    }
}

impl<'a> ListenerMap for PrivateFieldExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.object.report_effects(options);
    }

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.report_effects(options);

        let mut node = &self.object;
        loop {
            match node {
                Expression::ComputedMemberExpression(expr) => {
                    node = &expr.object;
                }
                Expression::StaticMemberExpression(expr) => node = &expr.object,
                Expression::PrivateInExpression(expr) => node = &expr.right,
                _ => {
                    break;
                }
            }
        }

        if let Expression::Identifier(ident) = node {
            ident.report_effects_when_called(options);
        } else {
            options.ctx.diagnostic(super::call_member(node.span()));
        }
    }

    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        self.report_effects(options);
        self.object.report_effects_when_mutated(options);
    }
}

impl<'a> ListenerMap for ArrayExpressionElement<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            match_expression!(Self) => self.to_expression().report_effects(options),
            Self::SpreadElement(spreed) => {
                spreed.argument.report_effects(options);
            }
            Self::Elision(_) => {}
        }
    }
}
