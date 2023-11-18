//! Utility macros for constructing the IR

#[macro_export]
macro_rules! format {
    ($p:ident, $s:expr) => {{
        $s.format($p)
    }};
}

/// Wrap a static string (ss)
#[macro_export]
macro_rules! ss {
    ($s:expr) => {{
        Doc::Str($s)
    }};
}

#[macro_export]
macro_rules! string {
    ($p:ident, $s:expr) => {{
        $p.str($s)
    }};
}

#[macro_export]
macro_rules! indent {
    ($p:ident, $( $x:expr ),* $(,)?) => {
        {
            let mut temp_vec = $p.vec();
            $(
                temp_vec.push($x);
            )*
            Doc::Indent(temp_vec)
        }
    };
}

#[macro_export]
macro_rules! indent_if_break {
    ($p:ident, $( $x:expr ),* $(,)?) => {
        {
            let mut temp_vec = $p.vec();
            $(
                temp_vec.push($x);
            )*
            Doc::IndentIfBreak($crate::doc::IndentIfBreak {
                contents: temp_vec,
                group_id: None,
            })
        }
    };
}

#[macro_export]
macro_rules! line {
    () => {
        Doc::Line
    };
}

#[macro_export]
macro_rules! softline {
    () => {
        Doc::Softline
    };
}

#[macro_export]
macro_rules! hardline {
    () => {
        Doc::Hardline
    };
}

#[macro_export]
macro_rules! array {
    ($p:ident, $( $x:expr ),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut temp_vec = $p.vec();
            $(
                temp_vec.push($x);
            )*
            Doc::Array(temp_vec)
        }
    };
}

#[macro_export]
macro_rules! group {
    ($p:ident, $( $x:expr ),* $(,)?) => {
        {
            let mut temp_vec = $p.vec();
            $(
                temp_vec.push($x);
            )*
            Doc::Group($crate::doc::Group { docs: temp_vec, group_id: None })
        }
    };
}

#[macro_export]
macro_rules! if_break {
    ($p:ident, $s:expr) => {{
        let mut temp_break_vec = $p.vec();
        temp_break_vec.push(Doc::Str($s));

        Doc::IfBreak($crate::doc::IfBreak {
            break_contents: temp_break_vec,
            flat_contents: $p.vec(),
            group_id: None,
        })
    }};
}

#[macro_export]
macro_rules! wrap {
    ($p:ident, $self:expr, $kind:ident, $block:block) => {{
        $p.enter_node(AstKind::$kind($p.alloc($self)));
        let doc = $block;
        $p.leave_node();
        doc
    }};
}
