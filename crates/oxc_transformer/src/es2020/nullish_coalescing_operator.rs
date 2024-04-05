use oxc_ast::VisitMut;

pub struct NullishCoalescingOperator;

impl VisitMut<'_> for NullishCoalescingOperator {}
