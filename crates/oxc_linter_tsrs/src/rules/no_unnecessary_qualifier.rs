// Port of internal/rules/no_unnecessary_qualifier/no_unnecessary_qualifier.go.

use tsrs_ast::{Kind, Node, Symbol, SymbolFlags};
use tsrs_checker::Checker;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_unnecessary_qualifier_message(name: &str) -> RuleMessage {
    RuleMessage::new(
        "unnecessaryQualifier",
        format!("Qualifier is unnecessary since '{name}' is in scope."),
    )
}

pub struct NoUnnecessaryQualifier;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnnecessaryQualifier))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::EnumDeclaration),
    Listener::Exit(Kind::EnumDeclaration),
    Listener::Enter(Kind::ModuleBlock),
    Listener::Exit(Kind::ModuleBlock),
    Listener::Enter(Kind::PropertyAccessExpression),
    Listener::Exit(Kind::PropertyAccessExpression),
    Listener::Enter(Kind::QualifiedName),
    Listener::Exit(Kind::QualifiedName),
];

impl Rule for NoUnnecessaryQualifier {
    fn name(&self) -> &'static str {
        "no-unnecessary-qualifier"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            namespaces_in_scope: Vec::new(),
            current_failed_namespace_expression: None,
        })
    }
}

struct Visitor {
    namespaces_in_scope: Vec<P<Node>>,
    current_failed_namespace_expression: Option<P<Node>>,
}

fn is_entity_name_expression(node: P<Node>) -> bool {
    match node.kind() {
        Kind::Identifier => true,
        Kind::PropertyAccessExpression => {
            is_entity_name_expression(node.as_property_access_expression().expression)
        }
        _ => false,
    }
}

impl Visitor {
    fn exit_declaration(&mut self) {
        self.namespaces_in_scope.pop();
    }

    fn symbol_is_namespace_in_scope(&self, c: &mut Checker, symbol: Option<P<Symbol>>) -> bool {
        let Some(symbol) = symbol else { return false };
        if symbol.declarations().iter().any(|d| self.namespaces_in_scope.contains(d)) {
            return true;
        }
        if utils::is_symbol_flag_set(Some(symbol), SymbolFlags::Alias) {
            let aliased = c.get_aliased_symbol(symbol);
            return self.symbol_is_namespace_in_scope(c, Some(aliased));
        }
        false
    }

    fn qualifier_is_unnecessary(&self, c: &mut Checker, qualifier: P<Node>, name: P<Node>) -> bool {
        let namespace_symbol = c.get_symbol_at_location_exported(qualifier);
        if namespace_symbol.is_none() || !self.symbol_is_namespace_in_scope(c, namespace_symbol) {
            return false;
        }
        let Some(accessed_symbol) = c.get_symbol_at_location_exported(name) else {
            return false;
        };
        let name_text = name.text();
        let in_scope_symbol = c
            .get_symbols_in_scope_exported(qualifier, accessed_symbol.flags())
            .into_iter()
            .find(|s| s.name() == name_text);
        // symbolsAreEqual
        in_scope_symbol
            .is_some_and(|in_scope| accessed_symbol == c.get_export_symbol_of_symbol(in_scope))
    }

    fn visit_namespace_access(
        &mut self,
        ctx: &mut Ctx,
        node: P<Node>,
        qualifier: P<Node>,
        name: P<Node>,
    ) {
        if self.current_failed_namespace_expression.is_some() {
            return;
        }
        // A qualifier can only ever be unnecessary when we are inside an enum or namespace declaration
        // (the only places that populate namespaces_in_scope). The vast majority of property accesses
        // occur outside any such scope, so bail out before doing any (expensive) symbol resolution.
        if self.namespaces_in_scope.is_empty() {
            return;
        }
        if !self.qualifier_is_unnecessary(ctx.checker, qualifier, name) {
            return;
        }

        self.current_failed_namespace_expression = Some(node);
        ctx.report_node_with_fixes(
            qualifier,
            build_unnecessary_qualifier_message(name.text()),
            |ctx| {
                let qualifier_start = ctx.trim(qualifier).0;
                let name_start = ctx.trim(name).0;
                vec![ctx.fix_remove_range(qualifier_start, name_start)]
            },
        );
    }

    fn reset_current_namespace_expression(&mut self, node: P<Node>) {
        if Some(node) == self.current_failed_namespace_expression {
            self.current_failed_namespace_expression = None;
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::EnumDeclaration) => self.namespaces_in_scope.push(node),
            Listener::Exit(Kind::EnumDeclaration) => self.exit_declaration(),
            Listener::Enter(Kind::ModuleBlock) => {
                if let Some(parent) = node.parent() {
                    if parent.kind() == Kind::ModuleDeclaration {
                        self.namespaces_in_scope.push(parent);
                    }
                }
            }
            Listener::Exit(Kind::ModuleBlock)
                if node.parent().is_some_and(|p| p.kind() == Kind::ModuleDeclaration) =>
            {
                self.exit_declaration();
            }
            Listener::Enter(Kind::PropertyAccessExpression) => {
                let pa = node.as_property_access_expression();
                let name = pa.name();
                if is_entity_name_expression(pa.expression) {
                    self.visit_namespace_access(ctx, node, pa.expression, name);
                }
            }
            Listener::Enter(Kind::QualifiedName) => {
                let qn = node.as_qualified_name();
                self.visit_namespace_access(ctx, node, qn.left, qn.right);
            }
            Listener::Exit(Kind::PropertyAccessExpression)
            | Listener::Exit(Kind::QualifiedName) => self.reset_current_namespace_expression(node),
            _ => {}
        }
    }
}
