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
        let doc = if $p.need_parens(kind) {
            $p.array(p_vec!($p, $p.text("("), doc, $p.text(")")))
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
