use std::mem;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{Atom, Span};
use oxc_syntax::unicode_id_start::{is_id_continue, is_id_start};

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

    fn create_new_var(&mut self, expr: &Expression<'a>) -> IdentifierReference {
        let name = self.ctx().scopes().generate_uid_based_on_node(expr);
        self.ctx().add_binding(name.clone());

        // Add `var name` to scope
        // TODO: hookup symbol id
        let binding_identifier = BindingIdentifier::new(Span::default(), name.clone());
        let binding_pattern_kind = self.ctx().ast.binding_pattern_identifier(binding_identifier);
        let binding = self.ctx().ast.binding_pattern(binding_pattern_kind, None, false);
        let kind = VariableDeclarationKind::Var;
        let decl = self.ctx().ast.variable_declarator(Span::default(), kind, binding, None, false);
        self.vars_mut().push(decl);
        // TODO: add reference id and flag
        IdentifierReference::new(Span::default(), name)
    }

    /// Possibly generate a memoised identifier if it is not static and has consequences.
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L578>
    fn maybe_generate_memoised(&mut self, expr: &Expression<'a>) -> Option<IdentifierReference> {
        if self.ctx().symbols().is_static(expr) {
            None
        } else {
            Some(self.create_new_var(expr))
        }
    }
}

pub const RESERVED_WORDS_ES3_ONLY: [&str; 24] = [
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

const RESERVED_WORD_STRICT: [&str; 9] = [
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

pub const KEYWORDS: [&str; 35] = [
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

/// https://github.com/babel/babel/blob/ff3481746a830e0e94626de4c4cb075ea5f2f5dc/packages/babel-helper-validator-identifier/src/identifier.ts#L85-L109
pub fn is_identifier_name(name: &Atom) -> bool {
    let string = name.as_str();
    if string.is_empty() {
        return false;
    }
    let mut is_first = true;
    for ch in string.chars() {
        if is_first {
            is_first = false;
            if !is_id_start(ch) {
                return false;
            }
        } else if !is_id_continue(ch) {
            return false;
        }
    }
    true
}

pub fn is_valid_identifier(name: &Atom, reserved: bool) -> bool {
    if reserved && (KEYWORDS.contains(&name.as_str()) || is_strict_reserved_word(name, true)) {
        return false;
    }
    is_identifier_name(name)
}

pub fn is_strict_reserved_word(name: &Atom, in_module: bool) -> bool {
    is_reserved_word(name, in_module) || RESERVED_WORD_STRICT.contains(&name.as_str())
}

pub fn is_reserved_word(name: &Atom, in_module: bool) -> bool {
    (in_module && name.as_str() == "await") || name.as_str() == "enum"
}

/// https://github.com/babel/babel/blob/main/packages/babel-types/src/validators/isValidES3Identifier.ts#L35
pub fn is_valid_es3_identifier(name: &Atom) -> bool {
    is_valid_identifier(name, true) && !RESERVED_WORDS_ES3_ONLY.contains(&name.as_str())
}
