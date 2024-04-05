use oxc_ast::VisitMut;

use crate::impl_plugin_transformation;

pub struct NullishCoalescingOperator;

impl_plugin_transformation!(NullishCoalescingOperator);

impl VisitMut<'_> for NullishCoalescingOperator {}
