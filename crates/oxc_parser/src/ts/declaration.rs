use crate::{lexer::Kind, Parser};

impl<'a> Parser<'a> {
    /// Check if the parser is at a start of a declaration
    fn at_start_of_ts_declaration_worker(&mut self) -> bool {
        loop {
            match self.cur_kind() {
                Kind::Var | Kind::Let | Kind::Const | Kind::Function | Kind::Class | Kind::Enum => {
                    return true;
                }
                Kind::Interface | Kind::Type => {
                    self.bump_any();
                    return self.cur_kind().is_binding_identifier()
                        && !self.cur_token().is_on_new_line;
                }
                Kind::Module | Kind::Namespace => {
                    self.bump_any();
                    return !self.cur_token().is_on_new_line
                        && (self.cur_kind().is_binding_identifier()
                            || self.cur_kind() == Kind::Str);
                }
                Kind::Abstract
                | Kind::Accessor
                | Kind::Async
                | Kind::Declare
                | Kind::Private
                | Kind::Protected
                | Kind::Public
                | Kind::Readonly => {
                    self.bump_any();
                    if self.cur_token().is_on_new_line {
                        return false;
                    }
                }
                Kind::Global => {
                    self.bump_any();
                    return matches!(self.cur_kind(), Kind::Ident | Kind::LCurly | Kind::Export);
                }
                Kind::Import => {
                    self.bump_any();
                    return matches!(self.cur_kind(), Kind::Str | Kind::Star | Kind::LCurly)
                        || self.cur_kind().is_identifier();
                }
                Kind::Export => {
                    self.bump_any();
                    let kind = if self.cur_kind() == Kind::Type {
                        self.peek_kind()
                    } else {
                        self.cur_kind()
                    };
                    // This allows constructs like
                    // `export *`, `export default`, `export {}`, `export = {}` along with all
                    // export [declaration]
                    if matches!(
                        kind,
                        Kind::Eq | Kind::Star | Kind::Default | Kind::LCurly | Kind::At | Kind::As
                    ) {
                        return true;
                    }
                    // falls through to check next token
                }
                Kind::Static => {
                    self.bump_any();
                }
                _ => {
                    return false;
                }
            }
        }
    }

    pub fn at_start_of_ts_declaration(&mut self) -> bool {
        self.lookahead(Self::at_start_of_ts_declaration_worker)
    }
}

#[cfg(test)]
mod test_is_declaration {
    use oxc_allocator::Allocator;
    use oxc_ast::SourceType;

    use super::*;

    fn run_check(source: &str, expected: bool) {
        let alloc = Allocator::default();
        let source_type = *SourceType::default().with_typescript(true);
        let mut parser = Parser::new(&alloc, source, source_type);
        // Get the parser to the first token.
        parser.bump_any();
        assert_eq!(expected, parser.at_start_of_ts_declaration());
    }

    #[test]
    fn test_lexical_decleration() {
        run_check("const a = 1", true);
        run_check("let a = 1", true);
    }

    #[test]
    fn test_combined_modifier() {
        // The order of modifiers shouldn't matter
        let source = "abstract async function a() { return 123; }";
        let source2 = "async abstract class C{}";
        run_check(source, true);
        run_check(source2, true);
    }

    #[test]
    fn test_contextual_keyword() {
        // Here abstract should not be parsed as starting a declaration
        run_check("abstract = 1", false);
        run_check("private = 'abc'", false);
        run_check("abstract\nclass A {}", false);
    }

    #[test]
    fn test_export() {
        run_check("export = {}", true);
        run_check("export *", true);
        // modifiers can be combined with expory
        run_check("abstract export type T", true);
    }

    #[test]
    fn test_declare_module() {
        run_check("declare module 'external1' {}", true);
    }

    #[test]
    fn test_const_enum() {
        run_check("const enum A {}", true);
    }

    #[test]
    fn test_type_alias() {
        run_check("type string = I", true);
        run_check("type void = I", false);
    }
}
