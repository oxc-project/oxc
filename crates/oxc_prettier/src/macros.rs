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

#[macro_export]
macro_rules! text {
    ($s:expr) => {{
        let s: &'static str = $s;
        $crate::ir::Doc::Str(s)
    }};
}

#[macro_export]
macro_rules! space {
    () => {{
        $crate::ir::Doc::Str(" ")
    }};
}

#[macro_export]
macro_rules! dynamic_text {
    ($p:ident, $s:expr) => {{
        let s = oxc_allocator::String::from_str_in($s, $p.allocator).into_bump_str();
        $crate::ir::Doc::Str(s)
    }};
}

#[macro_export]
macro_rules! line {
    () => {{
        $crate::ir::Doc::Line($crate::ir::Line::default())
    }};
}

/// Specify a line break.
/// The difference from line is that if the expression fits on one line, it will be replaced with nothing.
#[macro_export]
macro_rules! softline {
    () => {{
        $crate::ir::Doc::Line($crate::ir::Line { soft: true, ..Default::default() })
    }};
}

/// Specify a line break that is **always** included in the output,
/// no matter if the expression fits on one line or not.
#[macro_export]
macro_rules! hardline {
    () => {{
        let hardline = $crate::ir::Doc::Line($crate::ir::Line { hard: true, ..Default::default() });
        [hardline, $crate::ir::Doc::BreakParent]
    }};
}

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

#[macro_export]
macro_rules! conditional_group {
    ($p:ident, [$d:expr, $( $x:expr ),* $(,)?]) => {{
        let mut temp_single = oxc_allocator::Vec::with_capacity_in(1, $p.allocator);
        temp_single.push($d);
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

#[macro_export]
macro_rules! if_break {
    ($p:ident, $b:expr, $f:expr, $gid:expr) => {{
        $crate::ir::Doc::IfBreak($crate::ir::IfBreak {
            break_contents: oxc_allocator::Box::new_in($b, $p.allocator),
            flat_contents: oxc_allocator::Box::new_in($f, $p.allocator),
            group_id: $gid,
        })
    }};
    ($p:ident, $b:expr) => {{
        use $crate::text;
        if_break!($p, $b, text!(""), None)
    }};
}

#[macro_export]
macro_rules! indent_if_break {
    ($p:ident, $d:expr, $gid:expr) => {{
        $crate::ir::Doc::IndentIfBreak($crate::ir::IndentIfBreak {
            contents: oxc_allocator::Box::new_in($d, $p.allocator),
            group_id: $gid,
        })
    }};
}

#[macro_export]
macro_rules! break_parent {
    () => {{
        $crate::ir::Doc::BreakParent
    }};
}

#[macro_export]
macro_rules! join {
    ($p:ident, $sep:expr, $docs:expr) => {{
        let mut parts = oxc_allocator::Vec::new_in($p.allocator);
        for (i, doc) in $docs.into_iter().enumerate() {
            if i != 0 {
                match $sep {
                    $crate::ir::JoinSeparator::Softline => parts.push($crate::softline!()),
                    $crate::ir::JoinSeparator::Hardline => parts.extend($crate::hardline!()),
                    $crate::ir::JoinSeparator::CommaLine => {
                        parts.extend([$crate::text!(", "), $crate::line!()]);
                    }
                }
            }
            parts.push(doc);
        }
        $crate::ir::Doc::Array(parts)
    }};
}

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
