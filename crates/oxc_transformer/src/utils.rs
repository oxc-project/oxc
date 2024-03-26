use std::mem;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{CompactStr, Span};
use oxc_syntax::identifier::is_identifier_name;

use crate::context::TransformerCtx;

pub trait CreateVars<'a> {
    fn ctx(&self) -> &TransformerCtx<'a>;

    fn vars_mut(&mut self) -> &mut Vec<'a, VariableDeclarator<'a>>;

    fn add_vars_to_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        if self.vars_mut().is_empty() {
            return;
        }
        let new_vec = self.ctx().ast.new_vec();
        let decls = mem::replace(self.vars_mut(), new_vec);
        let kind = VariableDeclarationKind::Var;
        let decl =
            self.ctx().ast.variable_declaration(Span::default(), kind, decls, Modifiers::empty());
        let stmt = Statement::Declaration(Declaration::VariableDeclaration(decl));
        stmts.insert(0, stmt);
    }

    fn create_new_var_with_expression(&mut self, expr: &Expression<'a>) -> IdentifierReference<'a> {
        let name = self.ctx().scopes().generate_uid_based_on_node(expr);
        create_new_var(self, name)
    }

    fn create_new_named_var(&mut self, name: &'a str) -> IdentifierReference<'a> {
        let name = self.ctx().scopes().generate_uid(name);
        create_new_var(self, name)
    }

    /// Possibly generate a memoised identifier if it is not static and has consequences.
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L578>
    fn maybe_generate_memoised(
        &mut self,
        expr: &Expression<'a>,
    ) -> Option<IdentifierReference<'a>> {
        if self.ctx().symbols().is_static(expr) {
            None
        } else {
            Some(self.create_new_var_with_expression(expr))
        }
    }
}

pub const RESERVED_WORDS_ES3_ONLY: phf::Set<&str> = phf::phf_set![
    "abstract",
    "boolean",
    "byte",
    "char",
    "double",
    "enum",
    "final",
    "float",
    "goto",
    "implements",
    "int",
    "interface",
    "long",
    "native",
    "package",
    "private",
    "protected",
    "public",
    "short",
    "static",
    "synchronized",
    "throws",
    "transient",
    "volatile",
];

const RESERVED_WORD_STRICT: phf::Set<&str> = phf::phf_set![
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
];

pub const KEYWORDS: phf::Set<&str> = phf::phf_set![
    "break",
    "case",
    "catch",
    "continue",
    "debugger",
    "default",
    "do",
    "else",
    "finally",
    "for",
    "function",
    "if",
    "return",
    "switch",
    "throw",
    "try",
    "var",
    "const",
    "while",
    "with",
    "new",
    "this",
    "super",
    "class",
    "extends",
    "export",
    "import",
    "null",
    "true",
    "false",
    "in",
    "instanceof",
    "typeof",
    "void",
    "delete",
];

pub fn is_valid_identifier(name: &str, reserved: bool) -> bool {
    if reserved && (KEYWORDS.contains(name) || is_strict_reserved_word(name, true)) {
        return false;
    }
    is_identifier_name(name)
}

pub fn is_strict_reserved_word(name: &str, in_module: bool) -> bool {
    is_reserved_word(name, in_module) || RESERVED_WORD_STRICT.contains(name)
}

pub fn is_reserved_word(name: &str, in_module: bool) -> bool {
    (in_module && name == "await") || name == "enum"
}

/// https://github.com/babel/babel/blob/main/packages/babel-types/src/validators/isValidES3Identifier.ts#L35
pub fn is_valid_es3_identifier(name: &str) -> bool {
    is_valid_identifier(name, true) && !RESERVED_WORDS_ES3_ONLY.contains(name)
}

fn create_new_var<'a, V: CreateVars<'a> + ?Sized>(
    create_vars: &mut V,
    name: CompactStr,
) -> IdentifierReference<'a> {
    // Add `var name` to scope
    // TODO: hookup symbol id
    let atom = create_vars.ctx().ast.new_atom(name.as_str());
    let binding_identifier = BindingIdentifier::new(Span::default(), atom.clone());
    let binding_pattern_kind = create_vars.ctx().ast.binding_pattern_identifier(binding_identifier);
    let binding = create_vars.ctx().ast.binding_pattern(binding_pattern_kind, None, false);
    let kind = VariableDeclarationKind::Var;
    let decl =
        create_vars.ctx().ast.variable_declarator(Span::default(), kind, binding, None, false);
    create_vars.ctx().add_binding(name);
    create_vars.vars_mut().push(decl);
    // TODO: add reference id and flag
    IdentifierReference::new(Span::default(), atom)
}
