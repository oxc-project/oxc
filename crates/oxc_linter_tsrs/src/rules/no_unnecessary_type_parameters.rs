// Port of internal/rules/no_unnecessary_type_parameters/no_unnecessary_type_parameters.go.

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, Node, Symbol};
use tsrs_checker::{Checker, ObjectFlags, Signature, Type, TypeFlags};
use tsrs_core::{P, TextRange};

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleSuggestion,
    RuleVisitor,
};
use crate::utils;

fn build_sole_message(
    type_parameter_range: (i32, i32),
    type_parameter_reference: (i32, i32),
    name: &str,
    uses: &str,
    descriptor: &str,
) -> RuleDiagnostic {
    RuleDiagnostic {
        message: RuleMessage::new(
            "sole",
            format!("Type parameter {name} is {uses} in the {descriptor} signature."),
        ),
        pos: type_parameter_range.0,
        end: type_parameter_range.1,
        labeled_ranges: vec![LabeledRange {
            label: format!("This is the only usage of type parameter {name} in the signature."),
            pos: type_parameter_reference.0,
            end: type_parameter_reference.1,
        }],
    }
}

fn build_replace_usages_with_constraint_message() -> RuleMessage {
    RuleMessage::new(
        "replaceUsagesWithConstraint",
        "Replace all usages of type parameter with its constraint.",
    )
}

fn find_type_parameter_index(type_parameters: &[P<Node>], target: P<Node>) -> Option<usize> {
    type_parameters.iter().position(|&t| t == target)
}

fn is_complex_constraint(node: Option<P<Node>>) -> bool {
    node.is_some_and(|node| {
        matches!(node.kind(), Kind::UnionType | Kind::IntersectionType | Kind::ConditionalType)
    })
}

fn has_matching_ancestor_type(reference: P<Node>) -> bool {
    let Some(grandparent) = reference.parent().and_then(|p| p.parent()) else {
        return false;
    };
    matches!(
        grandparent.kind(),
        Kind::ArrayType | Kind::IndexedAccessType | Kind::IntersectionType | Kind::UnionType
    )
}

fn symbol_from_type_parameter(ctx: &mut Ctx, type_parameter: P<Node>) -> Option<P<Symbol>> {
    if !ast::is_type_parameter_declaration(type_parameter) {
        return None;
    }
    ctx.checker.get_symbol_at_location_exported(type_parameter.as_type_parameter_declaration().name)
}

fn collect_type_parameter_reference_nodes(
    ctx: &mut Ctx,
    node: P<Node>,
    symbol: P<Symbol>,
    declaration_name: P<Node>,
) -> Vec<P<Node>> {
    fn walk(
        c: &mut Checker,
        file: &'static ast::SourceFile,
        current: P<Node>,
        symbol: P<Symbol>,
        declaration_name: P<Node>,
        references: &mut Vec<P<Node>>,
    ) {
        if ast::is_identifier(current)
            && current != declaration_name
            && current.parent().is_some_and(ast::is_type_node)
            && c.get_symbol_at_location_exported(current) == Some(symbol)
        {
            references.push(current);
        }
        ast::for_each_child_and_jsdoc(current, file, &mut |child| {
            walk(c, file, child, symbol, declaration_name, references);
            false
        });
    }
    let mut references = Vec::with_capacity(8);
    walk(ctx.checker, ctx.file.get(), node, symbol, declaration_name, &mut references);
    references.sort_by_key(|r| r.pos());
    references
}

fn get_type_parameter_constraint_text(
    ctx: &Ctx,
    type_parameter: P<Node>,
) -> (String, Option<P<Node>>) {
    let constraint_node = type_parameter.as_type_parameter_declaration().constraint;
    match constraint_node {
        None => ("unknown".to_string(), None),
        Some(c) if c.kind() == Kind::AnyKeyword => ("unknown".to_string(), Some(c)),
        Some(c) => {
            (tsrs_scanner::get_source_text_of_node_from_source_file(ctx.file, c, false), Some(c))
        }
    }
}

fn get_type_parameter_list_removal_range(
    ctx: &Ctx,
    type_parameters: &[P<Node>],
    target_type_parameter: P<Node>,
) -> TextRange {
    let Some(index) = find_type_parameter_index(type_parameters, target_type_parameter) else {
        return TextRange::new(target_type_parameter.pos(), target_type_parameter.end());
    };
    let text = ctx.text().as_bytes();
    if type_parameters.len() == 1 {
        let first_type_parameter = type_parameters[0];
        let mut start = first_type_parameter.pos() - 1;
        if start < 0 {
            start = first_type_parameter.pos();
        }
        let mut end = first_type_parameter.end() as usize;
        while end < text.len() && text[end] != b'>' {
            end += 1;
        }
        if end < text.len() && text[end] == b'>' {
            end += 1;
        }
        let mut end = end as i32;
        if end <= start {
            end = first_type_parameter.end();
        }
        return TextRange::new(start, end);
    }
    if index == 0 {
        let mut end = type_parameters[index + 1].pos();
        let token_range = tsrs_scanner::get_range_of_token_at_position(ctx.file, end);
        if token_range.pos() > target_type_parameter.pos() {
            end = token_range.pos();
        }
        return TextRange::new(target_type_parameter.pos(), end);
    }
    let prev_end = type_parameters[index - 1].end();
    let mut start = prev_end;
    let mut i = prev_end;
    while i < target_type_parameter.pos() && (i as usize) < text.len() {
        if text[i as usize] == b',' {
            start = i;
            break;
        }
        i += 1;
    }
    TextRange::new(start, target_type_parameter.end())
}

fn class_members(node: P<Node>) -> &'static [P<Node>] {
    match node.kind() {
        Kind::ClassDeclaration | Kind::ClassExpression => node.members(),
        _ => &[],
    }
}

fn count_type_parameter_usage(
    ctx: &mut Ctx,
    node: P<Node>,
    target_symbols: &FxHashSet<P<Symbol>>,
) -> FxHashMap<P<Symbol>, i32> {
    let mut counts: FxHashMap<P<Symbol>, i32> = FxHashMap::default();
    let mut remaining_targets = target_symbols.len() as i32;
    if ast::is_class_like(node) {
        for &type_parameter in node.type_parameters() {
            if remaining_targets == 0 {
                break;
            }
            remaining_targets = collect_type_parameter_usage_counts(
                ctx.checker,
                type_parameter,
                &mut counts,
                target_symbols,
                remaining_targets,
                true,
            );
        }
        if let Some(heritage_clauses) = utils::get_heritage_clauses(node) {
            for &heritage_clause in heritage_clauses.nodes() {
                for &heritage_type in heritage_clause.as_heritage_clause().types.get().nodes() {
                    if remaining_targets == 0 {
                        break;
                    }
                    remaining_targets = collect_type_parameter_usage_counts(
                        ctx.checker,
                        heritage_type,
                        &mut counts,
                        target_symbols,
                        remaining_targets,
                        true,
                    );
                }
            }
        }
        for &member in class_members(node) {
            if remaining_targets == 0 {
                break;
            }
            remaining_targets = collect_type_parameter_usage_counts(
                ctx.checker,
                member,
                &mut counts,
                target_symbols,
                remaining_targets,
                true,
            );
        }
    } else {
        collect_type_parameter_usage_counts(
            ctx.checker,
            node,
            &mut counts,
            target_symbols,
            remaining_targets,
            false,
        );
    }
    counts
}

const MAX_INT: i32 = i32::MAX;

fn get_start_of_body(node: P<Node>) -> i32 {
    let type_end = |n: P<Node>| n.type_node().map(|t| t.end());
    let r = match node.kind() {
        Kind::ClassDeclaration | Kind::ClassExpression => node.member_list().map(|m| m.pos()),
        Kind::FunctionDeclaration | Kind::FunctionExpression | Kind::MethodDeclaration => {
            match node.body() {
                Some(body) => Some(body.pos()),
                None => type_end(node),
            }
        }
        Kind::ArrowFunction => match node.body() {
            // Returned function signatures contribute to the inferred return type.
            Some(body) if ast::is_function_like(body) => Some(MAX_INT),
            Some(body) => Some(body.pos()),
            None => type_end(node),
        },
        Kind::CallSignature
        | Kind::ConstructSignature
        | Kind::ConstructorType
        | Kind::FunctionType
        | Kind::MethodSignature => type_end(node),
        _ => None,
    };
    r.unwrap_or(MAX_INT)
}

fn is_type_parameter_repeated_in_ast(
    type_parameter: P<Node>,
    references: &[P<Node>],
    start_of_body: i32,
) -> bool {
    let mut total = 0;
    for &reference in references {
        // References inside the type parameter declaration itself don't count.
        if reference.pos() < type_parameter.end() && reference.end() > type_parameter.pos() {
            continue;
        }
        // References in the body don't count for fast-path usage checks.
        if reference.pos() > start_of_body {
            continue;
        }
        total += 1;
        if total >= 2 {
            return true;
        }
    }
    false
}

struct UsageCollector<'a> {
    found_identifier_usages: &'a mut FxHashMap<P<Symbol>, i32>,
    target_symbols: &'a FxHashSet<P<Symbol>>,
    remaining_targets: i32,
    from_class: bool,
    type_usages: FxHashMap<P<Type>, i32>,
    visited_constraints: FxHashSet<P<Node>>,
    visited_default: bool,
    function_like_type: bool,
}

impl UsageCollector<'_> {
    fn increment_identifier_count(&mut self, symbol: P<Symbol>, assume_multiple_uses: bool) {
        if self.remaining_targets == 0 {
            return;
        }
        if !self.target_symbols.contains(&symbol) {
            return;
        }
        let current = self.found_identifier_usages.get(&symbol).copied().unwrap_or(0);
        if current > 2 {
            return;
        }
        let value = if assume_multiple_uses { 2 } else { 1 };
        let updated = current + value;
        self.found_identifier_usages.insert(symbol, updated);
        if updated > 2 {
            self.remaining_targets -= 1;
        }
    }

    fn increment_type_usages(&mut self, t: P<Type>) -> i32 {
        let count = self.type_usages.entry(t).or_insert(0);
        *count += 1;
        *count
    }

    fn visit_types_list(&mut self, c: &mut Checker, types: &[P<Type>], assume_multiple_uses: bool) {
        for &t in types {
            if self.remaining_targets == 0 {
                return;
            }
            self.visit_type(c, Some(t), assume_multiple_uses, false);
        }
    }

    fn visit_symbols_list(
        &mut self,
        c: &mut Checker,
        symbols: &[P<Symbol>],
        assume_multiple_uses: bool,
    ) {
        for &symbol in symbols {
            if self.remaining_targets == 0 {
                return;
            }
            let t = c.get_type_of_symbol(symbol);
            self.visit_type(c, Some(t), assume_multiple_uses, false);
        }
    }

    fn visit_signature(&mut self, c: &mut Checker, signature: P<Signature>) {
        if self.remaining_targets == 0 {
            return;
        }
        if let Some(this_parameter) = signature.this_parameter() {
            let t = c.get_type_of_symbol(this_parameter);
            self.visit_type(c, Some(t), false, false);
        }
        for &parameter in signature.parameters() {
            if self.remaining_targets == 0 {
                return;
            }
            let t = c.get_type_of_symbol(parameter);
            self.visit_type(c, Some(t), false, false);
        }
        for &type_parameter in signature.type_parameters() {
            if self.remaining_targets == 0 {
                return;
            }
            self.visit_type(c, Some(type_parameter), false, false);
        }
        let mut return_type = c.get_return_type_of_signature(signature);
        if let Some(type_predicate) = c.get_type_predicate_of_signature_exported(signature) {
            if let Some(predicate_type) = type_predicate.type_() {
                return_type = predicate_type;
            }
        }
        self.visit_type(c, Some(return_type), false, true);
    }

    fn visit_type(
        &mut self,
        c: &mut Checker,
        t: Option<P<Type>>,
        assume_multiple_uses: bool,
        is_return_type: bool,
    ) {
        let Some(t) = t else { return };
        if self.remaining_targets == 0 || self.increment_type_usages(t) > 9 {
            return;
        }
        if utils::is_type_parameter(t) {
            if let Some(type_parameter_symbol) = t.symbol() {
                if let Some(&declaration) = type_parameter_symbol.declarations().first() {
                    if ast::is_type_parameter_declaration(declaration) {
                        self.increment_identifier_count(
                            type_parameter_symbol,
                            assume_multiple_uses,
                        );
                        let decl = declaration.as_type_parameter_declaration();
                        if let Some(constraint) = decl.constraint {
                            if self.visited_constraints.insert(constraint) {
                                let ct = c.get_type_at_location(constraint);
                                self.visit_type(c, Some(ct), false, false);
                            }
                        }
                        if let Some(default_type) = decl.default_type {
                            if !self.visited_default {
                                self.visited_default = true;
                                let dt = c.get_type_at_location(default_type);
                                self.visit_type(c, Some(dt), false, false);
                            }
                        }
                    }
                }
            }
            return;
        }
        if let Some(alias) = t.alias() {
            let alias_type_arguments = alias.type_arguments();
            if !alias_type_arguments.is_empty() {
                self.visit_types_list(c, alias_type_arguments, true);
                return;
            }
        }
        if utils::is_union_type(t) || utils::is_intersection_type(t) {
            self.visit_types_list(c, t.types(), assume_multiple_uses);
            return;
        }
        let flags = t.flags();
        if flags.intersects(TypeFlags::IndexedAccess) {
            let indexed_access_type = t.as_indexed_access_type();
            self.visit_type(c, indexed_access_type.object_type(), assume_multiple_uses, false);
            self.visit_type(c, indexed_access_type.index_type(), assume_multiple_uses, false);
            return;
        }
        if flags.intersects(TypeFlags::Object)
            && t.object_flags().intersects(ObjectFlags::Reference)
        {
            let type_arguments = c.get_type_arguments(t);
            if !type_arguments.is_empty() {
                let target = t.target();
                for &type_argument in type_arguments {
                    let mut this_assume_multiple_uses = self.from_class || assume_multiple_uses;
                    if target.is_some_and(tsrs_checker::is_tuple_type_exported) {
                        this_assume_multiple_uses = this_assume_multiple_uses
                            || (is_return_type && !target.unwrap().as_tuple_type().is_readonly());
                    } else if target.is_some_and(|target| c.is_array_type(target)) {
                        let symbol_name = t.symbol().map(|s| s.name()).unwrap_or("");
                        this_assume_multiple_uses =
                            this_assume_multiple_uses || (is_return_type && symbol_name == "Array");
                    } else {
                        this_assume_multiple_uses = true;
                    }
                    self.visit_type(
                        c,
                        Some(type_argument),
                        this_assume_multiple_uses,
                        is_return_type,
                    );
                }
            }
            return;
        }
        if flags.intersects(TypeFlags::TemplateLiteral) {
            self.visit_types_list(c, t.types(), assume_multiple_uses);
            return;
        }
        if flags.intersects(TypeFlags::Conditional) {
            let conditional_type = t.as_conditional_type();
            self.visit_type(c, conditional_type.check_type.get(), assume_multiple_uses, false);
            self.visit_type(c, conditional_type.extends_type.get(), assume_multiple_uses, false);
            return;
        }
        if utils::is_object_type(t) {
            let properties = c.get_properties_of_type(t);
            self.visit_symbols_list(c, properties, false);
            if t.object_flags().intersects(ObjectFlags::Mapped) {
                let mapped_type = t.as_mapped_type();
                self.visit_type(c, mapped_type.type_parameter.get(), false, false);
                if properties.is_empty() {
                    match mapped_type.template_type.get() {
                        Some(template_type) => {
                            self.visit_type(c, Some(template_type), false, false)
                        }
                        None => self.visit_type(c, mapped_type.constraint_type.get(), false, false),
                    }
                }
            }
            let number_index_type = c.get_number_index_type(t);
            self.visit_type(c, number_index_type, true, false);
            let string_index_type = c.get_string_index_type(t);
            self.visit_type(c, string_index_type, true, false);
            for &signature in c.get_call_signatures(t) {
                if self.remaining_targets == 0 {
                    return;
                }
                self.function_like_type = true;
                self.visit_signature(c, signature);
            }
            for &signature in c.get_construct_signatures(t) {
                if self.remaining_targets == 0 {
                    return;
                }
                self.function_like_type = true;
                self.visit_signature(c, signature);
            }
            return;
        }
        if flags.intersects(TypeFlags::Index) || flags.intersects(TypeFlags::StringMapping) {
            self.visit_type(c, t.target(), assume_multiple_uses, false);
        }
    }
}

fn collect_type_parameter_usage_counts(
    c: &mut Checker,
    node: P<Node>,
    found_identifier_usages: &mut FxHashMap<P<Symbol>, i32>,
    target_symbols: &FxHashSet<P<Symbol>>,
    remaining_targets: i32,
    from_class: bool,
) -> i32 {
    let mut collector = UsageCollector {
        found_identifier_usages,
        target_symbols,
        remaining_targets,
        from_class,
        type_usages: FxHashMap::default(),
        visited_constraints: FxHashSet::default(),
        visited_default: false,
        function_like_type: false,
    };
    if ast::is_call_signature_declaration(node) || ast::is_constructor_declaration(node) {
        collector.function_like_type = true;
        let signature = c.get_signature_from_declaration_exported(node);
        collector.visit_signature(c, signature);
    }
    if !collector.function_like_type {
        let t = c.get_type_at_location(node);
        collector.visit_type(c, Some(t), false, false);
    }
    collector.remaining_targets
}

struct CandidateTypeParameter {
    node: P<Node>,
    name_node: P<Node>,
    symbol: P<Symbol>,
    references: Vec<P<Node>>,
}

fn check_no_unnecessary_type_parameters_node(ctx: &mut Ctx, node: P<Node>, descriptor: &str) {
    let type_parameters = node.type_parameters();
    if type_parameters.is_empty() {
        return;
    }
    let start_of_body = get_start_of_body(node);
    let mut candidates: Vec<CandidateTypeParameter> = Vec::with_capacity(type_parameters.len());
    for &type_parameter in type_parameters {
        let Some(type_parameter_symbol) = symbol_from_type_parameter(ctx, type_parameter) else {
            continue;
        };
        let type_parameter_name_node = type_parameter.as_type_parameter_declaration().name;
        let references = collect_type_parameter_reference_nodes(
            ctx,
            node,
            type_parameter_symbol,
            type_parameter_name_node,
        );
        if is_type_parameter_repeated_in_ast(type_parameter, &references, start_of_body) {
            continue;
        }
        candidates.push(CandidateTypeParameter {
            node: type_parameter,
            name_node: type_parameter_name_node,
            symbol: type_parameter_symbol,
            references,
        });
    }
    if candidates.is_empty() {
        return;
    }
    let target_symbols: FxHashSet<P<Symbol>> = candidates.iter().map(|c| c.symbol).collect();
    let identifier_counts = count_type_parameter_usage(ctx, node, &target_symbols);
    for candidate in candidates {
        let Some(&identifier_count) = identifier_counts.get(&candidate.symbol) else {
            continue;
        };
        if identifier_count > 2 {
            continue;
        }
        let uses = if identifier_count == 1 { "never used" } else { "used only once" };
        let type_parameter_name = candidate.name_node.text();
        let (constraint_text, constraint_node) =
            get_type_parameter_constraint_text(ctx, candidate.node);
        let type_parameter_reference_range = match candidate.references.first() {
            Some(&first) => ctx.trim(first),
            None => ctx.trim(candidate.name_node),
        };
        let d = build_sole_message(
            ctx.trim(candidate.node),
            type_parameter_reference_range,
            type_parameter_name,
            uses,
            descriptor,
        );
        ctx.report_diagnostic_with_suggestions(d, |ctx| {
            let removal_range =
                get_type_parameter_list_removal_range(ctx, type_parameters, candidate.node);
            let mut fixes: Vec<RuleFix> = Vec::with_capacity(candidate.references.len() + 1);
            for &reference in &candidate.references {
                let reference_range = ctx.trim(reference);
                if reference_range.0 < removal_range.end()
                    && reference_range.1 > removal_range.pos()
                {
                    continue;
                }
                let mut replacement = constraint_text.clone();
                if is_complex_constraint(constraint_node) && has_matching_ancestor_type(reference) {
                    replacement = format!("({replacement})");
                }
                fixes.push(ctx.fix_replace(reference, replacement));
            }
            fixes.push(ctx.fix_remove_range(removal_range.pos(), removal_range.end()));
            vec![RuleSuggestion { message: build_replace_usages_with_constraint_message(), fixes }]
        });
    }
}

pub struct NoUnnecessaryTypeParameters;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnnecessaryTypeParameters))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ArrowFunction),
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Enter(Kind::CallSignature),
    Listener::Enter(Kind::ConstructSignature),
    Listener::Enter(Kind::ConstructorType),
    Listener::Enter(Kind::FunctionType),
    Listener::Enter(Kind::MethodSignature),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Enter(Kind::ClassDeclaration),
    Listener::Enter(Kind::ClassExpression),
];

impl Rule for NoUnnecessaryTypeParameters {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-parameters"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.type_parameters().is_empty() {
            return;
        }
        let descriptor = match node.kind() {
            Kind::ClassDeclaration | Kind::ClassExpression => "class",
            _ => "function",
        };
        check_no_unnecessary_type_parameters_node(ctx, node, descriptor);
    }
}
