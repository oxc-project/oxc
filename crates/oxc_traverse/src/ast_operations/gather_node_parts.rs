//! Gather node parts trait.
//!
//! Ported from: <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts>
//!
//! This trait is used to gather all the parts of a node that are identifiers.

use oxc_ast::ast::*;
use oxc_data_structures::string_ext::StringExt;
use oxc_ecmascript::BoundNames;

use super::to_identifier;

const MAX_LEN: usize = 20;

/// Get a variable name for an AST node, by concatenating identifiers used in the node.
///
/// The returned name is maximum 20 bytes.
///
/// This is based on Babel, but diverges slightly. Babel limits to 20 UTF-16 chars, not 20 UTF-8 bytes.
/// But the goal is just to avoid overly long variable names, the exact length is not really important.
pub fn get_var_name_from_node<'a, N: GatherNodeParts<'a>>(node: &N) -> String {
    let mut name = String::with_capacity(MAX_LEN);

    // Visit the node, gathering identifiers.
    //
    // Append them to `name`, capping the length at max `MAX_LEN` bytes.
    // Once `name` reaches `MAX_LEN` bytes, all further calls to the closure are no-ops.
    //
    // Length is strictly prevented from exceeding `MAX_LEN` bytes here, so `name` `String` never grows
    // beyond initial `MAX_LEN` capacity. Unsafe code in this closure relies on this invariant.
    // Be very careful to always uphold this invariant when making any changes to this code.
    node.gather(&mut |mut part| {
        // If `name` is already 20 bytes, ignore this part
        if name.len() == MAX_LEN {
            return;
        }

        debug_assert!(name.len() < MAX_LEN);

        if name.is_empty() {
            part = part.trim_start_matches('_');
        } else {
            // SAFETY: `name` is initialized with `MAX_LEN` capacity and never grows or shrinks,
            // so it has spare capacity for 1 more byte - we exited above if `name.len() == MAX_LEN`.
            unsafe { name.push_unchecked('$') };
        }

        // This addition cannot overflow `usize` because slices' length cannot exceed `isize::MAX`
        let new_len = name.len() + part.len();
        if new_len > MAX_LEN {
            // Truncate `part` so adding it to `name` results in `name` being `MAX_LEN` bytes.
            // If the cut point which would give `MAX_LEN` bytes is on a UTF-8 char boundary,
            // move the cut point forwards to the start of the UTF-8 char.
            debug_assert!(name.len() <= MAX_LEN);

            let mut cut_at = MAX_LEN - name.len();
            debug_assert!(cut_at < part.len());

            let bytes = part.as_bytes();

            // SAFETY:
            // * `name.len() + part.len() > MAX_LEN` (`new_len > MAX_LEN` check above)
            // * `name.len() <= MAX_LEN` (`name.len() < MAX_LEN` at top of closure, and we pushed at most 1 `$`)
            // * Therefore: `part.len() > MAX_LEN - name.len()`
            // * `cut_at = MAX_LEN - name.len()`
            // * Therefore: `part.len() > cut_at`
            // * Reversed: `cut_at < part.len()`
            // * `cut_at` is in bounds of `bytes`
            let cut_byte = unsafe { *bytes.get_unchecked(cut_at) };
            if !cut_byte.is_ascii() {
                // `cut_at` may be in the middle of a multi-byte char. Walk back to find the start byte.
                // Unicode is rare, so mark this as cold path.
                #[cold]
                #[inline]
                fn adjust_cut_at(bytes: &[u8], cut_at: &mut usize) {
                    // SAFETY: `bytes` comes from a valid UTF-8 `&str`.
                    // UTF-8 chars are maximum 4 bytes.
                    // Non-ASCII chars' byte sequences start with a byte which is `>= 0xC0`.
                    // We checked above that the byte at `cut_at` is not ASCII, so either:
                    // 1. Byte at `cut_at` is start of a multi-byte char, or
                    // 2. `cut` at is in middle of a multi-byte character, with 1st byte of character
                    //    at one of `cut_at - 1`, `cut_at - 2`, `cut_at - 3`.
                    unsafe {
                        if *bytes.get_unchecked(*cut_at) >= 0xC0 {
                            // `cut_at` already points to start of a multi-byte char
                        } else if *bytes.get_unchecked(*cut_at - 1) >= 0xC0 {
                            *cut_at -= 1;
                        } else if *bytes.get_unchecked(*cut_at - 2) >= 0xC0 {
                            *cut_at -= 2;
                        } else {
                            *cut_at -= 3;
                        }
                    }
                }
                adjust_cut_at(bytes, &mut cut_at);
            }

            // SAFETY: `cut_at` is `< part.len()` and on a UTF-8 character boundary
            unsafe {
                part = part.get_unchecked(..cut_at);
            }
        }

        // SAFETY: `name` is initialized with `MAX_LEN` capacity and never grows or shrinks.
        // `part` has been shortened above if required to fit within `name`'s remaining capacity.
        unsafe { name.push_str_unchecked(part) };
    });

    if name.is_empty() {
        const _: () = assert!(MAX_LEN >= 3);

        // SAFETY: `name` is initialized with `MAX_LEN` capacity and never grows or shrinks.
        // `name` is empty, so has capacity for 3 bytes.
        unsafe { name.push_str_unchecked("ref") };
    } else {
        name = to_identifier(name);
    }

    name
}

pub trait GatherNodeParts<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F);
}

// -------------------- ModuleDeclaration --------------------
impl<'a> GatherNodeParts<'a> for ImportDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.source.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportAllDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.source.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportNamedDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(source) = &self.source {
            source.gather(f);
        } else if let Some(declaration) = &self.declaration {
            declaration.gather(f);
        } else {
            for specifier in &self.specifiers {
                specifier.gather(f);
            }
        }
    }
}

impl<'a> GatherNodeParts<'a> for ExportDefaultDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.declaration.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportDefaultDeclarationKind<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => decl.gather(f),
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => decl.gather(f),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {}
            match_expression!(ExportDefaultDeclarationKind) => self.to_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ExportSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match &self.local {
            ModuleExportName::IdentifierName(ident) => ident.gather(f),
            ModuleExportName::IdentifierReference(ident) => ident.gather(f),
            ModuleExportName::StringLiteral(lit) => lit.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ModuleExportName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ModuleExportName::IdentifierName(ident) => ident.gather(f),
            ModuleExportName::IdentifierReference(ident) => ident.gather(f),
            ModuleExportName::StringLiteral(lit) => lit.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ImportSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ImportDefaultSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ImportNamespaceSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

// -------------------- Declaration --------------------

impl<'a> GatherNodeParts<'a> for Declaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            Self::FunctionDeclaration(decl) => decl.gather(f),
            Self::ClassDeclaration(decl) => decl.gather(f),
            _ => (),
        }
    }
}

// -------------------- Function --------------------

impl<'a> GatherNodeParts<'a> for Function<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(id) = &self.id {
            id.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for BindingRestElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

// -------------------- BindingPattern --------------------

impl<'a> GatherNodeParts<'a> for VariableDeclarator<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.id.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for BindingPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for ObjectPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for ArrayPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

// -------------------- Expression --------------------

impl<'a> GatherNodeParts<'a> for Expression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            Self::Identifier(ident) => ident.gather(f),
            Self::CallExpression(expr) => expr.gather(f),
            Self::NewExpression(expr) => expr.gather(f),
            Self::ObjectExpression(expr) => expr.gather(f),
            Self::ThisExpression(expr) => expr.gather(f),
            Self::Super(expr) => expr.gather(f),
            Self::ImportExpression(expr) => expr.gather(f),
            Self::YieldExpression(expr) => expr.gather(f),
            Self::AwaitExpression(expr) => expr.gather(f),
            Self::AssignmentExpression(expr) => expr.gather(f),
            Self::FunctionExpression(expr) => expr.gather(f),
            Self::ClassExpression(expr) => expr.gather(f),
            Self::ParenthesizedExpression(expr) => expr.gather(f),
            Self::UnaryExpression(expr) => expr.gather(f),
            Self::UpdateExpression(expr) => expr.gather(f),
            Self::ChainExpression(expr) => expr.gather(f),
            Self::MetaProperty(expr) => expr.gather(f),
            Self::JSXElement(expr) => expr.gather(f),
            Self::JSXFragment(expr) => expr.gather(f),
            Self::StringLiteral(expr) => expr.gather(f),
            Self::NumericLiteral(expr) => expr.gather(f),
            Self::BooleanLiteral(expr) => expr.gather(f),
            Self::BigIntLiteral(expr) => expr.gather(f),
            _ => (),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ChainExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.expression.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ChainElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ChainElement::CallExpression(expr) => expr.gather(f),
            ChainElement::TSNonNullExpression(expr) => expr.expression.gather(f),
            expr @ match_member_expression!(Self) => expr.to_member_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for MemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => {
                expr.gather(f);
            }
            MemberExpression::StaticMemberExpression(expr) => {
                expr.gather(f);
            }
            MemberExpression::PrivateFieldExpression(expr) => {
                expr.gather(f);
            }
        }
    }
}

impl<'a> GatherNodeParts<'a> for ComputedMemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.expression.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for StaticMemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.property.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for PrivateFieldExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.field.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for CallExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.callee.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for NewExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.callee.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ObjectExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        for prop in &self.properties {
            prop.gather(f);
        }
    }
}

impl GatherNodeParts<'_> for ThisExpression {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("this");
    }
}

impl GatherNodeParts<'_> for Super {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("super");
    }
}

impl<'a> GatherNodeParts<'a> for ImportExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("import");
    }
}

impl<'a> GatherNodeParts<'a> for YieldExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("yield");
        if let Some(argument) = &self.argument {
            argument.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for AwaitExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("await");
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.left.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ParenthesizedExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.expression.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for UnaryExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for UpdateExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for MetaProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.meta.gather(f);
        self.property.gather(f);
    }
}

// -------------------- AssignmentTarget --------------------
impl<'a> GatherNodeParts<'a> for AssignmentTarget<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            match_simple_assignment_target!(Self) => {
                self.to_simple_assignment_target().gather(f);
            }
            match_assignment_target_pattern!(Self) => {}
        }
    }
}

impl<'a> GatherNodeParts<'a> for SimpleAssignmentTarget<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gather(f),
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            _ => {}
        }
    }
}

// -------------------- Classes --------------------

impl<'a> GatherNodeParts<'a> for Class<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(id) = &self.id {
            id.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for ClassElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ClassElement::PropertyDefinition(def) => def.gather(f),
            ClassElement::MethodDefinition(def) => def.gather(f),
            ClassElement::AccessorProperty(def) => def.gather(f),
            _ => (),
        }
    }
}

impl<'a> GatherNodeParts<'a> for PropertyDefinition<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for MethodDefinition<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for AccessorProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

// -------------------- Objects --------------------

impl<'a> GatherNodeParts<'a> for ObjectPropertyKind<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => prop.gather(f),
            ObjectPropertyKind::SpreadProperty(prop) => prop.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ObjectProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for PropertyKey<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            PropertyKey::StaticIdentifier(ident) => ident.gather(f),
            PropertyKey::PrivateIdentifier(ident) => ident.gather(f),
            match_expression!(Self) => self.to_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for SpreadElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

// -------------------- Identifiers --------------------

impl<'a> GatherNodeParts<'a> for BindingIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierReference<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for PrivateIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

// -------------------- Literals --------------------

impl<'a> GatherNodeParts<'a> for StringLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.value.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for NumericLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(&self.raw_str());
    }
}

impl GatherNodeParts<'_> for BooleanLiteral {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if self.value {
            f("true");
        } else {
            f("false");
        }
    }
}

impl<'a> GatherNodeParts<'a> for BigIntLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.value.as_str());
    }
}

// -------------------- JSX --------------------

impl<'a> GatherNodeParts<'a> for JSXElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.opening_element.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXFragment<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.opening_fragment.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXOpeningElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.name.gather(f);
    }
}

impl GatherNodeParts<'_> for JSXOpeningFragment {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("Fragment");
    }
}

impl<'a> GatherNodeParts<'a> for JSXElementName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            JSXElementName::Identifier(ident) => ident.gather(f),
            JSXElementName::IdentifierReference(ident) => ident.gather(f),
            JSXElementName::NamespacedName(ns) => ns.gather(f),
            JSXElementName::MemberExpression(expr) => expr.gather(f),
            JSXElementName::ThisExpression(expr) => expr.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for JSXNamespacedName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.namespace.gather(f);
        self.name.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXMemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.property.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXMemberExpressionObject<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            JSXMemberExpressionObject::IdentifierReference(ident) => ident.gather(f),
            JSXMemberExpressionObject::MemberExpression(expr) => expr.gather(f),
            JSXMemberExpressionObject::ThisExpression(expr) => expr.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for JSXIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test wrappers that implement `GatherNodeParts` for testing purposes
    struct TestNode<'a, const LEN: usize>([&'a str; LEN]);

    impl<'a, const LEN: usize> GatherNodeParts<'a> for TestNode<'a, LEN> {
        fn gather<F: FnMut(&str)>(&self, f: &mut F) {
            for s in self.0 {
                f(s);
            }
        }
    }

    macro_rules! test_single {
        ($input:expr, $output:expr, $input_len:expr, $output_len:expr) => {{
            assert_eq!($input.len(), $input_len);
            assert_eq!($output.len(), $output_len);
            let node = TestNode([$input]);
            assert_eq!(get_var_name_from_node(&node), $output);
        }};
    }

    macro_rules! test_double {
        ($input1:expr, $input2:expr, $output:expr, $input1_len:expr, $input2_len:expr, $output_len:expr) => {{
            assert_eq!($input1.len(), $input1_len);
            assert_eq!($input2.len(), $input2_len);
            assert_eq!($output.len(), $output_len);
            let node = TestNode([$input1, $input2]);
            assert_eq!(get_var_name_from_node(&node), $output);
        }};
    }

    #[test]
    fn test_get_var_name_truncation_limits_to_approximately_20_bytes() {
        // ASCII
        test_single!("a", "a", 1, 1);
        test_single!("abcde", "abcde", 5, 5);
        test_single!("abcdefghijklmnopqrs", "abcdefghijklmnopqrs", 19, 19);
        test_single!("abcdefghijklmnopqrst", "abcdefghijklmnopqrst", 20, 20);
        test_single!("abcdefghijklmnopqrstu", "abcdefghijklmnopqrst", 21, 20);
        test_single!("abcdefghijklmnopqrstuvwxyz", "abcdefghijklmnopqrst", 26, 20);

        // 2-byte UTF-8 (Greek)
        test_single!("α", "α", 2, 2);
        test_single!("αβγδεζηθι", "αβγδεζηθι", 18, 18);
        test_single!("xαβγδεζηθι", "xαβγδεζηθι", 19, 19);
        test_single!("αβγδεζηθικ", "αβγδεζηθικ", 20, 20);
        test_single!("xαβγδεζηθικ", "xαβγδεζηθι", 21, 19);
        test_single!("αβγδεζηθικλ", "αβγδεζηθικ", 22, 20);
        test_single!("xαβγδεζηθικλ", "xαβγδεζηθι", 23, 19);
        test_single!("αβγδεζηθικλμνξοπρστυφ", "αβγδεζηθικ", 42, 20);
        test_single!("xαβγδεζηθικλμνξοπρστυφ", "xαβγδεζηθι", 43, 19);

        // 3-byte UTF-8 (Korean)
        test_single!("가", "가", 3, 3);
        test_single!("가나다라마바", "가나다라마바", 18, 18);
        test_single!("x가나다라마바", "x가나다라마바", 19, 19);
        test_single!("xx가나다라마바", "xx가나다라마바", 20, 20);
        test_single!("가나다라마바사", "가나다라마바", 21, 18);
        test_single!("x가나다라마바사", "x가나다라마바", 22, 19);
        test_single!("xx가나다라마바사", "xx가나다라마바", 23, 20);
        test_single!("가나다라마바사아자차", "가나다라마바", 30, 18);
        test_single!("x가나다라마바사아자차", "x가나다라마바", 31, 19);
        test_single!("xx가나다라마바사아자차", "xx가나다라마바", 32, 20);

        // 4-byte UTF-8 (CJK Ext B)
        test_single!("𠀀", "𠀀", 4, 4);
        test_single!("𠀀𠀁𠀂𠀃", "𠀀𠀁𠀂𠀃", 16, 16);
        test_single!("x𠀀𠀁𠀂𠀃", "x𠀀𠀁𠀂𠀃", 17, 17);
        test_single!("xx𠀀𠀁𠀂𠀃", "xx𠀀𠀁𠀂𠀃", 18, 18);
        test_single!("xxx𠀀𠀁𠀂𠀃", "xxx𠀀𠀁𠀂𠀃", 19, 19);
        test_single!("𠀀𠀁𠀂𠀃𠀄", "𠀀𠀁𠀂𠀃𠀄", 20, 20);
        test_single!("x𠀀𠀁𠀂𠀃𠀄", "x𠀀𠀁𠀂𠀃", 21, 17);
        test_single!("xx𠀀𠀁𠀂𠀃𠀄", "xx𠀀𠀁𠀂𠀃", 22, 18);
        test_single!("xxx𠀀𠀁𠀂𠀃𠀄", "xxx𠀀𠀁𠀂𠀃", 23, 19);
        test_single!("𠀀𠀁𠀂𠀃𠀄𠀅", "𠀀𠀁𠀂𠀃𠀄", 24, 20);
        test_single!("𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "𠀀𠀁𠀂𠀃𠀄", 40, 20);
        test_single!("x𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "x𠀀𠀁𠀂𠀃", 41, 17);
        test_single!("xx𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "xx𠀀𠀁𠀂𠀃", 42, 18);
        test_single!("xxx𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "xxx𠀀𠀁𠀂𠀃", 43, 19);
    }

    #[test]
    fn test_get_var_name_empty_returns_ref() {
        test_single!("", "ref", 0, 3);
        test_double!("", "", "ref", 0, 0, 3);
    }

    #[test]
    fn test_get_var_name_strips_leading_underscores() {
        test_single!("___foo", "foo", 6, 3);
        test_single!("___abcdefghijklmnopqrs", "abcdefghijklmnopqrs", 22, 19);
        test_single!("___abcdefghijklmnopqrst", "abcdefghijklmnopqrst", 23, 20);
        test_single!("___abcdefghijklmnopqrstu", "abcdefghijklmnopqrst", 24, 20);
    }

    #[test]
    fn test_get_var_name_double_joins_parts_with_dollar() {
        test_double!("foo", "bar", "foo$bar", 3, 3, 7);
        test_double!("a", "b", "a$b", 1, 1, 3);
        // Parts join to exactly `MAX_LEN` bytes (9 + `$` + 10) - no truncation.
        test_double!("foobarbaz", "abcdefghij", "foobarbaz$abcdefghij", 9, 10, 20);
    }

    #[test]
    fn test_get_var_name_double_empty_and_underscores() {
        // Empty second part still inserts the `$` separator.
        test_double!("foo", "", "foo$", 3, 0, 4);
        // Empty first part leaves `name` empty, so the second part is treated as the
        // first: no `$` separator, and leading underscores are trimmed.
        test_double!("", "bar", "bar", 0, 3, 3);
        // First part is all underscores -> trims to empty -> same as above.
        test_double!("___", "bar", "bar", 3, 3, 3);
        // Both parts empty (after trimming) -> fallback to `ref`.
        test_double!("___", "___", "ref", 3, 3, 3);
        // Leading underscores are only trimmed from the first non-empty part.
        // The second part keeps its underscores (name is non-empty by then).
        test_double!("_foo", "_bar", "foo$_bar", 4, 4, 8);
        test_double!("_foo", "___", "foo$___", 4, 3, 7);
    }

    #[test]
    fn test_get_var_name_double_first_part_fills_name() {
        // First part already fills `name` to `MAX_LEN`, so the second part is ignored
        // (early return at top of closure).
        test_double!("abcdefghijklmnopqrst", "xyz", "abcdefghijklmnopqrst", 20, 3, 20);
        // First part exceeds `MAX_LEN`, gets truncated to 20, second part ignored.
        test_double!("abcdefghijklmnopqrstu", "xyz", "abcdefghijklmnopqrst", 21, 3, 20);
    }

    #[test]
    fn test_get_var_name_double_dollar_reaches_max_len() {
        // Regression test for `cut_at == 0`:
        // First part is 19 bytes, pushing `$` brings `name` to exactly `MAX_LEN` (20).
        // The second part is then truncated to zero bytes (`cut_at == 0`), leaving a
        // trailing `$`. This previously tripped a `debug_assert!(cut_at > 0)`.
        test_double!("abcdefghijklmnopqrs", "x", "abcdefghijklmnopqrs$", 19, 1, 20);
        // Same, but second part starts with a multi-byte char - exercises the cold
        // `cut` path with `cut_at == 0` (must not underflow when indexing back).
        test_double!("abcdefghijklmnopqrs", "β", "abcdefghijklmnopqrs$", 19, 2, 20);
        // First part is 19 bytes of mixed ASCII + multi-byte UTF-8.
        test_double!("xαβγδεζηθι", "y", "xαβγδεζηθι$", 19, 1, 20);
    }

    #[test]
    fn test_get_var_name_double_truncates_second_part_ascii() {
        test_double!("foobar", "abcdefghijklmnopqrstuvwxyz", "foobar$abcdefghijklm", 6, 26, 20);
        test_double!("ab", "cdefghijklmnopqrstuvwxyz", "ab$cdefghijklmnopqrs", 2, 24, 20);
        // Second part fits exactly to `MAX_LEN` after the `$` - no truncation.
        test_double!("abcdefghijklmnopqr", "z", "abcdefghijklmnopqr$z", 18, 1, 20);
        // `cut_at == 1`: only the first byte of the second part survives.
        test_double!("abcdefghijklmnopqr", "yz", "abcdefghijklmnopqr$y", 18, 2, 20);
    }

    #[test]
    fn test_get_var_name_double_truncates_second_part_utf8() {
        // The second part is composed entirely of multi-byte UTF-8 chars. The cut point
        // lands inside it, and `cut` walks back to the start of the char it lands on -
        // so the walk-back distance is determined by which byte of the char is hit.
        // Cover every (char width, byte position) combination.

        // 2-byte chars (Greek): cut can land on byte 0 or 1.
        test_double!("abc", "αβγδεζηθικ", "abc$αβγδεζηθ", 3, 20, 20); // byte 0 (lead): walk back 0
        test_double!("ab", "αβγδεζηθικ", "ab$αβγδεζηθ", 2, 20, 19); // byte 1: walk back 1

        // 3-byte chars (Korean): cut can land on byte 0, 1 or 2.
        test_double!("abcd", "가나다라마바사", "abcd$가나다라마", 4, 21, 20); // byte 0 (lead): walk back 0
        test_double!("abc", "가나다라마바사", "abc$가나다라마", 3, 21, 19); // byte 1: walk back 1
        test_double!("ab", "가나다라마바사", "ab$가나다라마", 2, 21, 18); // byte 2: walk back 2

        // 4-byte chars (CJK Ext B): cut can land on byte 0, 1, 2 or 3.
        test_double!("abc", "𠀀𠀁𠀂𠀃𠀄𠀅", "abc$𠀀𠀁𠀂𠀃", 3, 24, 20); // byte 0 (lead): walk back 0
        test_double!("ab", "𠀀𠀁𠀂𠀃𠀄𠀅", "ab$𠀀𠀁𠀂𠀃", 2, 24, 19); // byte 1: walk back 1
        test_double!("a", "𠀀𠀁𠀂𠀃𠀄𠀅", "a$𠀀𠀁𠀂𠀃", 1, 24, 18); // byte 2: walk back 2
        test_double!("abcd", "𠀀𠀁𠀂𠀃𠀄𠀅", "abcd$𠀀𠀁𠀂", 4, 24, 17); // byte 3: walk back 3

        // Mixed ASCII + 2-byte: cut point lands mid-char after some ASCII bytes.
        test_double!("ab", "xyαβγδεζηθικ", "ab$xyαβγδεζη", 2, 22, 19);
    }

    #[test]
    fn test_get_var_name_double_passes_through_to_identifier() {
        // Digit mid-string is a valid identifier part - kept as-is.
        test_double!("foo", "1bar", "foo$1bar", 3, 4, 8);
        // Leading digit is not a valid identifier start - next char is capitalized.
        test_double!("1foo", "bar", "Foo$bar", 4, 3, 7);
        // Invalid identifier chars trigger camel-casing of the following char.
        test_double!("foo-bar.qux", "x", "fooBarQux$x", 11, 1, 11);
        test_double!("foo", "bar qux grep", "foo$barQuxGrep", 3, 12, 14);
    }
}
