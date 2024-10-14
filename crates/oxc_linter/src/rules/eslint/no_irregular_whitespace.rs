use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_irregular_whitespace_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected irregular whitespace")
        .with_help("Try to remove the irregular whitespace")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoIrregularWhitespace;

declare_oxc_lint!(
    /// ### What it does
    /// Disallows the use of irregular whitespaces in the code.
    ///
    /// ### Why is this bad
    /// The use of irregular whitespaces can hinder code readability and
    /// create inconsistencies, making maintenance and collaboration more challenging.
    ///
    /// ### Example
    /// ```javascript
    /// function  invalidExample  (  ) {
    ///     return  42;
    /// }
    /// ```
    NoIrregularWhitespace,
    correctness
);

impl Rule for NoIrregularWhitespace {
    fn run_once(&self, ctx: &LintContext) {
        let irregular_whitespaces = ctx.semantic().irregular_whitespaces();
        for irregular_whitespace in irregular_whitespaces {
            ctx.diagnostic(no_irregular_whitespace_diagnostic(*irregular_whitespace));
        }
    }
}

#[allow(clippy::unicode_not_nfc, clippy::invisible_characters)]
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"'\u000B';", None),
        (r"'\u000C';", None),
        (r"'\u0085';", None),
        (r"'\u00A0';", None),
        (r"'\u180E';", None),
        (r"'\ufeff';", None),
        (r"'\u2000';", None),
        (r"'\u2001';", None),
        (r"'\u2002';", None),
        (r"'\u2003';", None),
        (r"'\u2004';", None),
        (r"'\u2005';", None),
        (r"'\u2006';", None),
        (r"'\u2007';", None),
        (r"'\u2008';", None),
        (r"'\u2009';", None),
        (r"'\u200A';", None),
        (r"'\u200B';", None),
        (r"'\u2028';", None),
        (r"'\u2029';", None),
        (r"'\u202F';", None),
        (r"'\u205f';", None),
        (r"'\u3000';", None),
        (r"'';", None),
        (r"'';", None),
        (r"'';", None),
        (r"' ';", None),
        (r"'᠎';", None),
        (r"'﻿';", None),
        ("' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"'​';", None),
        (r"'\ ';", None),
        (r"'\ ';", None),
        (r"' ';", None),
        (r"' ';", None),
        (r"'　';", None),
        (r"// ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"// ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"// ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"// ᠎", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"// ﻿", Some(serde_json::json!([{ "skipComments": true }]))),
        // (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))), lint error
        // (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))), lint error
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        // (r"// ​", Some(serde_json::json!([{ "skipComments": true }]))), lint error
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//  ", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"// 　", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*  */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*  */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*  */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/* ᠎ */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/* ﻿ */", Some(serde_json::json!([{ "skipComments": true }]))),
        // (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))), lint error
        // (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))), lint error
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        // (r"/* ​ */", Some(serde_json::json!([{ "skipComments": true }]))), lint error
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/*   */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"/* 　 */", Some(serde_json::json!([{ "skipComments": true }]))),
        (r"//", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"//", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"//", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/᠎/", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/﻿/", Some(serde_json::json!([{ "skipRegExps": true }]))),
        // (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))), lint error
        // (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))), lint error
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        // (r"/​/", Some(serde_json::json!([{ "skipRegExps": true }]))),  lint error
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/ /", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"/　/", Some(serde_json::json!([{ "skipRegExps": true }]))),
        (r"``", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"``", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"``", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"`᠎`", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"`﻿`", Some(serde_json::json!([{ "skipTemplates": true }]))),
        // (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),  lint error
        // (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),  lint error
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        // (r"`​`", Some(serde_json::json!([{ "skipTemplates": true }]))),  lint error
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"` `", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"`　`", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"`　${foo}　`", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (r"const error = ` 　 `;", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (
            r"const error = `
			　`;",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"const error = `　
			`;",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"const error = `
			　
			`;",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"const error = `foo　bar
			foo　bar`;",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (r"<div></div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div></div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div></div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div>᠎</div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div>﻿</div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        // (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),  lint error
        // (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),  lint error
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        // (r"<div>​</div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),  lint error
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div> </div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        (r"<div>　</div>;", Some(serde_json::json!([{ "skipJSXText": true }]))),
        // (r"﻿console.log('hello BOM');", None),
    ];

    let fail = vec![
        (r"var any  = 'thing';", None),
        (r"var any  = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any ﻿ = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any   = 'thing';", None),
        (r"var any 　 = 'thing';", None),
        (
            r"var a = 'b', c = 'd',
          e = 'f' ",
            None,
        ),
        (
            r"var any 　 = 'thing', other 　 = 'thing';
			var third 　 = 'thing';",
            None,
        ),
        // (r"// ", None),
        // (r"// ", None),
        // (r"// ", None),
        // (r"//  ", None),
        // (r"// ᠎", None),
        // (r"// ﻿", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"// ​", None),
        // (r"//  ", None),
        // (r"//  ", None),
        // (r"// 　", None),
        // (r"/*  */", None),
        // (r"/*  */", None),
        // (r"/*  */", None),
        // (r"/*   */", None),
        // (r"/* ᠎ */", None),
        // (r"/* ﻿ */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/* ​ */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/*   */", None),
        // (r"/* 　 */", None),
        // (r"var any = /　/, other = //;", None),
        // (r"var any = '　', other = '';", Some(serde_json::json!([{ "skipStrings": false }]))),
        // (r"var any = `　`, other = ``;", Some(serde_json::json!([{ "skipTemplates": false }]))),
        (
            r"`something ${　 10} another thing`",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"`something ${10　} another thing`",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"　
			`　template`",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"　
			`　multiline
			template`",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (r"　`　template`", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (
            r"　`　multiline
			template`",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (r"`　template`　", Some(serde_json::json!([{ "skipTemplates": true }]))),
        (
            r"`　multiline
			template`　",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"`　template`
			　",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (
            r"`　multiline
			template`
			　",
            Some(serde_json::json!([{ "skipTemplates": true }])),
        ),
        (r"var foo =  bar;", None),
        (r"var foo =bar;", None),
        (r"var foo =  bar;", None),
        (r"var foo =  bar;", None),
        (r"var foo =   bar;", None),
        (r"var foo = bar;", None),
        (r"", None),
        ("   ", None),
        // (
        // r"var foo =
        // bar;",
        // None,
        // ),
        (
            r"var foo =
        bar;",
            None,
        ),
        (
            r"var foo =
        bar
        ;
        ",
            None,
        ),
        (r"var foo =  bar;", None),
        (r"var foo =  bar;", None),
        (r"var foo = bar; ", None),
        (r" ", None),
        (r"foo  ", None),
        (r"foo  ", None),
        (
            r"foo
         ",
            None,
        ),
        // (r"foo ", None),
        // (r"<div></div>;", None),
        // (r"<div></div>;", None),
        // (r"<div></div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div>᠎</div>;", None),
        // (r"<div>﻿</div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div>​</div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div> </div>;", None),
        // (r"<div>　</div>;", None),
    ];

    Tester::new(NoIrregularWhitespace::NAME, pass, fail).test_and_snapshot();
}
