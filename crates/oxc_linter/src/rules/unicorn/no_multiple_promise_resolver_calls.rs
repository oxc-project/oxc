use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::{VisitJs, walk_js};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{
    GlobalContext,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{IsGlobalReference, NodeId, Reference, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    operator::{AssignmentOperator, LogicalOperator, UnaryOperator},
    scope::ScopeFlags,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{AstNode, context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct NoMultiplePromiseResolverCalls;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows calling Promise executor resolver functions more than once on the same execution path.
    ///
    /// ### Why is this bad?
    ///
    /// A Promise settles only once. Subsequent calls to either resolver have no effect,
    /// and can hide a missing return or an incorrectly handled error.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new Promise((resolve, reject) => {
    ///   read((error, value) => {
    ///     if (error) reject(error);
    ///     resolve(value);
    ///   });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// new Promise((resolve, reject) => {
    ///   read((error, value) => {
    ///     if (error) return reject(error);
    ///     resolve(value);
    ///   });
    /// });
    /// ```
    NoMultiplePromiseResolverCalls,
    unicorn,
    nursery,
    version = "next",
    short_description = "Disallow calling Promise executor resolvers multiple times on the same execution path.",
);

impl Rule for NoMultiplePromiseResolverCalls {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new) = node.kind() else { return };
        let Expression::Identifier(callee) = new.callee.get_inner_expression() else { return };
        if callee.name != "Promise"
            || !callee.is_global_reference(ctx.scoping())
            || new.arguments.len() != 1
        {
            return;
        }
        let Some(executor) = new.arguments[0].as_expression().map(Expression::get_inner_expression)
        else {
            return;
        };
        let params = match executor {
            Expression::FunctionExpression(function) if !function.generator => &function.params,
            Expression::ArrowFunctionExpression(function) => &function.params,
            _ => return,
        };
        let resolvers: Vec<_> = params
            .items
            .iter()
            .take(2)
            .filter_map(|parameter| {
                let BindingPattern::BindingIdentifier(binding) = &parameter.pattern else {
                    return None;
                };
                let symbol = binding.symbol_id();
                (!ctx.scoping().get_resolved_references(symbol).any(Reference::is_write))
                    .then_some(symbol)
            })
            .collect();
        if resolvers.is_empty() {
            return;
        }
        let mut analysis = Analysis {
            ctx,
            resolvers,
            reported: FxHashSet::default(),
            analyzed: FxHashSet::default(),
            // There are only three reachable input masks. Reusing a statement transfer
            // prevents nested loop fixed points from repeatedly traversing the same
            // subtree, and diagnostics are collected in a set before emission.
            statement_cache: FxHashMap::default(),
        };
        if ctx.cfg().basic_block(ctx.nodes().cfg_id(node.id())).is_unreachable()
            || analysis.never_executed(node.id(), node.kind().span())
        {
            return;
        }
        analysis.function_expression(executor, true);
        let mut spans: Vec<_> = analysis.reported.into_iter().collect();
        spans.sort_unstable_by_key(|span| span.start);
        for span in spans {
            ctx.diagnostic(OxcDiagnostic::warn("Promise executor resolver functions may be called multiple times on the same execution path.")
                .with_help("Return after settling the Promise, or make the resolver calls mutually exclusive.")
                .with_label(span));
        }
    }
}

// A two-bit may-state: both bits may be present after control-flow paths merge.
// No state represents an unreachable path. Transfers only add bits at loop joins,
// so even nested loops reach a finite fixed point without enumerating paths.
const UNCALLED: u8 = 1;
const CALLED: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Completion<'a> {
    Return,
    Throw,
    Break(Option<&'a str>),
    Continue(Option<&'a str>, bool),
}

#[derive(Default, Clone)]
struct Flow<'a> {
    normal: u8,
    abrupt: FxHashMap<Completion<'a>, u8>,
}

impl<'a> Flow<'a> {
    fn normal(state: u8) -> Self {
        Self { normal: state, ..Self::default() }
    }
    fn add(&mut self, completion: Completion<'a>, state: u8) {
        if state != 0 {
            *self.abrupt.entry(completion).or_default() |= state;
        }
    }
    fn merge(&mut self, other: Self) {
        self.normal |= other.normal;
        for (completion, state) in other.abrupt {
            self.add(completion, state);
        }
    }
    fn then(&mut self, next: Self) {
        self.normal = next.normal;
        for (completion, state) in next.abrupt {
            self.add(completion, state);
        }
    }
    fn may_throw(&mut self) {
        self.add(Completion::Throw, self.normal);
    }
    fn finish(&mut self, completion: Completion<'a>) {
        self.add(completion, self.normal);
        self.normal = 0;
    }
    fn take(&mut self, completion: Completion<'a>) -> u8 {
        self.abrupt.remove(&completion).unwrap_or(0)
    }
}

// The native CFG provides function boundaries but its broad error harness edges
// do not identify the expression that can throw, and its finally joins merge
// normal and abrupt completion. Keep an ordered, completion-aware local overlay
// so an exception in resolve(mayThrow()) precedes settlement, and a finalizer can
// replace a pending return/throw without introducing a spurious continuation.
struct Analysis<'a, 'c> {
    ctx: &'c LintContext<'a>,
    resolvers: Vec<SymbolId>,
    reported: FxHashSet<Span>,
    analyzed: FxHashSet<NodeId>,
    // There are only three reachable input masks. Reusing a statement transfer
    // prevents nested loop fixed points from repeatedly traversing the same
    // subtree, and diagnostics are collected in a set before emission.
    statement_cache: FxHashMap<(NodeId, u8, Option<&'a str>), Flow<'a>>,
}

impl<'a> Analysis<'a, '_> {
    fn never_executed(&self, node_id: NodeId, span: Span) -> bool {
        let contains = |outer: Span| outer.start <= span.start && span.end <= outer.end;
        for ancestor in self.ctx.nodes().ancestors(node_id) {
            match ancestor.kind() {
                AstKind::WhileStatement(statement)
                    if contains(statement.body.span())
                        && self.truthiness(&statement.test) == Some(false) =>
                {
                    return true;
                }
                AstKind::ForStatement(statement)
                    if (contains(statement.body.span())
                        || statement
                            .update
                            .as_ref()
                            .is_some_and(|update| contains(update.span())))
                        && statement
                            .test
                            .as_ref()
                            .is_some_and(|test| self.truthiness(test) == Some(false)) =>
                {
                    return true;
                }
                AstKind::IfStatement(statement)
                    if (contains(statement.consequent.span())
                        && self.truthiness(&statement.test) == Some(false))
                        || (statement
                            .alternate
                            .as_ref()
                            .is_some_and(|alternate| contains(alternate.span()))
                            && self.truthiness(&statement.test) == Some(true)) =>
                {
                    return true;
                }
                AstKind::ForOfStatement(statement)
                    if contains(statement.body.span()) && self.empty_iterable(&statement.right) =>
                {
                    return true;
                }
                AstKind::ForInStatement(statement)
                    if contains(statement.body.span()) && self.empty_iterable(&statement.right) =>
                {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn is_resolver(&self, expr: &Expression<'a>) -> bool {
        let Expression::Identifier(identifier) = expr.get_inner_expression() else { return false };
        self.ctx
            .scoping()
            .get_reference(identifier.reference_id())
            .symbol_id()
            .is_some_and(|symbol| self.resolvers.contains(&symbol))
    }

    fn is_process_exit(&self, call: &CallExpression<'a>) -> bool {
        let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
            return false;
        };
        let Expression::Identifier(object) = &member.object else { return false };
        !call.optional
            && !member.optional
            && member.property.name == "exit"
            && object.name == "process"
            && object.is_global_reference(self.ctx.scoping())
    }

    fn function_expression(&mut self, expression: &Expression<'a>, executor: bool) {
        match expression {
            Expression::FunctionExpression(function) => self.function(function, executor),
            Expression::ArrowFunctionExpression(function)
                if self.analyzed.insert(function.node_id()) =>
            {
                let mut flow = self
                    .parameters(&function.params, executor || self.is_executor(function.node_id()));
                match &function.body {
                    ArrowFunctionBody::FunctionBody(body) => {
                        flow.then(self.statements(&body.statements, flow.normal));
                    }
                    _ => flow
                        .then(self.expression(function.body.as_expression().unwrap(), flow.normal)),
                }
            }
            _ => {}
        }
    }

    fn function(&mut self, function: &Function<'a>, executor: bool) {
        if self.analyzed.insert(function.node_id()) {
            let flow =
                self.parameters(&function.params, executor || self.is_executor(function.node_id()));
            if let Some(body) = &function.body {
                self.statements(&body.statements, flow.normal);
            }
        }
    }

    fn is_executor(&self, node_id: NodeId) -> bool {
        for ancestor in self.ctx.nodes().ancestors(node_id) {
            match ancestor.kind() {
                AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSTypeAssertion(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSInstantiationExpression(_) => {}
                AstKind::NewExpression(new) => {
                    return new.arguments.len() == 1
                        && matches!(new.callee.get_inner_expression(), Expression::Identifier(identifier) if identifier.name == "Promise" && identifier.is_global_reference(self.ctx.scoping()));
                }
                _ => return false,
            }
        }
        false
    }

    fn parameters(&mut self, params: &FormalParameters<'a>, executor: bool) -> Flow<'a> {
        let mut flow = Flow::normal(UNCALLED);
        for (index, parameter) in params.items.iter().enumerate() {
            if let Some(initializer) = &parameter.initializer
                && (!executor || index >= 2)
            {
                // Promise supplies exactly two arguments, so later defaults
                // always run; defaults in other functions may run.
                let default = self.expression(initializer, flow.normal);
                if executor {
                    flow.then(default);
                } else {
                    flow.merge(default);
                }
            }
            // A supplied argument skips only the parameter's outer default.
            // Nested destructuring defaults still execute when a property is
            // missing, and can call another captured resolver.
            flow.then(self.binding(&parameter.pattern, flow.normal));
        }
        flow
    }

    fn statements(&mut self, statements: &[Statement<'a>], state: u8) -> Flow<'a> {
        let mut flow = Flow::normal(state);
        for statement in statements {
            if flow.normal == 0 {
                break;
            }
            flow.then(self.statement(statement, flow.normal, None));
        }
        flow
    }

    fn statement(
        &mut self,
        statement: &Statement<'a>,
        state: u8,
        label: Option<&'a str>,
    ) -> Flow<'a> {
        if state == 0 {
            return Flow::default();
        }
        let key = (statement.node_id(), state, label);
        if let Some(flow) = self.statement_cache.get(&key) {
            return flow.clone();
        }
        let flow = self.statement_inner(statement, state, label);
        self.statement_cache.insert(key, flow.clone());
        flow
    }

    fn statement_inner(
        &mut self,
        statement: &Statement<'a>,
        state: u8,
        label: Option<&'a str>,
    ) -> Flow<'a> {
        if state == 0 {
            return Flow::default();
        }
        match statement {
            Statement::BlockStatement(block) => self.statements(&block.body, state),
            Statement::ExpressionStatement(statement) => {
                self.expression(&statement.expression, state)
            }
            Statement::VariableDeclaration(declaration) => self.declaration(declaration, state),
            Statement::FunctionDeclaration(function) => {
                self.function(function, false);
                Flow::normal(state)
            }
            Statement::ClassDeclaration(class) => self.class(class, state),
            Statement::ReturnStatement(statement) => {
                let mut flow = statement
                    .argument
                    .as_ref()
                    .map_or_else(|| Flow::normal(state), |expr| self.expression(expr, state));
                flow.finish(Completion::Return);
                flow
            }
            Statement::ThrowStatement(statement) => {
                let mut flow = self.expression(&statement.argument, state);
                flow.finish(Completion::Throw);
                flow
            }
            Statement::BreakStatement(statement) => {
                let mut flow = Flow::normal(state);
                flow.finish(Completion::Break(
                    statement.label.as_ref().map(|label| label.name.as_str()),
                ));
                flow
            }
            Statement::ContinueStatement(statement) => {
                let mut flow = Flow::normal(state);
                flow.finish(Completion::Continue(
                    statement.label.as_ref().map(|label| label.name.as_str()),
                    false,
                ));
                flow
            }
            Statement::IfStatement(statement) => {
                let mut flow = self.expression(&statement.test, state);
                let input = flow.normal;
                flow.normal = 0;
                let test = self.truthiness(&statement.test);
                if test != Some(false) {
                    flow.merge(self.statement(&statement.consequent, input, None));
                }
                if test != Some(true) {
                    flow.merge(statement.alternate.as_ref().map_or_else(
                        || Flow::normal(input),
                        |alternate| self.statement(alternate, input, None),
                    ));
                }
                flow
            }
            Statement::LabeledStatement(statement) => {
                let name = statement.label.name.as_str();
                let mut flow = self.statement(&statement.body, state, Some(name));
                flow.normal |= flow.take(Completion::Break(Some(name)));
                flow
            }
            Statement::WhileStatement(statement) => {
                self.loop_flow(&statement.body, Some(&statement.test), None, state, false, label)
            }
            Statement::DoWhileStatement(statement) => {
                self.loop_flow(&statement.body, Some(&statement.test), None, state, true, label)
            }
            Statement::ForStatement(statement) => {
                let mut flow = Flow::normal(state);
                if let Some(init) = &statement.init {
                    flow.then(match init {
                        ForStatementInit::VariableDeclaration(declaration) => {
                            self.declaration(declaration, flow.normal)
                        }
                        _ => self.expression(init.as_expression().unwrap(), flow.normal),
                    });
                }
                flow.then(self.loop_flow(
                    &statement.body,
                    statement.test.as_ref(),
                    statement.update.as_ref(),
                    flow.normal,
                    false,
                    label,
                ));
                flow
            }
            Statement::ForOfStatement(statement) => {
                self.iteration(&statement.left, &statement.right, &statement.body, state, label)
            }
            Statement::ForInStatement(statement) => {
                self.iteration(&statement.left, &statement.right, &statement.body, state, label)
            }
            Statement::TryStatement(statement) => self.try_statement(statement, state),
            Statement::SwitchStatement(statement) => self.switch(statement, state),
            Statement::WithStatement(statement) => {
                let mut flow = self.expression(&statement.object, state);
                flow.may_throw();
                flow.then(self.statement(&statement.body, flow.normal, None));
                flow
            }
            _ => {
                let mut visitor = FlowVisitor { analysis: self, flow: Flow::normal(state) };
                walk_js::walk_statement(&mut visitor, statement);
                visitor.flow
            }
        }
    }

    fn declaration(&mut self, declaration: &VariableDeclaration<'a>, state: u8) -> Flow<'a> {
        let mut flow = Flow::normal(state);
        for declarator in &declaration.declarations {
            if let Some(init) = &declarator.init {
                flow.then(self.expression(init, flow.normal));
            }
            flow.then(self.binding(&declarator.id, flow.normal));
        }
        if declaration.kind.is_using() {
            flow.may_throw();
        }
        flow
    }

    fn binding(&mut self, pattern: &BindingPattern<'a>, state: u8) -> Flow<'a> {
        let mut flow = Flow::normal(state);
        match pattern {
            BindingPattern::BindingIdentifier(_) => {}
            BindingPattern::AssignmentPattern(pattern) => {
                flow.merge(self.expression(&pattern.right, state));
                flow.then(self.binding(&pattern.left, flow.normal));
            }
            BindingPattern::ArrayPattern(pattern) => {
                flow.may_throw();
                for element in pattern.elements.iter().flatten() {
                    flow.then(self.binding(element, flow.normal));
                }
                if let Some(rest) = &pattern.rest {
                    flow.then(self.binding(&rest.argument, flow.normal));
                }
            }
            BindingPattern::ObjectPattern(pattern) => {
                flow.may_throw();
                for property in &pattern.properties {
                    if property.computed
                        && let Some(key) = property.key.as_expression()
                    {
                        flow.then(self.expression(key, flow.normal));
                    }
                    flow.then(self.binding(&property.value, flow.normal));
                }
                if let Some(rest) = &pattern.rest {
                    flow.then(self.binding(&rest.argument, flow.normal));
                }
            }
        }
        flow
    }

    fn loop_labels(&self, body: &Statement<'a>, label: Option<&'a str>) -> Vec<&'a str> {
        let mut labels: Vec<_> = label.into_iter().collect();
        let mut ancestors = self.ctx.nodes().ancestors(body.node_id());
        ancestors.next(); // The loop owning this body.
        for ancestor in ancestors {
            let AstKind::LabeledStatement(statement) = ancestor.kind() else {
                break;
            };
            let name = statement.label.name.as_str();
            if !labels.contains(&name) {
                labels.push(name);
            }
        }
        labels
    }

    fn loop_flow(
        &mut self,
        body: &Statement<'a>,
        test: Option<&Expression<'a>>,
        update: Option<&Expression<'a>>,
        state: u8,
        do_while: bool,
        label: Option<&'a str>,
    ) -> Flow<'a> {
        let mut output = Flow::default();
        let labels = self.loop_labels(body, label);
        let mut entry = state;
        loop {
            let mut condition = if do_while {
                Flow::normal(entry)
            } else {
                test.map_or_else(|| Flow::normal(entry), |test| self.expression(test, entry))
            };
            let truth = test.and_then(|test| self.truthiness(test)).or(if test.is_none() {
                Some(true)
            } else {
                None
            });
            let input = condition.normal;
            condition.normal = 0;
            if !do_while && truth != Some(true) {
                output.normal |= input;
            }
            output.merge(condition);
            let mut iteration = if do_while || truth != Some(false) {
                self.statement(body, input, None)
            } else {
                Flow::default()
            };
            output.normal |= iteration.take(Completion::Break(None));
            for label in &labels {
                output.normal |= iteration.take(Completion::Break(Some(label)));
            }
            let mut next = iteration.normal | iteration.take(Completion::Continue(None, false));
            for label in &labels {
                next |= iteration.take(Completion::Continue(Some(label), false));
            }
            iteration.normal = 0;
            // Match upstream v79's ignored continue-across-finally edges. Its
            // valid fixtures include `try { resolve(); continue; } finally {
            // cleanup(); }` and a break replaced by a finalizer's continue.
            // An ordinary `try { resolve(); } finally { continue; }` remains a
            // loop violation (covered by the upstream invalid fixtures). This
            // is the rule's compatibility boundary, not a claim that a callback
            // or a finally-crossing continue cannot run more than once.
            iteration
                .abrupt
                .retain(|completion, _| !matches!(completion, Completion::Continue(_, true)));
            output.merge(iteration);
            if let Some(update) = update {
                let mut flow = self.expression(update, next);
                next = flow.normal;
                flow.normal = 0;
                output.merge(flow);
            }
            if do_while {
                let mut condition =
                    test.map_or_else(|| Flow::normal(next), |test| self.expression(test, next));
                if truth != Some(true) {
                    output.normal |= condition.normal;
                }
                next = if truth == Some(false) { 0 } else { condition.normal };
                condition.normal = 0;
                output.merge(condition);
            }
            let merged = entry | next;
            if merged == entry {
                break;
            }
            entry = merged;
        }
        output
    }

    fn iteration(
        &mut self,
        left: &ForStatementLeft<'a>,
        right: &Expression<'a>,
        body: &Statement<'a>,
        state: u8,
        label: Option<&'a str>,
    ) -> Flow<'a> {
        let mut output = self.expression(right, state);
        if self.empty_iterable(right) {
            return output;
        }
        let labels = self.loop_labels(body, label);
        let mut entry = output.normal;
        output.normal = 0;
        loop {
            // Getting the next iterator value can throw before the body runs,
            // including on a later iteration after a resolver call.
            let mut header = Flow::normal(entry);
            header.may_throw();
            if let ForStatementLeft::VariableDeclaration(declaration) = left {
                header.then(self.declaration(declaration, header.normal));
            } else {
                header.may_throw();
            }
            let mut body_flow = self.statement(body, header.normal, None);
            output.normal |= header.normal | body_flow.take(Completion::Break(None));
            for label in &labels {
                output.normal |= body_flow.take(Completion::Break(Some(label)));
            }
            let mut next = body_flow.normal | body_flow.take(Completion::Continue(None, false));
            for label in &labels {
                next |= body_flow.take(Completion::Continue(Some(label), false));
            }
            body_flow.normal = 0;
            body_flow
                .abrupt
                .retain(|completion, _| !matches!(completion, Completion::Continue(_, true)));
            header.normal = 0;
            output.merge(header);
            output.merge(body_flow);
            let merged = entry | next;
            if merged == entry {
                break;
            }
            entry = merged;
        }
        output
    }

    fn try_statement(&mut self, statement: &TryStatement<'a>, state: u8) -> Flow<'a> {
        let mut flow = self.statements(&statement.block.body, state);
        if let Some(handler) = &statement.handler {
            let thrown = flow.take(Completion::Throw);
            let mut caught = Flow::normal(thrown);
            if let Some(parameter) = &handler.param {
                caught.then(self.binding(&parameter.pattern, caught.normal));
            }
            caught.then(self.statements(&handler.body.body, caught.normal));
            flow.merge(caught);
        }
        if let Some(finalizer) = &statement.finalizer {
            let mut output = Flow::default();
            let normal = self.statements(&finalizer.body, flow.normal);
            output.merge(normal);
            for (completion, state) in flow.abrupt {
                let mut final_flow = self.statements(&finalizer.body, state);
                let resumed = final_flow.normal;
                final_flow.normal = 0;
                let emitted_continues: Vec<_> = final_flow
                    .abrupt
                    .iter()
                    .filter_map(|(completion, state)| match completion {
                        Completion::Continue(label, false) => Some((*label, *state)),
                        _ => None,
                    })
                    .collect();
                for (label, state) in emitted_continues {
                    final_flow.take(Completion::Continue(label, false));
                    final_flow.add(Completion::Continue(label, true), state);
                }
                output.merge(final_flow);
                let completion = match completion {
                    Completion::Continue(label, _) => Completion::Continue(label, true),
                    other => other,
                };
                output.add(completion, resumed);
            }
            flow = output;
        }
        flow
    }

    fn switch(&mut self, statement: &SwitchStatement<'a>, state: u8) -> Flow<'a> {
        let mut output = self.expression(&statement.discriminant, state);
        let mut searching = output.normal;
        output.normal = 0;
        let mut entries = vec![0; statement.cases.len()];
        let mut default = None;
        for (index, case) in statement.cases.iter().enumerate() {
            if let Some(test) = &case.test {
                let mut flow = self.expression(test, searching);
                searching = flow.normal;
                entries[index] = searching;
                flow.normal = 0;
                output.merge(flow);
            } else {
                default = Some(index);
                entries[index] = searching;
            }
        }
        if let Some(default) = default {
            entries[default] |= searching;
        } else {
            output.normal |= searching;
        }
        let mut falling = 0;
        for (case, entry) in statement.cases.iter().zip(entries) {
            let mut flow = self.statements(&case.consequent, falling | entry);
            falling = flow.normal;
            flow.normal = 0;
            output.normal |= flow.take(Completion::Break(None));
            output.merge(flow);
        }
        output.normal |= falling;
        output
    }

    fn expression(&mut self, expression: &Expression<'a>, state: u8) -> Flow<'a> {
        if state == 0 {
            return Flow::default();
        }
        let expression = expression.get_inner_expression();
        match expression {
            Expression::CallExpression(call) => self.call(call, state),
            Expression::ChainExpression(chain) => self.chain(&chain.expression, state),
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
                self.function_expression(expression, false);
                Flow::normal(state)
            }
            Expression::ClassExpression(class) => self.class(class, state),
            Expression::SequenceExpression(sequence) => {
                let mut flow = Flow::normal(state);
                for expression in &sequence.expressions {
                    flow.then(self.expression(expression, flow.normal));
                }
                flow
            }
            Expression::ConditionalExpression(conditional) => {
                let mut flow = self.expression(&conditional.test, state);
                let input = flow.normal;
                flow.normal = 0;
                let truth = self.truthiness(&conditional.test);
                if truth != Some(false) {
                    flow.merge(self.expression(&conditional.consequent, input));
                }
                if truth != Some(true) {
                    flow.merge(self.expression(&conditional.alternate, input));
                }
                flow
            }
            Expression::LogicalExpression(logical) => {
                let mut flow = self.expression(&logical.left, state);
                let truth = match logical.operator {
                    LogicalOperator::And => self.truthiness(&logical.left),
                    LogicalOperator::Or => self.truthiness(&logical.left).map(|truth| !truth),
                    LogicalOperator::Coalesce => self.nullish(&logical.left),
                };
                if truth != Some(false) {
                    let right = self.expression(&logical.right, flow.normal);
                    if truth == Some(true) {
                        flow.then(right);
                    } else {
                        flow.merge(right);
                    }
                }
                flow
            }
            Expression::StaticMemberExpression(member) => {
                self.member(&member.object, None, member.optional, state)
            }
            Expression::ComputedMemberExpression(member) => {
                self.member(&member.object, Some(&member.expression), member.optional, state)
            }
            Expression::PrivateFieldExpression(member) => {
                self.member(&member.object, None, member.optional, state)
            }
            Expression::NewExpression(new) => {
                let mut flow = self.reference(&new.callee, state);
                for argument in &new.arguments {
                    flow.then(self.argument(argument, flow.normal));
                }
                flow.may_throw();
                flow
            }
            Expression::AwaitExpression(await_) => {
                let mut flow = self.expression(&await_.argument, state);
                if !matches!(await_.argument.get_inner_expression(),Expression::CallExpression(call) if self.is_resolver(&call.callee))
                {
                    flow.may_throw();
                }
                flow
            }
            Expression::YieldExpression(yield_) => {
                let mut flow = yield_.argument.as_ref().map_or_else(
                    || Flow::normal(state),
                    |argument| self.expression(argument, state),
                );
                flow.may_throw();
                flow
            }
            Expression::ImportExpression(import) => {
                let mut flow = self.expression(&import.source, state);
                if let Some(options) = &import.options {
                    flow.then(self.expression(options, flow.normal));
                }
                flow
            }
            Expression::BinaryExpression(binary) => {
                let mut flow = self.expression(&binary.left, state);
                flow.then(self.expression(&binary.right, flow.normal));
                flow
            }
            Expression::UnaryExpression(unary) => self.expression(&unary.argument, state),
            Expression::UpdateExpression(update) => {
                let mut flow = self.simple_assignment_target(&update.argument, state, true);
                // Coercion and PutValue happen after the target's object/key and
                // GetValue, so a resolver in the target precedes these errors.
                flow.may_throw();
                flow
            }
            Expression::AssignmentExpression(assignment) => {
                if assignment.left.as_simple_assignment_target().is_none() {
                    // Destructuring evaluates the RHS before any computed key,
                    // target reference, or default initializer in the pattern.
                    let mut flow = self.expression(&assignment.right, state);
                    flow.then(self.assignment_pattern(&assignment.left, flow.normal));
                    return flow;
                }
                let mut flow = self.assignment_target_before(
                    &assignment.left,
                    state,
                    assignment.operator != AssignmentOperator::Assign,
                );
                let optional = assignment.operator.is_logical();
                let right = self.expression(&assignment.right, flow.normal);
                if optional {
                    flow.merge(right);
                } else {
                    flow.then(right);
                }
                if assignment.operator != AssignmentOperator::Assign
                    || self.assignment_may_throw(&assignment.left)
                {
                    flow.may_throw();
                }
                flow
            }
            Expression::ArrayExpression(array) => {
                let mut flow = Flow::normal(state);
                for element in &array.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            flow.then(self.expression(&spread.argument, flow.normal));
                            flow.may_throw();
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        _ => flow
                            .then(self.expression(element.as_expression().unwrap(), flow.normal)),
                    }
                }
                flow
            }
            Expression::ObjectExpression(object) => {
                let mut flow = Flow::normal(state);
                for property in &object.properties {
                    match property {
                        ObjectPropertyKind::ObjectProperty(property) => {
                            if property.computed
                                && let Some(key) = property.key.as_expression()
                            {
                                flow.then(self.expression(key, flow.normal));
                            }
                            flow.then(self.expression(&property.value, flow.normal));
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            flow.then(self.expression(&spread.argument, flow.normal));
                            flow.may_throw();
                        }
                    }
                }
                flow
            }
            Expression::TemplateLiteral(template) => {
                let mut flow = Flow::normal(state);
                for expression in &template.expressions {
                    flow.then(self.expression(expression, flow.normal));
                }
                flow
            }
            Expression::TaggedTemplateExpression(tagged) => {
                let mut flow = self.reference(&tagged.tag, state);
                for expression in &tagged.quasi.expressions {
                    flow.then(self.expression(expression, flow.normal));
                }
                flow.may_throw();
                flow
            }
            _ => {
                let mut visitor = FlowVisitor { analysis: self, flow: Flow::normal(state) };
                walk_js::walk_expression(&mut visitor, expression);
                visitor.flow
            }
        }
    }

    fn call(&mut self, call: &CallExpression<'a>, state: u8) -> Flow<'a> {
        let resolver = self.is_resolver(&call.callee);
        let exit = self.is_process_exit(call);
        let mut flow = if resolver || exit {
            Flow::normal(state)
        } else {
            self.reference(&call.callee, state)
        };
        let skipped =
            if call.optional || Self::optional_chain(&call.callee) { flow.normal } else { 0 };
        for argument in &call.arguments {
            flow.then(self.argument(argument, flow.normal));
        }
        if resolver && flow.normal != 0 {
            if flow.normal & CALLED != 0 {
                self.reported.insert(call.span);
            }
            flow.normal = CALLED;
        } else if exit
            && call.arguments.iter().all(|argument| {
                argument.as_expression().is_some_and(|expression| self.safe(expression))
            })
        {
            // A real process exit bypasses catch and finally as well as normal flow.
            flow.normal = 0;
        } else {
            flow.may_throw();
        }
        flow.normal |= skipped;
        flow
    }

    fn argument(&mut self, argument: &Argument<'a>, state: u8) -> Flow<'a> {
        if let Argument::SpreadElement(spread) = argument {
            let mut flow = self.expression(&spread.argument, state);
            flow.may_throw();
            flow
        } else {
            self.expression(argument.as_expression().unwrap(), state)
        }
    }

    fn chain(&mut self, chain: &ChainElement<'a>, state: u8) -> Flow<'a> {
        match chain {
            ChainElement::CallExpression(call) => self.call(call, state),
            ChainElement::StaticMemberExpression(member) => {
                self.member(&member.object, None, member.optional, state)
            }
            ChainElement::ComputedMemberExpression(member) => {
                self.member(&member.object, Some(&member.expression), member.optional, state)
            }
            ChainElement::PrivateFieldExpression(member) => {
                self.member(&member.object, None, member.optional, state)
            }
            ChainElement::TSNonNullExpression(non_null) => {
                self.expression(&non_null.expression, state)
            }
        }
    }

    fn optional_chain(expression: &Expression<'a>) -> bool {
        match expression.get_inner_expression() {
            Expression::ChainExpression(_) => true,
            Expression::CallExpression(call) => call.optional || Self::optional_chain(&call.callee),
            Expression::StaticMemberExpression(member) => {
                member.optional || Self::optional_chain(&member.object)
            }
            Expression::ComputedMemberExpression(member) => {
                member.optional || Self::optional_chain(&member.object)
            }
            Expression::PrivateFieldExpression(member) => {
                member.optional || Self::optional_chain(&member.object)
            }
            _ => false,
        }
    }

    fn member(
        &mut self,
        object: &Expression<'a>,
        key: Option<&Expression<'a>>,
        optional: bool,
        state: u8,
    ) -> Flow<'a> {
        let mut flow = self.expression(object, state);
        let skip = optional || Self::optional_chain(object);
        let skipped = if skip { flow.normal } else { 0 };
        if !skip || self.nullish(object) != Some(true) {
            if let Some(key) = key {
                flow.then(self.expression(key, flow.normal));
            }
            flow.may_throw();
        }
        flow.normal |= skipped;
        flow
    }

    fn reference(&mut self, expression: &Expression<'a>, state: u8) -> Flow<'a> {
        let mut flow = self.expression(expression, state);
        if let Expression::Identifier(identifier) = expression.get_inner_expression()
            && !self.defined(identifier)
        {
            flow.may_throw();
        }
        flow
    }

    fn defined(&self, identifier: &IdentifierReference<'a>) -> bool {
        self.ctx.scoping().get_reference(identifier.reference_id()).symbol_id().is_some_and(
            |symbol| {
                let flags = self.ctx.scoping().symbol_flags(symbol);
                !(flags.is_block_scoped() || flags.is_class() || flags.is_import())
                    || self.ctx.scoping().symbol_span(symbol).start < identifier.span.start
            },
        )
    }

    fn readonly(&self, identifier: &IdentifierReference<'a>) -> bool {
        self.ctx.scoping().get_reference(identifier.reference_id()).symbol_id().is_some_and(
            |symbol| {
                let flags = self.ctx.scoping().symbol_flags(symbol);
                flags.is_const_variable() || flags.is_class() || flags.is_import()
            },
        )
    }

    fn assignment_target_before(
        &mut self,
        target: &AssignmentTarget<'a>,
        state: u8,
        read: bool,
    ) -> Flow<'a> {
        target.as_simple_assignment_target().map_or_else(
            || Flow::normal(state),
            |target| self.simple_assignment_target(target, state, read),
        )
    }

    fn simple_assignment_target(
        &mut self,
        target: &SimpleAssignmentTarget<'a>,
        state: u8,
        read: bool,
    ) -> Flow<'a> {
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                let mut flow = Flow::normal(state);
                if read && !self.defined(identifier) {
                    flow.may_throw();
                }
                flow
            }
            SimpleAssignmentTarget::StaticMemberExpression(member) => {
                self.assignment_member(&member.object, None, state, read)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                self.assignment_member(&member.object, Some(&member.expression), state, read)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(member) => {
                self.assignment_member(&member.object, None, state, read)
            }
            SimpleAssignmentTarget::TSAsExpression(_)
            | SimpleAssignmentTarget::TSSatisfiesExpression(_)
            | SimpleAssignmentTarget::TSNonNullExpression(_)
            | SimpleAssignmentTarget::TSTypeAssertion(_) => {
                self.expression(target.get_expression().unwrap(), state)
            }
        }
    }

    fn assignment_member(
        &mut self,
        object: &Expression<'a>,
        key: Option<&Expression<'a>>,
        state: u8,
        read: bool,
    ) -> Flow<'a> {
        let mut flow = self.reference(object, state);
        if let Some(key) = key {
            flow.then(self.expression(key, flow.normal));
        }
        if read || self.nullish(object) == Some(true) {
            flow.may_throw();
        }
        flow
    }

    fn assignment_pattern(&mut self, target: &AssignmentTarget<'a>, state: u8) -> Flow<'a> {
        let mut flow = Flow::normal(state);
        // Iterator/property operations can throw before evaluating an element.
        flow.may_throw();
        match target {
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                for element in array.elements.iter().flatten() {
                    flow.then(self.assignment_maybe_default(element, flow.normal));
                }
                if let Some(rest) = &array.rest {
                    flow.then(self.assignment_element(&rest.target, None, flow.normal));
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                for property in &object.properties {
                    match property {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                            if let Some(initializer) = &property.init {
                                flow.merge(self.expression(initializer, flow.normal));
                            }
                            if self.readonly(&property.binding) || !self.defined(&property.binding)
                            {
                                flow.may_throw();
                            }
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                            if property.computed
                                && let Some(key) = property.name.as_expression()
                            {
                                flow.then(self.expression(key, flow.normal));
                            }
                            flow.then(
                                self.assignment_maybe_default(&property.binding, flow.normal),
                            );
                        }
                    }
                }
                if let Some(rest) = &object.rest {
                    flow.then(self.assignment_element(&rest.target, None, flow.normal));
                }
            }
            _ => return self.assignment_element(target, None, state),
        }
        flow
    }

    fn assignment_maybe_default(
        &mut self,
        target: &AssignmentTargetMaybeDefault<'a>,
        state: u8,
    ) -> Flow<'a> {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => {
                self.assignment_element(&default.binding, Some(&default.init), state)
            }
            _ => self.assignment_element(target.to_assignment_target(), None, state),
        }
    }

    fn assignment_element(
        &mut self,
        target: &AssignmentTarget<'a>,
        initializer: Option<&Expression<'a>>,
        state: u8,
    ) -> Flow<'a> {
        // A simple element's reference precedes its default; a nested pattern
        // is evaluated after the value (and possible default) is obtained.
        let mut flow = self.assignment_target_before(target, state, false);
        if let Some(initializer) = initializer {
            flow.merge(self.expression(initializer, flow.normal));
        }
        if target.as_simple_assignment_target().is_none() {
            flow.then(self.assignment_pattern(target, flow.normal));
        } else if self.assignment_may_throw(target) {
            flow.may_throw();
        }
        flow
    }

    fn assignment_may_throw(&self, target: &AssignmentTarget<'a>) -> bool {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                self.readonly(identifier) || !self.defined(identifier)
            }
            _ => true,
        }
    }

    fn class(&mut self, class: &Class<'a>, state: u8) -> Flow<'a> {
        let mut flow = Flow::normal(state);
        if let Some(heritage) = &class.heritage {
            flow.then(self.reference(&heritage.expression, flow.normal));
            if !matches!(
                heritage.expression.get_inner_expression(),
                Expression::NullLiteral(_)
                    | Expression::FunctionExpression(_)
                    | Expression::ArrowFunctionExpression(_)
            ) {
                flow.may_throw();
            }
        }
        for element in &class.body.body {
            if flow.normal == 0 {
                break;
            }
            match element {
                ClassElement::StaticBlock(block) => {
                    let mut child = self.statements(&block.body, UNCALLED);
                    if child.take(Completion::Throw) != 0 {
                        flow.may_throw();
                    }
                    if child.normal == 0 {
                        flow.normal = 0;
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if method.computed
                        && let Some(key) = method.key.as_expression()
                    {
                        flow.then(self.expression(key, flow.normal));
                    }
                    if flow.normal != 0 {
                        self.function(&method.value, false);
                    }
                }
                ClassElement::PropertyDefinition(property) => {
                    if property.computed
                        && let Some(key) = property.key.as_expression()
                    {
                        flow.then(self.expression(key, flow.normal));
                    }
                    if let Some(value) = &property.value {
                        let mut child = self.expression(value, UNCALLED);
                        if property.r#static {
                            if child.take(Completion::Throw) != 0 {
                                flow.may_throw();
                            }
                            if child.normal == 0 {
                                flow.normal = 0;
                            }
                        }
                    }
                }
                ClassElement::AccessorProperty(property) => {
                    if property.computed
                        && let Some(key) = property.key.as_expression()
                    {
                        flow.then(self.expression(key, flow.normal));
                    }
                    if let Some(value) = &property.value {
                        let mut child = self.expression(value, UNCALLED);
                        if property.r#static {
                            if child.take(Completion::Throw) != 0 {
                                flow.may_throw();
                            }
                            if child.normal == 0 {
                                flow.normal = 0;
                            }
                        }
                    }
                }
                ClassElement::TSIndexSignature(_) => {}
            }
        }
        flow
    }

    fn initializer(&self, identifier: &IdentifierReference<'a>) -> Option<&Expression<'a>> {
        let symbol = self.ctx.scoping().get_reference(identifier.reference_id()).symbol_id()?;
        if self.ctx.scoping().get_resolved_references(symbol).any(Reference::is_write) {
            return None;
        }
        let AstKind::VariableDeclarator(declarator) =
            self.ctx.nodes().kind(self.ctx.scoping().symbol_declaration(symbol))
        else {
            return None;
        };
        let initializer = declarator.init.as_ref()?;
        // Avoid cycles and reads before the binding is initialized.
        (initializer.span().end < identifier.span.start).then_some(initializer)
    }

    fn truthiness(&self, expression: &Expression<'a>) -> Option<bool> {
        self.truthiness_at_depth(expression, 0)
    }

    fn truthiness_at_depth(&self, expression: &Expression<'a>, depth: u8) -> Option<bool> {
        if depth >= 16 {
            return None;
        }
        match expression.get_inner_expression() {
            Expression::BooleanLiteral(literal) => Some(literal.value),
            Expression::NullLiteral(_) => Some(false),
            Expression::NumericLiteral(literal) => {
                Some(literal.value != 0.0 && !literal.value.is_nan())
            }
            Expression::StringLiteral(literal) => Some(!literal.value.is_empty()),
            Expression::BigIntLiteral(literal) => Some(!literal.is_zero()),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                Some(template.quasis.iter().any(|quasi| !quasi.value.raw.is_empty()))
            }
            Expression::Identifier(identifier)
                if identifier.name == "undefined"
                    && identifier.is_global_reference(self.ctx.scoping()) =>
            {
                Some(false)
            }
            Expression::Identifier(identifier) => self
                .initializer(identifier)
                .and_then(|initializer| self.truthiness_at_depth(initializer, depth + 1)),
            Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
                self.truthiness_at_depth(&unary.argument, depth + 1).map(|truth| !truth)
            }
            Expression::UnaryExpression(unary)
                if unary.operator == UnaryOperator::Void && self.safe(&unary.argument) =>
            {
                Some(false)
            }
            Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_) => Some(true),
            _ => None,
        }
    }

    fn nullish(&self, expression: &Expression<'a>) -> Option<bool> {
        self.nullish_at_depth(expression, 0)
    }

    fn nullish_at_depth(&self, expression: &Expression<'a>, depth: u8) -> Option<bool> {
        if depth >= 16 {
            return None;
        }
        match expression.get_inner_expression() {
            Expression::NullLiteral(_) => Some(true),
            Expression::Identifier(identifier)
                if identifier.name == "undefined"
                    && identifier.is_global_reference(self.ctx.scoping()) =>
            {
                Some(true)
            }
            Expression::Identifier(identifier) => self
                .initializer(identifier)
                .and_then(|initializer| self.nullish_at_depth(initializer, depth + 1)),
            Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_) => Some(false),
            _ => None,
        }
    }

    fn safe(&self, expression: &Expression<'a>) -> bool {
        match expression.get_inner_expression() {
            Expression::BooleanLiteral(_) | Expression::NullLiteral(_) | Expression::NumericLiteral(_) | Expression::BigIntLiteral(_) | Expression::StringLiteral(_) | Expression::RegExpLiteral(_) | Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => true,
            Expression::Identifier(identifier) => self.defined(identifier) || (identifier.name == "undefined" && identifier.is_global_reference(self.ctx.scoping())),
            Expression::BinaryExpression(_) => !expression.may_have_side_effects(self),
            Expression::UnaryExpression(unary) => {
                if matches!(unary.operator, UnaryOperator::Void | UnaryOperator::LogicalNot | UnaryOperator::Typeof) { self.safe(&unary.argument) }
                else { !expression.may_have_side_effects(self) }
            }
            Expression::ConditionalExpression(conditional) => self.safe(&conditional.test) && match self.truthiness(&conditional.test) {
                Some(true) => self.safe(&conditional.consequent),
                Some(false) => self.safe(&conditional.alternate),
                None => self.safe(&conditional.consequent) && self.safe(&conditional.alternate),
            },
            Expression::LogicalExpression(logical) => self.safe(&logical.left) && self.safe(&logical.right),
            Expression::SequenceExpression(sequence) => sequence.expressions.iter().all(|expression| self.safe(expression)),
            Expression::ArrayExpression(array) => array.elements.iter().all(|element| matches!(element, ArrayExpressionElement::Elision(_)) || element.as_expression().is_some_and(|expression| self.safe(expression))),
            Expression::ObjectExpression(object) => object.properties.iter().all(|property| matches!(property, ObjectPropertyKind::ObjectProperty(property) if !property.computed && self.safe(&property.value))),
            Expression::TemplateLiteral(template) => template.expressions.iter().all(|expression| self.safe(expression)),
            _ => false,
        }
    }

    fn empty_iterable(&self, expression: &Expression<'a>) -> bool {
        self.empty_iterable_at_depth(expression, 0)
    }

    fn empty_iterable_at_depth(&self, expression: &Expression<'a>, depth: u8) -> bool {
        if depth >= 16 {
            return false;
        }
        match expression.get_inner_expression() {
            Expression::Identifier(identifier)
                if identifier.name == "undefined"
                    && identifier.is_global_reference(self.ctx.scoping()) =>
            {
                true
            }
            Expression::Identifier(identifier) => {
                self.initializer(identifier).is_some_and(|initializer| {
                    self.empty_iterable_at_depth(initializer, depth + 1)
                        && self
                            .ctx
                            .scoping()
                            .get_resolved_references(
                                self.ctx
                                    .scoping()
                                    .get_reference(identifier.reference_id())
                                    .symbol_id()
                                    .unwrap(),
                            )
                            .all(|reference| {
                                matches!(
                                    self.ctx.nodes().parent_kind(reference.node_id()),
                                    AstKind::ForOfStatement(_) | AstKind::ForInStatement(_)
                                )
                            })
                })
            }
            Expression::ArrayExpression(array) => array.elements.is_empty(),
            Expression::ObjectExpression(object) => object.properties.is_empty(),
            _ => self.truthiness_at_depth(expression, depth) == Some(false),
        }
    }
}

// Use Oxc's existing coercion/throw analysis for constant arithmetic rather
// than treating every binary expression as potentially throwing.
impl<'a> GlobalContext<'a> for Analysis<'a, '_> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool {
        reference.is_global_reference(self.ctx.scoping())
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for Analysis<'a, '_> {
    fn annotations(&self) -> bool {
        false
    }
    fn manual_pure_functions(&self, _callee: &Expression) -> bool {
        false
    }
    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        PropertyReadSideEffects::All
    }
    fn unknown_global_side_effects(&self) -> bool {
        true
    }
}

// Handle runtime children of syntax without special control-flow behavior (for
// example JSX containers and TypeScript enum initializers), without entering
// type-space nodes or flattening nested functions into their enclosing path.
struct FlowVisitor<'a, 'c, 'v> {
    analysis: &'v mut Analysis<'a, 'c>,
    flow: Flow<'a>,
}

impl<'a> VisitJs<'a> for FlowVisitor<'a, '_, '_> {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        self.flow.then(self.analysis.expression(expression, self.flow.normal));
    }

    fn visit_statement(&mut self, statement: &Statement<'a>) {
        self.flow.then(self.analysis.statement(statement, self.flow.normal, None));
    }

    fn visit_function(&mut self, function: &Function<'a>, _flags: ScopeFlags) {
        if self.flow.normal != 0 {
            self.analysis.function(function, false);
        }
    }
}

// Ported from eslint-plugin-unicorn v79.0.0, including all parameterized cases.
// https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v79.0.0/test/no-multiple-promise-resolver-calls.js
#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        r"new Promise(resolve => { while (``) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { for (; ``;) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { if (``) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => {
	try {
		resolve();
		process.exit(1);
	} catch {
		resolve();
	}
});",
        r"new Promise(resolve => {
	try {
		process.exit(1);
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	const condition = getCondition();

	try {
		condition ? process.exit(1) : process.exit(2);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			try {
				resolve();
				process.exit(1);
			} catch {}
		} while (otherCondition);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"function f(condition) {
	new Promise(resolve => {
		try {
			do {
				while (condition) {}
			} while (process.exit(1));
		} catch {}

		resolve(value);
		resolve(otherValue);
	});
}",
        r"new Promise(resolve => {
	try {
		do {
			switch (1) {
				case 1: {
					void 0;
					break;
				}
				default: {
					void 0;
					break;
				}
			}
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		outer: do {
			switch (1) {
				case 1: {
					{
						void 0;
						continue outer;
					}
				}
				default: {
					{
						void 0;
						continue outer;
					}
				}
			}
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		outer: do {
			switch (1) {
				case 1:
					try {
						continue outer;
					} finally {
						void 0;
					}
				default:
					continue outer;
			}
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		outer: do {
			while (true) {
				continue outer;
			}
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			switch (1) {
				case 1: {
					{
						void 0;
						break;
					}
				}
				default: {
					{
						void 0;
						break;
					}
				}
			}
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const condition = getCondition();

	try {
		outer: do {
			while (true) {
				if (condition) {
					continue outer;
				} else {
					continue;
				}
			}
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			switch (1) {
				case 1: {
					break;
				}
				default: {
					break;
				}
			}
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			if (false) maybeThrow();
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const condition = getCondition();

	try {
		if (condition) {
			process.exit(1);
		} else {
			process.exit(2);
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		function helper() {}
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		while (process.exit(1)) {}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			continue;
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			resolve();
			continue;
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			;
			continue;
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		label: do {
			continue label;
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const condition = true;

	try {
		do {
			if (condition) continue;
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		for (; true; process.exit(1)) {}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			while (true) {
				resolve();
				process.exit(1);
			}
		} while (otherCondition);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			resolve();
			process.exit(1);
		} while (condition);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const condition = getCondition();

	try {
		do {
			if (condition) {
				resolve();
				process.exit(1);
			} else {
				process.exit(2);
			}
		} while (otherCondition);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const condition = true;

	try {
		do {
			;
			if (condition) continue;
			else process.exit(1);
		} while (process.exit(2));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const condition = true;

	try {
		do {
			;
			if (condition) continue;
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const condition = true;

	try {
		do {
			if (condition) continue;
			process.exit(1);
		} while (process.exit(2));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do;
	while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		for (;;) {
			process.exit(1);
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		while (1) {
			process.exit(1);
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		for (; 1;) {
			process.exit(1);
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const value = 1;

	outer: {
		try {
			switch (value) {
				case 1:
					break outer;
				case process.exit(1):
					{}
			}
		} catch {}

		resolve(value);
		resolve(otherValue);
	}
});",
        r"new Promise((resolve, reject) => {
	resolve(value);
	process.exit(1);
	reject(error);
});",
        r"new Promise(resolve => {
	[...process.exit(1)];
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	callback(...process.exit(1));
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = process.exit(1)) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = void process.exit(1)) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = (process.exit(1), value)) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = condition ? process.exit(1) : process.exit(2)) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = process.exit(1) || value) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = process.exit(1) && value) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = foo(process.exit(1))) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = process.exit(1) + 1) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = process.exit(1) as never) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		process.exit(1);
	} finally {
		resolve(value);
	}
});",
        r"new Promise(resolve => {
	try {
		process.exit(1)?.toString();
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		new (process.exit(1))();
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		({})[process.exit(1)];
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		const value = 1, exitCode = process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		const exitCode = 1;
		process.exit(exitCode);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		'use strict';
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		if (true) return;
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		[process.exit(1)];
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		[1, process.exit(1)];
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		({value: process.exit(1)});
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		({value: 1, exitCode: process.exit(1)});
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		value = process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		value = process.exit(1);
	} catch {}

	let value;
	resolve(firstValue);
	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		({value} = process.exit(1));
	} catch {}

	resolve(firstValue);
	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		import(process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const report = () => {};
	try {
		report(process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		throw process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		switch (process.exit(1)) {}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		process.exit(1)`tagged`;
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		({}).value = process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		(1, process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		true && process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		true ? process.exit(1) : value;
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		switch (true) {
			default:
				process.exit(1);
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const value = 1;
new Promise(resolve => {
	try {
		switch (value) {
			case 1:
				process.exit(1);
			default:
				process.exit(2);
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		throw error;
	} catch {
		if (condition) {
			process.exit(1);
		} else {
			throw error;
		}
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		cleanup();
	} finally {
		process.exit(1);
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		switch (value) {
			case 1:
				process.exit(1);
				cleanup();
			default:
				process.exit(2);
		}
	} finally {
		resolve(value);
		resolve(otherValue);
	}
});",
        r"new Promise(resolve => {
	try {
		switch (value) {
			case 1:
				process.exit(1);
			default:
				process.exit(2);
		}
	} finally {
		cleanup();
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		process.exit(1);
	} finally {
		resolve(value);
		resolve(otherValue);
	}
});",
        r"new Promise(resolve => {
	try {
		void process.exit(1);
	} finally {
		resolve(value);
		resolve(otherValue);
	}
});",
        r"new Promise(resolve => {
	try {
		throw error;
	} catch {
		process.exit(1);
	} finally {
		resolve(value);
		resolve(otherValue);
	}
});",
        r"new Promise(resolve => {
	class Example {
		static {
			process.exit(1);
		}

		static {
			resolve(value);
			resolve(otherValue);
		}
	}
});",
        r"new Promise(resolve => {
	try {
		class Example {
			static {}
			static { process.exit(1); }
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		class Example extends process.exit(1) {}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const value = 1;
new Promise(resolve => {
	try {
		switch (value) {
			case process.exit(1):
			case 1:
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	class Example {
		static field = condition ? process.exit(1) : process.exit(1);
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	class Example {
		static field = process.exit(1);
		static otherField = (resolve(value), resolve(otherValue));
	}
});",
        r"new Promise(resolve => {
	try {
		class Example {
			static field = 1;
			static {
				process.exit(1);
			}
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		class Example {
			[process.exit(1)]() {}
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	class Example {
		static {
			try {
				process.exit(1);
			} finally {
				cleanup();
			}
		}
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	class Example {
		static {
			try {
				cleanup();
			} finally {
				process.exit(1);
			}
		}
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	class Example {
		static {
			if (condition) {
				process.exit(1);
			} else {
				process.exit(1);
			}
		}
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => resolve(value));",
        r"new Promise((resolve, reject) => {
	if (error) {
		reject(error);
	} else {
		resolve(value);
	}
});",
        r"new Promise((resolve, reject) => condition ? resolve(value) : reject(error));",
        r"new Promise((resolve, reject) => {
	if (error) {
		reject(error);
		return;
	}

	resolve(value);
});",
        r"new Promise(resolve => {
	resolve(value);
	return;
	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	read((error, value) => {
		if (error) {
			reject(error);
		} else {
			resolve(value);
		}
	});
});",
        r"new Promise<string>((resolve: (value: string) => void, reject: (error: Error) => void) => resolve(value));",
        r"const Promise = CustomPromise;
new Promise(resolve => {
	resolve(value);
	resolve(otherValue);
});",
        r"new globalThis.Promise(resolve => {
	resolve(value);
	resolve(otherValue);
});",
        r"new CustomPromise(resolve => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	resolve(value);
	resolve(otherValue);
}, extraArgument);",
        r"new Promise(function * (resolve) {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	resolve = otherResolve;
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const finish = resolve;
	finish(value);
	finish(otherValue);
});",
        r"new Promise(({resolve}, reject) => {
	resolve(value);
	resolve(otherValue);
	reject.call(undefined, error);
	reject.apply(undefined, [otherError]);
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(mayThrow());
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		process.exit(1);
		cleanup();
	} catch {
		reject(error);
	}

	resolve(value);
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
	} catch (error) {
		reject(error);
		reject(otherError);
	}
});",
        r"new Promise(async (resolve, reject) => {
	try {
		resolve(await getValue());
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(object.value);
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		if (condition) {
			resolve(value);
		} else {
			mayThrow();
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		try {
			mayThrow();
		} catch {} finally {
			resolve();
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		try {
			mayThrow();
		} finally {
			resolve();
			return;
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		if (condition) {
			mayThrow();
		} else {
			resolve(value);
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	onSuccess(() => resolve(value));
	onFailure(error => reject(error));
});",
        r"const executor = (resolve, reject) => {
	resolve(value);
	reject(error);
};
new Promise(executor);",
        r"new Promise(resolve => {
	while (condition) {
		resolve(value);
		break;
	}
});",
        r"new Promise(resolve => {
	while (condition) {
		try {
			resolve(value);
			continue;
		} finally {
			break;
		}
	}
});",
        r"new Promise(resolve => {
	outer: while (condition) {
		try {
			resolve(value);
			continue outer;
		} finally {
			break outer;
		}
	}
});",
        r"new Promise(resolve => {
	while (condition) {
		try {
			resolve(value);
			continue;
		} finally {
			cleanup();
		}
	}
});",
        r"new Promise(resolve => {
	while (condition) {
		try {
			resolve(value);
			break;
		} finally {
			continue;
		}
	}
});",
        r"new Promise(resolve => {
	do {
		resolve(value);
	} while (false);
});",
        r"new Promise(resolve => {
	do {
		resolve(value);
	} while (false as boolean);
});",
        r"new Promise(resolve => {
	while (false) {
		resolve(value);
	}
});",
        r"new Promise(resolve => {
	resolve(value);
	while (false) {
		resolve(otherValue);
	}
});",
        r"new Promise(resolve => {
	resolve(value);
	for (; false;) {
		resolve(otherValue);
	}
});",
        r"new Promise(resolve => {
	while (false) {
		consume(() => {
			resolve(value);
			resolve(otherValue);
		});
	}
});",
        r"new Promise((resolve, reject = resolve()) => { resolve(); });",
        r"new Promise(resolveOuter => {
	new Promise((resolveInner, rejectInner = resolveOuter()) => {
		resolveOuter();
	});
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		const value = 1;
		({value: 1});
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		const value = 1;
		value;
		({value});
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		import('missing');
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		import('missing');
	} catch (error) {
		reject(error);
		reject(otherError);
	}
});",
        r"new Promise(async (resolve, reject) => {
	try {
		await resolve();
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		missingValue;
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	switch (value) {
		case 1: {
			resolve();
			break;
		}
		default: {
			reject();
		}
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		const value: Foo = 1;
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise(resolve => {
	function nested(resolve) {
		resolve(value);
		resolve(otherValue);
	}

	nested(callback);
});",
        r"new Promise((resolve, reject, extra) => {
	extra(value);
	extra(otherValue);
});",
        r"new Promise(resolve => {
	resolve(value);
	(() => resolve(otherValue))();
});",
        r"new Promise(resolve => {
	resolve(value);

	class Example {
		static {
			resolve(otherValue);
		}
	}
});",
        r"new Promise((resolve, reject, extra = 1 + process.exit(1)) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = new Foo(process.exit(1))) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	for (; false; resolve()) {}
	resolve(value);
});",
        r"new Promise(resolve => {
	for (let index = 0; index < 1; resolve()) {
		try {
			continue;
		} finally {
			cleanup();
		}
	}
});",
        r"new Promise(resolve => {
	try {
		while (condition) {
			try {
				foo();
			} finally {
				cleanup();
			}

			try {
				bar();
			} finally {
				cleanup();
			}
		}
	} catch {}

	resolve(value);
});",
        r"const callback = () => {};
new Promise(resolve => {
	try {
		callback(process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		const Example = class {
			static { process.exit(1); }
		};
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const tag = () => {};
new Promise(resolve => {
	try {
		tag`${process.exit(1)}`;
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const value = 1;
new Promise(resolve => {
	try {
		switch (value) {
			case process.exit(1):
			default:
				process.exit(2);
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const value = 1;
new Promise(resolve => {
	try {
		value += process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		process.exit!(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => { reject = other; reject(1); reject(2); });",
        r"new Promise(({resolve}) => { resolve(1); resolve(2); });",
        r"new Promise((resolve, reject, extra) => { extra(1); extra(2); });",
        r"new Promise(resolve => { if (false) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { if (false) { resolve(1); resolve(2); } });",
        r"new Promise(resolve => { for (const x of []) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { for (const x in {}) { resolve(1); } resolve(2); });",
        r"const empty = []; new Promise(resolve => { for (const x of empty) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { for (const x of null) { resolve(1); resolve(2); } });",
        r"new Promise(resolve => { for (const x in null) { resolve(1); resolve(2); } });",
        r"new Promise(resolve => { for (const x of undefined) { resolve(1); resolve(2); } });",
        r#"new Promise(resolve => { for (const x of "") { resolve(1); resolve(2); } });"#,
        r"new Promise(resolve => { for (const x of 0) { resolve(1); resolve(2); } });",
        r#"new Promise(resolve => { for (const x in "") { resolve(1); resolve(2); } });"#,
    ];
    assert_eq!(pass.len(), 169);
    let fail = vec![
        r"new Promise(resolve => { do { resolve(1); } while (``); resolve(2); });",
        r"new Promise(resolve => { while (`${value}`) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { while (tag``) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => {
	const modes = new Set();
	modes.add('foo');

	if (!modes.size) {
		return;
	}

	resolve();
	resolve();
});",
        r"new Promise(resolve => {
	const modes = new Set(['foo']);
	modes.clear();

	if (modes.size || condition) {
		return;
	}

	resolve();
	resolve();
});",
        r"new Promise(resolve => {
	const object = {value: true};
	Object.defineProperty(object, 'value', {get() { return false; }});

	if (object.value) {
		return;
	}

	resolve();
	resolve();
});",
        r"new Promise(resolve => {
	try {
		switch (value) {
			default:
				break;
			case process.exit(1):
				{}
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	try {
		try {
			throw error;
		} catch {
			throw error;
		}

		process.exit(1);
	} catch {
		resolve(value);
	}

	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	try {
		maybeThrow();
		process.exit(1);
	} catch {
		resolve(value);
	}

	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	try {
		(maybeThrow(), process.exit(1));
	} catch {
		resolve(value);
	}

	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	if (condition) {
		process.exit(1);
	} else {
		resolve(value);
	}

	reject(error);
});",
        r"new Promise((resolve, reject) => {
	resolve(value);
	const process = customProcess;
	process.exit(1);
	reject(error);
});",
        r"import process from 'node:process';

new Promise((resolve, reject) => {
	resolve(value);
	process.exit(1);
	reject(error);
});",
        r"new Promise(resolve => {
	class Example {
		static field = () => process.exit(1);
	}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = condition ? process.exit(1) : value) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	resolve(value);
	reject(error);
});",
        r"new Promise(resolve => {
	if (condition) {
		resolve(value);
	}

	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	if (condition) {
		resolve(value);
	} else {
		reject(error);
	}

	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => { condition ? resolve(value) : reject(error); resolve(otherValue); });",
        r"new Promise((resolve, reject) => { condition && resolve(value); reject(error); });",
        r"new Promise((resolve, reject) => {
	read((error, value) => {
		if (error) {
			reject(error);
		}

		resolve(value);
	});
});",
        r"new Promise((resolve, reject) => {
	try {
		reject(error);
	} finally {
		resolve(value);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(value);
		mayThrow();
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		object.value;
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		new Constructor();
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	consume(function * () {
		try {
			resolve();
			yield value;
		} catch (error) {
			reject(error);
		}
	});
});",
        r"new Promise((resolve, reject) => {
	try {
		if (condition) {
			resolve(value);
			mayThrow();
		} else {
			otherMayThrow();
			resolve(otherValue);
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		if (condition) {
			otherMayThrow();
			resolve(otherValue);
		} else {
			resolve(value);
			mayThrow();
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(value);
		throw error;
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(mayThrow());
	} catch (error) {
		reject(error);
	}

	resolve(fallbackValue);
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(mayThrow());
	} catch (error) {
		reject(error);
		reject(otherError);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(firstValue);
		resolve(mayThrow());
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		if (condition) {
			resolve(firstValue);
		}

		resolve(mayThrow());
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise(resolve => {
	while (condition) {
		resolve(value);
	}
});",
        r"new Promise(resolve => {
	for (const value of values) {
		resolve(value);
	}

	resolve(fallbackValue);
});",
        r"new Promise(resolveOuter => {
	new Promise(resolveInner => {
		resolveOuter(value);
		resolveInner(innerValue);
		resolveOuter(otherValue);
	});
});",
        r"new Promise(resolveOuter => {
	new Promise(resolveInner => {
		resolveInner(innerValue);
		resolveOuter(value);
		resolveInner(otherInnerValue);
	});
});",
        r"new Promise(async (resolve, reject) => {
	resolve(await getValue());
	reject(error);
});",
        r"new Promise(resolve => {
	resolve?.(value);
	resolve(otherValue);
});",
        r"new Promise<string>((resolve: (value: string) => void) => { resolve(value); resolve(otherValue); });",
        r"new Promise((resolve: (value?: unknown) => void) => { resolve!(); (resolve as typeof resolve)(); });",
        r"new Promise(((resolve: (value?: unknown) => void) => { resolve(); resolve(); }) as Executor);",
        r"new Promise((<Type>(resolve: (value?: unknown) => void) => { resolve(); resolve(); })<unknown>);",
        r"new Promise(((resolve: (value?: unknown) => void) => { resolve(); resolve(); }) satisfies Executor);",
        r"new Promise((resolve, reject, extra = resolve()) => { resolve(); });",
        r"new Promise((resolve, reject) => { resolve = otherResolve; reject(); reject(); });",
        r"new Promise(function (resolve, reject) {
	resolve(value);
	reject(error);
});",
        r"new Promise(async (resolve, reject) => {
	try {
		resolve();
		await import('missing');
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise(async (resolve, reject) => {
	try {
		resolve();
		await value;
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	switch (value) {
		case 1: {
			resolve();
		}
		default: {
			reject();
		}
	}
});",
        r"new Promise(resolve => {
	try {
		const value = maybeThrow(), exitCode = process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		if (maybeThrow()) return;
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		[maybeThrow(), process.exit(1)];
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		({[maybeThrow()]: process.exit(1)});
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		using resource = 1;
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		class Example {
			static {
				maybeThrow();
				process.exit(1);
			}
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		class Example {
			static field = (maybeThrow(), process.exit(1));
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		(maybeThrow(), {})[process.exit(1)];
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject, extra = foo?.(process.exit(1))) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			for (const value = maybeThrow(); condition;) {}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			for (; condition; maybeThrow()) {}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			for (const value of maybeThrow()) {}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"function f(value, condition) {
	new Promise(resolve => {
		try {
			do {
				for (; condition; value++) {}
			} while (process.exit(1));
		} catch {
			resolve(firstValue);
		}

		resolve(secondValue);
	});
}",
        r"new Promise(resolve => {
	try {
		do {
			for (const {value} of [null]) {}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			for ({value} of [null]) {}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			while (maybeThrow()) {
				break;
			}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			switch (maybeThrow()) {}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		do {
			switch (value) {
				default:
					maybeThrow();
			}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		resolve(firstValue);
		outer: do {
			switch (value) {
				case 1:
					continue outer;
				default:
					process.exit(1);
			}
		} while (condition);
	} catch {}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		outer: do {
			switch (1) {
				case 1:
					try {
						continue outer;
					} finally {
						maybeThrow();
					}
				default:
					continue outer;
			}
		} while (process.exit(1));
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise((resolve, reject, process) => {
	try {
		process.exit(1);
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"import process from 'node:process';

new Promise(resolve => {
	try {
		process.exit(1);
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		label: do {
			maybeThrow();
			process.exit(1);
		} while (condition);
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		resolve(firstValue);
		do {
			while (condition) {
				maybeThrow();
			}
		} while (process.exit(1));
	} catch {
		resolve(secondValue);
	}
});",
        r"new Promise(resolve => {
	try {
		outer: do {
			switch (value) {
				case 1:
					continue outer;
				default:
					process.exit(1);
			}
		} while (condition);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const condition = true;
new Promise(resolve => {
	try {
		do {
			maybeThrow();
			if (condition) continue;
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const condition = getCondition();
new Promise(resolve => {
	try {
		do {
			if (condition) continue;
			else maybeThrow();
		} while (process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		process.exit((maybeThrow(), 1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	const callback = (value = process.exit()) => {};

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		process.exit(maybeThrow());
	} catch {
		resolve(value);
		resolve(otherValue);
	}
});",
        r"new Promise(resolve => {
	try {
		missing(process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"'use strict';
new Promise(resolve => {
	try {
		missing = 1;
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"let value = Symbol();
new Promise(resolve => {
	try {
		value += 1;
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const value = 0;
new Promise(resolve => {
	try {
		(value = 0) || process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"const value = 0;
new Promise(resolve => {
	try {
		(value = 0) ? process.exit(1) : value;
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		const Example = class extends undefined {
			static { process.exit(1); }
		};
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((finish, fail) => { finish(); fail(); finish(); });",
        r"new Promise(resolve => { resolve(); resolve(); });",
        r"new Promise((resolve, reject = process.exit(1)) => {
	resolve(value);
	resolve(otherValue);
});",
        r"new Promise((resolve, reject) => {
	if (error) {
		resolve();
	}

	if (!error) {
		reject();
	}
});",
        r"new Promise((resolve, reject) => { if (resolve()) { reject(); } });",
        r"new Promise((resolve, reject) => { resolve(reject(error)); });",
        r"new Promise((resolve, reject) => { try { resolve(reject(mayThrow())); } catch (error) { reject(error); } });",
        r"new Promise(resolve => {
	while (condition) {
		try {
			resolve(value);
		} finally {
			cleanup();
		}
	}
});",
        r"new Promise(resolve => {
	while (condition) {
		try {
			resolve(value);
		} finally {
			continue;
		}
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		mayThrow();
	} catch (error) {
		reject(error);
	} finally {
		resolve(fallbackValue);
	}
});",
        r"new Promise((resolve, reject) => {
	class Example {
		field = (resolve(value), reject(error));
	}
});",
        r"new Promise(resolve => {
	class Example {
		static {
			resolve(value);
			resolve(otherValue);
		}
	}
});",
        r"new Promise((resolve, reject) => {
	while (condition) {
		try {
			resolve(mayThrow());
		} catch (error) {
			reject(error);
		}
	}
});",
        r"new Promise((resolve, reject) => {
	resolve(firstValue);

	try {
		resolve(2);
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		reject();
	} catch {
		resolve();
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve();
		reject(mayThrow());
	} catch {
		resolve();
	}
});",
        r"new Promise((resolve, reject) => {
	resolve(1);

	try {
		try {
			mayThrow();
		} catch {}

		resolve(2);
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	resolve(1);

	try {
		try {
			mayThrow();
		} finally {}

		resolve(2);
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		try {
			mayThrow();
		} finally {
			resolve();
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		try {
			mayThrow();
		} catch (error) {
			resolve(error);
			throw error;
		}
	} catch (error) {
		reject(error);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		resolve(mayThrow());
	} finally {
		reject(error);
	}
});",
        r"new Promise(resolve => {
	try {
		class Example {
			static {}
		}
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		object.method(process.exit(1));
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		callback(process.exit(1));
	} catch {}

	let callback;
	resolve(firstValue);
	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		const [value] = 1;
		process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	try {
		null.value = process.exit(1);
	} catch {}

	resolve(value);
	resolve(otherValue);
});",
        r"new Promise(resolve => {
	let value;
	try {
		([value] = 1, process.exit(1));
	} catch {}

	resolve(firstValue);
	resolve(secondValue);
});",
        r"const value = 0;
new Promise(resolve => {
	try {
		(value = 1, process.exit(1));
	} catch {}

	resolve(firstValue);
	resolve(secondValue);
});",
        r"new Promise(resolve => {
	try {
		if (true) return;
		process.exit(1);
	} catch {} finally {
		resolve(value);
		resolve(otherValue);
	}
});",
        r"new Promise((resolve, reject) => {
	try {
		if (true) {
			mayThrow();
			process.exit(1);
		}
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"const value = 0;
new Promise((resolve, reject) => {
	try {
		if (value = 1) {
			process.exit(1);
		}
	} catch {
		resolve(firstValue);
	}

	resolve(secondValue);
});",
        r"const value = 1;
new Promise(resolve => {
	try {
		switch (value) {
			case 1:
				mayThrow();
				process.exit(1);
			default:
				process.exit(2);
		}
	} catch {}

	resolve(firstValue);
	resolve(secondValue);
});",
        r"new Promise((resolve, reject = other) => { reject(1); reject(2); });",
        r"new Promise((resolve = other, reject) => { resolve(1); resolve(2); });",
        r"new Promise(resolve => { if (true) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { for (const x of [1]) { resolve(1); } resolve(2); });",
        r"new Promise(resolve => { if (false) { a(); } else { resolve(1); } resolve(2); });",
        r"const array = []; array.push(1); new Promise(resolve => { for (const x of array) { resolve(1); } resolve(2); });",
    ];
    assert_eq!(fail.len(), 124);
    Tester::new(
        NoMultiplePromiseResolverCalls::NAME,
        NoMultiplePromiseResolverCalls::PLUGIN,
        pass,
        fail,
    )
    // The upstream TypeScript cases include a generic arrow that is ambiguous
    // in TSX. A parse error must not stand in for the expected lint diagnostic.
    .change_rule_path_extension("ts")
    .test_and_snapshot();
}

#[test]
fn regression() {
    use crate::tester::Tester;

    let pass = vec![
        "new Promise(resolve => { ([value = resolve()] = process.exit(1)); resolve(); resolve(); });",
        "new Promise(resolve => { ({[resolve()]: target} = process.exit(1)); resolve(); resolve(); });",
        "new Promise(resolve => { try { resolve(); return; } finally { cleanup(); } resolve(); });",
        "new Promise(resolve => { if (0x0n) resolve(); resolve(); });",
        "new Promise(resolve => { try { process.exit(1 + 1); } catch {} resolve(); resolve(); });",
        "new Promise(resolve => { try { process.exit(true ? 0 : 1); } catch {} resolve(); resolve(); });",
        "new Promise(resolve => { if (0o0n) resolve(); resolve(); });",
        "new Promise(resolve => { if (0b0n) resolve(); resolve(); });",
        "new Promise(resolve => { if (0x0_0n) resolve(); resolve(); });",
        "if (0x0n) new Promise(resolve => { resolve(); resolve(); });",
        "new Promise((resolve, reject) => { try { try { throw error; } finally { resolve(); return; } } catch { reject(); } });",
        "new Promise(resolve => { while (condition) { try { resolve(); } finally { break; } } });",
        "new Promise((resolve, reject) => { try { for (const item of values) { resolve(); break; } } catch { reject(); } });",
        "function f() { return; new Promise(resolve => { resolve(); resolve(); }); }",
        "if (false) { new Promise(resolve => { resolve(); resolve(); }); }",
        "while (false) { new Promise(resolve => { resolve(); resolve(); }); }",
        "new Promise(resolve => { if (condition) return resolve(); else return resolve(); });",
        "new Promise(resolve => { resolve(); consume(() => resolve()); });",
        "new Promise(resolve => { class Example { static { resolve(); } static { resolve(); } } });",
    ];
    let fail = vec![
        "new Promise(resolve => { object[resolve()]++; resolve(); });",
        "new Promise(resolve => { resolve(); object[resolve()]++; });",
        "new Promise(resolve => { ([value = resolve()] = []); resolve(); });",
        "new Promise(resolve => { resolve(); ({[resolve()]: target} = {}); });",
        "new Promise(resolve => { try { resolve(); return; } finally { resolve(); } });",
        "new Promise(resolve => { if (0x1n) resolve(); resolve(); });",
        "new Promise((resolve, reject, {a = resolve()} = {}) => { resolve(); });",
        "new Promise((resolve, reject, [a = resolve()] = []) => { resolve(); });",
        "new Promise(resolve => { outer: inner: while (condition) { resolve(); continue outer; } });",
        "new Promise(resolve => { do { try { resolve(); } finally {} } while (condition); });",
        "new Promise(resolve => { while (condition) { try { resolve(); } finally { if (stop) break; } } });",
        "new Promise(resolve => { const node = <Element value={resolve()} other={resolve()} />; });",
        "new Promise(resolve => { enum Result { First = resolve(), Second = resolve() } });",
        "new Promise(resolve => { consume(() => { resolve(); resolve(); }); });",
    ];
    Tester::new(
        NoMultiplePromiseResolverCalls::NAME,
        NoMultiplePromiseResolverCalls::PLUGIN,
        pass,
        fail,
    )
    .with_snapshot_suffix("regressions")
    .test_and_snapshot();
}

#[test]
fn bounded_analysis() {
    use crate::tester::Tester;

    let mut aliases = String::from("const value0 = unknown;");
    for index in 1..2_000 {
        use std::fmt::Write;
        write!(aliases, "const value{index} = value{};", index - 1).unwrap();
    }
    let pass = vec![format!(
        "{aliases} new Promise(resolve => {{ if (value1999) return; else return; resolve(); resolve(); }});"
    )];
    let fail = vec![format!(
        "new Promise(resolve => {{ {} resolve(); {} }});",
        "while (condition) {".repeat(32),
        "}".repeat(32),
    )];
    Tester::new(
        NoMultiplePromiseResolverCalls::NAME,
        NoMultiplePromiseResolverCalls::PLUGIN,
        pass,
        fail,
    )
    .with_snapshot_suffix("bounded")
    .test_and_snapshot();
}
