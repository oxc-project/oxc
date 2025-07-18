mod meta;
mod transpile_runner;

use std::path::{Path, PathBuf};

use self::meta::{CompilerSettings, TestCaseContent, TestUnitData};
pub use self::transpile_runner::{TranspileRunner, TypeScriptTranspileCase};
use crate::suite::{Case, Suite, TestResult};

const TESTS_ROOT: &str = "typescript/tests";

pub struct TypeScriptSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TypeScriptSuite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(TESTS_ROOT).join("cases"), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TypeScriptSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        // stack overflows in compiler tests
        #[cfg(any(coverage, coverage_nightly))]
        let supported_paths = ["conformance"].iter().any(|p| path.to_string_lossy().contains(p));
        #[cfg(not(any(coverage, coverage_nightly)))]
        let supported_paths =
            ["conformance", "compiler"].iter().any(|p| path.to_string_lossy().contains(p));
        let unsupported_tests = [
            // these 2 relies on the ts "target" option
            "functionWithUseStrictAndSimpleParameterList.ts",
            "parameterInitializerBeforeDestructuringEmit.ts",
            // these also relies on "target: es5" option w/ RegExp `u` flag
            "unicodeExtendedEscapesInRegularExpressions01.ts",
            "unicodeExtendedEscapesInRegularExpressions02.ts",
            "unicodeExtendedEscapesInRegularExpressions03.ts",
            "unicodeExtendedEscapesInRegularExpressions04.ts",
            "unicodeExtendedEscapesInRegularExpressions05.ts",
            "unicodeExtendedEscapesInRegularExpressions06.ts",
            "unicodeExtendedEscapesInRegularExpressions08.ts",
            "unicodeExtendedEscapesInRegularExpressions09.ts",
            "unicodeExtendedEscapesInRegularExpressions10.ts",
            "unicodeExtendedEscapesInRegularExpressions11.ts",
            "unicodeExtendedEscapesInRegularExpressions13.ts",
            "unicodeExtendedEscapesInRegularExpressions15.ts",
            "unicodeExtendedEscapesInRegularExpressions16.ts",
            "unicodeExtendedEscapesInRegularExpressions18.ts",
            // TS18010, but requires JSDoc TS parsing
            "privateNamesIncompatibleModifiersJs.ts",
            // Exporting JSDoc types from `.js`
            "importingExportingTypes.ts",
            // This is just a binary file
            "TransportStream.ts",
            // Emits TS5052 (complains compilerOptions are invalid, which we do not support), also implies TS2564 but not emitted
            "esDecorators-emitDecoratorMetadata.ts",
        ]
        .iter()
        .any(|p| path.to_string_lossy().contains(p));
        !supported_paths || unsupported_tests
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct TypeScriptCase {
    path: PathBuf,
    pub code: String,
    pub units: Vec<TestUnitData>,
    pub settings: CompilerSettings,
    error_codes: Vec<String>,
    pub result: TestResult,
}

impl TypeScriptCase {
    /// Simple check for usage such as `semantic`.
    /// `should_fail()` will return `true` only if there are still error codes remaining
    /// after filtering out the not-supported ones.
    pub fn should_fail_with_any_error_codes(&self) -> bool {
        !self.error_codes.is_empty()
    }
}

impl Case for TypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let TestCaseContent { tests, settings, error_codes } =
            TestCaseContent::make_units_from_test(&path, &code);
        Self { path, code, units: tests, settings, error_codes, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        // If there are still error codes to be supported, it should fail
        self.error_codes.iter().any(|code| !NOT_SUPPORTED_ERROR_CODES.contains(code.as_str()))
    }

    fn always_strict(&self) -> bool {
        self.settings.always_strict
    }

    fn run(&mut self) {
        let result = self
            .units
            .iter()
            .map(|unit| self.parse(&unit.content, unit.source_type))
            .find(Result::is_err)
            .unwrap_or(Ok(()));
        self.result = self.evaluate_result(result);
    }
}

// spellchecker:off
static NOT_SUPPORTED_ERROR_CODES: phf::Set<&'static str> = phf::phf_set![
    // TODO: More not-supported error codes here
    "2315",  // Type 'U' is not generic.
    "2665", // Invalid module name in augmentation. Module 'foo' resolves to an untyped module at '/node_modules/foo/index.js', which cannot be augmented.
    "5009", // Cannot find the common subdirectory path for the input files.
    "5052", // Option 'checkJs' cannot be specified without specifying option 'allowJs'.
    "5053", // Option 'mapRoot' cannot be specified with option 'inlineSourceMap'.
    "5055", // Cannot write file 'a.d.ts' because it would overwrite input file.
    "5056", // Cannot write file 'a.js' because it would be overwritten by multiple input files.
    "5059", // Invalid value for '--reactNamespace'. 'my-React-Lib' is not a valid identifier.
    "5061", // Pattern 'too*many*asterisks' can have at most one '*' character.
    "5062", // Substitution '*2*' in pattern '*1*' can have at most one '*' character.
    "5063", // Substitutions for pattern '*' should be an array.
    "5064", // Substitution '1' for pattern '*' has incorrect type, expected 'string', got 'number'.
    "5066", // Substitutions for pattern 'foo' shouldn't be an empty array.
    "5067", // Invalid value for 'jsxFactory'. 'Element.createElement=' is not a valid identifier or qualified-name.
    "5069", // Option 'emitDeclarationOnly' cannot be specified without specifying option 'declaration' or option 'composite'.
    "5070", // Option '--resolveJsonModule' cannot be specified when 'moduleResolution' is set to 'classic'.
    "5071", // Option '--resolveJsonModule' cannot be specified when 'module' is set to 'none', 'system', or 'umd'.
    "5074", // Option '--incremental' can only be specified using tsconfig, emitting to single file or when option '--tsBuildInfoFile' is specified.
    "5088", // The inferred type of 'foo' references a type with a cyclic structure which cannot be trivially serialized. A type annotation is necessary.
    "5090", // Non-relative paths are not allowed when 'baseUrl' is not set. Did you forget a leading './'?
    "5091", // Option 'preserveConstEnums' cannot be disabled when 'isolatedModules' is enabled.
    "5095", // Option 'bundler' can only be used when 'module' is set to 'preserve' or to 'es2015' or later.
    "5097", // An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.
    "5098", // Option 'customConditions' can only be used when 'moduleResolution' is set to 'node16', 'nodenext', or 'bundler'.
    "5101", // Option 'noImplicitUseStrict' is deprecated and will stop functioning in TypeScript 5.5. Specify compilerOption '"ignoreDeprecations": "5.0"' to silence this error.
    "5102", // Option 'noImplicitUseStrict' has been removed. Please remove it from your configuration.
    "5103", // Invalid value for '--ignoreDeprecations'.
    "5105", // Option 'verbatimModuleSyntax' cannot be used when 'module' is set to 'UMD', 'AMD', or 'System'.
    "5107", // Option 'target=ES3' is deprecated and will stop functioning in TypeScript 5.5. Specify compilerOption '"ignoreDeprecations": "5.0"' to silence this error.
    "5108", // Option 'target=ES3' has been removed. Please remove it from your configuration.
    "5109", // Option 'moduleResolution' must be set to 'Node16' (or left unspecified) when option 'module' is set to 'Node18'.
    "5110", // Option 'module' must be set to 'Node16' when option 'moduleResolution' is set to 'Node16'.
    "6053", // File 'invalid.ts' not found.
    "6054", // File 'b.js.map' has an unsupported extension. The only supported extensions are '.ts', '.tsx', '.d.ts', '.js', '.jsx', '.cts', '.d.cts', '.cjs', '.mts', '.d.mts', '.mjs'.
    "6082", // Only 'amd' and 'system' modules are supported alongside --outFile.
    "6131", // Cannot compile modules using option 'outFile' unless the '--module' flag is 'amd' or 'system'.
    "6133", // 'f1' is declared but its value is never read.
    "6137", // Cannot import type declaration files. Consider importing 'foo-bar' instead of '@types/foo-bar'.
    "6138", // Property 'used' is declared but its value is never read.
    "6142", // Module '/foo' was resolved to '/foo.jsx', but '--jsx' is not set.
    "6192", // All imports in import declaration are unused.
    "6196", // 'i1' is declared but never used.
    "6198", // All destructured elements are unused.
    "6199", // All variables are unused.
    "6200", // Definitions of the following identifiers conflict with those in another file: A, B, C, D, E, F, G, H, I
    "6205", // All type parameters are unused.
    "6229", // Tag 'MyComp4' expects at least '4' arguments, but the JSX factory 'React.createElement' provides at most '2'.
    "6231", // Could not resolve the path 'invalid' with the extensions: '.ts', '.tsx', '.d.ts', '.js', '.jsx', '.cts', '.d.cts', '.cjs', '.mts', '.d.mts', '.mjs'.
    "6232", // Declaration augments declaration in another file. This cannot be serialized.
    "6234", // This expression is not callable because it is a 'get' accessor. Did you mean to use it without '()'?
    "6263", // Module './dir/native.node' was resolved to 'dir/native.d.node.ts', but '--allowArbitraryExtensions' is not set.
    "6379", // Composite projects may not disable incremental compilation.
    "7005", // Variable 'x' implicitly has an 'any' type.
    "7006", // Parameter 'x' implicitly has an 'any' type.
    "7008", // Member 'v' implicitly has an 'any' type.
    "7009", // 'new' expression, whose target lacks a construct signature, implicitly has an 'any' type.
    "7010", // 'temp', which lacks return-type annotation, implicitly has an 'any' return type.
    "7011", // Function expression, which lacks return-type annotation, implicitly has an 'any' return type.
    "7012", // This overload implicitly returns the type 'any' because it lacks a return type annotation.
    "7013", // Construct signature, which lacks return-type annotation, implicitly has an 'any' return type.
    "7014", // Function type, which lacks return-type annotation, implicitly has an 'any' return type.
    "7015", // Element implicitly has an 'any' type because index expression is not of type 'number'.
    "7016", // Could not find a declaration file for module './b'. '/src/b.js' implicitly has an 'any' type.
    "7017", // Element implicitly has an 'any' type because type 'typeof globalThis' has no index signature.
    "7018", // Object literal's property 's' implicitly has an 'any' type.
    "7019", // Rest parameter 'r' implicitly has an 'any[]' type.
    "7020", // Call signature, which lacks return-type annotation, implicitly has an 'any' return type.
    "7022", // 'value1' implicitly has type 'any' because it does not have a type annotation and is referenced directly or indirectly in its own initializer.
    "7023", // 'next' implicitly has return type 'any' because it does not have a return type annotation and is referenced directly or indirectly in one of its return expressions.
    "7024", // Function implicitly has return type 'any' because it does not have a return type annotation and is referenced directly or indirectly in one of its return expressions.
    "7025", // Generator implicitly has yield type 'any'. Consider supplying a return type annotation.
    "7026", // JSX element implicitly has type 'any' because no interface 'JSX.IntrinsicElements' exists.
    "7027", // Unreachable code detected.
    "7028", // Unused label.
    "7029", // Fallthrough case in switch.
    "7030", // Not all code paths return a value.
    "7031", // Binding element 'a5' implicitly has an 'any' type.
    "7032", // Property 'message' implicitly has type 'any', because its set accessor lacks a parameter type annotation.
    "7033", // Property 'message' implicitly has type 'any', because its get accessor lacks a return type annotation.
    "7034", // Variable 'x' implicitly has type 'any[]' in some locations where its type cannot be determined.
    "7036", // Dynamic import's specifier must be of type 'string', but here has type 'null'.
    "7039", // Mapped object type implicitly has an 'any' template type.
    "7041", // The containing arrow function captures the global value of 'this'.
    "7052", // Element implicitly has an 'any' type because type '{ get: (key: string) => string; }' has no index signature. Did you mean to call 'c.get'?
    "7053", // Element implicitly has an 'any' type because expression of type 'string' can't be used to index type '{}'.
    "7055", // 'h', which lacks return-type annotation, implicitly has an 'any' yield type.
    "7056", // The inferred type of this node exceeds the maximum length the compiler will serialize. An explicit type annotation is needed.
    "7057", // 'yield' expression implicitly results in an 'any' type because its containing generator lacks a return-type annotation.
    "8021", // JSDoc '@typedef' tag should either have a type annotation or be followed by '@property' or '@member' tags.
    "8022", // JSDoc '@extends' is not attached to a class.
    "8023", // JSDoc '@extends Mismatch' does not match the 'extends B' clause.
    "8024", // JSDoc '@param' tag has name 's', but there is no parameter with that name.
    "8026", // Expected A<T> type arguments; provide these with an '@extends' tag.
    "8029", // JSDoc '@param' tag has name 'rest', but there is no parameter with that name. It would match 'arguments' if it had an array type.
    "8030", // The type of a function declaration must match the function's signature.
    "8032", // Qualified name 'xyz.p' is not allowed without a leading '@param {object} xyz'.
    "8033", // A JSDoc '@typedef' comment may not contain multiple '@type' tags.
    "8039", // A JSDoc '@template' tag may not follow a '@typedef', '@callback', or '@overload' tag
    "9005", // Declaration emit for this file requires using private name 'Sub'. An explicit type annotation may unblock declaration emit.
    "9006", // DeclaDeclaration emit for this file requires using private name 'Item' from module '"some-mod"'. An explicit type annotation may unblock declaration emit.
    "9007", // FunctDeclaion must have an explicit return type annotation with --isolatedDeclarations.
    "9008", // MethoDeclad must have an explicit return type annotation with --isolatedDeclarations.
    "9009", // At leDeclaast one accessor must have an explicit type annotation with --isolatedDeclarations.
    "9010", // VariaDeclable must have an explicit type annotation with --isolatedDeclarations.
    "9011", // ParamDeclaeter must have an explicit type annotation with --isolatedDeclarations.
    "9012", // PropeDeclarty must have an explicit type annotation with --isolatedDeclarations.
    "9013", // ExpreDeclassion type can't be inferred with --isolatedDeclarations.
    "9015", // ObjecDeclats that contain spread assignments can't be inferred with --isolatedDeclarations.
    "9016", // ObjecDeclats that contain shorthand properties can't be inferred with --isolatedDeclarations.
    "9017", // Only Declaconst arrays can be inferred with --isolatedDeclarations.
    "9018", // ArrayDeclas with spread elements can't inferred with --isolatedDeclarations.
    "9019", // BindiDeclang elements can't be exported directly with --isolatedDeclarations.
    "9020", // Enum Declamember initializers must be computable without references to external symbols with --isolatedDeclarations.
    "9021", // ExtenDeclads clause can't contain an expression with --isolatedDeclarations.
    "9022", // InferDeclaence from class expressions is not supported with --isolatedDeclarations.
    "9023", // AssigDeclaning properties to functions without declaring them is not supported with --isolatedDeclarations. Add an explicit declaration for the properties assigned to this function.
    "9026", // DeclaDeclaration emit for this file requires preserving this import for augmentations. This is not supported with --isolatedDeclarations.
    "9037", // DefauDeclalt exports can't be inferred with --isolatedDeclarations.
    "9038", // CompuDeclated property names on class or object literals cannot be inferred with --isolatedDeclarations.
    "17004", // Cannot use JSX unless the '--jsx' flag is provided.
    "17016", // The 'jsxFragmentFactory' compiler option must be provided to use JSX fragments with the 'jsxFactory' compiler option.
    "17017", // An @jsxFrag pragma is required when using an @jsx pragma with JSX fragments.
    "18028", // Private identifiers are only available when targeting ECMAScript 2015 and higher.
    "18033", // Type 'Number' is not assignable to type 'number' as required for computed enum member values.
    "18035", // Invalid value for 'jsxFragmentFactory'. '234' is not a valid identifier or qualified-name.
    "18042", // 'Prop' is a type and cannot be imported in JavaScript files. Use 'import("./component").Prop' in a JSDoc type annotation.
    "18043", // Types cannot appear in export declarations in JavaScript files.
    "18045", // Properties with the 'accessor' modifier are only available when targeting ECMAScript 2015 and higher.
    "18046", // 'x' is of type 'unknown'.
    "18047", // 'x' is possibly 'null'.
    "18048", // 'x' is possibly 'undefined'.
    "18049", // 'x' is possibly 'null' or 'undefined'.
    "18055", // 'A.a' has a string type, but must have syntactically recognizable string syntax when 'isolatedModules' is enabled.
    "18057", // String literal import and export names are not supported when the '--module' flag is set to 'es2015' or 'es2020'.
];
// spellchecker:on
