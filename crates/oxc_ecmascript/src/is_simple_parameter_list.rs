use oxc_ast::ast::FormalParameters;

/// [`IsSimpleParameterList`](https://tc39.es/ecma262/#sec-static-semantics-issimpleparameterlist)
pub trait IsSimpleParameterList {
    fn is_simple_parameter_list(&self) -> bool;
}

impl<'a> IsSimpleParameterList for FormalParameters<'a> {
    fn is_simple_parameter_list(&self) -> bool {
        self.items.iter().all(|pat| pat.pattern.kind.is_binding_identifier()) && self.rest.is_none()
    }
}
