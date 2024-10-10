use cow_utils::CowUtils;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::CompressorPass;

/// Minimize With Known Methods
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeReplaceKnownMethods.java>
pub struct PeepholeReplaceKnownMethods {
    changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeReplaceKnownMethods {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeReplaceKnownMethods {
    fn enter_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.try_fold_known_string_methods(node, ctx);
    }
}

impl PeepholeReplaceKnownMethods {
    pub fn new() -> Self {
        Self { changed: false }
    }

    fn try_fold_known_string_methods<'a>(
        &mut self,
        node: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::CallExpression(call_expr) = node else { return };

        let Expression::StaticMemberExpression(member) = &call_expr.callee else { return };
        if let Expression::StringLiteral(string_lit) = &member.object {
            if call_expr.arguments.len() == 0 {
                let transformed_value = match member.property.name.as_str() {
                    "toLowerCase" => Some(
                        ctx.ast.string_literal(call_expr.span, string_lit.value.cow_to_lowercase()),
                    ),
                    "toUpperCase" => Some(
                        ctx.ast.string_literal(call_expr.span, string_lit.value.cow_to_uppercase()),
                    ),
                    "trim" => Some(ctx.ast.string_literal(call_expr.span, string_lit.value.trim())),
                    _ => None,
                };

                if let Some(transformed_value) = transformed_value {
                    self.changed = true;
                    *node = ctx.ast.expression_from_string_literal(transformed_value);
                }
            }
        }
    }
}

/// Port from: <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeReplaceKnownMethodsTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, positive: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeReplaceKnownMethods::new();
        tester::test(&allocator, source_text, positive, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    fn fold_same(js: &str) {
        test_same(js);
    }

    fn fold(js: &str, expected: &str) {
        test(js, expected);
    }

    #[test]
    #[ignore]
    fn test_string_index_of() {
        fold("x = 'abcdef'.indexOf('g')", "x = -1");
        fold("x = 'abcdef'.indexOf('b')", "x = 1");
        fold("x = 'abcdefbe'.indexOf('b', 2)", "x = 6");
        fold("x = 'abcdef'.indexOf('bcd')", "x = 1");
        fold("x = 'abcdefsdfasdfbcdassd'.indexOf('bcd', 4)", "x = 13");

        fold("x = 'abcdef'.lastIndexOf('b')", "x = 1");
        fold("x = 'abcdefbe'.lastIndexOf('b')", "x = 6");
        fold("x = 'abcdefbe'.lastIndexOf('b', 5)", "x = 1");

        // Both elements must be strings. Don't do anything if either one is not
        // string.
        fold("x = 'abc1def'.indexOf(1)", "x = 3");
        fold("x = 'abcNaNdef'.indexOf(NaN)", "x = 3");
        fold("x = 'abcundefineddef'.indexOf(undefined)", "x = 3");
        fold("x = 'abcnulldef'.indexOf(null)", "x = 3");
        fold("x = 'abctruedef'.indexOf(true)", "x = 3");

        // The following test case fails with JSC_PARSE_ERROR. Hence omitted.
        // fold_same("x = 1.indexOf('bcd');");
        fold_same("x = NaN.indexOf('bcd')");
        fold_same("x = undefined.indexOf('bcd')");
        fold_same("x = null.indexOf('bcd')");
        fold_same("x = true.indexOf('bcd')");
        fold_same("x = false.indexOf('bcd')");

        // Avoid dealing with regex or other types.
        fold_same("x = 'abcdef'.indexOf(/b./)");
        fold_same("x = 'abcdef'.indexOf({a:2})");
        fold_same("x = 'abcdef'.indexOf([1,2])");

        // Template Strings
        fold_same("x = `abcdef`.indexOf('b')");
        fold_same("x = `Hello ${name}`.indexOf('a')");
        fold_same("x = tag `Hello ${name}`.indexOf('a')");
    }

    #[test]
    #[ignore]
    fn test_string_join_add_sparse() {
        fold("x = [,,'a'].join(',')", "x = ',,a'");
    }

    #[test]
    #[ignore]
    fn test_no_string_join() {
        fold_same("x = [].join(',',2)");
        fold_same("x = [].join(f)");
    }

    #[test]
    #[ignore]
    fn test_string_join_add() {
        fold("x = ['a', 'b', 'c'].join('')", "x = \"abc\"");
        fold("x = [].join(',')", "x = \"\"");
        fold("x = ['a'].join(',')", "x = \"a\"");
        fold("x = ['a', 'b', 'c'].join(',')", "x = \"a,b,c\"");
        fold("x = ['a', foo, 'b', 'c'].join(',')", "x = [\"a\",foo,\"b,c\"].join()");
        fold("x = [foo, 'a', 'b', 'c'].join(',')", "x = [foo,\"a,b,c\"].join()");
        fold("x = ['a', 'b', 'c', foo].join(',')", "x = [\"a,b,c\",foo].join()");

        // Works with numbers
        fold("x = ['a=', 5].join('')", "x = \"a=5\"");
        fold("x = ['a', '5'].join(7)", "x = \"a75\"");

        // Works on boolean
        fold("x = ['a=', false].join('')", "x = \"a=false\"");
        fold("x = ['a', '5'].join(true)", "x = \"atrue5\"");
        fold("x = ['a', '5'].join(false)", "x = \"afalse5\"");

        // Only optimize if it's a size win.
        fold(
            "x = ['a', '5', 'c'].join('a very very very long chain')",
            "x = [\"a\",\"5\",\"c\"].join(\"a very very very long chain\")",
        );

        // Template strings
        fold("x = [`a`, `b`, `c`].join(``)", "x = 'abc'");
        fold("x = [`a`, `b`, `c`].join('')", "x = 'abc'");

        // TODO(user): Its possible to fold this better.
        fold_same("x = ['', foo].join('-')");
        fold_same("x = ['', foo, ''].join()");

        fold(
            "x = ['', '', foo, ''].join(',')", //
            "x = [ ','  , foo, ''].join()",
        );
        fold(
            "x = ['', '', foo, '', ''].join(',')", //
            "x = [ ',',   foo,  ','].join()",
        );

        fold(
            "x = ['', '', foo, '', '', bar].join(',')", //
            "x = [ ',',   foo,  ',',   bar].join()",
        );

        fold(
            "x = [1,2,3].join('abcdef')", //
            "x = '1abcdef2abcdef3'",
        );

        fold("x = [1,2].join()", "x = '1,2'");
        fold("x = [null,undefined,''].join(',')", "x = ',,'");
        fold("x = [null,undefined,0].join(',')", "x = ',,0'");
        // This can be folded but we don't currently.
        fold_same("x = [[1,2],[3,4]].join()"); // would like: "x = '1,2,3,4'"
    }

    #[test]
    #[ignore]
    fn test_string_join_add_b1992789() {
        fold("x = ['a'].join('')", "x = \"a\"");
        fold_same("x = [foo()].join('')");
        fold_same("[foo()].join('')");
        fold("[null].join('')", "''");
    }

    #[test]
    #[ignore]
    fn test_fold_string_substr() {
        fold("x = 'abcde'.substr(0,2)", "x = 'ab'");
        fold("x = 'abcde'.substr(1,2)", "x = 'bc'");
        fold("x = 'abcde'.substr(2)", "x = 'cde'");

        // we should be leaving negative indexes alone for now
        fold_same("x = 'abcde'.substr(-1)");
        fold_same("x = 'abcde'.substr(1, -2)");
        fold_same("x = 'abcde'.substr(1, 2, 3)");
        fold_same("x = 'a'.substr(0, 2)");

        // Template strings
        fold_same("x = `abcdef`.substr(0,2)");
        fold_same("x = `abc ${xyz} def`.substr(0,2)");
    }

    #[test]
    #[ignore]
    fn test_fold_string_replace() {
        fold("'c'.replace('c','x')", "'x'");
        fold("'ac'.replace('c','x')", "'ax'");
        fold("'ca'.replace('c','x')", "'xa'");
        fold("'ac'.replace('c','xxx')", "'axxx'");
        fold("'ca'.replace('c','xxx')", "'xxxa'");

        // only one instance replaced
        fold("'acaca'.replace('c','x')", "'axaca'");
        fold("'ab'.replace('','x')", "'xab'");

        fold_same("'acaca'.replace(/c/,'x')"); // this will affect the global RegExp props
        fold_same("'acaca'.replace(/c/g,'x')"); // this will affect the global RegExp props

        // not a literal
        fold_same("x.replace('x','c')");

        fold_same("'Xyz'.replace('Xyz', '$$')"); // would fold to '$'
        fold_same("'PreXyzPost'.replace('Xyz', '$&')"); // would fold to 'PreXyzPost'
        fold_same("'PreXyzPost'.replace('Xyz', '$`')"); // would fold to 'PrePrePost'
        fold_same("'PreXyzPost'.replace('Xyz', '$\\'')"); // would fold to  'PrePostPost'
        fold_same("'PreXyzPostXyz'.replace('Xyz', '$\\'')"); // would fold to 'PrePostXyzPostXyz'
        fold_same("'123'.replace('2', '$`')"); // would fold to '113'
    }

    #[test]
    #[ignore]
    fn test_fold_string_replace_all() {
        fold("x = 'abcde'.replaceAll('bcd','c')", "x = 'ace'");
        fold("x = 'abcde'.replaceAll('c','xxx')", "x = 'abxxxde'");
        fold("x = 'abcde'.replaceAll('xxx','c')", "x = 'abcde'");
        fold("'ab'.replaceAll('','x')", "'xaxbx'");

        fold("x = 'c_c_c'.replaceAll('c','x')", "x = 'x_x_x'");

        fold_same("x = 'acaca'.replaceAll(/c/,'x')"); // this should throw
        fold_same("x = 'acaca'.replaceAll(/c/g,'x')"); // this will affect the global RegExp props

        // not a literal
        fold_same("x.replaceAll('x','c')");

        fold_same("'Xyz'.replaceAll('Xyz', '$$')"); // would fold to '$'
        fold_same("'PreXyzPost'.replaceAll('Xyz', '$&')"); // would fold to 'PreXyzPost'
        fold_same("'PreXyzPost'.replaceAll('Xyz', '$`')"); // would fold to 'PrePrePost'
        fold_same("'PreXyzPost'.replaceAll('Xyz', '$\\'')"); // would fold to  'PrePostPost'
        fold_same("'PreXyzPostXyz'.replaceAll('Xyz', '$\\'')"); // would fold to 'PrePostXyzPost'
        fold_same("'123'.replaceAll('2', '$`')"); // would fold to '113'
    }

    #[test]
    #[ignore]
    fn test_fold_string_substring() {
        fold("x = 'abcde'.substring(0,2)", "x = 'ab'");
        fold("x = 'abcde'.substring(1,2)", "x = 'b'");
        fold("x = 'abcde'.substring(2)", "x = 'cde'");

        // we should be leaving negative, out-of-bound, and inverted indices alone for now
        fold_same("x = 'abcde'.substring(-1)");
        fold_same("x = 'abcde'.substring(1, -2)");
        fold_same("x = 'abcde'.substring(1, 2, 3)");
        fold_same("x = 'abcde'.substring(2, 0)");
        fold_same("x = 'a'.substring(0, 2)");

        // Template strings
        fold_same("x = `abcdef`.substring(0,2)");
        fold_same("x = `abcdef ${abc}`.substring(0,2)");
    }

    #[test]
    #[ignore]
    fn test_fold_string_slice() {
        fold("x = 'abcde'.slice(0,2)", "x = 'ab'");
        fold("x = 'abcde'.slice(1,2)", "x = 'b'");
        fold("x = 'abcde'.slice(2)", "x = 'cde'");

        // we should be leaving negative, out-of-bound, and inverted indices alone for now
        fold_same("x = 'abcde'.slice(-1)");
        fold_same("x = 'abcde'.slice(1, -2)");
        fold_same("x = 'abcde'.slice(1, 2, 3)");
        fold_same("x = 'abcde'.slice(2, 0)");
        fold_same("x = 'a'.slice(0, 2)");

        // Template strings
        fold_same("x = `abcdef`.slice(0,2)");
        fold_same("x = `abcdef ${abc}`.slice(0,2)");
    }

    #[test]
    #[ignore]
    fn test_fold_string_char_at() {
        fold("x = 'abcde'.charAt(0)", "x = 'a'");
        fold("x = 'abcde'.charAt(1)", "x = 'b'");
        fold("x = 'abcde'.charAt(2)", "x = 'c'");
        fold("x = 'abcde'.charAt(3)", "x = 'd'");
        fold("x = 'abcde'.charAt(4)", "x = 'e'");
        fold_same("x = 'abcde'.charAt(5)"); // or x = ''
        fold_same("x = 'abcde'.charAt(-1)"); // or x = ''
        fold_same("x = 'abcde'.charAt(y)");
        fold_same("x = 'abcde'.charAt()"); // or x = 'a'
        fold_same("x = 'abcde'.charAt(0, ++z)"); // or (++z, 'a')
        fold_same("x = 'abcde'.charAt(null)"); // or x = 'a'
        fold_same("x = 'abcde'.charAt(true)"); // or x = 'b'
                                               // fold("x = '\\ud834\udd1e'.charAt(0)", "x = '\\ud834'");
                                               // fold("x = '\\ud834\udd1e'.charAt(1)", "x = '\\udd1e'");

        // Template strings
        fold_same("x = `abcdef`.charAt(0)");
        fold_same("x = `abcdef ${abc}`.charAt(0)");
    }

    #[test]
    #[ignore]
    fn test_fold_string_char_code_at() {
        fold("x = 'abcde'.charCodeAt(0)", "x = 97");
        fold("x = 'abcde'.charCodeAt(1)", "x = 98");
        fold("x = 'abcde'.charCodeAt(2)", "x = 99");
        fold("x = 'abcde'.charCodeAt(3)", "x = 100");
        fold("x = 'abcde'.charCodeAt(4)", "x = 101");
        fold_same("x = 'abcde'.charCodeAt(5)"); // or x = (0/0)
        fold_same("x = 'abcde'.charCodeAt(-1)"); // or x = (0/0)
        fold_same("x = 'abcde'.charCodeAt(y)");
        fold_same("x = 'abcde'.charCodeAt()"); // or x = 97
        fold_same("x = 'abcde'.charCodeAt(0, ++z)"); // or (++z, 97)
        fold_same("x = 'abcde'.charCodeAt(null)"); // or x = 97
        fold_same("x = 'abcde'.charCodeAt(true)"); // or x = 98
                                                   // fold("x = '\\ud834\udd1e'.charCodeAt(0)", "x = 55348");
                                                   // fold("x = '\\ud834\udd1e'.charCodeAt(1)", "x = 56606");

        // Template strings
        fold_same("x = `abcdef`.charCodeAt(0)");
        fold_same("x = `abcdef ${abc}`.charCodeAt(0)");
    }

    #[test]
    #[ignore]
    fn test_fold_string_split() {
        // late = false;
        fold("x = 'abcde'.split('foo')", "x = ['abcde']");
        fold("x = 'abcde'.split()", "x = ['abcde']");
        fold("x = 'abcde'.split(null)", "x = ['abcde']");
        fold("x = 'a b c d e'.split(' ')", "x = ['a','b','c','d','e']");
        fold("x = 'a b c d e'.split(' ', 0)", "x = []");
        fold("x = 'abcde'.split('cd')", "x = ['ab','e']");
        fold("x = 'a b c d e'.split(' ', 1)", "x = ['a']");
        fold("x = 'a b c d e'.split(' ', 3)", "x = ['a','b','c']");
        fold("x = 'a b c d e'.split(null, 1)", "x = ['a b c d e']");
        fold("x = 'aaaaa'.split('a')", "x = ['', '', '', '', '', '']");
        fold("x = 'xyx'.split('x')", "x = ['', 'y', '']");

        // Empty separator
        fold("x = 'abcde'.split('')", "x = ['a','b','c','d','e']");
        fold("x = 'abcde'.split('', 3)", "x = ['a','b','c']");

        // Empty separator AND empty string
        fold("x = ''.split('')", "x = []");

        // Separator equals string
        fold("x = 'aaa'.split('aaa')", "x = ['','']");
        fold("x = ' '.split(' ')", "x = ['','']");

        fold_same("x = 'abcde'.split(/ /)");
        fold_same("x = 'abcde'.split(' ', -1)");

        // Template strings
        fold_same("x = `abcdef`.split()");
        fold_same("x = `abcdef ${abc}`.split()");

        // late = true;
        // fold_same("x = 'a b c d e'.split(' ')");
    }

    #[test]
    #[ignore]
    fn test_join_bug() {
        fold("var x = [].join();", "var x = '';");
        fold_same("var x = [x].join();");
        fold_same("var x = [x,y].join();");
        fold_same("var x = [x,y,z].join();");

        // fold_same(
        // lines(
        // "shape['matrix'] = [",
        // "    Number(headingCos2).toFixed(4),",
        // "    Number(-headingSin2).toFixed(4),",
        // "    Number(headingSin2 * yScale).toFixed(4),",
        // "    Number(headingCos2 * yScale).toFixed(4),",
        // "    0,",
        // "    0",
        // "  ].join()"));
    }

    #[test]
    #[ignore]
    fn test_join_spread1() {
        fold_same("var x = [...foo].join('');");
        fold_same("var x = [...someMap.keys()].join('');");
        fold_same("var x = [foo, ...bar].join('');");
        fold_same("var x = [...foo, bar].join('');");
        fold_same("var x = [...foo, 'bar'].join('');");
        fold_same("var x = ['1', ...'2', '3'].join('');");
        fold_same("var x = ['1', ...['2'], '3'].join('');");
    }

    #[test]
    #[ignore]
    fn test_join_spread2() {
        fold("var x = [...foo].join(',');", "var x = [...foo].join();");
        fold("var x = [...someMap.keys()].join(',');", "var x = [...someMap.keys()].join();");
        fold("var x = [foo, ...bar].join(',');", "var x = [foo, ...bar].join();");
        fold("var x = [...foo, bar].join(',');", "var x = [...foo, bar].join();");
        fold("var x = [...foo, 'bar'].join(',');", "var x = [...foo, 'bar'].join();");
        fold("var x = ['1', ...'2', '3'].join(',');", "var x = ['1', ...'2', '3'].join();");
        fold("var x = ['1', ...['2'], '3'].join(',');", "var x = ['1', ...['2'], '3'].join();");
    }

    #[test]
    fn test_to_upper() {
        fold("'a'.toUpperCase()", "'A'");
        fold("'A'.toUpperCase()", "'A'");
        fold("'aBcDe'.toUpperCase()", "'ABCDE'");

        fold_same("`abc`.toUpperCase()");
        fold_same("`a ${bc}`.toUpperCase()");

        /*
         * Make sure things aren't totally broken for non-ASCII strings, non-exhaustive.
         *
         * <p>This includes things like:
         *
         * <ul>
         *   <li>graphemes with multiple code-points
         *   <li>graphemes represented by multiple graphemes in other cases
         *   <li>graphemes whose case changes are not round-trippable
         *   <li>graphemes that change case in a position sentitive way
         * </ul>
         */
        fold("'\u{0049}'.toUpperCase()", "'\u{0049}'");
        fold("'\u{0069}'.toUpperCase()", "'\u{0049}'");
        fold("'\u{0130}'.toUpperCase()", "'\u{0130}'");
        fold("'\u{0131}'.toUpperCase()", "'\u{0049}'");
        fold("'\u{0049}\u{0307}'.toUpperCase()", "'\u{0049}\u{0307}'");
        fold("'ß'.toUpperCase()", "'SS'");
        fold("'SS'.toUpperCase()", "'SS'");
        fold("'σ'.toUpperCase()", "'Σ'");
        fold("'σς'.toUpperCase()", "'ΣΣ'");
    }

    #[test]
    fn test_to_lower() {
        fold("'A'.toLowerCase()", "'a'");
        fold("'a'.toLowerCase()", "'a'");
        fold("'aBcDe'.toLowerCase()", "'abcde'");

        fold_same("`ABC`.toLowerCase()");
        fold_same("`A ${BC}`.toLowerCase()");

        /*
         * Make sure things aren't totally broken for non-ASCII strings, non-exhaustive.
         *
         * <p>This includes things like:
         *
         * <ul>
         *   <li>graphemes with multiple code-points
         *   <li>graphemes with multiple representations
         *   <li>graphemes represented by multiple graphemes in other cases
         *   <li>graphemes whose case changes are not round-trippable
         *   <li>graphemes that change case in a position sentitive way
         * </ul>
         */
        fold("'\u{0049}'.toLowerCase()", "'\u{0069}'");
        fold("'\u{0069}'.toLowerCase()", "'\u{0069}'");
        fold("'\u{0130}'.toLowerCase()", "'\u{0069}\u{0307}'");
        fold("'\u{0131}'.toLowerCase()", "'\u{0131}'");
        fold("'\u{0049}\u{0307}'.toLowerCase()", "'\u{0069}\u{0307}'");
        fold("'ß'.toLowerCase()", "'ß'");
        fold("'SS'.toLowerCase()", "'ss'");
        fold("'Σ'.toLowerCase()", "'σ'");
        fold("'ΣΣ'.toLowerCase()", "'σς'");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_bug() {
        fold_same("Math[0]()");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_abs() {
        fold_same("Math.abs(Math.random())");

        fold("Math.abs('-1')", "1");
        fold("Math.abs(-2)", "2");
        fold("Math.abs(null)", "0");
        fold("Math.abs('')", "0");
        fold("Math.abs([])", "0");
        fold("Math.abs([2])", "2");
        fold("Math.abs([1,2])", "NaN");
        fold("Math.abs({})", "NaN");
        fold("Math.abs('string');", "NaN");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_imul() {
        fold_same("Math.imul(Math.random(),2)");
        fold("Math.imul(-1,1)", "-1");
        fold("Math.imul(2,2)", "4");
        fold("Math.imul(2)", "0");
        fold("Math.imul(2,3,5)", "6");
        fold("Math.imul(0xfffffffe, 5)", "-10");
        fold("Math.imul(0xffffffff, 5)", "-5");
        fold("Math.imul(0xfffffffffffff34f, 0xfffffffffff342)", "13369344");
        fold("Math.imul(0xfffffffffffff34f, -0xfffffffffff342)", "-13369344");
        fold("Math.imul(NaN, 2)", "0");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_ceil() {
        fold_same("Math.ceil(Math.random())");

        fold("Math.ceil(1)", "1");
        fold("Math.ceil(1.5)", "2");
        fold("Math.ceil(1.3)", "2");
        fold("Math.ceil(-1.3)", "-1");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_floor() {
        fold_same("Math.floor(Math.random())");

        fold("Math.floor(1)", "1");
        fold("Math.floor(1.5)", "1");
        fold("Math.floor(1.3)", "1");
        fold("Math.floor(-1.3)", "-2");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_fround() {
        fold_same("Math.fround(Math.random())");

        fold("Math.fround(NaN)", "NaN");
        fold("Math.fround(Infinity)", "Infinity");
        fold("Math.fround(1)", "1");
        fold("Math.fround(0)", "0");
    }

    #[test]
    #[ignore]
    // @GwtIncompatible // TODO(b/155511629): Enable this test for J2CL
    fn test_fold_math_functions_fround_j2cl() {
        fold_same("Math.fround(1.2)");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_round() {
        fold_same("Math.round(Math.random())");
        fold("Math.round(NaN)", "NaN");
        fold("Math.round(3.5)", "4");
        fold("Math.round(-3.5)", "-3");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_sign() {
        fold_same("Math.sign(Math.random())");
        fold("Math.sign(NaN)", "NaN");
        fold("Math.sign(3.5)", "1");
        fold("Math.sign(-3.5)", "-1");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_trunc() {
        fold_same("Math.trunc(Math.random())");
        fold("Math.sign(NaN)", "NaN");
        fold("Math.trunc(3.5)", "3");
        fold("Math.trunc(-3.5)", "-3");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_clz32() {
        fold("Math.clz32(0)", "32");
        let mut x = 1;
        for i in (0..=31).rev() {
            fold(&format!("{x}.leading_zeros()"), &i.to_string());
            fold(&format!("{}.leading_zeros()", 2 * x - 1), &i.to_string());
            x *= 2;
        }
        fold("Math.clz32('52')", "26");
        fold("Math.clz32([52])", "26");
        fold("Math.clz32([52, 53])", "32");

        // Overflow cases
        fold("Math.clz32(0x100000000)", "32");
        fold("Math.clz32(0x100000001)", "31");

        // NaN -> 0
        fold("Math.clz32(NaN)", "32");
        fold("Math.clz32('foo')", "32");
        fold("Math.clz32(Infinity)", "32");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_max() {
        fold_same("Math.max(Math.random(), 1)");

        fold("Math.max()", "-Infinity");
        fold("Math.max(0)", "0");
        fold("Math.max(0, 1)", "1");
        fold("Math.max(0, 1, -1, 200)", "200");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_min() {
        fold_same("Math.min(Math.random(), 1)");

        fold("Math.min()", "Infinity");
        fold("Math.min(3)", "3");
        fold("Math.min(0, 1)", "0");
        fold("Math.min(0, 1, -1, 200)", "-1");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_pow() {
        fold("Math.pow(1, 2)", "1");
        fold("Math.pow(2, 0)", "1");
        fold("Math.pow(2, 2)", "4");
        fold("Math.pow(2, 32)", "4294967296");
        fold("Math.pow(Infinity, 0)", "1");
        fold("Math.pow(Infinity, 1)", "Infinity");
        fold("Math.pow('a', 33)", "NaN");
    }

    #[test]
    #[ignore]
    fn test_fold_number_functions_is_safe_integer() {
        fold("Number.isSafeInteger(1)", "true");
        fold("Number.isSafeInteger(1.5)", "false");
        fold("Number.isSafeInteger(9007199254740991)", "true");
        fold("Number.isSafeInteger(9007199254740992)", "false");
        fold("Number.isSafeInteger(-9007199254740991)", "true");
        fold("Number.isSafeInteger(-9007199254740992)", "false");
    }

    #[test]
    #[ignore]
    fn test_fold_number_functions_is_finite() {
        fold("Number.isFinite(1)", "true");
        fold("Number.isFinite(1.5)", "true");
        fold("Number.isFinite(NaN)", "false");
        fold("Number.isFinite(Infinity)", "false");
        fold("Number.isFinite(-Infinity)", "false");
        fold_same("Number.isFinite('a')");
    }

    #[test]
    #[ignore]
    fn test_fold_number_functions_is_nan() {
        fold("Number.isNaN(1)", "false");
        fold("Number.isNaN(1.5)", "false");
        fold("Number.isNaN(NaN)", "true");
        fold_same("Number.isNaN('a')");
        // unknown function may have side effects
        fold_same("Number.isNaN(+(void unknown()))");
    }

    #[test]
    #[ignore]
    fn test_fold_parse_numbers() {
        // Template Strings
        fold_same("x = parseInt(`123`)");
        fold_same("x = parseInt(` 123`)");
        fold_same("x = parseInt(`12 ${a}`)");
        fold_same("x = parseFloat(`1.23`)");

        // setAcceptedLanguage(LanguageMode.ECMASCRIPT5);

        fold("x = parseInt('123')", "x = 123");
        fold("x = parseInt(' 123')", "x = 123");
        fold("x = parseInt('123', 10)", "x = 123");
        fold("x = parseInt('0xA')", "x = 10");
        fold("x = parseInt('0xA', 16)", "x = 10");
        fold("x = parseInt('07', 8)", "x = 7");
        fold("x = parseInt('08')", "x = 8");
        fold("x = parseInt('0')", "x = 0");
        fold("x = parseInt('-0')", "x = -0");
        fold("x = parseFloat('0')", "x = 0");
        fold("x = parseFloat('1.23')", "x = 1.23");
        fold("x = parseFloat('-1.23')", "x = -1.23");
        fold("x = parseFloat('1.2300')", "x = 1.23");
        fold("x = parseFloat(' 0.3333')", "x = 0.3333");
        fold("x = parseFloat('0100')", "x = 100");
        fold("x = parseFloat('0100.000')", "x = 100");

        // Mozilla Dev Center test cases
        fold("x = parseInt(' 0xF', 16)", "x = 15");
        fold("x = parseInt(' F', 16)", "x = 15");
        fold("x = parseInt('17', 8)", "x = 15");
        fold("x = parseInt('015', 10)", "x = 15");
        fold("x = parseInt('1111', 2)", "x = 15");
        fold("x = parseInt('12', 13)", "x = 15");
        fold("x = parseInt(15.99, 10)", "x = 15");
        fold("x = parseInt(-15.99, 10)", "x = -15");
        // Java's Integer.parseInt("-15.99", 10) throws an exception, because of the decimal point.
        fold_same("x = parseInt('-15.99', 10)");
        fold("x = parseFloat('3.14')", "x = 3.14");
        fold("x = parseFloat(3.14)", "x = 3.14");
        fold("x = parseFloat(-3.14)", "x = -3.14");
        fold("x = parseFloat('-3.14')", "x = -3.14");
        fold("x = parseFloat('-0')", "x = -0");

        // Valid calls - unable to fold
        fold_same("x = parseInt('FXX123', 16)");
        fold_same("x = parseInt('15*3', 10)");
        fold_same("x = parseInt('15e2', 10)");
        fold_same("x = parseInt('15px', 10)");
        fold_same("x = parseInt('-0x08')");
        fold_same("x = parseInt('1', -1)");
        fold_same("x = parseFloat('3.14more non-digit characters')");
        fold_same("x = parseFloat('314e-2')");
        fold_same("x = parseFloat('0.0314E+2')");
        fold_same("x = parseFloat('3.333333333333333333333333')");

        // Invalid calls
        fold_same("x = parseInt('0xa', 10)");
        fold_same("x = parseInt('')");

        // setAcceptedLanguage(LanguageMode.ECMASCRIPT3);
        fold_same("x = parseInt('08')");
    }

    #[test]
    #[ignore]
    fn test_fold_parse_octal_numbers() {
        // setAcceptedLanguage(LanguageMode.ECMASCRIPT5);

        fold("x = parseInt('021', 8)", "x = 17");
        fold("x = parseInt('-021', 8)", "x = -17");
    }

    #[test]
    #[ignore]
    fn test_replace_with_char_at() {
        // enableTypeCheck();
        // replaceTypesWithColors();
        // disableCompareJsDoc();

        fold_string_typed("a.substring(0, 1)", "a.charAt(0)");
        fold_same_string_typed("a.substring(-4, -3)");
        fold_same_string_typed("a.substring(i, j + 1)");
        fold_same_string_typed("a.substring(i, i + 1)");
        fold_same_string_typed("a.substring(1, 2, 3)");
        fold_same_string_typed("a.substring()");
        fold_same_string_typed("a.substring(1)");
        fold_same_string_typed("a.substring(1, 3, 4)");
        fold_same_string_typed("a.substring(-1, 3)");
        fold_same_string_typed("a.substring(2, 1)");
        fold_same_string_typed("a.substring(3, 1)");

        fold_string_typed("a.slice(4, 5)", "a.charAt(4)");
        fold_same_string_typed("a.slice(-2, -1)");
        fold_string_typed("var /** number */ i; a.slice(0, 1)", "var /** number */ i; a.charAt(0)");
        fold_same_string_typed("a.slice(i, j + 1)");
        fold_same_string_typed("a.slice(i, i + 1)");
        fold_same_string_typed("a.slice(1, 2, 3)");
        fold_same_string_typed("a.slice()");
        fold_same_string_typed("a.slice(1)");
        fold_same_string_typed("a.slice(1, 3, 4)");
        fold_same_string_typed("a.slice(-1, 3)");
        fold_same_string_typed("a.slice(2, 1)");
        fold_same_string_typed("a.slice(3, 1)");

        fold_string_typed("a.substr(0, 1)", "a.charAt(0)");
        fold_string_typed("a.substr(2, 1)", "a.charAt(2)");
        fold_same_string_typed("a.substr(-2, 1)");
        fold_same_string_typed("a.substr(bar(), 1)");
        fold_same_string_typed("''.substr(bar(), 1)");
        fold_same_string_typed("a.substr(2, 1, 3)");
        fold_same_string_typed("a.substr(1, 2, 3)");
        fold_same_string_typed("a.substr()");
        fold_same_string_typed("a.substr(1)");
        fold_same_string_typed("a.substr(1, 2)");
        fold_same_string_typed("a.substr(1, 2, 3)");

        // enableTypeCheck();

        fold_same("function f(/** ? */ a) { a.substring(0, 1); }");
        fold_same("function f(/** ? */ a) { a.substr(0, 1); }");
        // fold_same(lines(
        //     "/** @constructor */ function A() {};",
        //     "A.prototype.substring = function(begin, end) {};",
        //     "function f(/** !A */ a) { a.substring(0, 1); }",
        // ));
        // fold_same(lines(
        //     "/** @constructor */ function A() {};",
        //     "A.prototype.slice = function(begin, end) {};",
        //     "function f(/** !A */ a) { a.slice(0, 1); }",
        // ));

        // useTypes = false;
        fold_same_string_typed("a.substring(0, 1)");
        fold_same_string_typed("a.substr(0, 1)");
        fold_same_string_typed("''.substring(i, i + 1)");
    }

    #[test]
    #[ignore]
    fn test_fold_concat_chaining() {
        // enableTypeCheck();

        fold("[1,2].concat(1).concat(2,['abc']).concat('abc')", "[1,2].concat(1,2,['abc'],'abc')");
        fold("[].concat(['abc']).concat(1).concat([2,3])", "['abc'].concat(1,[2,3])");

        // cannot fold concat based on type information
        fold_same("returnArrayType().concat(returnArrayType()).concat(1).concat(2)");
        fold_same("returnArrayType().concat(returnUnionType()).concat(1).concat(2)");
        fold(
            "[1,2,1].concat(1).concat(returnArrayType()).concat(2)",
            "[1,2,1].concat(1).concat(returnArrayType(),2)",
        );
        fold(
            "[1].concat(1).concat(2).concat(returnArrayType())",
            "[1].concat(1,2).concat(returnArrayType())",
        );
        fold_same("[].concat(1).concat(returnArrayType())");
        fold_same("obj.concat([1,2]).concat(1)");
    }

    #[test]
    #[ignore]
    fn test_remove_array_literal_from_front_of_concat() {
        // enableTypeCheck();

        fold("[].concat([1,2,3],1)", "[1,2,3].concat(1)");

        fold_same("[1,2,3].concat(returnArrayType())");
        // Call method with the same name as Array.prototype.concat
        fold_same("obj.concat([1,2,3])");

        fold_same("[].concat(1,[1,2,3])");
        fold_same("[].concat(1)");
        fold("[].concat([1])", "[1].concat()");

        // Chained folding of empty array lit
        fold("[].concat([], [1,2,3], [4])", "[1,2,3].concat([4])");
        fold("[].concat([]).concat([1]).concat([2,3])", "[1].concat([2,3])");

        // Cannot fold based on type information
        fold_same("[].concat(returnArrayType(),1)");
        fold_same("[].concat(returnArrayType())");
        fold_same("[].concat(returnUnionType())");
    }

    #[test]
    #[ignore]
    fn test_array_of_spread() {
        fold("x = Array.of(...['a', 'b', 'c'])", "x = [...['a', 'b', 'c']]");
        fold("x = Array.of(...['a', 'b', 'c',])", "x = [...['a', 'b', 'c']]");
        fold("x = Array.of(...['a'], ...['b', 'c'])", "x = [...['a'], ...['b', 'c']]");
        fold("x = Array.of('a', ...['b', 'c'])", "x = ['a', ...['b', 'c']]");
        fold("x = Array.of('a', ...['b', 'c'])", "x = ['a', ...['b', 'c']]");
    }

    #[test]
    #[ignore]
    fn test_array_of_no_spread() {
        fold("x = Array.of('a', 'b', 'c')", "x = ['a', 'b', 'c']");
        fold("x = Array.of('a', ['b', 'c'])", "x = ['a', ['b', 'c']]");
        fold("x = Array.of('a', ['b', 'c'],)", "x = ['a', ['b', 'c']]");
    }

    #[test]
    #[ignore]
    fn test_array_of_no_args() {
        fold("x = Array.of()", "x = []");
    }

    #[test]
    #[ignore]
    fn test_array_of_no_change() {
        fold_same("x = Array.of.apply(window, ['a', 'b', 'c'])");
        fold_same("x = ['a', 'b', 'c']");
        fold_same("x = [Array.of, 'a', 'b', 'c']");
    }

    #[test]
    #[ignore]
    fn test_fold_array_bug() {
        fold_same("Array[123]()");
    }

    fn fold_same_string_typed(js: &str) {
        fold_string_typed(js, js);
    }

    fn fold_string_typed(js: &str, expected: &str) {
        let left = "function f(/** string */ a) {".to_string() + js + "}";
        let right = "function f(/** string */ a) {".to_string() + expected + "}";
        test(left.as_str(), right.as_str());
    }
}
