use oxc_ast::ast::*;

use crate::TailwindcssOptions;

/// Check if a JSX attribute is a tailwind class attribute (class/className or custom tailwindAttributes)
pub fn is_tailwind_jsx_attribute(
    attr_name: &JSXAttributeName<'_>,
    tailwind_options: &TailwindcssOptions,
) -> bool {
    let JSXAttributeName::Identifier(ident) = attr_name else {
        return false;
    };
    let name = ident.name.as_str();

    // Default attributes: `class` and `className`
    let is_default_attr = name == "class" || name == "className";

    if is_default_attr {
        return true;
    }

    // Custom attributes from `tailwindAttributes` option
    tailwind_options
        .tailwind_attributes
        .as_ref()
        .is_some_and(|attrs| attrs.iter().any(|a| a == name))
}

/// Check if a callee expression is a tailwind function (e.g., `clsx`, `cn`, `tw`)
pub fn is_tailwind_function_call(
    callee: &Expression<'_>,
    tailwind_options: &TailwindcssOptions,
) -> bool {
    let Some(functions) = &tailwind_options.tailwind_functions else {
        return false;
    };

    let Expression::Identifier(ident) = callee else {
        return false;
    };

    functions.iter().any(|f| f == ident.name.as_str())
}
