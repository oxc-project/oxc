use oxc_ast::ast::JSXAttributeName;

pub fn get_attribute_name(attr: &JSXAttributeName) -> String {
    match attr {
        JSXAttributeName::Identifier(ident) => ident.name.to_string(),
        JSXAttributeName::NamespacedName(namespaced_name) => {
            format!("{}:{}", namespaced_name.namespace.name, namespaced_name.property.name)
        }
    }
}
