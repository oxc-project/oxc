use oxc_ast::VisitMut;

use crate::impl_plugin_transformation;

pub struct UnicodeSetsRegex;

impl_plugin_transformation!(UnicodeSetsRegex);

impl VisitMut<'_> for UnicodeSetsRegex {}
