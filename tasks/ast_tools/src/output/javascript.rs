use std::{
    io::Write,
    process::{Command, Stdio},
};

use lazy_regex::{Captures, Lazy, Regex, lazy_regex};

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::{
    AstBuilder,
    ast::{Expression, Program, Statement, UnaryOperator},
};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_codegen::Codegen;
use oxc_minifier::{
    CompressOptions, CompressOptionsKeepNames, Minifier, MinifierOptions, PropertyReadSideEffects,
    TreeShakeOptions,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::{logln, utils::write_it};

use super::add_header;

/// Format Javascript/Typescript code, and add header.
pub fn print_javascript(code: &str, generator_path: &str) -> String {
    let code = add_header(code, generator_path, "//");
    format(&code)
}

/// Format JS/TS code with `dprint`.
fn format(source_text: &str) -> String {
    let mut dprint = Command::new("dprint")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(["fmt", "--stdin", "placeholder_filename.ts"])
        .spawn()
        .expect("Failed to run dprint (is it installed?)");

    let stdin = dprint.stdin.as_mut().unwrap();
    stdin.write_all(source_text.as_bytes()).unwrap();
    stdin.flush().unwrap();

    let output = dprint.wait_with_output().unwrap();
    if output.status.success() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        // Formatting failed. Return unformatted code, to aid debugging.
        let error =
            String::from_utf8(output.stderr).unwrap_or_else(|_| "Unknown error".to_string());
        logln!("FAILED TO FORMAT JS/TS code:\n{error}");
        source_text.to_string()
    }
}

/// Trait to generate several variants of JS code.
///
/// `const` statements will be inserted at the top of the file for the flags listed in `FLAG_NAMES`.
///
/// For each variant, these variables are set to the values provided.
/// Minifier is then run to shake out any dead code.
///
/// ## Basic example
///
/// ```
/// struct Gen;
///
/// impl VariantGenerator<1> for Gen { // `1` is the number of flags
///     const FLAG_NAMES: [&str; 1] = ["DO_STUFF"];
/// }
///
/// // Generate 2 variants with `DO_STUFF = false` and `DO_STUFF = true`
/// let mut variants: Vec<String> = Gen.generate(&code);
/// assert_eq!(variants.len(), 2);
/// let no_stuff = &variants[0];
/// let with_stuff = &variants[1];
/// ```
///
/// ```js
/// // This is inserted by `VariantGenerator`
/// const DO_STUFF = false; // or `true`
///
/// // Code given as input to `generate` can use these consts to gate code.
/// // Minifier will remove it from the variants where `DO_STUFF` is `false`.
/// if (DO_STUFF) doStuff();
/// ```
///
/// ## Specifying variants
///
/// By default, `generate` produces variants for every possible combination of flags.
/// To reduce the number of variants, implement `variants` method.
///
/// ```
/// struct Gen;
///
/// impl VariantGenerator<3> for Gen {
///     const FLAG_NAMES: [&str; 3] = ["STUFF", "OTHER_STUFF", "MORE_STUFF"];
///
///     /// Only generate 2 variants.
///     fn variants(&mut self) -> Vec<[bool; 3]> {
///         vec![
///             [/* STUFF */ true, /* OTHER_STUFF */ false, /* MORE_STUFF */ false],
///             [/* STUFF */ false, /* OTHER_STUFF */ true, /* MORE_STUFF */ true],
///         ]
///     }
/// }
/// ```
///
/// ## Pre-processing
///
/// If you need to modify the AST in other ways for certain variants, implement `pre_process_variant` method.
///
/// `pre_process_variant` is called for each variant, *before* minifier is applied to AST.
///
/// ```
/// struct Gen;
///
/// impl VariantGenerator<2> for Gen {
///     const FLAG_NAMES: [&str; 2] = ["STUFF", "OTHER_STUFF"];
///
///     fn pre_process_variant<'a>(
///         &mut self, program: &mut Program<'a>, flags: [bool; 2], allocator: &'a Allocator
///     ) {
///         if /* OTHER_STUFF */ flags[1] {
///             OtherStuffVisitor(allocator).visit_program(program);
///         }
///     }
/// }
///
/// struct OtherStuffVisitor<'a>(&'a Allocator);
///
/// impl<'a> VisitMut<'a> for OtherStuffVisitor<'a> {
///     // ... modify AST ...
/// }
/// ```
pub trait VariantGenerator<const FLAG_COUNT: usize> {
    /// Names of flag consts in code.
    const FLAG_NAMES: [&str; FLAG_COUNT];

    /// Get variants required.
    ///
    /// Return `Vec` of flag values, in same order as `FLAG_NAMES`.
    ///
    /// By default generates a variant for every possible combination of flags.
    fn variants(&mut self) -> Vec<[bool; FLAG_COUNT]> {
        let variant_count = 1_usize << FLAG_COUNT;
        (0..variant_count)
            .map(|variant_index| {
                let mut flags = [false; FLAG_COUNT];
                for (flag_index, flag) in flags.iter_mut().enumerate() {
                    *flag = (variant_index & (1 << flag_index)) != 0;
                }
                flags
            })
            .collect()
    }

    /// Perform optional pre-processing on AST of a variant before it's minified.
    ///
    /// By default, does nothing.
    #[expect(unused_variables)]
    #[inline]
    fn pre_process_variant<'a>(
        &mut self,
        program: &mut Program<'a>,
        flags: [bool; FLAG_COUNT],
        allocator: &'a Allocator,
    ) {
    }

    /// Generate variants.
    fn generate(&mut self, code: &str) -> Vec<String> {
        // Add `const` statements to top of file
        let input_code = code;
        let mut code = String::new();
        for flag_name in Self::FLAG_NAMES {
            write_it!(code, "const {flag_name} = false;\n");
        }
        code.push('\n');
        code.push_str(input_code);

        // Parse
        let allocator = Allocator::new();
        let mut program = parse_js(&code, &allocator);

        // Get details of variants.
        let mut variants = self.variants();

        // Generate variants.
        // Handle last separately to avoid cloning AST 1 more time than necessary.
        let mut print_allocator = Allocator::new();
        let mut outputs = Vec::with_capacity(variants.len());

        let Some(last_variant) = variants.pop() else { return outputs };

        for flags in variants {
            let mut program = program.clone_in(&print_allocator);
            outputs.push(self.generate_variant(&mut program, flags, &print_allocator));
            print_allocator.reset();
        }

        outputs.push(self.generate_variant(&mut program, last_variant, &print_allocator));

        outputs
    }

    /// Generate variants for a set of flags.
    fn generate_variant<'a>(
        &mut self,
        program: &mut Program<'a>,
        flags: [bool; FLAG_COUNT],
        allocator: &'a Allocator,
    ) -> String {
        for (stmt_index, value) in flags.into_iter().enumerate() {
            let stmt = &mut program.body[stmt_index];
            let Statement::VariableDeclaration(var_decl) = stmt else { unreachable!() };
            let declarator = &mut var_decl.declarations[0];
            let Some(Expression::BooleanLiteral(bool_lit)) = &mut declarator.init else {
                unreachable!()
            };
            bool_lit.value = value;
        }

        self.pre_process_variant(program, flags, allocator);

        print_minified(program, allocator)
    }
}

/// Parse file.
pub fn parse_js<'a>(source_text: &'a str, allocator: &'a Allocator) -> Program<'a> {
    let source_type = SourceType::mjs();
    let parser_ret = Parser::new(allocator, source_text, source_type).parse();
    assert!(parser_ret.errors.is_empty(), "Parse errors: {:#?}", parser_ret.errors);
    parser_ret.program
}

/// Print AST with minified syntax.
///
/// Do not remove whitespace, or mangle symbols.
/// Purpose is not to compress length of code, but to remove dead code.
pub fn print_minified<'a>(program: &mut Program<'a>, allocator: &'a Allocator) -> String {
    // Minify
    let minify_options = MinifierOptions {
        mangle: None,
        compress: Some(CompressOptions {
            keep_names: CompressOptionsKeepNames::all_true(),
            sequences: false,
            treeshake: TreeShakeOptions {
                property_read_side_effects: PropertyReadSideEffects::None,
                ..TreeShakeOptions::default()
            },
            ..CompressOptions::default()
        }),
    };
    Minifier::new(minify_options).minify(allocator, program);

    // Revert minification of `true` to `!0` and `false` to `!1`. It hurts readability.
    let mut unminifier = BooleanUnminifier::new(allocator);
    unminifier.visit_program(program);

    // Print
    let code = Codegen::new().build(program).code;

    // Add back line breaks between function declarations and exports, to aid readability
    #[expect(clippy::items_after_statements)]
    static REGEX: Lazy<Regex> = lazy_regex!(r"\n(export|function|class|const|let) ");

    REGEX
        .replace_all(&code, |caps: &Captures| {
            // `format!("\n\n{} ", &caps[1])` would be simpler, but this avoids allocations
            match &caps[1] {
                "export" => "\n\nexport ",
                "function" => "\n\nfunction ",
                "class" => "\n\nclass ",
                "const" => "\n\nconst ",
                _ => "\n\nlet ",
            }
        })
        .into_owned()
}

/// Visitor which converts `!0` to `true` and `!1` to `false`.
struct BooleanUnminifier<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> BooleanUnminifier<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self { ast: AstBuilder::new(allocator) }
    }
}

impl<'a> VisitMut<'a> for BooleanUnminifier<'a> {
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        if let Expression::UnaryExpression(unary_expr) = expr
            && unary_expr.operator == UnaryOperator::LogicalNot
            && let Expression::NumericLiteral(lit) = &unary_expr.argument
        {
            *expr = self.ast.expression_boolean_literal(unary_expr.span, lit.value == 0.0);
            return;
        }
        walk_mut::walk_expression(self, expr);
    }
}
