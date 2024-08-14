use std::collections::HashSet;
use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_ast::Trivias;
use oxc_codegen::{CodeGenerator, CommentOptions, WhitespaceRemover};
use oxc_diagnostics::OxcDiagnostic;
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{SourceType, Span};
use oxc_transformer::{TransformOptions, Transformer};

use crate::suite::TestResult;

#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct Driver {
    pub path: PathBuf,
    // options
    pub transform: Option<TransformOptions>,
    pub compress: bool,
    pub remove_whitespace: bool,
    pub codegen: bool,
    pub allow_return_outside_function: bool,
    // results
    pub panicked: bool,
    pub errors: Vec<OxcDiagnostic>,
    pub printed: String,
}

impl Driver {
    pub fn errors(&mut self) -> Vec<OxcDiagnostic> {
        std::mem::take(&mut self.errors)
    }

    pub fn idempotency(
        mut self,
        case: &'static str,
        source_text: &str,
        source_type: SourceType,
    ) -> TestResult {
        self.run(source_text, source_type);
        let printed1 = self.printed.clone();
        self.run(&printed1, source_type);
        let printed2 = self.printed.clone();
        if printed1 == printed2 {
            TestResult::Passed
        } else {
            TestResult::Mismatch(case, printed1, printed2)
        }
    }

    pub fn run(&mut self, source_text: &str, source_type: SourceType) {
        let allocator = Allocator::default();
        let ParserReturn { mut program, errors, trivias, panicked } =
            Parser::new(&allocator, source_text, source_type)
                .allow_return_outside_function(self.allow_return_outside_function)
                .parse();
        self.panicked = panicked;

        if self.check_comments(&trivias) {
            return;
        }

        // Make sure serialization doesn't crash; also for code coverage.
        let _serializer = program.serializer();

        if !errors.is_empty() {
            self.errors.extend(errors);
        }

        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(trivias.clone())
            .with_check_syntax_error(true)
            .build_module_record(self.path.clone(), &program)
            .build(&program);

        if !semantic_ret.errors.is_empty() {
            self.errors.extend(semantic_ret.errors);
            return;
        }

        // TODO
        // if self.check_semantic(&semantic_ret.semantic) {
        // return;
        // }

        if let Some(options) = self.transform.clone() {
            Transformer::new(
                &allocator,
                &self.path,
                source_type,
                source_text,
                trivias.clone(),
                options,
            )
            .build(&mut program);
        }

        if self.compress {
            Compressor::new(&allocator, CompressOptions::all_true()).build(&mut program);
        }

        if self.codegen {
            let comment_options = CommentOptions { preserve_annotate_comments: true };

            let printed = if self.remove_whitespace {
                WhitespaceRemover::new().build(&program).source_text
            } else {
                CodeGenerator::new()
                    .enable_comment(source_text, trivias, comment_options)
                    .build(&program)
                    .source_text
            };

            self.printed = printed;
        }
    }

    fn check_comments(&mut self, trivias: &Trivias) -> bool {
        let mut uniq: HashSet<Span> = HashSet::new();
        for comment in trivias.comments() {
            if !uniq.insert(comment.span) {
                self.errors
                    .push(OxcDiagnostic::error("Duplicate Comment").with_label(comment.span));
                return true;
            }
        }
        false
    }

    #[allow(unused)]
    fn check_semantic(&mut self, semantic: &Semantic<'_>) -> bool {
        if are_all_identifiers_resolved(semantic) {
            return false;
        }
        self.errors.push(OxcDiagnostic::error("symbol or reference is not set"));
        false
    }
}

#[allow(unused)]
fn are_all_identifiers_resolved(semantic: &Semantic<'_>) -> bool {
    use oxc_ast::AstKind;
    use oxc_semantic::AstNode;

    let ast_nodes = semantic.nodes();
    let has_non_resolved = ast_nodes.iter().any(|node| {
        match node.kind() {
            AstKind::BindingIdentifier(id) => {
                let mut parents = ast_nodes.iter_parents(node.id()).map(AstNode::kind);
                parents.next(); // Exclude BindingIdentifier itself
                if let (Some(AstKind::Function(_)), Some(AstKind::IfStatement(_))) =
                    (parents.next(), parents.next())
                {
                    return false;
                }
                id.symbol_id.get().is_none()
            }
            AstKind::IdentifierReference(ref_id) => ref_id.reference_id.get().is_none(),
            _ => false,
        }
    });

    !has_non_resolved
}
