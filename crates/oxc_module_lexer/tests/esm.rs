//! <https://github.com/guybedford/es-module-lexer/blob/main/test/_unit.cjs>

use oxc_allocator::Allocator;
use oxc_module_lexer::ImportType;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug, Clone)]
pub struct ImportSpecifier {
    pub n: Option<String>,
    pub s: u32,
    pub e: u32,
    pub ss: u32,
    pub se: u32,
    pub d: ImportType,
    pub a: Option<u32>,
    pub t: bool,
}

impl From<oxc_module_lexer::ImportSpecifier<'_>> for ImportSpecifier {
    fn from(value: oxc_module_lexer::ImportSpecifier) -> Self {
        Self {
            n: value.n.map(|n| n.to_string()),
            s: value.s,
            e: value.e,
            ss: value.ss,
            se: value.se,
            d: value.d,
            a: value.a,
            t: value.t,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportSpecifier {
    pub n: String,
    pub ln: Option<String>,
    pub s: u32,
    pub e: u32,
    pub ls: Option<u32>,
    pub le: Option<u32>,
    pub t: bool,
}

impl From<oxc_module_lexer::ExportSpecifier<'_>> for ExportSpecifier {
    fn from(value: oxc_module_lexer::ExportSpecifier) -> Self {
        Self {
            n: value.n.to_string(),
            ln: value.ln.map(|ln| ln.to_string()),
            s: value.s,
            e: value.e,
            ls: value.ls,
            le: value.le,
            t: value.t,
        }
    }
}

#[non_exhaustive]
pub struct ModuleLexer {
    pub imports: Vec<ImportSpecifier>,
    pub exports: Vec<ExportSpecifier>,
    pub has_module_syntax: bool,
    pub facade: bool,
}

fn parse(source: &str) -> ModuleLexer {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(ret.errors.is_empty(), "{source} should not produce errors.\n{:?}", ret.errors);
    let module_lexer = oxc_module_lexer::ModuleLexer::new().build(&ret.program);
    // Copy data over because `ModuleLexer<'a>` can't be returned
    ModuleLexer {
        imports: module_lexer.imports.into_iter().map(Into::into).collect(),
        exports: module_lexer.exports.into_iter().map(Into::into).collect(),
        has_module_syntax: module_lexer.has_module_syntax,
        facade: module_lexer.facade,
    }
}

trait Slice {
    fn slice(&self, start: u32, end: u32) -> &'static str;
}

impl Slice for &'static str {
    fn slice(&self, start: u32, end: u32) -> &'static str {
        &self[start as usize..end as usize]
    }
}

#[allow(clippy::needless_pass_by_value, clippy::similar_names)]
fn assert_export_is(
    source: &str,
    actual: &ExportSpecifier,
    expected_n: &str,
    expected_ln: Option<&str>,
) {
    // Commented out because:
    // * there are no tests hitting the true branch
    // * `&source[s..e]` is a Rust string, `expected.n.as_str()` is a escaped JavaScript string,
    // which will never be cause for escaped strings.
    // let s = actual.s as usize;
    // let e = actual.e as usize;
    // if matches!(&source[s..s], "\"" | "'") {
    // assert_eq!(&source[s..s], &source[e - 1..e - 1]);
    // } else {
    // assert_eq!(&source[s..e], &expected.n.as_str());
    // }

    let ls = actual.ls;
    let le = actual.le;
    if let Some(expected_ln) = expected_ln {
        if expected_ln.is_empty() {
            assert_eq!(ls, None);
            assert_eq!(le, None);
        } else if let Some(ls) = ls {
            let ls = ls as usize;
            let le = le.unwrap() as usize;
            // Commented out because "true" branch never got him.
            // if matches!(&source[ls..ls], "\"" | "'") {
            // assert_eq!(&source[ls..ls], &source[le - 1..le - 1]);
            // } else {
            assert_eq!(&source[ls..le], expected_ln);
            // }
        }
    }
    assert_eq!(actual.n, expected_n, "n");
    assert_eq!(actual.ln.as_deref(), expected_ln, "ln");
}

/* Added by Oxc */

#[test]
fn named_imports() {
    let source = "import { a, b, c } from 'foo'";
    let imports = &parse(source).imports;
    assert_eq!(imports.len(), 1);
    // assert_eq!(source.slice(impt.ss, impt.se), r#"import(("asdf"))"#);
    // assert_eq!(source.slice(impt.s, impt.e), r#"("asdf")"#);
}

/* Suite Lexer */

#[test]
fn dynamic_import_expression_range() {
    let source = r#"import(("asdf"))"#;
    let impt = &parse(source).imports[0];
    assert_eq!(source.slice(impt.ss, impt.se), r#"import(("asdf"))"#);
    assert_eq!(source.slice(impt.s, impt.e), r#"("asdf")"#);
}

#[test]
fn dynamic_import_expression_range_2() {
    let source = r"import(/* comment */ `asdf` /* comment */)";
    let impt = &parse(source).imports[0];
    assert_eq!(source.slice(impt.ss, impt.se), r"import(/* comment */ `asdf` /* comment */)");
    assert_eq!(source.slice(impt.s, impt.e), r"`asdf`");
}

#[test]
fn dynamic_import_expression_range_3() {
    let source = "import(`asdf` // comment\n)";
    let impt = &parse(source).imports[0];
    assert_eq!(source.slice(impt.ss, impt.se), "import(`asdf` // comment\n)");
    assert_eq!(source.slice(impt.s, impt.e), "`asdf`");
}

#[test]
fn dynamic_import_expression_range_4() {
    let source = "import(\"foo\" + /* comment */ \"bar\")";
    let impt = &parse(source).imports[0];
    assert_eq!(source.slice(impt.ss, impt.se), "import(\"foo\" + /* comment */ \"bar\")");
    assert_eq!(source.slice(impt.s, impt.e), "\"foo\" + /* comment */ \"bar\"");
}

#[test]
fn dynamic_import_expression_range_5() {
    let source = "import((() => { return \"foo\" })() /* comment */)";
    let impt = &parse(source).imports[0];
    assert_eq!(
        source.slice(impt.ss, impt.se),
        "import((() => { return \"foo\" })() /* comment */)"
    );
    assert_eq!(source.slice(impt.s, impt.e), "(() => { return \"foo\" })()");
}

#[test]
fn dynamic_import_expression_range_6() {
    let source = "import(/* comment */ `asdf` /* comment */ /* comment 2 */)";
    let impt = &parse(source).imports[0];
    assert_eq!(
        source.slice(impt.ss, impt.se),
        "import(/* comment */ `asdf` /* comment */ /* comment 2 */)"
    );
    assert_eq!(source.slice(impt.s, impt.e), "`asdf`");
}

#[test]
fn simple_export_destructuring() {
    let source = "
    export const{URI,Utils,...Another}=LIB
    export var p, { z } = {};

    export var { aa, qq: { z } } = { qq: {} }, pp = {};
    ";
    let exports = parse(source).exports;
    assert_eq!(
        exports.iter().map(|e| e.n.clone()).collect::<Vec<_>>(),
        // NOTE: esm-module-lexer does not have "Another", "z", and "pp", and has an extra "qq"
        // vec!["URI", "Utils", "p", "aa", "qq"]
        vec!["URI", "Utils", "Another", "p", "z", "aa", "z", "pp"]
    );
}

#[test]
fn export_default_cases() {
    let source = "
    export default \"export default a\"
    export default \"export default 'a'\"
    export default \"export function foo() {}\"
    export default \"export function foo() {return bar}\"
    ";
    let exports = parse(source).exports;
    assert_eq!(
        exports.iter().map(|expt| expt.n.clone()).collect::<Vec<_>>(),
        vec!["default", "default", "default", "default"]
    );
}

#[test]
fn import_meta_spread() {
    let source = "console.log(...import.meta.obj);";
    let impts = parse(source).imports;
    assert_eq!(impts.len(), 1);
    assert_eq!(source.slice(impts[0].s, impts[0].e), "import.meta");
}

#[test]
fn template_string_default_bracket() {
    let source = "export default{};";
    let expts = &parse(source).exports;
    let expt = &expts[0];
    assert_eq!(source.slice(expt.s, expt.e), "default");
    assert_eq!(expt.ls, None);
    assert_eq!(expt.le, None);
    assert_eq!(expt.n, "default");
    assert_eq!(expt.ln, None);
}

#[test]
fn template_string_default() {
    let source = "const css = String.raw;
export default css`:host { solid 1px black }`;";
    let expts = &parse(source).exports;
    let expt = &expts[0];
    assert_eq!(source.slice(expt.s, expt.e), "default");
    assert_eq!(expt.ls, None);
    assert_eq!(expt.le, None);
    assert_eq!(expt.n, "default");
    assert_eq!(expt.ln, None);
}

#[test]
fn class_fn_asi() {
    parse("class a{friendlyName;import}n();");
}

#[test]
fn division_const_after_class_parse_case() {
    let source = "class a{}const Ti=a/yi;";
    parse(source);
}

#[test]
fn multiline_dynamic_import_on_windows() {
    let source = "import(\n\"./statehash\\u{1011}.js\"\r)";
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 1);
    assert_eq!(source.slice(imports[0].s, imports[0].e), "\"./statehash\\u{1011}.js\"");
}

#[test]
fn basic_nested_dynamic_import_support() {
    let source = "await import (await import ('foo'))";
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 2);
    // We can't obtain the left of `(`
    // assert_eq!(source.slice(imports[0].ss, imports[0].d.as_dynamic_import().unwrap()), "import ");
    assert_eq!(source.slice(imports[0].ss, imports[0].se), "import (await import ('foo'))");
    assert_eq!(source.slice(imports[0].s, imports[0].e), "await import ('foo')");
    // We can't obtain the left of `(`
    // assert_eq!(source.slice(imports[1].ss, imports[1].d.as_dynamic_import().unwrap()), "import ");
    assert_eq!(source.slice(imports[1].ss, imports[1].se), "import ('foo')");
    assert_eq!(source.slice(imports[1].s, imports[1].e), "'foo'");
}

#[test]
fn import_assertions() {
    let source = r#"
import json from "./foo.json" assert { type: "json" };
import("foo.json", { assert: { type: "json" } });

import test from './asdf' assert { not: 'an assertion!' }
export var p = 5;
"#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 3);
    assert_eq!(source.slice(imports[0].s, imports[0].e), "./foo.json");
    assert_eq!(source.slice(imports[0].a.unwrap(), imports[0].se), "{ type: \"json\" };");
    assert_eq!(
        source.slice(imports[1].a.unwrap(), imports[1].se),
        "{ assert: { type: \"json\" } })"
    );
    assert_eq!(source.slice(imports[1].s, imports[1].e), "\"foo.json\"");
    assert_eq!(imports[1].n.as_ref().unwrap(), "foo.json");
    assert_eq!(imports[2].n.as_ref().unwrap(), "./asdf");
    assert_eq!(imports[2].a, None);
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "p", Some("p"));
}

#[test]
fn import_attributes() {
    let source = "
import json from \"./foo.json\" with { type: \"json\" };
import(\"foo.json\", { with: { type: \"json\" } });

import test from './asdf'
with { not: 'an assertion!' }
export var p = 5;
";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 3);
    assert_eq!(source.slice(imports[0].s, imports[0].e), "./foo.json");
    assert_eq!(source.slice(imports[0].a.unwrap(), imports[0].se), "{ type: \"json\" };");
    assert_eq!(source.slice(imports[1].a.unwrap(), imports[1].se), "{ with: { type: \"json\" } })");
    assert_eq!(source.slice(imports[1].s, imports[1].e), "\"foo.json\"");
    assert_eq!(imports[1].n.clone().unwrap(), "foo.json");
    assert_eq!(imports[2].n.clone().unwrap(), "./asdf");
    assert_eq!(imports[2].a, None);
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "p", Some("p"));
}

#[test]
fn import_meta_inside_dynamic_import() {
    let source = "import(import.meta.url)";
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 2);
    assert_eq!(source.slice(imports[0].s, imports[0].e), "import.meta.url");
}

#[test]
fn export() {
    let source = "export var p=5";
    let exports = parse(source).exports;
    assert_export_is(source, &exports[0], "p", Some("p"));
}

#[test]
fn string_encoding() {
    let imports = parse(
        "
        import './\\x61\\x62\\x63.js';
        import './\\u{20204}.js';
        import('./\\u{20204}.js');
        import('./\\u{20204}.js' + dyn);
        import('./\\u{20204}.js' );
        import('./\\u{20204}.js' ());
    ",
    )
    .imports;
    assert_eq!(imports.len(), 6);
    assert_eq!(imports[0].n.clone().unwrap(), "./abc.js");
    assert_eq!(imports[1].n.clone().unwrap(), "./𠈄.js");
    assert_eq!(imports[2].n.clone().unwrap(), "./𠈄.js");
    assert_eq!(imports[3].n, None);
    assert_eq!(imports[4].n.clone().unwrap(), "./𠈄.js");
    assert_eq!(imports[5].n, None);
}

#[test]
fn regexp_case() {
    parse(
        "
        class Number {

        }

        /(\"|')(?<value>(\\\\(\\1)|[^\\1])*)?(\\1)/.exec(`'\\\\\"\\\\'aa'`);

        const x = `\"${label.replace(/\"/g, \"\\\\\\\"\")}\"`;
    ",
    );
}

#[test]
fn regexp_keyword_prefixes() {
    let imports = parse(
        "
        x: while (true) {
            if (foo) break
            /import(\"a\")/.test(bar) || baz()
            if (foo) continue
            /import(\"b\")/.test(bar) || baz()
            if (foo) break x
            /import(\"c\")/.test(bar) || baz()
            if (foo) continue x
            /import(\"d\")/.test(bar) || baz()
        }
    ",
    )
    .imports;
    assert_eq!(imports.len(), 0);
}

#[test]
fn regexp_division() {
    parse("\nconst x = num / /'/.exec(l)[0].slice(1, -1)//'");
}

#[test]
fn multiline_string_escapes() {
    parse(
        "const str = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB4AAAAeCAYAAAA7MK6iAAAABmJLR0QA/wAAAAAzJ3zzAAAGTElEQV\\\n\t\tRIx+VXe1BU1xn/zjn7ugvL4sIuQnll5U0ELAQxig7WiQYz6NRHa6O206qdSXXSxs60dTK200zNY9q0dcRpMs1jkrRNWmaijCVoaU';\n",
    );
}

#[test]
fn dotted_number() {
    parse(
        "
       const x = 5. / 10;
    ",
    );
}

#[test]
fn division_operator_case() {
    parse("
        function log(r){
            if(g>=0){u[g++]=m;g>=n.logSz&&(g=0)}else{u.push(m);u.length>=n.logSz&&(g=0)}/^(DBG|TICK): /.test(r)||t.Ticker.tick(454,o.slice(0,200));
        }

        (function(n){
        })();
    ");
}

#[test]
fn single_parse_cases() {
    parse("export { x }");
    parse("'asdf'");
    parse("/asdf/");
    parse("`asdf`");
    parse("/**/\n");
    parse(" //");
}

#[test]
fn simple_export_with_unicode_conversions() {
    let source = "export var p𓀀s,q";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 0);
    assert_eq!(exports.len(), 2);
    assert_export_is(source, &exports[0], "p𓀀s", Some("p𓀀s"));
    assert_export_is(source, &exports[1], "q", Some("q"));
}

#[test]
fn simple_import() {
    let source = "
      import test from \"test\";
      console.log(test);
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.d, ImportType::StaticImport);
    assert_eq!(import.n.clone().unwrap(), "test");
    assert_eq!(source.slice(import.ss, import.se), "import test from \"test\";");
    assert_eq!(exports.len(), 0);
}

#[test]
fn empty_single_quote_string_import() {
    let source = "import ''";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.d, ImportType::StaticImport);
    assert_eq!(source.slice(import.s, import.e), "");
    assert_eq!(source.slice(import.ss, import.se), "import ''");
    assert_eq!(exports.len(), 0);
}

#[test]
fn empty_double_quote_string_import() {
    let source = "import \"\"";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.d, ImportType::StaticImport);
    assert_eq!(source.slice(import.s, import.e), "");
    assert_eq!(source.slice(import.ss, import.se), "import \"\"");
    assert_eq!(exports.len(), 0);
}

#[test]
fn import_export_with_comments() {
    let source = "

import /* 'x' */
 'a';

import /* 'x' */
 'b';

export var z /*  */
      export {
        a,
// b,
/* c */
 d
      };
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 2);
    assert_eq!(source.slice(imports[0].s, imports[0].e), "a");
    assert_eq!(source.slice(imports[0].ss, imports[0].se), "import /* 'x' */\n 'a';");
    assert_eq!(source.slice(imports[1].s, imports[1].e), "b");
    assert_eq!(source.slice(imports[1].ss, imports[1].se), "import /* 'x' */\n 'b';");
    assert_eq!(exports.len(), 3);
    assert_export_is(source, &exports[0], "z", Some("z"));
    assert_export_is(source, &exports[1], "a", Some("a"));
    assert_export_is(source, &exports[2], "d", Some("d"));
}

#[test]
fn exported_function_and_class() {
    let source = "
      export function a𓀀 () {

      }
      export class Q{

      }
    ";
    let exports = parse(source).exports;
    assert_eq!(exports.len(), 2);
    assert_export_is(source, &exports[0], "a𓀀", Some("a𓀀"));
    assert_export_is(source, &exports[1], "Q", Some("Q"));
}

#[test]
fn export_destructuring() {
    let source = "
      export const { a, b } = foo;

      export { ok };
    ";
    let exports = parse(source).exports;
    assert_eq!(exports.len(), 3);
    assert_export_is(source, &exports[0], "a", Some("a"));
}

#[test]
fn minified_import_syntax() {
    let source = r#"import{TemplateResult as t}from"lit-html";import{a as e}from"./chunk-4be41b30.js";export{j as SVGTemplateResult,i as TemplateResult,g as html,h as svg}from"./chunk-4be41b30.js";window.JSCompiler_renameProperty='asdf';"#;
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 3);
    assert_eq!(imports[0].s, 32);
    assert_eq!(imports[0].e, 40);
    assert_eq!(imports[0].ss, 0);
    assert_eq!(imports[0].se, 42);
    assert_eq!(imports[1].s, 61);
    assert_eq!(imports[1].e, 80);
    assert_eq!(imports[1].ss, 42);
    assert_eq!(imports[1].se, 82);
    assert_eq!(imports[2].s, 156);
    assert_eq!(imports[2].e, 175);
    assert_eq!(imports[2].ss, 82);
    assert_eq!(imports[2].se, 177);
}

#[test]
fn more_minified_imports() {
    let source = r#"import"some/import.js";"#;
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].s, 7);
    assert_eq!(imports[0].e, 21);
    assert_eq!(imports[0].ss, 0);
    assert_eq!(imports[0].se, 23);
}

#[test]
fn plus_plus_division() {
    parse(
        "\
tick++/fetti;f=(1)+\")\";
",
    );
}

#[test]
fn return_bracket_division() {
    let source = "function variance(){return s/(a-1)}";
    parse(source);
}

#[test]
fn simple_reexport() {
    let source = r#"export { hello as default } from "test-dep";"#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.d, ImportType::StaticImport);
    assert_eq!(source.slice(import.s, import.e), "test-dep");
    assert_eq!(
        source.slice(import.ss, import.se),
        "export { hello as default } from \"test-dep\";"
    );
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "default", None);
}

#[test]
fn import_meta() {
    let source = r"
      export var hello = 'world';
      console.log(import.meta.url);
    ";
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.d, ImportType::ImportMeta);
    assert_eq!(import.ss, 53);
    assert_eq!(import.se, 64);
    assert_eq!(source.slice(import.s, import.e), "import.meta");
}

#[test]
fn import_meta_edge_cases() {
    let source = r"
      // Import meta
      import.
       meta
      // Not import meta
      a.
      import.
        meta
    ";
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.d, ImportType::ImportMeta);
    assert_eq!(import.ss, 28);
    assert_eq!(import.se, 47);
    assert_eq!(source.slice(import.s, import.e), "import.\n       meta");
}

#[test]
fn dynamic_import_method() {
    let source = r"
    class A {
        import() {
        }
    }
    ";
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 0);
}

#[test]
fn dynamic_import_edge_cases() {
    let source = r"
      ({
        // not a dynamic import!
        import(not1) {}
      });
      {
        // is a dynamic import!
        import(is1);
      }
      a.
      // not a dynamic import!
      import(not2);
      a.
      b()
      // is a dynamic import!
      import(is2);

      const myObject = {
        import: ()=> import(some_url)
      }
    ";
    let imports = parse(source).imports;
    assert_eq!(imports.len(), 3);
    let imp = &imports[0];
    assert_eq!(imp.ss + 6, imp.d.as_dynamic_import().unwrap());
    assert_eq!(imp.se, imp.e + 1);
    assert_eq!(source.slice(imp.d.as_dynamic_import().unwrap(), imp.se), "(is1)");
    assert_eq!(source.slice(imp.s, imp.e), "is1");

    let imp = &imports[1];
    assert_eq!(imp.ss + 6, imp.d.as_dynamic_import().unwrap());
    assert_eq!(imp.se, imp.e + 1);
    assert_eq!(source.slice(imp.s, imp.e), "is2");

    let imp = &imports[2];
    assert_eq!(imp.ss + 6, imp.d.as_dynamic_import().unwrap());
    assert_eq!(imp.se, imp.e + 1);
    assert_eq!(source.slice(imp.s, imp.e), "some_url");
}

#[test]
fn import_after_code() {
    let source = "\
export function f () {
g();
}

import { g } from './test-circular2.js';
";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 1);
    let import = &imports[0];
    assert_eq!(import.d, ImportType::StaticImport);
    assert_eq!(source.slice(import.s, import.e), "./test-circular2.js");
    assert_eq!(source.slice(import.ss, import.se), "import { g } from './test-circular2.js';");
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "f", Some("f"));
}

#[test]
fn comments() {
    let source = " /*\n                   VERSION\n                 */\nimport util from 'util';\n\n//\nfunction x() {\n}\n\n/**/\n// '\n/* / */\n/*\n\n   * export { b }\n\\*/\nexport { a }\n\n      function d() {\n/***/\n      }\n    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 1);
    assert_eq!(source.slice(imports[0].s, imports[0].e), "util");
    assert_eq!(source.slice(imports[0].ss, imports[0].se), "import util from 'util';");
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "a", Some("a"));
}

#[test]
fn strings() {
    let source = r#"
      "";
      `
        ${
          import(`test/${ import(b)}`)
        }
      `
      export { a }
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 2);
    assert_ne!(imports[0].d, ImportType::StaticImport);
    assert_eq!(imports[0].ss + 6, imports[0].d.as_dynamic_import().unwrap());
    assert_eq!(imports[0].se, imports[0].e + 1);
    assert_eq!(source.slice(imports[0].ss, imports[0].s), "import(");
    assert_ne!(imports[1].d, ImportType::StaticImport);
    assert_eq!(imports[1].ss + 6, imports[1].d.as_dynamic_import().unwrap());
    assert_eq!(imports[1].se, imports[1].e + 1);
    assert_eq!(source.slice(imports[1].ss, imports[1].d.as_dynamic_import().unwrap()), "import");
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "a", Some("a"));
}

#[test]
fn bracket_matching() {
    parse(
        "
      instance.extend('parseExprAtom', function (nextMethod) {
        return function () {
          function parseExprAtom(refDestructuringErrors) {
            if (this.type === tt._import) {
              return parseDynamicImport.call(this);
            }
            return c(refDestructuringErrors);
          }
        }();
      });
      export { a }
    ",
    );
}

#[test]
fn division_regex_ambiguity() {
    let source = r"
      /asdf/; x();
      a / 2; '  /  '
      while (true)
        /test'/
      x-/a'/g
      try {}
      finally{}/a'/g
      (x);{f()}/d'export { b }/g
      ;{}/e'/g;
      {}/f'/g
      a / 'b' / c;
      /a'/ - /b'/;
      +{} /g -'/g'
      ('a')/h -'/g'
      if //x
      ('a')/i'/g;
      /asdf/ / /as'df/; // '
      p = `\${/test/ + 5}`;
      /regex/ / x;
      function m() {
        return /*asdf8*// 5/;
      }
      export { a };
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 0);
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "a", Some("a"));
}

#[test]
fn template_string_expression_ambiguity() {
    let source = r"
        `$`
        import 'a';
        ``
        export { b };
        `a$b`
        import(`$`);
        `{$}`
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 2);
    assert_eq!(exports.len(), 1);
    assert_export_is(source, &exports[0], "b", Some("b"));
}

#[test]
fn many_exports() {
    let exports = parse(
        r"
      export { _iconsCache as fas, prefix, faAbacus, faAcorn, faAd, faAddressBook, faAddressCard, faAdjust, faAirFreshener, faAlarmClock, faAlarmExclamation, faAlarmPlus, faAlarmSnooze, faAlicorn, faAlignCenter, faAlignJustify, faAlignLeft, faAlignRight, faAlignSlash, faAllergies, faAmbulance, faAmericanSignLanguageInterpreting, faAnalytics, faAnchor, faAngel, faAngleDoubleDown, faAngleDoubleLeft, faAngleDoubleRight, faAngleDoubleUp, faAngleDown, faAngleLeft, faAngleRight, faAngleUp, faAngry, faAnkh, faAppleAlt, faAppleCrate, faArchive, faArchway, faArrowAltCircleDown, faArrowAltCircleLeft, faArrowAltCircleRight, faArrowAltCircleUp, faArrowAltDown, faArrowAltFromBottom, faArrowAltFromLeft, faArrowAltFromRight, faArrowAltFromTop, faArrowAltLeft, faArrowAltRight, faArrowAltSquareDown, faArrowAltSquareLeft, faArrowAltSquareRight, faArrowAltSquareUp, faArrowAltToBottom, faArrowAltToLeft, faArrowAltToRight, faArrowAltToTop, faArrowAltUp, faArrowCircleDown, faArrowCircleLeft, faArrowCircleRight, faArrowCircleUp, faArrowDown, faArrowFromBottom, faArrowFromLeft, faArrowFromRight, faArrowFromTop, faArrowLeft, faArrowRight, faArrowSquareDown, faArrowSquareLeft, faArrowSquareRight, faArrowSquareUp, faArrowToBottom, faArrowToLeft, faArrowToRight, faArrowToTop, faArrowUp, faArrows, faArrowsAlt, faArrowsAltH, faArrowsAltV, faArrowsH, faArrowsV, faAssistiveListeningSystems, faAsterisk, faAt, faAtlas, faAtom, faAtomAlt, faAudioDescription, faAward, faAxe, faAxeBattle, faBaby, faBabyCarriage, faBackpack, faBackspace, faBackward, faBacon, faBadge, faBadgeCheck, faBadgeDollar, faBadgePercent, faBadgerHoney, faBagsShopping, faBalanceScale, faBalanceScaleLeft, faBalanceScaleRight, faBallPile, faBallot, faBallotCheck, faBan, faBandAid, faBarcode, faBarcodeAlt, faBarcodeRead, faBarcodeScan, faBars, faBaseball, faBaseballBall, faBasketballBall, faBasketballHoop, faBat, faBath, faBatteryBolt, faBatteryEmpty, faBatteryFull, faBatteryHalf, faBatteryQuarter, faBatterySlash, faBatteryThreeQuarters, faBed, faBeer, faBell, faBellExclamation, faBellPlus, faBellSchool, faBellSchoolSlash, faBellSlash, faBells, faBezierCurve, faBible, faBicycle, faBiking, faBikingMountain, faBinoculars, faBiohazard, faBirthdayCake, faBlanket, faBlender, faBlenderPhone, faBlind, faBlog, faBold, faBolt, faBomb, faBone, faBoneBreak, faBong, faBook, faBookAlt, faBookDead, faBookHeart, faBookMedical, faBookOpen, faBookReader, faBookSpells, faBookUser, faBookmark, faBooks, faBooksMedical, faBoot, faBoothCurtain, faBorderAll, faBorderBottom, faBorderCenterH, faBorderCenterV, faBorderInner, faBorderLeft, faBorderNone, faBorderOuter, faBorderRight, faBorderStyle, faBorderStyleAlt, faBorderTop, faBowArrow, faBowlingBall, faBowlingPins, faBox, faBoxAlt, faBoxBallot, faBoxCheck, faBoxFragile, faBoxFull, faBoxHeart, faBoxOpen, faBoxUp, faBoxUsd, faBoxes, faBoxesAlt, faBoxingGlove, faBrackets, faBracketsCurly, faBraille, faBrain, faBreadLoaf, faBreadSlice, faBriefcase, faBriefcaseMedical, faBringForward, faBringFront, faBroadcastTower, faBroom, faBrowser, faBrush, faBug, faBuilding, faBullhorn, faBullseye, faBullseyeArrow, faBullseyePointer, faBurgerSoda, faBurn, faBurrito, faBus, faBusAlt, faBusSchool, faBusinessTime, faCabinetFiling, faCalculator, faCalculatorAlt, faCalendar, faCalendarAlt, faCalendarCheck, faCalendarDay, faCalendarEdit, faCalendarExclamation, faCalendarMinus, faCalendarPlus, faCalendarStar, faCalendarTimes, faCalendarWeek, faCamera, faCameraAlt, faCameraRetro, faCampfire, faCampground, faCandleHolder, faCandyCane, faCandyCorn, faCannabis, faCapsules, faCar, faCarAlt, faCarBattery, faCarBuilding, faCarBump, faCarBus, faCarCrash, faCarGarage, faCarMechanic, faCarSide, faCarTilt, faCarWash, faCaretCircleDown, faCaretCircleLeft, faCaretCircleRight, faCaretCircleUp, faCaretDown, faCaretLeft, faCaretRight, faCaretSquareDown, faCaretSquareLeft, faCaretSquareRight, faCaretSquareUp, faCaretUp, faCarrot, faCars, faCartArrowDown, faCartPlus, faCashRegister, faCat, faCauldron, faCertificate, faChair, faChairOffice, faChalkboard, faChalkboardTeacher, faChargingStation, faChartArea, faChartBar, faChartLine, faChartLineDown, faChartNetwork, faChartPie, faChartPieAlt, faChartScatter, faCheck, faCheckCircle, faCheckDouble, faCheckSquare, faCheese, faCheeseSwiss, faCheeseburger, faChess, faChessBishop, faChessBishopAlt, faChessBoard, faChessClock, faChessClockAlt, faChessKing, faChessKingAlt, faChessKnight, faChessKnightAlt, faChessPawn, faChessPawnAlt, faChessQueen, faChessQueenAlt, faChessRook, faChessRookAlt, faChevronCircleDown, faChevronCircleLeft, faChevronCircleRight, faChevronCircleUp, faChevronDoubleDown, faChevronDoubleLeft, faChevronDoubleRight, faChevronDoubleUp, faChevronDown, faChevronLeft, faChevronRight, faChevronSquareDown, faChevronSquareLeft, faChevronSquareRight, faChevronSquareUp, faChevronUp, faChild, faChimney, faChurch, faCircle, faCircleNotch, faCity, faClawMarks, faClinicMedical, faClipboard, faClipboardCheck, faClipboardList, faClipboardListCheck, faClipboardPrescription, faClipboardUser, faClock, faClone, faClosedCaptioning, faCloud, faCloudDownload, faCloudDownloadAlt, faCloudDrizzle, faCloudHail, faCloudHailMixed, faCloudMeatball, faCloudMoon, faCloudMoonRain, faCloudRain, faCloudRainbow, faCloudShowers, faCloudShowersHeavy, faCloudSleet, faCloudSnow, faCloudSun, faCloudSunRain, faCloudUpload, faCloudUploadAlt, faClouds, faCloudsMoon, faCloudsSun, faClub, faCocktail, faCode, faCodeBranch, faCodeCommit, faCodeMerge, faCoffee, faCoffeeTogo, faCoffin, faCog, faCogs, faCoin, faCoins, faColumns, faComment, faCommentAlt, faCommentAltCheck, faCommentAltDollar, faCommentAltDots, faCommentAltEdit, faCommentAltExclamation, faCommentAltLines, faCommentAltMedical, faCommentAltMinus, faCommentAltPlus, faCommentAltSlash, faCommentAltSmile, faCommentAltTimes, faCommentCheck, faCommentDollar, faCommentDots, faCommentEdit, faCommentExclamation, faCommentLines, faCommentMedical, faCommentMinus, faCommentPlus, faCommentSlash, faCommentSmile, faCommentTimes, faComments, faCommentsAlt, faCommentsAltDollar, faCommentsDollar, faCompactDisc, faCompass, faCompassSlash, faCompress, faCompressAlt, faCompressArrowsAlt, faCompressWide, faConciergeBell, faConstruction, faContainerStorage, faConveyorBelt, faConveyorBeltAlt, faCookie, faCookieBite, faCopy, faCopyright, faCorn, faCouch, faCow, faCreditCard, faCreditCardBlank, faCreditCardFront, faCricket, faCroissant, faCrop, faCropAlt, faCross, faCrosshairs, faCrow, faCrown, faCrutch, faCrutches, faCube, faCubes, faCurling, faCut, faDagger, faDatabase, faDeaf, faDebug, faDeer, faDeerRudolph, faDemocrat, faDesktop, faDesktopAlt, faDewpoint, faDharmachakra, faDiagnoses, faDiamond, faDice, faDiceD10, faDiceD12, faDiceD20, faDiceD4, faDiceD6, faDiceD8, faDiceFive, faDiceFour, faDiceOne, faDiceSix, faDiceThree, faDiceTwo, faDigging, faDigitalTachograph, faDiploma, faDirections, faDisease, faDivide, faDizzy, faDna, faDoNotEnter, faDog, faDogLeashed, faDollarSign, faDolly, faDollyEmpty, faDollyFlatbed, faDollyFlatbedAlt, faDollyFlatbedEmpty, faDonate, faDoorClosed, faDoorOpen, faDotCircle, faDove, faDownload, faDraftingCompass, faDragon, faDrawCircle, faDrawPolygon, faDrawSquare, faDreidel, faDrone, faDroneAlt, faDrum, faDrumSteelpan, faDrumstick, faDrumstickBite, faDryer, faDryerAlt, faDuck, faDumbbell, faDumpster, faDumpsterFire, faDungeon, faEar, faEarMuffs, faEclipse, faEclipseAlt, faEdit, faEgg, faEggFried, faEject, faElephant, faEllipsisH, faEllipsisHAlt, faEllipsisV, faEllipsisVAlt, faEmptySet, faEngineWarning, faEnvelope, faEnvelopeOpen, faEnvelopeOpenDollar, faEnvelopeOpenText, faEnvelopeSquare, faEquals, faEraser, faEthernet, faEuroSign, faExchange, faExchangeAlt, faExclamation, faExclamationCircle, faExclamationSquare, faExclamationTriangle, faExpand, faExpandAlt, faExpandArrows, faExpandArrowsAlt, faExpandWide, faExternalLink, faExternalLinkAlt, faExternalLinkSquare, faExternalLinkSquareAlt, faEye, faEyeDropper, faEyeEvil, faEyeSlash, faFan, faFarm, faFastBackward, faFastForward, faFax, faFeather, faFeatherAlt, faFemale, faFieldHockey, faFighterJet, faFile, faFileAlt, faFileArchive, faFileAudio, faFileCertificate, faFileChartLine, faFileChartPie, faFileCheck, faFileCode, faFileContract, faFileCsv, faFileDownload, faFileEdit, faFileExcel, faFileExclamation, faFileExport, faFileImage, faFileImport, faFileInvoice, faFileInvoiceDollar, faFileMedical, faFileMedicalAlt, faFileMinus, faFilePdf, faFilePlus, faFilePowerpoint, faFilePrescription, faFileSearch, faFileSignature, faFileSpreadsheet, faFileTimes, faFileUpload, faFileUser, faFileVideo, faFileWord, faFilesMedical, faFill, faFillDrip, faFilm, faFilmAlt, faFilter, faFingerprint, faFire, faFireAlt, faFireExtinguisher, faFireSmoke, faFireplace, faFirstAid, faFish, faFishCooked, faFistRaised, faFlag, faFlagAlt, faFlagCheckered, faFlagUsa, faFlame, faFlask, faFlaskPoison, faFlaskPotion, faFlower, faFlowerDaffodil, faFlowerTulip, faFlushed, faFog, faFolder, faFolderMinus, faFolderOpen, faFolderPlus, faFolderTimes, faFolderTree, faFolders, faFont, faFontAwesomeLogoFull, faFontCase, faFootballBall, faFootballHelmet, faForklift, faForward, faFragile, faFrenchFries, faFrog, faFrostyHead, faFrown, faFrownOpen, faFunction, faFunnelDollar, faFutbol, faGameBoard, faGameBoardAlt, faGamepad, faGasPump, faGasPumpSlash, faGavel, faGem, faGenderless, faGhost, faGift, faGiftCard, faGifts, faGingerbreadMan, faGlass, faGlassChampagne, faGlassCheers, faGlassCitrus, faGlassMartini, faGlassMartiniAlt, faGlassWhiskey, faGlassWhiskeyRocks, faGlasses, faGlassesAlt, faGlobe, faGlobeAfrica, faGlobeAmericas, faGlobeAsia, faGlobeEurope, faGlobeSnow, faGlobeStand, faGolfBall, faGolfClub, faGopuram, faGraduationCap, faGreaterThan, faGreaterThanEqual, faGrimace, faGrin, faGrinAlt, faGrinBeam, faGrinBeamSweat, faGrinHearts, faGrinSquint, faGrinSquintTears, faGrinStars, faGrinTears, faGrinTongue, faGrinTongueSquint, faGrinTongueWink, faGrinWink, faGripHorizontal, faGripLines, faGripLinesVertical, faGripVertical, faGuitar, faHSquare, faH1, faH2, faH3, faH4, faHamburger, faHammer, faHammerWar, faHamsa, faHandHeart, faHandHolding, faHandHoldingBox, faHandHoldingHeart, faHandHoldingMagic, faHandHoldingSeedling, faHandHoldingUsd, faHandHoldingWater, faHandLizard, faHandMiddleFinger, faHandPaper, faHandPeace, faHandPointDown, faHandPointLeft, faHandPointRight, faHandPointUp, faHandPointer, faHandReceiving, faHandRock, faHandScissors, faHandSpock, faHands, faHandsHeart, faHandsHelping, faHandsUsd, faHandshake, faHandshakeAlt, faHanukiah, faHardHat, faHashtag, faHatChef, faHatSanta, faHatWinter, faHatWitch, faHatWizard, faHaykal, faHdd, faHeadSide, faHeadSideBrain, faHeadSideMedical, faHeadVr, faHeading, faHeadphones, faHeadphonesAlt, faHeadset, faHeart, faHeartBroken, faHeartCircle, faHeartRate, faHeartSquare, faHeartbeat, faHelicopter, faHelmetBattle, faHexagon, faHighlighter, faHiking, faHippo, faHistory, faHockeyMask, faHockeyPuck, faHockeySticks, faHollyBerry, faHome, faHomeAlt, faHomeHeart, faHomeLg, faHomeLgAlt, faHoodCloak, faHorizontalRule, faHorse, faHorseHead, faHospital, faHospitalAlt, faHospitalSymbol, faHospitalUser, faHospitals, faHotTub, faHotdog, faHotel, faHourglass, faHourglassEnd, faHourglassHalf, faHourglassStart, faHouseDamage, faHouseFlood, faHryvnia, faHumidity, faHurricane, faICursor, faIceCream, faIceSkate, faIcicles, faIcons, faIconsAlt, faIdBadge, faIdCard, faIdCardAlt, faIgloo, faImage, faImages, faInbox, faInboxIn, faInboxOut, faIndent, faIndustry, faIndustryAlt, faInfinity, faInfo, faInfoCircle, faInfoSquare, faInhaler, faIntegral, faIntersection, faInventory, faIslandTropical, faItalic, faJackOLantern, faJedi, faJoint, faJournalWhills, faKaaba, faKerning, faKey, faKeySkeleton, faKeyboard, faKeynote, faKhanda, faKidneys, faKiss, faKissBeam, faKissWinkHeart, faKite, faKiwiBird, faKnifeKitchen, faLambda, faLamp, faLandmark, faLandmarkAlt, faLanguage, faLaptop, faLaptopCode, faLaptopMedical, faLaugh, faLaughBeam, faLaughSquint, faLaughWink, faLayerGroup, faLayerMinus, faLayerPlus, faLeaf, faLeafHeart, faLeafMaple, faLeafOak, faLemon, faLessThan, faLessThanEqual, faLevelDown, faLevelDownAlt, faLevelUp, faLevelUpAlt, faLifeRing, faLightbulb, faLightbulbDollar, faLightbulbExclamation, faLightbulbOn, faLightbulbSlash, faLightsHoliday, faLineColumns, faLineHeight, faLink, faLips, faLiraSign, faList, faListAlt, faListOl, faListUl, faLocation, faLocationArrow, faLocationCircle, faLocationSlash, faLock, faLockAlt, faLockOpen, faLockOpenAlt, faLongArrowAltDown, faLongArrowAltLeft, faLongArrowAltRight, faLongArrowAltUp, faLongArrowDown, faLongArrowLeft, faLongArrowRight, faLongArrowUp, faLoveseat, faLowVision, faLuchador, faLuggageCart, faLungs, faMace, faMagic, faMagnet, faMailBulk, faMailbox, faMale, faMandolin, faMap, faMapMarked, faMapMarkedAlt, faMapMarker, faMapMarkerAlt, faMapMarkerAltSlash, faMapMarkerCheck, faMapMarkerEdit, faMapMarkerExclamation, faMapMarkerMinus, faMapMarkerPlus, faMapMarkerQuestion, faMapMarkerSlash, faMapMarkerSmile, faMapMarkerTimes, faMapPin, faMapSigns, faMarker, faMars, faMarsDouble, faMarsStroke, faMarsStrokeH, faMarsStrokeV, faMask, faMeat, faMedal, faMedkit, faMegaphone, faMeh, faMehBlank, faMehRollingEyes, faMemory, faMenorah, faMercury, faMeteor, faMicrochip, faMicrophone, faMicrophoneAlt, faMicrophoneAltSlash, faMicrophoneSlash, faMicroscope, faMindShare, faMinus, faMinusCircle, faMinusHexagon, faMinusOctagon, faMinusSquare, faMistletoe, faMitten, faMobile, faMobileAlt, faMobileAndroid, faMobileAndroidAlt, faMoneyBill, faMoneyBillAlt, faMoneyBillWave, faMoneyBillWaveAlt, faMoneyCheck, faMoneyCheckAlt, faMoneyCheckEdit, faMoneyCheckEditAlt, faMonitorHeartRate, faMonkey, faMonument, faMoon, faMoonCloud, faMoonStars, faMortarPestle, faMosque, faMotorcycle, faMountain, faMountains, faMousePointer, faMug, faMugHot, faMugMarshmallows, faMugTea, faMusic, faNarwhal, faNetworkWired, faNeuter, faNewspaper, faNotEqual, faNotesMedical, faObjectGroup, faObjectUngroup, faOctagon, faOilCan, faOilTemp, faOm, faOmega, faOrnament, faOtter, faOutdent, faOverline, faPageBreak, faPager, faPaintBrush, faPaintBrushAlt, faPaintRoller, faPalette, faPallet, faPalletAlt, faPaperPlane, faPaperclip, faParachuteBox, faParagraph, faParagraphRtl, faParking, faParkingCircle, faParkingCircleSlash, faParkingSlash, faPassport, faPastafarianism, faPaste, faPause, faPauseCircle, faPaw, faPawAlt, faPawClaws, faPeace, faPegasus, faPen, faPenAlt, faPenFancy, faPenNib, faPenSquare, faPencil, faPencilAlt, faPencilPaintbrush, faPencilRuler, faPennant, faPeopleCarry, faPepperHot, faPercent, faPercentage, faPersonBooth, faPersonCarry, faPersonDolly, faPersonDollyEmpty, faPersonSign, faPhone, faPhoneAlt, faPhoneLaptop, faPhoneOffice, faPhonePlus, faPhoneSlash, faPhoneSquare, faPhoneSquareAlt, faPhoneVolume, faPhotoVideo, faPi, faPie, faPig, faPiggyBank, faPills, faPizza, faPizzaSlice, faPlaceOfWorship, faPlane, faPlaneAlt, faPlaneArrival, faPlaneDeparture, faPlay, faPlayCircle, faPlug, faPlus, faPlusCircle, faPlusHexagon, faPlusOctagon, faPlusSquare, faPodcast, faPodium, faPodiumStar, faPoll, faPollH, faPollPeople, faPoo, faPooStorm, faPoop, faPopcorn, faPortrait, faPoundSign, faPowerOff, faPray, faPrayingHands, faPrescription, faPrescriptionBottle, faPrescriptionBottleAlt, faPresentation, faPrint, faPrintSearch, faPrintSlash, faProcedures, faProjectDiagram, faPumpkin, faPuzzlePiece, faQrcode, faQuestion, faQuestionCircle, faQuestionSquare, faQuidditch, faQuoteLeft, faQuoteRight, faQuran, faRabbit, faRabbitFast, faRacquet, faRadiation, faRadiationAlt, faRainbow, faRaindrops, faRam, faRampLoading, faRandom, faReceipt, faRectangleLandscape, faRectanglePortrait, faRectangleWide, faRecycle, faRedo, faRedoAlt, faRegistered, faRemoveFormat, faRepeat, faRepeat1, faRepeat1Alt, faRepeatAlt, faReply, faReplyAll, faRepublican, faRestroom, faRetweet, faRetweetAlt, faRibbon, faRing, faRingsWedding, faRoad, faRobot, faRocket, faRoute, faRouteHighway, faRouteInterstate, faRss, faRssSquare, faRubleSign, faRuler, faRulerCombined, faRulerHorizontal, faRulerTriangle, faRulerVertical, faRunning, faRupeeSign, faRv, faSack, faSackDollar, faSadCry, faSadTear, faSalad, faSandwich, faSatellite, faSatelliteDish, faSausage, faSave, faScalpel, faScalpelPath, faScanner, faScannerKeyboard, faScannerTouchscreen, faScarecrow, faScarf, faSchool, faScrewdriver, faScroll, faScrollOld, faScrubber, faScythe, faSdCard, faSearch, faSearchDollar, faSearchLocation, faSearchMinus, faSearchPlus, faSeedling, faSendBack, faSendBackward, faServer, faShapes, faShare, faShareAll, faShareAlt, faShareAltSquare, faShareSquare, faSheep, faShekelSign, faShield, faShieldAlt, faShieldCheck, faShieldCross, faShip, faShippingFast, faShippingTimed, faShishKebab, faShoePrints, faShoppingBag, faShoppingBasket, faShoppingCart, faShovel, faShovelSnow, faShower, faShredder, faShuttleVan, faShuttlecock, faSickle, faSigma, faSign, faSignIn, faSignInAlt, faSignLanguage, faSignOut, faSignOutAlt, faSignal, faSignal1, faSignal2, faSignal3, faSignal4, faSignalAlt, faSignalAlt1, faSignalAlt2, faSignalAlt3, faSignalAltSlash, faSignalSlash, faSignature, faSimCard, faSitemap, faSkating, faSkeleton, faSkiJump, faSkiLift, faSkiing, faSkiingNordic, faSkull, faSkullCrossbones, faSlash, faSledding, faSleigh, faSlidersH, faSlidersHSquare, faSlidersV, faSlidersVSquare, faSmile, faSmileBeam, faSmilePlus, faSmileWink, faSmog, faSmoke, faSmoking, faSmokingBan, faSms, faSnake, faSnooze, faSnowBlowing, faSnowboarding, faSnowflake, faSnowflakes, faSnowman, faSnowmobile, faSnowplow, faSocks, faSolarPanel, faSort, faSortAlphaDown, faSortAlphaDownAlt, faSortAlphaUp, faSortAlphaUpAlt, faSortAlt, faSortAmountDown, faSortAmountDownAlt, faSortAmountUp, faSortAmountUpAlt, faSortDown, faSortNumericDown, faSortNumericDownAlt, faSortNumericUp, faSortNumericUpAlt, faSortShapesDown, faSortShapesDownAlt, faSortShapesUp, faSortShapesUpAlt, faSortSizeDown, faSortSizeDownAlt, faSortSizeUp, faSortSizeUpAlt, faSortUp, faSoup, faSpa, faSpaceShuttle, faSpade, faSparkles, faSpellCheck, faSpider, faSpiderBlackWidow, faSpiderWeb, faSpinner, faSpinnerThird, faSplotch, faSprayCan, faSquare, faSquareFull, faSquareRoot, faSquareRootAlt, faSquirrel, faStaff, faStamp, faStar, faStarAndCrescent, faStarChristmas, faStarExclamation, faStarHalf, faStarHalfAlt, faStarOfDavid, faStarOfLife, faStars, faSteak, faSteeringWheel, faStepBackward, faStepForward, faStethoscope, faStickyNote, faStocking, faStomach, faStop, faStopCircle, faStopwatch, faStore, faStoreAlt, faStream, faStreetView, faStretcher, faStrikethrough, faStroopwafel, faSubscript, faSubway, faSuitcase, faSuitcaseRolling, faSun, faSunCloud, faSunDust, faSunHaze, faSunglasses, faSunrise, faSunset, faSuperscript, faSurprise, faSwatchbook, faSwimmer, faSwimmingPool, faSword, faSwords, faSynagogue, faSync, faSyncAlt, faSyringe, faTable, faTableTennis, faTablet, faTabletAlt, faTabletAndroid, faTabletAndroidAlt, faTabletRugged, faTablets, faTachometer, faTachometerAlt, faTachometerAltAverage, faTachometerAltFast, faTachometerAltFastest, faTachometerAltSlow, faTachometerAltSlowest, faTachometerAverage, faTachometerFast, faTachometerFastest, faTachometerSlow, faTachometerSlowest, faTaco, faTag, faTags, faTally, faTanakh, faTape, faTasks, faTasksAlt, faTaxi, faTeeth, faTeethOpen, faTemperatureFrigid, faTemperatureHigh, faTemperatureHot, faTemperatureLow, faTenge, faTennisBall, faTerminal, faText, faTextHeight, faTextSize, faTextWidth, faTh, faThLarge, faThList, faTheaterMasks, faThermometer, faThermometerEmpty, faThermometerFull, faThermometerHalf, faThermometerQuarter, faThermometerThreeQuarters, faTheta, faThumbsDown, faThumbsUp, faThumbtack, faThunderstorm, faThunderstormMoon, faThunderstormSun, faTicket, faTicketAlt, faTilde, faTimes, faTimesCircle, faTimesHexagon, faTimesOctagon, faTimesSquare, faTint, faTintSlash, faTire, faTireFlat, faTirePressureWarning, faTireRugged, faTired, faToggleOff, faToggleOn, faToilet, faToiletPaper, faToiletPaperAlt, faTombstone, faTombstoneAlt, faToolbox, faTools, faTooth, faToothbrush, faTorah, faToriiGate, faTornado, faTractor, faTrademark, faTrafficCone, faTrafficLight, faTrafficLightGo, faTrafficLightSlow, faTrafficLightStop, faTrain, faTram, faTransgender, faTransgenderAlt, faTrash, faTrashAlt, faTrashRestore, faTrashRestoreAlt, faTrashUndo, faTrashUndoAlt, faTreasureChest, faTree, faTreeAlt, faTreeChristmas, faTreeDecorated, faTreeLarge, faTreePalm, faTrees, faTriangle, faTrophy, faTrophyAlt, faTruck, faTruckContainer, faTruckCouch, faTruckLoading, faTruckMonster, faTruckMoving, faTruckPickup, faTruckPlow, faTruckRamp, faTshirt, faTty, faTurkey, faTurtle, faTv, faTvRetro, faUmbrella, faUmbrellaBeach, faUnderline, faUndo, faUndoAlt, faUnicorn, faUnion, faUniversalAccess, faUniversity, faUnlink, faUnlock, faUnlockAlt, faUpload, faUsdCircle, faUsdSquare, faUser, faUserAlt, faUserAltSlash, faUserAstronaut, faUserChart, faUserCheck, faUserCircle, faUserClock, faUserCog, faUserCrown, faUserEdit, faUserFriends, faUserGraduate, faUserHardHat, faUserHeadset, faUserInjured, faUserLock, faUserMd, faUserMdChat, faUserMinus, faUserNinja, faUserNurse, faUserPlus, faUserSecret, faUserShield, faUserSlash, faUserTag, faUserTie, faUserTimes, faUsers, faUsersClass, faUsersCog, faUsersCrown, faUsersMedical, faUtensilFork, faUtensilKnife, faUtensilSpoon, faUtensils, faUtensilsAlt, faValueAbsolute, faVectorSquare, faVenus, faVenusDouble, faVenusMars, faVial, faVials, faVideo, faVideoPlus, faVideoSlash, faVihara, faVoicemail, faVolcano, faVolleyballBall, faVolume, faVolumeDown, faVolumeMute, faVolumeOff, faVolumeSlash, faVolumeUp, faVoteNay, faVoteYea, faVrCardboard, faWalker, faWalking, faWallet, faWand, faWandMagic, faWarehouse, faWarehouseAlt, faWasher, faWatch, faWatchFitness, faWater, faWaterLower, faWaterRise, faWaveSine, faWaveSquare, faWaveTriangle, faWebcam, faWebcamSlash, faWeight, faWeightHanging, faWhale, faWheat, faWheelchair, faWhistle, faWifi, faWifi1, faWifi2, faWifiSlash, faWind, faWindTurbine, faWindWarning, faWindow, faWindowAlt, faWindowClose, faWindowMaximize, faWindowMinimize, faWindowRestore, faWindsock, faWineBottle, faWineGlass, faWineGlassAlt, faWonSign, faWreath, faWrench, faXRay, faYenSign, faYinYang };
    ",
    ).exports;
    assert_eq!(exports.len(), 1651);
}

#[test]
fn empty_export() {
    let source = r"
      export {};
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 0);
    assert_eq!(exports.len(), 0);
}

#[test]
fn export_star_as() {
    let source = r"
      export * as X from './asdf';
      export *  as  yy from './g';
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 2);
    assert_eq!(exports.len(), 2);
    assert_export_is(source, &exports[0], "X", None);
    assert_export_is(source, &exports[1], "yy", None);
}

/* Suite Import From */

#[test]
fn non_identifier_string_double_quote() {
    let source = r#"
        import { "~123" as foo0 } from './mod0.js';
        import { "ab cd" as foo1 } from './mod1.js';
        import { "not identifier" as foo2 } from './mod2.js';
        import { "-notidentifier" as foo3 } from './mod3.js';
        import { "%notidentifier" as foo4 } from './mod4.js';
        import { "@notidentifier" as foo5 } from './mod5.js';
        import { " notidentifier" as foo6 } from './mod6.js';
        import { "notidentifier " as foo7 } from './mod7.js';
        import { " notidentifier " as foo8 } from './mod8.js';
        import LionCombobox from './src/LionCombobox.js'; // assuming LionCombobox is imported directly
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(exports.len(), 0);
    assert_eq!(imports.len(), 10);
    assert_eq!(imports[0].n.clone().unwrap(), "./mod0.js");
    assert_eq!(imports[1].n.clone().unwrap(), "./mod1.js");
    assert_eq!(imports[2].n.clone().unwrap(), "./mod2.js");
    assert_eq!(imports[3].n.clone().unwrap(), "./mod3.js");
    assert_eq!(imports[4].n.clone().unwrap(), "./mod4.js");
    assert_eq!(imports[5].n.clone().unwrap(), "./mod5.js");
    assert_eq!(imports[6].n.clone().unwrap(), "./mod6.js");
    assert_eq!(imports[7].n.clone().unwrap(), "./mod7.js");
    assert_eq!(imports[8].n.clone().unwrap(), "./mod8.js");
}

#[test]
fn non_identifier_string_single_quote() {
    let source = r"
        import { '~123' as foo0 } from './mod0.js';
        import { 'ab cd' as foo1 } from './mod1.js';
        import { 'not identifier' as foo2 } from './mod2.js';
        import { '-notidentifier' as foo3 } from './mod3.js';
        import { '%notidentifier' as foo4 } from './mod4.js';
        import { '@notidentifier' as foo5 } from './mod5.js';
        import { ' notidentifier' as foo6 } from './mod6.js';
        import { 'notidentifier ' as foo7 } from './mod7.js';
        import { ' notidentifier ' as foo8 } from './mod8.js';
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(exports.len(), 0);
    assert_eq!(imports.len(), 9);
    assert_eq!(imports[0].n.clone().unwrap(), "./mod0.js");
    assert_eq!(imports[1].n.clone().unwrap(), "./mod1.js");
    assert_eq!(imports[2].n.clone().unwrap(), "./mod2.js");
    assert_eq!(imports[3].n.clone().unwrap(), "./mod3.js");
    assert_eq!(imports[4].n.clone().unwrap(), "./mod4.js");
    assert_eq!(imports[5].n.clone().unwrap(), "./mod5.js");
    assert_eq!(imports[6].n.clone().unwrap(), "./mod6.js");
    assert_eq!(imports[7].n.clone().unwrap(), "./mod7.js");
    assert_eq!(imports[8].n.clone().unwrap(), "./mod8.js");
}

#[test]
fn with_backslash_keywords_double_quote() {
    let source = r#"
        import { " slash\\ " as foo0 } from './mod0.js';
        import { " quote\" " as foo1 } from './mod1.js';
        import { " quote\\\" " as foo2 } from './mod2.js';
        import { " quote' " as foo3 } from './mod3.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(exports.len(), 0);
    assert_eq!(imports.len(), 4);
    assert_eq!(imports[0].n.clone().unwrap(), "./mod0.js");
    assert_eq!(imports[1].n.clone().unwrap(), "./mod1.js");
    assert_eq!(imports[2].n.clone().unwrap(), "./mod2.js");
    assert_eq!(imports[3].n.clone().unwrap(), "./mod3.js");
}

#[test]
fn with_backslash_keywords_single_quote() {
    let source = r"
        import { ' slash\\ ' as foo0 } from './mod0.js';
        import { ' quote\' ' as foo1 } from './mod1.js';
        import { ' quote\\\' ' as foo2 } from './mod2.js';
        import { ' quote\' ' as foo3 } from './mod3.js';
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(exports.len(), 0);
    assert_eq!(imports.len(), 4);
    assert_eq!(imports[0].n.clone().unwrap(), "./mod0.js");
    assert_eq!(imports[1].n.clone().unwrap(), "./mod1.js");
    assert_eq!(imports[2].n.clone().unwrap(), "./mod2.js");
    assert_eq!(imports[3].n.clone().unwrap(), "./mod3.js");
}

#[test]
fn with_emoji_as() {
    let source = r#"
        import { "hm🤔" as foo0 } from './mod0.js';
        import { " 🚀rocket space " as foo1 } from './mod1.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(exports.len(), 0);
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].n.clone().unwrap(), "./mod0.js");
    assert_eq!(imports[1].n.clone().unwrap(), "./mod1.js");
}

#[test]
fn double_quotes_and_curly_bracket() {
    // cannot be parsed
    // let source = "
    // import { asdf as \"b} from 'wrong'\" } from 'mod0';";
    let source = "
        import { asdf as x } from 'mod0';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(exports.len(), 0);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].n.clone().unwrap(), "mod0");
}

#[test]
fn single_quotes_and_curly_bracket() {
    // cannot be parsed
    // let source = "
    // import { asdf as 'b} from \"wrong\"' } from 'mod0';";
    let source = "
        import { asdf as x } from 'mod0';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(exports.len(), 0);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].n.clone().unwrap(), "mod0");
}

/* Export From */

#[test]
fn identifier_only() {
    let source = "
        export { x } from './asdf';
        export { x1, x2 } from './g';
        export { foo, x2 as bar, zoo } from './g2';
        export {
            /** @type{HTMLElement} */
            LionCombobox
        } from './src/LionCombobox.js';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 7);
    assert_export_is(source, &exports[0], "x", None);
    assert_export_is(source, &exports[1], "x1", None);
    assert_export_is(source, &exports[2], "x2", None);
    assert_export_is(source, &exports[3], "foo", None);
    assert_export_is(source, &exports[4], "bar", None);
    assert_export_is(source, &exports[5], "zoo", None);
    assert_export_is(source, &exports[6], "LionCombobox", None);
}

#[test]
fn non_identifier_string_as_variable_double_quote() {
    let source = "
        export { \"~123\" as foo0 } from './mod0.js';
        export { \"ab cd\" as foo1 } from './mod1.js';
        export { \"not identifier\" as foo2 } from './mod2.js';
        export { \"-notidentifier\" as foo3 } from './mod3.js';
        export { \"%notidentifier\" as foo4 } from './mod4.js';
        export { \"@notidentifier\" as foo5 } from './mod5.js';
        export { \" notidentifier\" as foo6 } from './mod6.js';
        export { \"notidentifier \" as foo7 } from './mod7.js';
        export { \" notidentifier \" as foo8 } from './mod8.js';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "foo0", None);
    assert_export_is(source, &exports[1], "foo1", None);
    assert_export_is(source, &exports[2], "foo2", None);
    assert_export_is(source, &exports[3], "foo3", None);
    assert_export_is(source, &exports[4], "foo4", None);
    assert_export_is(source, &exports[5], "foo5", None);
    assert_export_is(source, &exports[6], "foo6", None);
    assert_export_is(source, &exports[7], "foo7", None);
    assert_export_is(source, &exports[8], "foo8", None);
}

#[test]
fn non_identifier_string_as_variable_single_quote() {
    let source = "
        export { '~123' as foo0 } from './mod0.js';
        export { 'ab cd' as foo1 } from './mod1.js';
        export { 'not identifier' as foo2 } from './mod2.js';
        export { '-notidentifier' as foo3 } from './mod3.js';
        export { '%notidentifier' as foo4 } from './mod4.js';
        export { '@notidentifier' as foo5 } from './mod5.js';
        export { ' notidentifier' as foo6 } from './mod6.js';
        export { 'notidentifier ' as foo7 } from './mod7.js';
        export { ' notidentifier ' as foo8 } from './mod8.js';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "foo0", None);
    assert_export_is(source, &exports[1], "foo1", None);
    assert_export_is(source, &exports[2], "foo2", None);
    assert_export_is(source, &exports[3], "foo3", None);
    assert_export_is(source, &exports[4], "foo4", None);
    assert_export_is(source, &exports[5], "foo5", None);
    assert_export_is(source, &exports[6], "foo6", None);
    assert_export_is(source, &exports[7], "foo7", None);
    assert_export_is(source, &exports[8], "foo8", None);
}

#[test]
fn with_backslash_keywords_as_variable_double_quote() {
    let source = r#"
        export { " slash\\ " as foo0 } from './mod0.js';
        export { " quote\" " as foo1 } from './mod1.js';
        export { " quote\\\" " as foo2 } from './mod2.js';
        export { " quote' " as foo3 } from './mod3.js';"#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], "foo0", None);
    assert_export_is(source, &exports[1], "foo1", None);
    assert_export_is(source, &exports[2], "foo2", None);
    assert_export_is(source, &exports[3], "foo3", None);
}

#[test]
fn with_backslash_keywords_as_variable_single_quote() {
    let source = r"
        export { ' slash\\ ' as foo0 } from './mod0.js';
        export { ' quote\' ' as foo1 } from './mod1.js';
        export { ' quote\\\' ' as foo2 } from './mod2.js';
        export { ' quote\' ' as foo3 } from './mod3.js';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], "foo0", None);
    assert_export_is(source, &exports[1], "foo1", None);
    assert_export_is(source, &exports[2], "foo2", None);
    assert_export_is(source, &exports[3], "foo3", None);
}

#[test]
fn with_emoji_as_2() {
    let source = r#"
        export { "hm🤔" as foo0 } from './mod0.js';
        export { " 🚀rocket space " as foo1 } from './mod1.js';"#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 2);
    assert_eq!(exports.len(), 2);
    assert_export_is(source, &exports[0], "foo0", None);
    assert_export_is(source, &exports[1], "foo1", None);
}

#[test]
fn non_identifier_string_double_quote_2() {
    let source = r#"
        export { "~123" } from './mod0.js';
        export { "ab cd" } from './mod1.js';
        export { "not identifier" } from './mod2.js';
        export { "-notidentifier" } from './mod3.js';
        export { "%notidentifier" } from './mod4.js';
        export { "@notidentifier" } from './mod5.js';
        export { " notidentifier" } from './mod6.js';
        export { "notidentifier " } from './mod7.js';
        export { " notidentifier " } from './mod8.js';"#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "~123", None);
    assert_export_is(source, &exports[1], "ab cd", None);
    assert_export_is(source, &exports[2], "not identifier", None);
    assert_export_is(source, &exports[3], "-notidentifier", None);
    assert_export_is(source, &exports[4], "%notidentifier", None);
    assert_export_is(source, &exports[5], "@notidentifier", None);
    assert_export_is(source, &exports[6], " notidentifier", None);
    assert_export_is(source, &exports[7], "notidentifier ", None);
    assert_export_is(source, &exports[8], " notidentifier ", None);
}

#[test]
fn non_identifier_string_single_quote_2() {
    let source = r"
        export { '~123' } from './mod0.js';
        export { 'ab cd' } from './mod1.js';
        export { 'not identifier' } from './mod2.js';
        export { '-notidentifier' } from './mod3.js';
        export { '%notidentifier' } from './mod4.js';
        export { '@notidentifier' } from './mod5.js';
        export { ' notidentifier' } from './mod6.js';
        export { 'notidentifier ' } from './mod7.js';
        export { ' notidentifier ' } from './mod8.js';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "~123", None);
    assert_export_is(source, &exports[1], "ab cd", None);
    assert_export_is(source, &exports[2], "not identifier", None);
    assert_export_is(source, &exports[3], "-notidentifier", None);
    assert_export_is(source, &exports[4], "%notidentifier", None);
    assert_export_is(source, &exports[5], "@notidentifier", None);
    assert_export_is(source, &exports[6], " notidentifier", None);
    assert_export_is(source, &exports[7], "notidentifier ", None);
    assert_export_is(source, &exports[8], " notidentifier ", None);
}

#[test]
fn with_backslash_keywords_double_quote_2() {
    let source = r#"
        export { " slash\\ " } from './mod0.js';
        export { " quote\" " } from './mod1.js';
        export { " quote\\\" " } from './mod2.js';
        export { " quote' " } from './mod3.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], " slash\\ ", None);
    assert_export_is(source, &exports[1], " quote\" ", None);
    assert_export_is(source, &exports[2], " quote\\\" ", None);
    assert_export_is(source, &exports[3], " quote' ", None);
}

#[test]
fn with_backslash_keywords_single_quote_2() {
    let source = r"
      export { ' slash\\ ' } from './mod0.js';
      export { ' quote\' ' } from './mod1.js';
      export { ' quote\\\' ' } from './mod2.js';
      export { ' quote\' ' } from './mod3.js';
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], " slash\\ ", None);
    assert_export_is(source, &exports[1], " quote' ", None);
    assert_export_is(source, &exports[2], " quote\\' ", None);
    assert_export_is(source, &exports[3], " quote' ", None);
}

#[test]
fn variable_as_non_identifier_string_double_quote() {
    let source = r#"
        export { foo0 as "~123" } from './mod0.js';
        export { foo1 as "ab cd" } from './mod1.js';
        export { foo2 as "not identifier" } from './mod2.js';
        export { foo3 as "-notidentifier" } from './mod3.js';
        export { foo4 as "%notidentifier" } from './mod4.js';
        export { foo5 as "@notidentifier" } from './mod5.js';
        export { foo6 as " notidentifier" } from './mod6.js';
        export { foo7 as "notidentifier " } from './mod7.js';
        export { foo8 as " notidentifier " } from './mod8.js';"#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "~123", None);
    assert_export_is(source, &exports[1], "ab cd", None);
    assert_export_is(source, &exports[2], "not identifier", None);
    assert_export_is(source, &exports[3], "-notidentifier", None);
    assert_export_is(source, &exports[4], "%notidentifier", None);
    assert_export_is(source, &exports[5], "@notidentifier", None);
    assert_export_is(source, &exports[6], " notidentifier", None);
    assert_export_is(source, &exports[7], "notidentifier ", None);
    assert_export_is(source, &exports[8], " notidentifier ", None);
}

#[test]
fn variable_as_non_identifier_string_single_quote() {
    let source = r"
        export { foo0 as '~123' } from './mod0.js';
        export { foo1 as 'ab cd' } from './mod1.js';
        export { foo2 as 'not identifier' } from './mod2.js';
        export { foo3 as '-notidentifier' } from './mod3.js';
        export { foo4 as '%notidentifier' } from './mod4.js';
        export { foo5 as '@notidentifier' } from './mod5.js';
        export { foo6 as ' notidentifier' } from './mod6.js';
        export { foo7 as 'notidentifier ' } from './mod7.js';
        export { foo8 as ' notidentifier ' } from './mod8.js';";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "~123", None);
    assert_export_is(source, &exports[1], "ab cd", None);
    assert_export_is(source, &exports[2], "not identifier", None);
    assert_export_is(source, &exports[3], "-notidentifier", None);
    assert_export_is(source, &exports[4], "%notidentifier", None);
    assert_export_is(source, &exports[5], "@notidentifier", None);
    assert_export_is(source, &exports[6], " notidentifier", None);
    assert_export_is(source, &exports[7], "notidentifier ", None);
    assert_export_is(source, &exports[8], " notidentifier ", None);
}

#[test]
fn variable_as_with_backslash_keywords_double_quote() {
    let source = r#"
        export { foo0 as " slash\\ " } from './mod0.js';
        export { foo1 as " quote\" " } from './mod1.js';
        export { foo2 as " quote\\\" " } from './mod2.js';
        export { foo3 as " quote' " } from './mod3.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], " slash\\ ", None);
    assert_export_is(source, &exports[1], " quote\" ", None);
    assert_export_is(source, &exports[2], " quote\\\" ", None);
    assert_export_is(source, &exports[3], " quote' ", None);
}

#[test]
fn variable_as_with_backslash_keywords_single_quote() {
    let source = r"
        export { foo0 as ' slash\\ ' } from './mod0.js';
        export { foo1 as ' quote\' ' } from './mod1.js';
        export { foo2 as ' quote\\\' ' } from './mod2.js';
        export { foo3 as ' quote\' ' } from './mod3.js';
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], " slash\\ ", None);
    assert_export_is(source, &exports[1], " quote' ", None);
    assert_export_is(source, &exports[2], " quote\\' ", None);
    assert_export_is(source, &exports[3], " quote' ", None);
}

#[test]
fn non_identifier_string_as_non_identifier_string_double_quote() {
    let source = r#"
        export { "~123" as "~123" } from './mod0.js';
        export { "ab cd" as "ab cd" } from './mod1.js';
        export { "not identifier" as "not identifier" } from './mod2.js';
        export { "-notidentifier" as "-notidentifier" } from './mod3.js';
        export { "%notidentifier" as "%notidentifier" } from './mod4.js';
        export { "@notidentifier" as "@notidentifier" } from './mod5.js';
        export { " notidentifier" as " notidentifier" } from './mod6.js';
        export { "notidentifier " as "notidentifier " } from './mod7.js';
        export { " notidentifier " as " notidentifier " } from './mod8.js';
        "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "~123", None);
    assert_export_is(source, &exports[1], "ab cd", None);
    assert_export_is(source, &exports[2], "not identifier", None);
    assert_export_is(source, &exports[3], "-notidentifier", None);
    assert_export_is(source, &exports[4], "%notidentifier", None);
    assert_export_is(source, &exports[5], "@notidentifier", None);
    assert_export_is(source, &exports[6], " notidentifier", None);
    assert_export_is(source, &exports[7], "notidentifier ", None);
    assert_export_is(source, &exports[8], " notidentifier ", None);
}

#[test]
fn non_identifier_string_as_non_identifier_string_single_quote() {
    let source = r"
      export { '~123' as '~123' } from './mod0.js';
      export { 'ab cd' as 'ab cd' } from './mod1.js';
      export { 'not identifier' as 'not identifier' } from './mod2.js';
      export { '-notidentifier' as '-notidentifier' } from './mod3.js';
      export { '%notidentifier' as '%notidentifier' } from './mod4.js';
      export { '@notidentifier' as '@notidentifier' } from './mod5.js';
      export { ' notidentifier' as ' notidentifier' } from './mod6.js';
      export { 'notidentifier ' as 'notidentifier ' } from './mod7.js';
      export { ' notidentifier ' as ' notidentifier ' } from './mod8.js';
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 9);
    assert_eq!(exports.len(), 9);
    assert_export_is(source, &exports[0], "~123", None);
    assert_export_is(source, &exports[1], "ab cd", None);
    assert_export_is(source, &exports[2], "not identifier", None);
    assert_export_is(source, &exports[3], "-notidentifier", None);
    assert_export_is(source, &exports[4], "%notidentifier", None);
    assert_export_is(source, &exports[5], "@notidentifier", None);
    assert_export_is(source, &exports[6], " notidentifier", None);
    assert_export_is(source, &exports[7], "notidentifier ", None);
    assert_export_is(source, &exports[8], " notidentifier ", None);
}

#[test]
fn with_backslash_keywords_as_with_backslash_keywords_double_quote() {
    let source = r#"
        export { " slash\\ " as " slash\\ " } from './mod0.js';
        export { " quote\"" as " quote\" " } from './mod1.js'
        export { " quote\\\" " as " quote\\\" " } from './mod2.js';
        export { " quote' " as " quote' " } from './mod3.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], " slash\\ ", None);
    assert_export_is(source, &exports[1], " quote\" ", None);
    assert_export_is(source, &exports[2], " quote\\\" ", None);
    assert_export_is(source, &exports[3], " quote' ", None);
}

#[test]
fn with_backslash_keywords_as_with_backslash_keywords_single_quote() {
    let source = r"
        export { ' slash\\ ' as ' slash\\ ' } from './mod0.js';
        export { ' quote\'' as ' quote\' ' } from './mod1.js'
        export { ' quote\\\' ' as ' quote\\\' ' } from './mod2.js';
        export { ' quote\' ' as ' quote\' ' } from './mod3.js';
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 4);
    assert_eq!(exports.len(), 4);
    assert_export_is(source, &exports[0], " slash\\ ", None);
    assert_export_is(source, &exports[1], " quote' ", None);
    assert_export_is(source, &exports[2], " quote\\' ", None);
    assert_export_is(source, &exports[3], " quote' ", None);
}

#[test]
fn curly_brace_double_quote() {
    let source = r#"
      export { " right-curlybrace} " } from './mod0.js';
      export { " {left-curlybrace " } from './mod1.js';
      export { " {curlybrackets} " } from './mod2.js';
      export { ' right-curlybrace} ' } from './mod0.js';
      export { ' {left-curlybrace ' } from './mod1.js';
      export { ' {curlybrackets} ' } from './mod2.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 6);
    assert_eq!(exports.len(), 6);
    assert_export_is(source, &exports[0], " right-curlybrace} ", None);
    assert_export_is(source, &exports[1], " {left-curlybrace ", None);
    assert_export_is(source, &exports[2], " {curlybrackets} ", None);
    assert_export_is(source, &exports[3], " right-curlybrace} ", None);
    assert_export_is(source, &exports[4], " {left-curlybrace ", None);
    assert_export_is(source, &exports[5], " {curlybrackets} ", None);
}

#[test]
fn as_curly_brace_double_quote() {
    let source = r#"
      export { foo as " right-curlybrace} " } from './mod0.js';
      export { foo as " {left-curlybrace " } from './mod1.js';
      export { foo as " {curlybrackets} " } from './mod2.js';
      export { foo as ' right-curlybrace} ' } from './mod0.js';
      export { foo as ' {left-curlybrace ' } from './mod1.js';
      export { foo as ' {curlybrackets} ' } from './mod2.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 6);
    assert_eq!(exports.len(), 6);
    assert_export_is(source, &exports[0], " right-curlybrace} ", None);
    assert_export_is(source, &exports[1], " {left-curlybrace ", None);
    assert_export_is(source, &exports[2], " {curlybrackets} ", None);
    assert_export_is(source, &exports[3], " right-curlybrace} ", None);
    assert_export_is(source, &exports[4], " {left-curlybrace ", None);
    assert_export_is(source, &exports[5], " {curlybrackets} ", None);
}

#[test]
fn curly_brace_as_curly_brace_double_quote() {
    let source = r#"
      export { " right-curlybrace} " as " right-curlybrace} " } from './mod0.js';
      export { " {left-curlybrace " as " {left-curlybrace " } from './mod1.js';
      export { " {curlybrackets} " as " {curlybrackets} " } from './mod2.js';
      export { ' right-curlybrace} ' as ' right-curlybrace} ' } from './mod0.js';
      export { ' {left-curlybrace ' as ' {left-curlybrace ' } from './mod1.js';
      export { ' {curlybrackets} ' as ' {curlybrackets} ' } from './mod2.js';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 6);
    assert_eq!(exports.len(), 6);
    assert_export_is(source, &exports[0], " right-curlybrace} ", None);
    assert_export_is(source, &exports[1], " {left-curlybrace ", None);
    assert_export_is(source, &exports[2], " {curlybrackets} ", None);
    assert_export_is(source, &exports[3], " right-curlybrace} ", None);
    assert_export_is(source, &exports[4], " {left-curlybrace ", None);
    assert_export_is(source, &exports[5], " {curlybrackets} ", None);
}

#[test]
fn complex_and_edge_cases() {
    let source = r#"
        export {
        foo,
        foo1 as foo2,
        " {left-curlybrace ",
        " {curly-brackets}" as "@notidentifier",
        "?" as "identifier",
        } from './mod0.js';
        export { "p as 'z' from 'asdf'" as "z'" } from 'asdf';
        export { "z'" as "p as 'z' from 'asdf'" } from 'asdf';
    "#;
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 3);
    assert_eq!(exports.len(), 7);
    assert_export_is(source, &exports[0], "foo", None);
    assert_export_is(source, &exports[1], "foo2", None);
    assert_export_is(source, &exports[2], " {left-curlybrace ", None);
    assert_export_is(source, &exports[3], "@notidentifier", None);
    assert_export_is(source, &exports[4], "identifier", None);
    assert_export_is(source, &exports[5], "z'", None);
    assert_export_is(source, &exports[6], "p as 'z' from 'asdf'", None);
}

#[test]
fn export_default() {
    let source = r"
        export default async function example   () {};
        export const a = '1';
        export default a;
        export default function example1() {};
        export default function() {};
        export default class className {/* ... */};
        export default class {}
        export default function* generatorFunctionName(){/* ... */};
        export default function* ()  {};
        const async = 1
        export default async

        function x() {}

        const asyncVar = 1
        export default asyncVar

        function functionName () {};
        export default functionName;
    ";
    let ModuleLexer { imports, exports, .. } = parse(source);
    assert_eq!(imports.len(), 0);
    assert_eq!(exports.len(), 12);
    assert_export_is(source, &exports[0], "default", Some("example"));
    assert_export_is(source, &exports[1], "a", Some("a"));
    assert_export_is(source, &exports[2], "default", None);
    assert_export_is(source, &exports[3], "default", Some("example1"));
    assert_export_is(source, &exports[4], "default", None);
    assert_export_is(source, &exports[5], "default", Some("className"));
    assert_export_is(source, &exports[6], "default", None);
    assert_export_is(source, &exports[7], "default", Some("generatorFunctionName"));
    assert_export_is(source, &exports[8], "default", None);
    assert_export_is(source, &exports[9], "default", None);
    assert_export_is(source, &exports[10], "default", None);
    assert_export_is(source, &exports[11], "default", None);
}

/* Suite Invalid Syntax */

fn expect_parse_error(source: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(!ret.errors.is_empty());
}

#[test]
fn unterminated_object() {
    let source = r"
      const foo = };
      const bar = {};
    ";
    expect_parse_error(source);
}

#[test]
fn invalid_string() {
    let source = r"import './export.js';

    import d from './export.js';

    import { s as p } from './reexport1.js';

    import { z, q as r } from './reexport2.js';

       '

    import * as q from './reexport1.js';

    export { d as a, p as b, z as c, r as d, q }`";
    expect_parse_error(source);
}

#[test]
fn invalid_export() {
    let source = "export { a = }";
    expect_parse_error(source);
}

/* has_module_syntax */

#[test]
fn has_module_syntax_import1() {
    let has_module_syntax = parse(r#"import foo from "./foo""#).has_module_syntax;
    assert!(has_module_syntax);
}

#[test]
fn has_module_syntax_import2() {
    let has_module_syntax = parse(r#"const foo = "import""#).has_module_syntax;
    assert!(!has_module_syntax);
}

#[test]
fn has_module_syntax_import3() {
    let has_module_syntax = parse(r#"import("./foo")"#).has_module_syntax;
    // dynamic imports can be used in non-ESM files as well
    assert!(!has_module_syntax);
}

#[test]
fn has_module_syntax_import4() {
    let has_module_syntax = parse(r"import.meta.url").has_module_syntax;
    assert!(has_module_syntax);
}

#[test]
fn has_module_syntax_export1() {
    let has_module_syntax = parse(r#"export const foo = "foo""#).has_module_syntax;
    assert!(has_module_syntax);
}

#[test]
fn has_module_syntax_export2() {
    let has_module_syntax = parse(r"export {}").has_module_syntax;
    assert!(has_module_syntax);
}

#[test]
fn has_module_syntax_export3() {
    let has_module_syntax = parse(r#"export * from "./foo""#).has_module_syntax;
    assert!(has_module_syntax);
}

/* facade */

#[test]
fn facade() {
    let facade = parse(
        r"
            export * from 'external';
            import * as ns from 'external2';
            export { a as b } from 'external3';
            export { ns };
        ",
    )
    .facade;
    assert!(facade);
}

#[test]
fn facade_default() {
    let facade = parse(
        r"
            import * as ns from 'external';
            export default ns;
        ",
    )
    .facade;
    assert!(!facade);
}

#[test]
fn facade_declaration1() {
    let facade = parse(r"export function p () {}").facade;
    assert!(!facade);
}

#[test]
fn facade_declaration2() {
    let facade = parse(r"export var p").facade;
    assert!(!facade);
}

#[test]
fn facade_declaration3() {
    let facade = parse(r"export {}").facade;
    assert!(facade);
}

#[test]
fn facade_declaration4() {
    let facade = parse(r"export class Q{}").facade;
    assert!(!facade);
}

#[test]
fn facade_side_effect() {
    let facade = parse(r"console.log('any non esm syntax')").facade;
    assert!(!facade);
}
