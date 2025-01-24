//! Prettier IR builder macros
//! Ref: <https://github.com/prettier/prettier/blob/3.4.2/commands.md>

// NOTE: In addition to those defined here, there are still some that are not yet implemented.
// In terms of macro implementation, there are 2 forms: the most frequently used form and the most flexible form.

/// Arrays are used to concatenate a list of `Doc`s to be printed sequentially into a single doc.
/// (In Prettier(.js) this is just an array literal.)
/// ```
/// array!(p, [a, b, c]);
/// array!(p, vec);
/// ```
#[macro_export]
macro_rules! array {
    ($p:ident, [$( $x:expr ),* $(,)?]) => {{
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        $(
            temp_vec.push($x);
        )*
        array!($p, temp_vec)
    }};
    ($p:ident, $vec:expr) => {{
        $crate::ir::Doc::Array($vec)
    }};
}

/// Strings are printed directly as is.
/// However for the algorithm to work properly they shouldn't contain line break characters.
/// (In Prettier(.js) this is just a string literal.)
/// ```
/// text!("const");
/// ```
#[macro_export]
macro_rules! text {
    ($str:expr) => {{
        let s: &'static str = $str;
        $crate::ir::Doc::Str(s)
    }};
}

/// Strings are printed directly as is.
/// However for the algorithm to work properly they shouldn't contain line break characters.
/// (In Prettier(.js) this is just a string literal.)
/// ```
/// dynamic_text!(value.as_str());
/// ```
#[macro_export]
macro_rules! dynamic_text {
    ($p:ident, $str:expr) => {{
        let s = $p.allocator.alloc_str($str);
        $crate::ir::Doc::Str(s)
    }};
}

/// Mark a group of items which the printer should try to fit on one line.
/// This is the basic command to tell the printer when to break.
/// Groups are usually nested, and the printer will try to fit everything on one line,
/// but if it doesn't fit it will break the outermost group first and try again.
/// It will continue breaking groups until everything fits (or there are no more groups to break).
///
/// A group is forced to break if it's created with the `should_break` option set to true or if it includes `break_parent`.
/// A hard and literal line breaks automatically include this so they always break parent groups.
/// Breaks are propagated to all parent groups, so if a deeply nested expression has a hard break, everything will break.
/// This only matters for "hard" breaks, i.e. newlines that are printed no matter what and can be statically analyzed.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#group>
/// ```
/// group!(p, [a, b, c], true, Some(group_id));
/// group!(p, vec, true, None);
/// group!(p, [a, b, c]);
/// group!(p, vec);
/// ```
#[macro_export]
macro_rules! group {
    ($p:ident, [$( $x:expr ),* $(,)?], $should_break:expr, $group_id:expr) => {{
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        $(
            temp_vec.push($x);
        )*
        $crate::ir::Doc::Group($crate::ir::Group {
            contents: temp_vec,
            should_break: $should_break,
            expanded_states: None,
            group_id: $group_id,
        })
    }};
    ($p:ident, $vec:expr, $should_break:expr, $group_id:expr) => {{
        $crate::ir::Doc::Group($crate::ir::Group {
            contents: $vec,
            should_break: $should_break,
            expanded_states: None,
            group_id: $group_id,
        })
    }};
    ($p:ident, [$( $x:expr ),* $(,)?]) => {{
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        $(
            temp_vec.push($x);
        )*
        group!($p, temp_vec, false, None)
    }};
    ($p:ident, $vec:expr) => {{
        group!($p, $vec, false, None)
    }};
}

/// This should be used as last resort as it triggers an exponential complexity when nested.
/// This will try to print the first alternative, if it fit use it, otherwise go to the next one and so on.
/// The alternatives is an array of documents going from the least expanded (most flattened) representation first to the most expanded.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#conditionalgroup>
/// ```
/// conditional_group!(p, [a, b, c]);
/// ```
#[macro_export]
macro_rules! conditional_group {
    ($p:ident, [$doc:expr, $( $x:expr ),* $(,)?]) => {{
        let mut temp_single = oxc_allocator::Vec::with_capacity_in(1, $p.allocator);
        temp_single.push($doc);
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        $(
            temp_vec.push($x);
        )*

        $crate::ir::Doc::Group($crate::ir::Group {
            contents: temp_single,
            should_break: false,
            expanded_states: Some(temp_vec),
            group_id: None,
        })
    }};
}

/// This is an alternative type of group which behaves like text layout:
/// it's going to add a break whenever the next element doesn't fit in the line anymore.
/// The difference with `group` is that it's not going to break all the separators, just the ones that are at the end of lines.
///
/// Expects the arguments to be an array of alternating content and line breaks.
/// In other words, elements with odd indices must be line breaks (e.g., `softline`).
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#fill>
/// ```
/// fill!(p, [a, line!(), b, line!(), c]);
/// fill!(p, vec);
/// ```
#[macro_export]
macro_rules! fill {
    ($p:ident, [$( $x:expr ),* $(,)?]) => {{
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        $(
            temp_vec.push($x);
        )*
        fill!($p, temp_vec)
    }};
    ($p:ident, $vec:expr) => {{
        $crate::ir::Doc::Fill($crate::ir::Fill { contents: $vec })
    }};
}

/// Print something if the current group or the current element of fill breaks and something else if it doesn't.
/// `group_id` can be used to check another already printed group instead of the current group.
///
/// If a `hardline` or `break_parent` is present within the possible contents,
/// the parent groups will be broken regardless of said content being printed, which might not be desirable.
/// This behaviour is a design limitation.
/// Usually the desired result can be achieved in a different way.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#ifbreak>
/// ```
/// if_break!(p, a, b, Some(group_id));
/// if_break!(p, a);
/// ```
#[macro_export]
macro_rules! if_break {
    ($p:ident, $break:expr, $flat:expr, $group_id:expr) => {{
        $crate::ir::Doc::IfBreak($crate::ir::IfBreak {
            break_contents: oxc_allocator::Box::new_in($break, $p.allocator),
            flat_contents: oxc_allocator::Box::new_in($flat, $p.allocator),
            group_id: $group_id,
        })
    }};
    ($p:ident, $break:expr) => {{
        use $crate::text;
        if_break!($p, $break, text!(""), None)
    }};
}

/// Include this anywhere to force all parent groups to break. See `group` for more info.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#breakparent>
/// ```
/// break_parent!();
/// ```
#[macro_export]
macro_rules! break_parent {
    () => {{
        $crate::ir::Doc::BreakParent
    }};
}

/// Join an array of docs with a separator.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#join>
/// ```
/// join!(p, JoinSeparator::Softline, vec);
/// ```
#[macro_export]
macro_rules! join {
    ($p:ident, $sep:expr, $vec:expr) => {{
        let mut parts = oxc_allocator::Vec::new_in($p.allocator);
        for (i, doc) in $vec.into_iter().enumerate() {
            if i != 0 {
                match $sep {
                    $crate::ir::JoinSeparator::Softline => parts.push($crate::softline!()),
                    $crate::ir::JoinSeparator::Hardline => parts.push($crate::hardline!($p)),
                    $crate::ir::JoinSeparator::CommaLine => {
                        parts.extend([$crate::text!(","), $crate::line!()]);
                    }
                    $crate::ir::JoinSeparator::CommaSpace => parts.push($crate::text!(", ")),
                    $crate::ir::JoinSeparator::Literalline => parts.push($crate::literalline!($p)),
                }
            }
            parts.push(doc);
        }
        $crate::ir::Doc::Array(parts)
    }};
}

/// Specify a line break.
/// If an expression fits on one line, the line break will be replaced with a space.
/// Line breaks always indent the next line with the current level of indentation.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#line>
/// ```
/// line!();
/// ```
#[macro_export]
macro_rules! line {
    () => {{
        $crate::ir::Doc::Line($crate::ir::Line::default())
    }};
}

/// Specify a line break.
/// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#softline>
/// ```
/// softline!();
/// ```
#[macro_export]
macro_rules! softline {
    () => {{
        $crate::ir::Doc::Line($crate::ir::Line { soft: true, ..Default::default() })
    }};
}

/// Specify a line break that is always included in the output, no matter if the expression fits on one line or not.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#hardline>
/// ```
/// hardline!(p);
/// ```
#[macro_export]
macro_rules! hardline {
    ($p:ident) => {{
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        temp_vec.push($crate::ir::Doc::Line($crate::ir::Line { hard: true, ..Default::default() }));
        temp_vec.push($crate::ir::Doc::BreakParent);
        $crate::ir::Doc::Array(temp_vec)
    }};
}

/// Specify a line break that is always included in the output and doesn't indent the next line.
/// Also, unlike hardline, this kind of line break preserves trailing whitespace on the line it ends.
/// This is used for template literals.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#literalline>
/// ```
/// literalline!(p);
/// ```
#[macro_export]
macro_rules! literalline {
    ($p:ident) => {{
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        temp_vec.push($crate::ir::Doc::Line($crate::ir::Line {
            hard: true,
            literal: true,
            ..Default::default()
        }));
        temp_vec.push($crate::ir::Doc::BreakParent);
        $crate::ir::Doc::Array(temp_vec)
    }};
}

/// In cases where you embed code inside of templates, comments shouldn't be able to leave the code part.
/// `line_suffix_boundary` is an explicit marker you can use to flush the `line_suffix` buffer in addition to line breaks.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#linesuffixboundary>
/// ```
/// line_suffix_boundary!();
/// ```
#[macro_export]
macro_rules! line_suffix_boundary {
    () => {{
        $crate::ir::Doc::LineSuffixBoundary
    }};
}

/// Increase the level of indentation.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#indent>
/// ```
/// indent!(p, [a, b, c]);
/// indent!(p, vec);
/// ```
#[macro_export]
macro_rules! indent {
    ($p:ident, [$( $x:expr ),* $(,)?]) => {{
        let mut temp_vec = oxc_allocator::Vec::new_in($p.allocator);
        $(
            temp_vec.push($x);
        )*
        $crate::ir::Doc::Indent(temp_vec)
    }};
    ($p:ident, $vec:expr) => {{
        $crate::ir::Doc::Indent($vec)
    }};
}

/// An optimized version of `if_break(indent(doc), doc, group_id)`.
/// It doesn't make sense to apply `indent_if_break` to the current group,
/// because "indent if the current group is broken" is the normal behavior of indent.
/// That's why `group_id` is required.
/// <https://github.com/prettier/prettier/blob/3.4.2/commands.md#indentifbreak>
/// ```
/// indent_if_break!(p, a, group_id);
/// ```
#[macro_export]
macro_rules! indent_if_break {
    ($p:ident, $doc:expr, $group_id:expr) => {{
        $crate::ir::Doc::IndentIfBreak($crate::ir::IndentIfBreak {
            contents: oxc_allocator::Box::new_in($doc, $p.allocator),
            group_id: $group_id,
        })
    }};
}

// ---

/// `wrap!` macro,
/// - to save the reference of the current node to be used as parent node later
/// - to print parens and comments
///
/// NOTE: `wrap!` is not used by all AST nodes that implement `Format` trait.
/// This may be or may not be a problem.
#[macro_export]
macro_rules! wrap {
    ($p:ident, $self:expr, $kind:ident, $block:block) => {{
        let kind = oxc_ast::AstKind::$kind($p.alloc($self));
        $p.enter_node(kind);

        let leading = $p.print_leading_comments(kind.span());

        let doc = $block;
        let doc = if $p.need_parens(kind) {
            $crate::array!($p, [$crate::text!("("), doc, $crate::text!(")")])
        } else {
            doc
        };

        // TODO: dangling comments?
        let trailing = $p.print_trailing_comments(kind.span());

        let doc = $p.print_comments(leading, doc, trailing);

        $p.leave_node();
        doc
    }};
}
