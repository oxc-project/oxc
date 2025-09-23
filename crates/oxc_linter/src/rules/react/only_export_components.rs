use oxc_ast::ast::*;
use oxc_ast::ast_kind::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn only_export_components_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

fn export_all_dignostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("This rule can't verify that `export *` only exports components.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct OnlyExportComponentsConfig {
    /// If you use a framework that handles HMR of some specific exports, you
    /// can use this option to avoid warning for them.
    ///
    /// Example for Remix:
    /// ```json
    /// {
    ///   "react-refresh/only-export-components": [
    ///     "error",
    ///     { "allowExportNames": ["meta", "links", "headers", "loader", "action"] }
    ///   ]
    /// }
    /// ```
    allow_export_names: Vec<String>,

    /// Don't warn when a constant (string, number, boolean, templateLiteral) is
    /// exported aside one or more components.
    ///
    /// This should be enabled if the fast refresh implementation correctly
    /// handles this case (HMR when the constant doesn't change, propagate
    /// update to importers when the constant changes.). Vite supports it, PR
    /// welcome if you notice other integrations works well.
    ///
    /// Enabling this option allows code such as the following:
    ///
    /// ```tsx
    /// export const CONSTANT = 3;
    /// export const Foo = () => <></>;
    /// ```
    allow_constant_exports: bool,
    /// If you're using JSX inside `.js` files (which I don't recommend because it
    /// forces you to configure every tool you use to switch the parser), you
    /// can still use the plugin by enabling this option. To reduce the number
    /// of false positive, only files importing `react` are checked.
    check_js: bool,

    /// If you're exporting a component wrapped in a custom HOC, you can use
    /// this option to avoid false positives.
    ///
    /// ```json
    /// {
    ///     "react-refresh/only-export-components": [
    ///       "error",
    ///       { "customHOCs": ["observer", "withAuth"] }
    ///     ]
    /// }
    /// ```
    #[serde(rename = "customHOCs")]
    custom_hocs: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct OnlyExportComponents(Box<OnlyExportComponentsConfig>);

impl std::ops::Deref for OnlyExportComponents {
    type Target = OnlyExportComponentsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<OnlyExportComponentsConfig> for OnlyExportComponents {
    fn from(config: OnlyExportComponentsConfig) -> Self {
        Self(Box::new(config))
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    OnlyExportComponents,
    react,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for OnlyExportComponents {
    // fn run_once<'a>(&self, ctx: &LintContext<'a>) {
    //     ctx.module_record().
    // }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportAllDeclaration(export) => {
                if export.export_kind.is_type() {
                    return;
                }
                ctx.diagnostic(export_all_dignostic(export.span));
            }
            AstKind::ExportDefaultDeclaration(export) => {
                self.handle_export_default_declaration(export, ctx)
            }
            AstKind::ExportNamedDeclaration(export) => {
                self.handle_export_named_declaration(export, ctx)
            }
            AstKind::ExportSpecifier(export) => self.handle_export_specifier(export, ctx),
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        const TEST_OR_STORY_PATTERNS: [&str; 4] = [".test.", ".spec", ".cy.", ".stories"];
        const JSX_EXT: [&str; 2] = [".jsx", ".tsx"];

        let Some(filepath) = ctx.file_path().to_str() else {
            return false;
        };

        // skip test and story files
        // for test_pattern in &[".test.", ".spec", ".cy.", ".stories"] {
        //     if filepath.contains(test_pattern) {
        //         return false;
        //     }
        // }
        if TEST_OR_STORY_PATTERNS.iter().any(|pattern| filepath.contains(pattern)) {
            return false;
        }

        // allow .jsx and .tsx files
        if JSX_EXT.iter().any(|ext| filepath.ends_with(ext)) {
            return true;
        }
        // allow .js files if check_js is true
        // TODO: check for 'react' import
        if self.check_js && filepath.ends_with(".js") {
            return true;
        }

        false
    }
}

impl OnlyExportComponents {
    fn handle_export_default_declaration<'a>(
        &self,
        export: &ExportDefaultDeclaration<'a>,
        ctx: &LintContext<'a>,
    ) {
        todo!()
    }

    fn handle_export_named_declaration<'a>(
        &self,
        export: &ExportNamedDeclaration<'a>,
        ctx: &LintContext<'a>,
    ) {
        todo!()
    }

    fn handle_export_specifier<'a>(&self, export: &ExportSpecifier<'a>, ctx: &LintContext<'a>) {
        todo!()
    }
}

// #!/usr/bin/env tnode
// import { test } from "bun:test";
// import parser from "@typescript-eslint/parser";
// import { RuleTester } from "eslint";
// import { onlyExportComponents } from "./only-export-components.ts";

// const ruleTester = new RuleTester({ languageOptions: { parser } });

// const valid = [
//   {
//     name: "Direct export named component",
//     code: "export function Foo() {};",
//   },
//   {
//     name: "Export named component",
//     code: "function Foo() {}; export { Foo };",
//   },
//   {
//     name: "Export default named component",
//     code: "function Foo() {}; export default Foo;",
//   },
//   {
//     name: "Direct export default named component",
//     code: "export default function Foo() {}",
//   },
//   {
//     name: "Direct export AF component",
//     code: "export const Foo = () => {};",
//   },
//   {
//     name: "Direct export AF component with number",
//     code: "export const Foo2 = () => {};",
//   },
//   {
//     name: "Direct export uppercase function",
//     code: "export function CMS() {};",
//   },
//   {
//     name: "Uppercase component with forwardRef",
//     code: "export const SVG = forwardRef(() => <svg/>);",
//   },
//   {
//     name: "Direct export uppercase component",
//     code: "export const CMS = () => {};",
//   },
//   {
//     name: "Export AF component",
//     code: "const Foo = () => {}; export { Foo };",
//   },
//   {
//     name: "Default export AF component",
//     code: "const Foo = () => {}; export default Foo;",
//   },
//   {
//     name: "Two components & local variable",
//     code: "const foo = 4; export const Bar = () => {}; export const Baz = () => {};",
//   },
//   {
//     name: "Two components & local function",
//     code: "const foo = () => {}; export const Bar = () => {}; export const Baz = () => {};",
//   },
//   {
//     name: "styled components",
//     code: "export const Foo = () => {}; export const Bar = styled.div`padding-bottom: 6px;`;",
//   },
//   {
//     name: "Direct export variable",
//     code: "export const foo = 3;",
//   },
//   {
//     name: "Export variables",
//     code: "const foo = 3; const bar = 'Hello'; export { foo, bar };",
//   },
//   {
//     name: "Direct export AF",
//     code: "export const foo = () => {};",
//   },
//   {
//     name: "Direct export default AF",
//     code: "export default function foo () {};",
//   },
//   {
//     name: "export default memo function",
//     code: "export default memo(function Foo () {});",
//   },
//   {
//     name: "export default React.memo function",
//     code: "export default React.memo(function Foo () {});",
//   },
//   {
//     name: "export default memo function assignment",
//     code: "const Foo = () => {}; export default memo(Foo);",
//   },
//   {
//     name: "export default React.memo function assignment",
//     code: "const Foo = () => {}; export default React.memo(Foo);",
//   },
//   {
//     name: "export default memo function declaration",
//     code: "function Foo() {}; export default memo(Foo);",
//   },
//   {
//     name: "export default React.memo function declaration",
//     code: "function Foo() {}; export default React.memo(Foo);",
//   },
//   {
//     name: "export default React.memo function declaration with type assertion",
//     code: "function Foo() {}; export default React.memo(Foo) as typeof Foo;",
//   },
//   {
//     name: "export type *",
//     code: "export type * from './module';",
//     filename: "Test.tsx",
//   },
//   {
//     name: "export type { foo }",
//     code: "type foo = string; export const Foo = () => null; export type { foo };",
//     filename: "Test.tsx",
//   },
//   {
//     name: "export type foo",
//     code: "export type foo = string; export const Foo = () => null;",
//     filename: "Test.tsx",
//   },
//   {
//     name: "Mixed export in JS without checkJS",
//     code: "export const foo = () => {}; export const Bar = () => {};",
//     filename: "Test.js",
//   },
//   {
//     name: "Mixed export in JS without react import",
//     code: "export const foo = () => {}; export const Bar = () => {};",
//     filename: "Test.js",
//     options: [{ checkJS: true }],
//   },
//   {
//     name: "Component and number constant with allowConstantExport",
//     code: "export const foo = 4; export const Bar = () => {};",
//     options: [{ allowConstantExport: true }],
//   },
//   {
//     name: "Component and negative number constant with allowConstantExport",
//     code: "export const foo = -4; export const Bar = () => {};",
//     options: [{ allowConstantExport: true }],
//   },
//   {
//     name: "Component and string constant with allowConstantExport",
//     code: "export const CONSTANT = 'Hello world'; export const Foo = () => {};",
//     options: [{ allowConstantExport: true }],
//   },
//   {
//     name: "Component and template literal with allowConstantExport",
//     // eslint-disable-next-line no-template-curly-in-string
//     code: "const foo = 'world'; export const CONSTANT = `Hello ${foo}`; export const Foo = () => {};",
//     options: [{ allowConstantExport: true }],
//   },
//   {
//     name: "Component and allowed export",
//     code: "export const loader = () => {}; export const Bar = () => {};",
//     options: [{ allowExportNames: ["loader", "meta"] }],
//   },
//   {
//     name: "Component and allowed function export",
//     code: "export function loader() {}; export const Bar = () => {};",
//     options: [{ allowExportNames: ["loader", "meta"] }],
//   },
//   {
//     name: "Only allowed exports without component",
//     code: "export const loader = () => {}; export const meta = { title: 'Home' };",
//     options: [{ allowExportNames: ["loader", "meta"] }],
//   },
//   {
//     name: "Export as default",
//     code: "export { App as default }; const App = () => <>Test</>;",
//   },
//   {
//     name: "Allow connect from react-redux",
//     code: "const MyComponent = () => {}; export default connect(() => ({}))(MyComponent);",
//   },
//   {
//     name: "Two components, one of them with 'Context' in its name",
//     code: "export const MyComponent = () => {}; export const ChatContext = () => {};",
//   },
//   {
//     name: "Component & local React context",
//     code: "export const MyComponent = () => {}; const MyContext = createContext('test');",
//   },
//   {
//     name: "Only React context",
//     code: "export const MyContext = createContext('test');",
//   },
//   {
//     name: "Custom HOCs like mobx's observer",
//     code: "const MyComponent = () => {}; export default observer(MyComponent);",
//     options: [{ customHOCs: ["observer"] }],
//   },
//   {
//     name: "Local constant with component casing and non component function",
//     code: "const SomeConstant = 42; export function someUtility() { return SomeConstant }",
//   },
//   {
//     name: "Component and as const constant with allowConstantExport",
//     code: "export const MyComponent = () => {}; export const MENU_WIDTH = 232 as const;",
//     options: [{ allowConstantExport: true }],
//   },
//   {
//     name: "Type assertion in memo export",
//     code: "export const MyComponent = () => {}; export default memo(MyComponent as any);",
//   },
//   {
//     name: "Type assertion for memo export",
//     code: "export const MyComponent = () => {}; export default memo(MyComponent) as any;",
//   },
//   {
//     name: "Nested memo HOC",
//     code: "export const MyComponent = () => {}; export default memo(forwardRef(MyComponent));",
//   },
// ];

// const invalid = [
//   {
//     name: "Component and function",
//     code: "export const foo = () => {}; export const Bar = () => {};",
//     errorId: "namedExport",
//   },
//   {
//     name: "Component and function with allowConstantExport",
//     code: "export const foo = () => {}; export const Bar = () => {};",
//     errorId: "namedExport",
//     options: [{ allowConstantExport: true }],
//   },
//   {
//     name: "Component and variable (direct export)",
//     code: "export const foo = 4; export const Bar = () => {};",
//     errorId: "namedExport",
//   },
//   {
//     name: "Component and PascalCase variable",
//     code: "export function Component() {}; export const Aa = 'a'",
//     errorId: "namedExport",
//   },
//   {
//     name: "Component and variable",
//     code: "const foo = 4; const Bar = () => {}; export { foo, Bar };",
//     errorId: "namedExport",
//   },
//   {
//     name: "Export all",
//     code: "export * from './foo';",
//     errorId: "exportAll",
//   },
//   {
//     name: "Export default anonymous AF",
//     code: "export default () => {};",
//     errorId: "anonymousExport",
//   },
//   {
//     name: "export default anonymous memo AF",
//     code: "export default memo(() => {});",
//     errorId: "anonymousExport",
//   },
//   {
//     name: "Export default anonymous function",
//     code: "export default function () {};",
//     errorId: "anonymousExport",
//   },
//   {
//     name: "Component and constant",
//     code: "export const CONSTANT = 3; export const Foo = () => {};",
//     errorId: "namedExport",
//   },
//   {
//     name: "Component and enum",
//     code: "export enum Tab { Home, Settings }; export const Bar = () => {};",
//     errorId: "namedExport",
//   },
//   {
//     name: "Unexported component and export",
//     code: "const Tab = () => {}; export const tabs = [<Tab />, <Tab />];",
//     errorId: "localComponents",
//   },
//   {
//     name: "Unexported component and no export",
//     code: "const App = () => {}; createRoot(document.getElementById('root')).render(<App />);",
//     errorId: "noExport",
//   },
//   {
//     name: "Mixed export in JS with react import",
//     code: `
//      import React from 'react';
//      export const CONSTANT = 3; export const Foo = () => {};
//     `,
//     filename: "Test.js",
//     options: [{ checkJS: true }],
//     errorId: "namedExport",
//   },
//   {
//     name: "export default compose",
//     code: "export default compose()(MainView);",
//     filename: "Test.jsx",
//     errorId: "anonymousExport",
//   },
//   {
//     name: "Component and export non in allowExportNames",
//     code: "export const loader = () => {}; export const Bar = () => {}; export const foo = () => {};",
//     options: [{ allowExportNames: ["loader", "meta"] }],
//     errorId: "namedExport",
//   },
//   {
//     name: "Export with arbitrary module identifier",
//     code: 'const Foo = () => {}; export { Foo as "🍌"}',
//     errorId: "localComponents",
//   },
//   {
//     name: "Component and React Context",
//     code: "export const MyComponent = () => {}; export const MyContext = createContext('test');",
//     errorId: "reactContext",
//   },
//   {
//     name: "Component and React Context with React import",
//     code: "export const MyComponent = () => {}; export const MyContext = React.createContext('test');",
//     errorId: "reactContext",
//   },
//   {
//     name: "should be invalid when custom HOC is used without adding it to the rule configuration",
//     code: "const MyComponent = () => {}; export default observer(MyComponent);",
//     errorId: ["localComponents", "anonymousExport"],
//   },
// ];

// const it = (name: string, cases: Parameters<typeof ruleTester.run>[2]) => {
//   test(name, () => {
//     ruleTester.run(
//       "only-export-components",
//       // @ts-expect-error Mismatch between typescript-eslint and eslint
//       onlyExportComponents,
//       cases,
//     );
//   });
// };

// for (const { name, code, filename, options = [] } of valid) {
//   it(name, {
//     valid: [{ filename: filename ?? "Test.jsx", code, options }],
//     invalid: [],
//   });
// }

// for (const { name, code, errorId, filename, options = [] } of invalid) {
//   it(name, {
//     valid: [],
//     invalid: [
//       {
//         filename: filename ?? "Test.jsx",
//         code,
//         errors: Array.isArray(errorId)
//           ? errorId.map((messageId) => ({ messageId }))
//           : [{ messageId: errorId }],
//         options,
//       },
//     ],
//   });
// }
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "export function Foo() {};",
        "function Foo() {}; export { Foo };",
        "function Foo() {}; export default Foo;",
        "export default function Foo() {}",
        "export const Foo = () => {};",
        "export const Foo2 = () => {};",
        "export function CMS() {};",
        "export const SVG = forwardRef(() => <svg/>);",
        "export const CMS = () => {};",
        "const Foo = () => {}; export { Foo };",
        "const Foo = () => {}; export default Foo;",
        "const foo = 4; export const Bar = () => {}; export const Baz = () => {};",
        "const foo = () => {}; export const Bar = () => {}; export const Baz = () => {};",
        "export const Foo = () => {}; export const Bar = styled.div`padding-bottom: 6px;`;",
        "export const foo = 3;",
        "const foo = 3; const bar = 'Hello'; export { foo, bar };",
        "export const foo = () => {};",
        "export default function foo () {};",
        "export default memo(function Foo () {});",
        "export default React.memo(function Foo () {});",
        "const Foo = () => {}; export default memo(Foo);",
        "const Foo = () => {}; export default React.memo(Foo);",
        "function Foo() {}; export default memo(Foo);",
        "function Foo() {}; export default React.memo(Foo);",
        "function Foo() {}; export default React.memo(Foo) as typeof Foo;",
        "export type * from './module';",
        "type foo = string; export const Foo = () => null; export type { foo };",
        "export type foo = string; export const Foo = () => null;",
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = 4; export const Bar = () => {};",
        "export const foo = -4; export const Bar = () => {};",
        "export const CONSTANT = 'Hello world'; export const Foo = () => {};",
        "const foo = 'world'; export const CONSTANT = `Hello ${foo}`; export const Foo = () => {};",
        "export const loader = () => {}; export const Bar = () => {};",
        "export function loader() {}; export const Bar = () => {};",
        "export const loader = () => {}; export const Bar = () => {};",
        "export const loader = () => {}; export const Bar = () => {};",
        "export { App as default }; const App = () => <>Test</>;",
        "const MyComponent = () => {}; export default connect(() => ({}))(MyComponent);",
        "export const MyComponent = () => {}; export const ChatContext = () => {};",
        "export const MyComponent = () => {}; const MyContext = createContext('test');",
        "export const MyContext = createContext('test');",
        "export const MyComponent = () => {}; export const MyContext = createContext('test');",
        "export const MyComponent = () => {}; export const MyContext = React.createContext('test');",
        "const MyComponent = () => {}; export default observer(MyComponent);",
        "const SomeConstant = 42; export function someUtility() { return SomeConstant }",
        "export const MyComponent = () => {}; export const MENU_WIDTH = 232 as const;",
        "export const MyComponent = () => {}; export default memo(MyComponent as any);",
        "export const MyComponent = () => {}; export default memo(MyComponent) as any;",
        "export const MyComponent = () => {}; export default memo(forwardRef(MyComponent));",
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = 4; export const Bar = () => {};",
        "export const foo = -4; export const Bar = () => {};",
    ];

    let fail = vec![
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = 4; export const Bar = () => {};",
        "export const foo = -4; export const Bar = () => {};",
        "export const CONSTANT = 'Hello world'; export const Foo = () => {};",
        "const foo = 'world'; export const CONSTANT = `Hello ${foo}`; export const Foo = () => {};",
        "export const loader = () => {}; export const Bar = () => {};",
        "export function loader() {}; export const Bar = () => {};",
        "export const loader = () => {}; export const Bar = () => {};",
        "export { App as default }; const App = () => <>Test</>;",
        "const MyComponent = () => {}; export default connect(() => ({}))(MyComponent);",
        "export const MyComponent = () => {}; export const ChatContext = () => {};",
        "export const MyComponent = () => {}; const MyContext = createContext('test');",
        "export const MyContext = createContext('test');",
        "export const MyComponent = () => {}; export const MyContext = createContext('test');",
        "export const MyComponent = () => {}; export const MyContext = React.createContext('test');",
        "const MyComponent = () => {}; export default observer(MyComponent);",
        "const SomeConstant = 42; export function someUtility() { return SomeConstant }",
        "export const MyComponent = () => {}; export const MENU_WIDTH = 232 as const;",
        "export const MyComponent = () => {}; export default memo(MyComponent as any);",
        "export const MyComponent = () => {}; export default memo(MyComponent) as any;",
        "export const MyComponent = () => {}; export default memo(forwardRef(MyComponent));",
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = () => {}; export const Bar = () => {};",
        "export const foo = 4; export const Bar = () => {};",
        "export const foo = -4; export const Bar = () => {};",
        "export const CONSTANT = 'Hello world'; export const Foo = () => {};",
        "const foo = 'world'; export const CONSTANT = `Hello ${foo}`; export const Foo = () => {};",
        "export const loader = () => {}; export const Bar = () => {};",
        "export function loader() {}; export const Bar = () => {};",
        "export const loader = () => {}; export const Bar = () => {};",
        "export { App as default }; const App = () => <>Test</>;",
        "const MyComponent = () => {}; export default connect(() => ({}))(MyComponent);",
        "export const MyComponent = () => {}; export const ChatContext = () => {};",
        "export const MyComponent = () => {}; const MyContext = createContext('test');",
        "export const MyContext = createContext('test');",
        "export const MyComponent = () => {}; export const MyContext = createContext('test');",
        "export const MyComponent = () => {}; export const MyContext = React.createContext('test');",
    ];

    Tester::new(OnlyExportComponents::NAME, OnlyExportComponents::PLUGIN, pass, fail)
        .test_and_snapshot();
}
