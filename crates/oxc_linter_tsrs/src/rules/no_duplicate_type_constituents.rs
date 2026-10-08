// Port of internal/rules/no_duplicate_type_constituents/no_duplicate_type_constituents.go.

use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

#[derive(Clone, Copy, PartialEq, Eq)]
enum UnionOrIntersection {
    Union,
    Intersection,
}

fn build_duplicate_message(
    union_or_intersection: UnionOrIntersection,
    previous: &str,
) -> RuleMessage {
    let msg = if union_or_intersection == UnionOrIntersection::Intersection {
        "Intersection"
    } else {
        "Union"
    };
    RuleMessage::new("duplicate", format!("{msg} type constituent is duplicated with {previous}."))
}
fn build_unnecessary_message() -> RuleMessage {
    RuleMessage::new("unnecessary", "Explicit undefined is unnecessary on an optional parameter.")
}

pub struct NoDuplicateTypeConstituents {
    listeners: Vec<Listener>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let ignore_intersections = opt_bool(&m, "ignoreIntersections", false);
    let ignore_unions = opt_bool(&m, "ignoreUnions", false);
    let mut listeners = Vec::with_capacity(2);
    if !ignore_intersections {
        listeners.push(Listener::Enter(Kind::IntersectionType));
    }
    if !ignore_unions {
        listeners.push(Listener::Enter(Kind::UnionType));
    }
    Ok(Box::new(NoDuplicateTypeConstituents { listeners }))
}

impl Rule for NoDuplicateTypeConstituents {
    fn name(&self) -> &'static str {
        "no-duplicate-type-constituents"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoDuplicateTypeConstituents,
}

fn unwinded_parent_type(mut node: P<Node>, kind: Kind) -> Option<P<Node>> {
    loop {
        node = node.parent()?;
        if node.kind() == kind {
            return Some(node);
        }
        if node.kind() != Kind::ParenthesizedType {
            return None;
        }
    }
}

fn remove_range(r: tsrs_core::TextRange) -> RuleFix {
    RuleFix { text: String::new(), pos: r.pos(), end: r.end() }
}

fn compute_fixes(
    ctx: &mut Ctx,
    union_or_intersection: UnionOrIntersection,
    constituent_node: P<Node>,
) -> Vec<RuleFix> {
    let kind = if union_or_intersection == UnionOrIntersection::Intersection {
        Kind::IntersectionType
    } else {
        Kind::UnionType
    };
    let parent = unwinded_parent_type(constituent_node, kind).unwrap();
    let mut s = tsrs_scanner::get_scanner_for_source_file(ctx.file, parent.pos());
    let mut found_before = false;
    let mut prev_start = 0;
    let mut bracket_before_tokens: Vec<tsrs_core::TextRange> = Vec::new();
    loop {
        if s.token_start() >= constituent_node.pos() {
            break;
        }
        if s.token() == Kind::AmpersandToken || s.token() == Kind::BarToken {
            found_before = true;
            prev_start = s.token_start();
            bracket_before_tokens.clear();
        } else if s.token() == Kind::OpenParenToken {
            bracket_before_tokens.push(s.token_range());
        }
        s.scan();
    }
    let (pos, end) = ctx.trim(constituent_node);
    let mut fixes = vec![ctx.fix_remove_range(pos, end)];
    if found_before {
        fixes.push(ctx.fix_remove_range(prev_start, prev_start + 1));
        for &before in &bracket_before_tokens {
            fixes.push(remove_range(before));
        }
        s.reset_pos(constituent_node.end());
        for _ in &bracket_before_tokens {
            s.scan();
            if s.token() != Kind::CloseParenToken {
                panic!("expected next scanned token to be ')', got '{:?}'", s.token());
            }
            fixes.push(remove_range(s.token_range()));
        }
    } else {
        s.reset_pos(constituent_node.end());
        let mut closing_parens_count = 0;
        loop {
            s.scan();
            if s.token_start() >= parent.end() {
                panic!("couldn't find '&' or '|' token");
            }
            if s.token() == Kind::AmpersandToken || s.token() == Kind::BarToken {
                fixes.push(remove_range(s.token_range()));
                break;
            }
            if s.token() != Kind::CloseParenToken {
                panic!("expected next scanned token to be ')', got '{:?}'", s.token());
            }
            closing_parens_count += 1;
            fixes.push(remove_range(s.token_range()));
        }

        let mut opening_parens: Vec<tsrs_core::TextRange> =
            Vec::with_capacity(closing_parens_count);
        s.reset_pos(parent.pos());
        for _ in 0..closing_parens_count {
            s.scan();
            if s.token() == Kind::OpenParenToken {
                if opening_parens.len() < closing_parens_count {
                    opening_parens.push(s.token_range());
                }
            } else {
                opening_parens.clear();
            }
            if s.token_start() == constituent_node.pos() {
                if opening_parens.len() != closing_parens_count {
                    panic!(
                        "expected to find {} opening parens, found only {}",
                        closing_parens_count,
                        opening_parens.len()
                    );
                }
                break;
            }
            for &r in &opening_parens {
                fixes.push(remove_range(r));
            }
        }
    }
    fixes
}

fn report(
    ctx: &mut Ctx,
    with_fix: bool,
    union_or_intersection: UnionOrIntersection,
    message: RuleMessage,
    constituent_node: P<Node>,
    first_occurrence: Option<P<Node>>,
) {
    let mut labeled_ranges = Vec::new();
    if let Some(first) = first_occurrence {
        let (fpos, fend) = ctx.trim(first);
        let first_text = &ctx.text()[fpos as usize..fend as usize];
        labeled_ranges.push(LabeledRange {
            label: format!("Type '{first_text}' is first declared here."),
            pos: fpos,
            end: fend,
        });
    }
    let (pos, end) = ctx.trim(constituent_node);
    let d = RuleDiagnostic { pos, end, message, labeled_ranges };
    if !with_fix {
        ctx.report_diagnostic(d);
        return;
    }
    ctx.report_diagnostic_with_fixes(d, |ctx| {
        compute_fixes(ctx, union_or_intersection, constituent_node)
    });
}

/// The Go forEachNodeType callback of the union listener: reports explicit `undefined` on an
/// optional parameter. `union_node` is the listener's node.
fn for_each_node_type(ctx: &mut Ctx, union_node: P<Node>, t: P<Type>, constituent_node: P<Node>) {
    let Some(parent) = union_node.parent() else {
        return;
    };
    if !ast::is_parameter_declaration(parent) {
        return;
    }
    if parent.as_parameter_declaration().question_token.get().is_none() {
        return;
    }
    if utils::is_type_flag_set(t, TypeFlags::Undefined) {
        report(
            ctx,
            true,
            UnionOrIntersection::Union,
            build_unnecessary_message(),
            constituent_node,
            None,
        );
    }
}

fn check_duplicate_recursively(
    ctx: &mut Ctx,
    with_fix: bool,
    union_or_intersection: UnionOrIntersection,
    constituent_node: P<Node>,
    cached_type_map: &mut FxHashMap<P<Type>, P<Node>>,
    union_node: Option<P<Node>>,
) -> bool {
    let t = ctx.checker.get_type_at_location(constituent_node);
    if utils::is_intrinsic_error_type(t) {
        return false;
    }

    if let Some(&previous) = cached_type_map.get(&t) {
        let previous_text = &ctx.text()[previous.pos() as usize..previous.end() as usize];
        report(
            ctx,
            with_fix,
            union_or_intersection,
            build_duplicate_message(union_or_intersection, previous_text),
            constituent_node,
            Some(previous),
        );
        return true;
    }

    if let Some(union_node) = union_node {
        for_each_node_type(ctx, union_node, t, constituent_node);
    }
    cached_type_map.insert(t, constituent_node);

    let without_parens = ast::skip_type_parentheses(constituent_node);
    let types = if union_or_intersection == UnionOrIntersection::Union
        && without_parens.kind() == Kind::UnionType
    {
        without_parens.as_union_type_node().types().nodes()
    } else if union_or_intersection == UnionOrIntersection::Intersection
        && without_parens.kind() == Kind::IntersectionType
    {
        without_parens.as_intersection_type_node().types().nodes()
    } else {
        return false;
    };

    let mut all_prev_removed = true;
    for (i, &constituent) in types.iter().enumerate() {
        if !check_duplicate_recursively(
            ctx,
            i < types.len() - 1 || !all_prev_removed,
            union_or_intersection,
            constituent,
            cached_type_map,
            union_node,
        ) {
            all_prev_removed = false;
        }
    }
    false
}

fn check_duplicate(ctx: &mut Ctx, node: P<Node>, union_node: Option<P<Node>>) {
    let mut cached_type_map: FxHashMap<P<Type>, P<Node>> = FxHashMap::default();
    let (union_or_intersection, types) = match node.kind() {
        Kind::IntersectionType => {
            (UnionOrIntersection::Intersection, node.as_intersection_type_node().types().nodes())
        }
        Kind::UnionType => (UnionOrIntersection::Union, node.as_union_type_node().types().nodes()),
        k => panic!("expected union or intersection, got {k:?}"),
    };
    for &t in types {
        check_duplicate_recursively(
            ctx,
            true,
            union_or_intersection,
            t,
            &mut cached_type_map,
            union_node,
        );
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &self.rule.listeners
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::IntersectionType => {
                if unwinded_parent_type(node, Kind::IntersectionType).is_some() {
                    return;
                }
                check_duplicate(ctx, node, None);
            }
            Kind::UnionType => {
                if unwinded_parent_type(node, Kind::UnionType).is_some() {
                    return;
                }
                check_duplicate(ctx, node, Some(node));
            }
            _ => {}
        }
    }
}
