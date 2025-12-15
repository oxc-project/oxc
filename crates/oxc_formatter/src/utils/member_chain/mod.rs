pub mod chain_member;
pub mod groups;
pub mod simple_argument;

use std::iter;

use crate::{
    JsLabels,
    ast_nodes::{AstNode, AstNodes},
    best_fitting,
    formatter::{Buffer, Format, Formatter, prelude::*},
    utils::{
        is_long_curried_call,
        member_chain::{
            chain_member::{CallExpressionPosition, ChainMember},
            groups::{MemberChainGroup, MemberChainGroupsBuilder, TailChainGroups},
            simple_argument::SimpleArgument,
        },
    },
    write,
};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use super::typecast::is_type_cast_node;

#[derive(Debug)]
pub struct MemberChain<'a, 'b> {
    root: &'b AstNode<'a, CallExpression<'a>>,
    head: MemberChainGroup<'a, 'b>,
    tail: TailChainGroups<'a, 'b>,
}

impl<'a, 'b> MemberChain<'a, 'b> {
    pub(crate) fn from_call_expression(
        call_expression: &'b AstNode<'a, CallExpression<'a>>,
        f: &Formatter<'_, 'a>,
    ) -> Self {
        let mut chain_members = chain_members_iter(call_expression, f).collect::<Vec<_>>();
        chain_members.reverse();

        // as explained before, the first group is particular, so we calculate it
        let remaining_members_start_index = get_split_index_of_head_and_tail_groups(&chain_members);

        // `flattened_items` now contains only the nodes that should have a sequence of
        // `[ StaticMemberExpression -> AnyNode + CallExpression ]`
        let tail_groups =
            compute_remaining_groups(chain_members.drain(remaining_members_start_index..), f);
        let head_group = MemberChainGroup::from(chain_members);

        let mut member_chain = Self { head: head_group, tail: tail_groups, root: call_expression };

        // Here we check if the first element of Groups::groups can be moved inside the head.
        // If so, then we extract it and concatenate it together with the head.
        member_chain.maybe_merge_with_first_group(call_expression.parent, f);

        member_chain
    }

    /// Here we check if the first group can be merged to the head. If so, then
    /// we move out the first group out of the groups
    fn maybe_merge_with_first_group(&mut self, parent: &AstNodes<'a>, f: &Formatter<'_, 'a>) {
        if self.should_merge_tail_with_head(parent, f) {
            let group = self.tail.pop_first().unwrap();
            self.head.extend_members(group.into_members());
        }
    }

    /// This function checks if the current grouping should be merged with the first group.
    fn should_merge_tail_with_head(&self, parent: &AstNodes<'a>, f: &Formatter<'_, 'a>) -> bool {
        let Some(first_group) = self.tail.first() else {
            return false;
        };

        let has_comment = first_group.members().first().is_some_and(|member| {
            matches!(member, ChainMember::StaticMember(expression)
                if f.context().comments().has_comment_in_range(
                    expression.object().span().end,
                    expression.property().span.start
                )
            )
        });

        if has_comment {
            return false;
        }

        let has_computed_property =
            first_group.members().first().is_some_and(ChainMember::is_computed_expression);

        if self.head.members().len() == 1
            && let ChainMember::Node(node) = self.head.members()[0]
        {
            match node.as_ref() {
                Expression::Identifier(identifier) => {
                    has_computed_property ||
                    is_factory(&identifier.name) ||
                    // If an identifier has a name that is shorter than the tab width, then we join it with the "head"
                    (matches!(parent.without_chain_expression(), AstNodes::ExpressionStatement(stmt) if !stmt.is_arrow_function_body())
                        && has_short_name(&identifier.name, f.options().indent_width.value()))
                }
                Expression::ThisExpression(_) => true,
                _ => false,
            }
        } else if let Some(ChainMember::StaticMember(expression)) = self.head.members().last() {
            has_computed_property || is_factory(&expression.property().name)
        } else {
            false
        }
    }

    /// To keep the formatting order consistent, we need to inspect all member chain groups in order.
    fn inspect_member_chain_groups(&self, f: &mut Formatter<'_, 'a>) {
        self.head.inspect(false, f);

        for member in self.tail.iter() {
            member.inspect(true, f);
        }
    }

    /// It tells if the groups should break on multiple lines
    fn groups_should_break(&self, f: &Formatter<'_, 'a>) -> bool {
        let mut call_expressions = self
            .members()
            .filter_map(|member| match member {
                ChainMember::CallExpression { expression, .. } => Some(expression),
                _ => None,
            })
            .peekable();

        let mut calls_count = 0;
        let mut has_function_like_argument = false;
        let mut has_complex_args = false;

        while let Some(call) = call_expressions.next() {
            calls_count += 1;

            if call_expressions.peek().is_some() {
                has_function_like_argument =
                    has_function_like_argument || has_arrow_or_function_expression_arg(call);
            }

            has_complex_args = has_complex_args || !has_simple_arguments(call);

            if calls_count > 2 && has_complex_args {
                return true;
            }
        }

        if !self.tail.is_empty() && self.head.will_break(f) {
            return true;
        }

        if self.last_call_breaks(f) && has_function_like_argument {
            return true;
        }

        self.tail.any_except_last_will_break(f)
    }

    /// We retrieve all the call expressions inside the group and we check if
    /// their arguments are not simple.
    fn last_call_breaks(&self, f: &Formatter<'_, 'a>) -> bool {
        let last_group = self.last_group();

        if matches!(last_group.members().last(), Some(ChainMember::CallExpression { .. })) {
            last_group.will_break(f)
        } else {
            false
        }
    }

    fn last_group(&self) -> &MemberChainGroup<'a, 'b> {
        self.tail.last().unwrap_or(&self.head)
    }

    /// Returns an iterator over all members in the member chain
    fn members(&self) -> impl Iterator<Item = &ChainMember<'a, 'b>> {
        self.head.members().iter().chain(self.tail.members())
    }

    fn has_comment(&self, f: &Formatter<'_, 'a>) -> bool {
        let comments = f.comments();

        for member in self.members() {
            if matches!(
                member,
                ChainMember::StaticMember(member)
                    if comments.has_comment_in_range(member.object().span().end, member.property().span.start)
            ) {
                return true;
            }
        }

        false
    }
}

impl<'a> Format<'a> for MemberChain<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let has_comment = self.has_comment(f);
        let format_one_line = format_with(|f| {
            f.join().entries(iter::once(&self.head).chain(self.tail.iter()));
        });

        self.inspect_member_chain_groups(f);

        let has_new_line_or_comment_between =
            self.tail.iter().any(MemberChainGroup::needs_empty_line);

        if self.tail.len() <= 1 && !has_comment && !has_new_line_or_comment_between {
            return if is_long_curried_call(self.root) {
                write!(f, [format_one_line]);
            } else {
                write!(f, [group(&format_one_line)]);
            };
        }

        let format_tail = format_with(|f| {
            for group in self.tail.iter() {
                if group.needs_empty_line() {
                    write!(f, [empty_line()]);
                } else {
                    write!(f, [hard_line_break()]);
                }
                write!(f, [group]);
            }
        });
        let format_expanded = format_with(|f| write!(f, [self.head, indent(&format_tail)]));

        let format_content = format_with(|f| {
            if has_comment || has_new_line_or_comment_between || self.groups_should_break(f) {
                write!(f, [group(&format_expanded)]);
            } else {
                let has_empty_line_before_tail =
                    self.tail.first().is_some_and(MemberChainGroup::needs_empty_line);

                if has_empty_line_before_tail || self.last_group().will_break(f) {
                    write!(f, [expand_parent()]);
                }

                write!(f, [best_fitting!(format_one_line, format_expanded)]);
            }
        });

        write!(f, [labelled(LabelId::of(JsLabels::MemberChain), &format_content)]);
    }
}

/// Returns the index where the head group ends and the tail groups begin.
fn get_split_index_of_head_and_tail_groups(members: &[ChainMember<'_, '_>]) -> usize {
    // This where we apply the first two points explained in the description of the main public function.
    // We want to keep iterating over the items until we have call expressions
    // - `something()()()()`
    // - `something[1][2][4]`
    // - `something[1]()[3]()`
    // - `something()[2].something.else[0]`
    let non_call_or_array_member_access_start = members
        .iter()
        // The first member is always part of the first group
        .skip(1)
        .position(|member| match member {
            ChainMember::CallExpression { .. } | ChainMember::TSNonNullExpression(_) => false,
            ChainMember::ComputedMember(expression) => {
                !matches!(&expression.expression, Expression::NumericLiteral(_))
            }
            _ => true,
        })
        .map_or(members.len(), |index| {
            // `skip(1)` causes the first member to be skipped, so we need to add 1 to the index
            index + 1
        });

    if members.first().is_some_and(ChainMember::is_call_expression) {
        non_call_or_array_member_access_start
    } else {
        // Take as many member access chains as possible
        let rest = &members[non_call_or_array_member_access_start..];

        let member_end = rest
            .iter()
            .position(|member| {
                !matches!(
                    member,
                    ChainMember::StaticMember { .. } | ChainMember::ComputedMember { .. }
                )
            })
            .map_or(rest.len(), |index| {
                // `index - 1` is the last member of the group
                index - 1
            });

        non_call_or_array_member_access_start + member_end
    }
}

/// computes groups coming after the first group
fn compute_remaining_groups<'a, 'b>(
    members: impl IntoIterator<Item = ChainMember<'a, 'b>>,
    f: &Formatter<'_, 'a>,
) -> TailChainGroups<'a, 'b> {
    let mut has_seen_call_expression = false;
    let mut groups_builder = MemberChainGroupsBuilder::default();

    for member in members {
        let span = member.span();
        let has_trailing_comment =
            f.comments().comments_after(span.end).first().is_some_and(|comment| {
                f.source_text().bytes_range(span.end, comment.span.start).trim_ascii().is_empty()
            });

        match member {
            // [0] should be appended at the end of the group instead of the
            // beginning of the next one
            ChainMember::ComputedMember(_) if is_computed_array_member_access(&member) => {
                groups_builder.start_or_continue_group(member);
            }
            ChainMember::StaticMember(_) | ChainMember::ComputedMember(_) => {
                // if we have seen a CallExpression, we want to close the group.
                // The resultant group will be something like: [ . , then, () ];
                // `.` and `then` belong to the previous StaticMemberExpression,
                // and `()` belong to the call expression we just encountered
                if has_seen_call_expression {
                    groups_builder.close_group();
                    groups_builder.start_group(member);
                    has_seen_call_expression = false;
                } else {
                    groups_builder.start_or_continue_group(member);
                }
            }
            ChainMember::CallExpression { .. } => {
                groups_builder.start_or_continue_group(member);
                has_seen_call_expression = true;
            }
            ChainMember::TSNonNullExpression(_) => {
                groups_builder.start_or_continue_group(member);
            }
            ChainMember::Node(_) => unreachable!("Remaining members never have a `Node` variant"),
        }

        // Close the group immediately if the node had any trailing comments to
        // ensure those are printed in a trailing position for the token they
        // were originally commenting
        if has_trailing_comment {
            groups_builder.close_group();
            has_seen_call_expression = false;
        }
    }

    groups_builder.finish()
}

fn is_computed_array_member_access(member: &ChainMember<'_, '_>) -> bool {
    matches!(member, ChainMember::ComputedMember(expression)
        if matches!(&expression.expression, Expression::NumericLiteral(_))
    )
}

fn has_arrow_or_function_expression_arg(call: &AstNode<'_, CallExpression<'_>>) -> bool {
    call.as_ref().arguments.iter().any(|argument| {
        matches!(&argument, Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_))
    })
}

fn has_simple_arguments<'a>(call: &AstNode<'a, CallExpression<'a>>) -> bool {
    call.arguments().iter().all(|argument| SimpleArgument::new(argument).is_simple())
}

/// In order to detect those cases, we use an heuristic: if the first
/// node is an identifier with the name starting with a capital
/// letter or just a sequence of _$. The rationale is that they are
/// likely to be factories.
fn is_factory(token: &str) -> bool {
    let mut bytes = token.bytes();

    match bytes.next() {
        // Any sequence of '$' or '_' characters
        Some(b'_' | b'$') => bytes.all(|b| matches!(b, b'_' | b'$')),
        Some(c) => c.is_ascii_uppercase(),
        _ => false,
    }
}

/// Here we check if the length of the groups exceeds the cutoff or there are comments
/// This function is the inverse of the prettier function
/// [Prettier applies]: <https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/member-chain.js#L342>
pub fn is_member_call_chain<'a>(
    expression: &AstNode<'a, CallExpression<'a>>,
    f: &Formatter<'_, 'a>,
) -> bool {
    MemberChain::from_call_expression(expression, f).tail.is_member_call_chain()
}

fn has_short_name(name: &Atom, tab_width: u8) -> bool {
    name.as_str().len() <= tab_width as usize
}

fn chain_members_iter<'a, 'b>(
    root: &'b AstNode<'a, CallExpression<'a>>,
    f: &Formatter<'_, 'a>,
) -> impl Iterator<Item = ChainMember<'a, 'b>> {
    let mut is_root = true;
    let mut next: Option<&'b AstNode<'a, Expression<'a>>> = None;

    iter::from_fn(move || {
        let handle_call_expression =
            |position: CallExpressionPosition,
             expr: &'b AstNode<'a, CallExpression<'a>>,
             next: &mut Option<&'b AstNode<'a, Expression<'a>>>| {
                let callee = expr.callee();

                let is_chain = matches!(
                    callee.as_ref(),
                    Expression::StaticMemberExpression(_)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::CallExpression(_)
                );

                if is_chain {
                    *next = Some(callee);
                }

                ChainMember::CallExpression { expression: expr, position }
            };

        if is_root {
            is_root = false;
            return Some(handle_call_expression(CallExpressionPosition::End, root, &mut next));
        }

        let expression = next.take()?;

        if is_type_cast_node(expression, f).is_some() {
            return ChainMember::Node(expression).into();
        }

        let member = match expression.as_ast_nodes() {
            AstNodes::CallExpression(expr) => {
                let callee = expr.callee();
                let is_chain = matches!(
                    callee.as_ref(),
                    Expression::StaticMemberExpression(_)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::CallExpression(_)
                );
                let position = if is_chain {
                    CallExpressionPosition::Middle
                } else {
                    CallExpressionPosition::Start
                };
                handle_call_expression(position, expr, &mut next)
            }
            AstNodes::StaticMemberExpression(expr) => {
                next = Some(expr.object());
                ChainMember::StaticMember(expr)
            }
            AstNodes::ComputedMemberExpression(expr) => {
                next = Some(expr.object());
                ChainMember::ComputedMember(expr)
            }
            AstNodes::TSNonNullExpression(expr) => {
                next = Some(expr.expression());
                ChainMember::TSNonNullExpression(expr)
            }
            _ => ChainMember::Node(expression),
        };

        Some(member)
    })
}
