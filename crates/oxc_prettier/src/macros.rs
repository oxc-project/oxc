//! Utility macros for constructing the IR

#[macro_export]
macro_rules! group {
    ($p:ident, $( $x:expr ),* $(,)?) => {{
        use $crate::ir::Group;
        let mut temp_vec = $p.vec();
        $(
            temp_vec.push($x);
        )*
        Doc::Group(Group::new(temp_vec))
    }};
}

#[macro_export]
macro_rules! conditional_group {
    ($p:ident, $c: expr, $( $x:expr ),* $(,)?) => {{
        use $crate::ir::Group;
        let mut temp_vec = $p.vec();
        $(
            temp_vec.push($x);
        )*
        let contents = $p.vec_single($c);
        Doc::Group(Group::new_conditional_group(contents, temp_vec))
    }};
}

#[macro_export]
macro_rules! group_break {
    ($p:ident, $( $x:expr ),* $(,)?) => {{
        use $crate::ir::Group;
        let mut temp_vec = $p.vec();
        $(
            temp_vec.push($x);
        )*
        Doc::Group(Group::new(temp_vec).with_break(true))
    }};
}

#[macro_export]
macro_rules! if_break {
    ($p:ident, $s:expr, $flat:expr, $group_id:expr) => {{
        use $crate::ir::IfBreak;
        Doc::IfBreak(IfBreak {
            break_contents: $p.boxed(Doc::Str($s)),
            flat_content: $p.boxed(Doc::Str($flat)),
            group_id: $group_id,
        })
    }};
    ($p:ident, $s:expr, $flat:expr) => {{
        if_break!($p, $s, $flat, None)
    }};
    ($p:ident, $s:expr) => {{
        if_break!($p, $s, "", None)
    }};
}

// ---

#[macro_export]
macro_rules! p_str {
    ($p:ident, $s:expr) => {{
        $p.string($s)
    }};
}

#[macro_export]
macro_rules! p_vec {
    ($p:ident, $( $x:expr ),* $(,)?) => {{
        let mut temp_vec = $p.vec();
        $(
            temp_vec.push($x);
        )*
        temp_vec
    }};
}

#[macro_export]
macro_rules! wrap {
    ($p:ident, $self:expr, $kind:ident, $block:block) => {{
        let kind = AstKind::$kind($p.alloc($self));
        $p.enter_node(kind);

        let leading = $p.print_leading_comments(kind.span());

        let doc = $block;
        let doc =
            if $p.need_parens(kind) { array(p_vec!($p, text("("), doc, text(")"))) } else { doc };

        // TODO: dangling comments?
        let trailing = $p.print_trailing_comments(kind.span());

        let doc = $p.print_comments(leading, doc, trailing);

        $p.leave_node();
        doc
    }};
}
