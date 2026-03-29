use crate::{test, test_same, test_target};

#[track_caller]
fn test_value(code: &str, expected: &str) {
    test(format!("x = {code}").as_str(), format!("x = {expected}").as_str());
}

#[track_caller]
fn test_same_value(code: &str) {
    test_same(format!("x = {code}").as_str());
}

#[expect(clippy::literal_string_with_formatting_args)]
#[test]
fn test_string_index_of() {
    test("x = 'abcdef'.indexOf('g')", "x = -1");
    test("x = 'abcdef'.indexOf('b')", "x = 1");
    test("x = 'abcdefbe'.indexOf('b', 2)", "x = 6");
    test("x = 'abcdef'.indexOf('bcd')", "x = 1");
    test("x = 'abcdefsdfasdfbcdassd'.indexOf('bcd', 4)", "x = 13");
    test_same("x = 'abcdef'.indexOf(...a, 1)");
    test_same("x = 'abcdef'.indexOf('b', ...a)");
    test_same("x = 'abcdef'.indexOf(a, 1)");
    test_same("x = 'abcdef'.indexOf('b', a)");

    test("x = 'abcdef'.lastIndexOf('b')", "x = 1");
    test("x = 'abcdefbe'.lastIndexOf('b')", "x = 6");
    test("x = 'abcdefbe'.lastIndexOf('b', 5)", "x = 1");

    test("x = 'abc1def'.indexOf(1)", "x = 3");
    test("x = 'abcNaNdef'.indexOf(NaN)", "x = 3");
    test("x = 'abcundefineddef'.indexOf(undefined)", "x = 3");
    test("x = 'abcnulldef'.indexOf(null)", "x = 3");
    test("x = 'abctruedef'.indexOf(true)", "x = 3");

    test_same("x = 1 .indexOf('bcd');");
    test_same("x = NaN.indexOf('bcd')");
    test("x = undefined.indexOf('bcd')", "x = (void 0).indexOf('bcd')");
    test_same("x = null.indexOf('bcd')");
    test_same("x = (!0).indexOf('bcd')");
    test_same("x = (!1).indexOf('bcd')");

    // dealing with regex or other types.
    test("x = 'abcdef/b./'.indexOf(/b./)", "x = 6");
    test("x = 'abcdef[object Object]'.indexOf({a:2})", "x = 6");
    test("x = 'abcdef1,2'.indexOf([1,2])", "x = 6");

    // Template Strings
    test("x = `abcdef`.indexOf('b')", "x = 1");
    test_same("x = `Hello ${name}`.indexOf('a')");
    test_same("x = tag `Hello ${name}`.indexOf('a')");
}

#[test]
#[ignore = "TODO: Array.join optimization with sparse arrays not yet implemented"]
fn test_string_join_add_sparse() {
    test("x = [,,'a'].join(',')", "x = ',,a'");
}

#[test]
#[ignore = "TODO: Array.join optimization edge cases not yet implemented"]
fn test_no_string_join() {
    test_same("x = [].join(',',2)");
    test_same("x = [].join(f)");
}

#[test]
#[ignore = "TODO: Array.join to string concatenation optimization not yet implemented"]
fn test_string_join_add() {
    test("x = ['a', 'b', 'c'].join('')", "x = \"abc\"");
    test("x = [].join(',')", "x = \"\"");
    test("x = ['a'].join(',')", "x = \"a\"");
    test("x = ['a', 'b', 'c'].join(',')", "x = \"a,b,c\"");
    test("x = ['a', foo, 'b', 'c'].join(',')", "x = [\"a\",foo,\"b,c\"].join()");
    test("x = [foo, 'a', 'b', 'c'].join(',')", "x = [foo,\"a,b,c\"].join()");
    test("x = ['a', 'b', 'c', foo].join(',')", "x = [\"a,b,c\",foo].join()");

    // Works with numbers
    test("x = ['a=', 5].join('')", "x = \"a=5\"");
    test("x = ['a', '5'].join(7)", "x = \"a75\"");

    // Works on boolean
    test("x = ['a=', false].join('')", "x = \"a=false\"");
    test("x = ['a', '5'].join(true)", "x = \"atrue5\"");
    test("x = ['a', '5'].join(false)", "x = \"afalse5\"");

    // Only optimize if it's a size win.
    test(
        "x = ['a', '5', 'c'].join('a very very very long chain')",
        "x = [\"a\",\"5\",\"c\"].join(\"a very very very long chain\")",
    );

    // Template strings
    test("x = [`a`, `b`, `c`].join(``)", "x = 'abc'");
    test("x = [`a`, `b`, `c`].join('')", "x = 'abc'");

    // TODO(user): Its possible to fold this better.
    test_same("x = ['', foo].join('-')");
    test_same("x = ['', foo, ''].join()");

    test(
        "x = ['', '', foo, ''].join(',')", //
        "x = [ ','  , foo, ''].join()",
    );
    test(
        "x = ['', '', foo, '', ''].join(',')", //
        "x = [ ',',   foo,  ','].join()",
    );

    test(
        "x = ['', '', foo, '', '', bar].join(',')", //
        "x = [ ',',   foo,  ',',   bar].join()",
    );

    test(
        "x = [1,2,3].join('abcdef')", //
        "x = '1abcdef2abcdef3'",
    );

    test("x = [1,2].join()", "x = '1,2'");
    test("x = [null,undefined,''].join(',')", "x = ',,'");
    test("x = [null,undefined,0].join(',')", "x = ',,0'");
    // This can be folded but we don't currently.
    test_same("x = [[1,2],[3,4]].join()"); // would like: "x = '1,2,3,4'"
}

#[test]
#[ignore = "TODO: Array.join single element optimization not yet implemented"]
fn test_string_join_add_b1992789() {
    test("x = ['a'].join('')", "x = \"a\"");
    test_same("x = [foo()].join('')");
    test_same("[foo()].join('')");
    test("[null].join('')", "''");
}

#[test]
fn test_fold_string_replace() {
    test("x = 'c'.replace('c','x')", "x = 'x'");
    test("x = 'ac'.replace('c','x')", "x = 'ax'");
    test("x = 'ca'.replace('c','x')", "x = 'xa'");
    test("x = 'ac'.replace('c','xxx')", "x = 'axxx'");
    test("x = 'ca'.replace('c','xxx')", "x = 'xxxa'");
    test_same("x = 'c'.replace((foo(), 'c'), 'b')");

    test_same("x = '[object Object]'.replace({}, 'x')"); // can be folded to "x"
    test_same("x = 'a'.replace({ [Symbol.replace]() { return 'x' } }, 'c')"); // can be folded to "x"

    // only one instance replaced
    test("x = 'acaca'.replace('c','x')", "x = 'axaca'");
    test("x = 'ab'.replace('','x')", "x = 'xab'");

    test_same("'acaca'.replace(/c/,'x')"); // this will affect the global RegExp props
    test_same("'acaca'.replace(/c/g,'x')"); // this will affect the global RegExp props

    // not a literal
    test_same("x.replace('x','c')");

    test_same("'Xyz'.replace('Xyz', '$$')"); // would fold to '$'
    test_same("'PreXyzPost'.replace('Xyz', '$&')"); // would fold to 'PreXyzPost'
    test_same("'PreXyzPost'.replace('Xyz', '$`')"); // would fold to 'PrePrePost'
    test_same("'PreXyzPost'.replace('Xyz', '$\\'')"); // would fold to  'PrePostPost'
    test_same("'PreXyzPostXyz'.replace('Xyz', '$\\'')"); // would fold to 'PrePostXyzPostXyz'
    test_same("'123'.replace('2', '$`')"); // would fold to '113'
}

#[test]
fn test_fold_string_replace_all() {
    test("x = 'abcde'.replaceAll('bcd','c')", "x = 'ace'");
    test("x = 'abcde'.replaceAll('c','xxx')", "x = 'abxxxde'");
    test("x = 'abcde'.replaceAll('xxx','c')", "x = 'abcde'");
    test("x = 'ab'.replaceAll('','x')", "x = 'xaxbx'");

    test("x = 'c_c_c'.replaceAll('c','x')", "x = 'x_x_x'");
    test("x = 'acaca'.replaceAll('c',/x/)", "x = 'a/x/a/x/a'");

    test_same("x = '[object Object]'.replaceAll({}, 'x')"); // can be folded to "x"
    test_same("x = 'a'.replaceAll({ [Symbol.replace]() { return 'x' } }, 'c')"); // can be folded to "x"

    test_same("x = 'acaca'.replaceAll(/c/,'x')"); // this should throw
    test_same("x = 'acaca'.replaceAll(/c/g,'x')"); // this will affect the global RegExp props

    // not a literal
    test_same("x.replaceAll('x','c')");

    test_same("'Xyz'.replaceAll('Xyz', '$$')"); // would fold to '$'
    test_same("'PreXyzPost'.replaceAll('Xyz', '$&')"); // would fold to 'PreXyzPost'
    test_same("'PreXyzPost'.replaceAll('Xyz', '$`')"); // would fold to 'PrePrePost'
    test_same("'PreXyzPost'.replaceAll('Xyz', '$\\'')"); // would fold to  'PrePostPost'
    test_same("'PreXyzPostXyz'.replaceAll('Xyz', '$\\'')"); // would fold to 'PrePostXyzPost'
    test_same("'123'.replaceAll('2', '$`')"); // would fold to '113'
}

#[test]
fn test_fold_string_substring() {
    test("x = 'abcde'.substring(0,2)", "x = 'ab'");
    test("x = 'abcde'.substring(1,2)", "x = 'b'");
    test("x = 'abcde'.substring(2)", "x = 'cde'");
    test_same("x = 'abcde'.substring(...a, 1)");
    test_same("x = 'abcde'.substring(1, ...a)");
    test_same("x = 'abcde'.substring(a, 1)");
    test_same("x = 'abcde'.substring(1, a)");

    // we should be leaving negative, out-of-bound, and inverted indices alone for now
    test_same("x = 'abcde'.substring(-1)");
    test_same("x = 'abcde'.substring(1, -2)");
    test_same("x = 'abcde'.substring(1, 2, 3)");
    test_same("x = 'abcde'.substring(2, 0)");
    test_same("x = 'a'.substring(0, 2)");

    // Template strings
    test("x = `abcdef`.substring(0,2)", "x = 'ab'");
    test_same("x = `abcdef ${abc}`.substring(0,2)");
}

#[test]
fn test_fold_string_slice() {
    test("x = 'abcde'.slice(0,2)", "x = 'ab'");
    test("x = 'abcde'.slice(1,2)", "x = 'b'");
    test("x = 'abcde'.slice(2)", "x = 'cde'");

    // we should be leaving negative, out-of-bound, and inverted indices alone for now
    test_same("x = 'abcde'.slice(-1)");
    test_same("x = 'abcde'.slice(1, -2)");
    test_same("x = 'abcde'.slice(1, 2, 3)");
    test_same("x = 'abcde'.slice(2, 0)");
    test_same("x = 'a'.slice(0, 2)");

    // Template strings
    test("x = `abcdef`.slice(0, 2)", "x = 'ab'");
    test_same("x = `abcdef ${abc}`.slice(0,2)");
}

#[test]
fn test_fold_string_char_at() {
    test("x = 'abcde'.charAt(0)", "x = 'a'");
    test("x = 'abcde'.charAt(1)", "x = 'b'");
    test("x = 'abcde'.charAt(2)", "x = 'c'");
    test("x = 'abcde'.charAt(3)", "x = 'd'");
    test("x = 'abcde'.charAt(4)", "x = 'e'");
    test("x = 'abcde'.charAt(5)", "x = ''");
    test("x = 'abcde'.charAt(-1)", "x = ''");
    test("x = 'abcde'.charAt()", "x = 'a'");
    test_same("x = 'abcde'.charAt(...foo)");
    test_same("x = 'abcde'.charAt(0, ++z)");
    test_same("x = 'abcde'.charAt(y)");
    test("x = 'abcde'.charAt(null)", "x = 'a'");
    test("x = 'abcde'.charAt(!0)", "x = 'b'");
    test_same("x = '\\ud834\\udd1e'.charAt(0)"); // or x = '\\ud834'
    test_same("x = '\\ud834\\udd1e'.charAt(1)"); // or x = '\\udd1e'

    // Template strings
    test("x = `abcdef`.charAt(0)", "x = 'a'");
    test_same("x = `abcdef ${abc}`.charAt(0)");
}

#[test]
fn test_fold_string_char_code_at() {
    test("x = 'abcde'.charCodeAt()", "x = 97");
    test("x = 'abcde'.charCodeAt(0)", "x = 97");
    test("x = 'abcde'.charCodeAt(1)", "x = 98");
    test("x = 'abcde'.charCodeAt(2)", "x = 99");
    test("x = 'abcde'.charCodeAt(3)", "x = 100");
    test("x = 'abcde'.charCodeAt(4)", "x = 101");
    test("x = 'abcde'.charCodeAt(5)", "x = NaN");
    test("x = 'abcde'.charCodeAt(-1)", "x = NaN");
    test_same("x = 'abcde'.charCodeAt(...foo)");
    test_same("x = 'abcde'.charCodeAt(y)");
    test("x = 'abcde'.charCodeAt()", "x = 97");
    test("x = 'abcde'.charCodeAt(0, ++z)", "x = 97");
    test("x = 'abcde'.charCodeAt(null)", "x = 97");
    test("x = 'abcde'.charCodeAt(true)", "x = 98");
    test("x = '\\ud834\\udd1e'.charCodeAt(0)", "x = 55348");
    test("x = '\\ud834\\udd1e'.charCodeAt(1)", "x = 56606");
    test("x = `abcdef`.charCodeAt(0)", "x = 97");
    test_same("x = `abcdef ${abc}`.charCodeAt(0)");
}

#[test]
#[ignore = "TODO: String.split optimization not yet implemented"]
fn test_fold_string_split() {
    // late = false;
    test("x = 'abcde'.split('foo')", "x = ['abcde']");
    test("x = 'abcde'.split()", "x = ['abcde']");
    test("x = 'abcde'.split(null)", "x = ['abcde']");
    test("x = 'a b c d e'.split(' ')", "x = ['a','b','c','d','e']");
    test("x = 'a b c d e'.split(' ', 0)", "x = []");
    test("x = 'abcde'.split('cd')", "x = ['ab','e']");
    test("x = 'a b c d e'.split(' ', 1)", "x = ['a']");
    test("x = 'a b c d e'.split(' ', 3)", "x = ['a','b','c']");
    test("x = 'a b c d e'.split(null, 1)", "x = ['a b c d e']");
    test("x = 'aaaaa'.split('a')", "x = ['', '', '', '', '', '']");
    test("x = 'xyx'.split('x')", "x = ['', 'y', '']");

    // Empty separator
    test("x = 'abcde'.split('')", "x = ['a','b','c','d','e']");
    test("x = 'abcde'.split('', 3)", "x = ['a','b','c']");

    // Empty separator AND empty string
    test("x = ''.split('')", "x = []");

    // Separator equals string
    test("x = 'aaa'.split('aaa')", "x = ['','']");
    test("x = ' '.split(' ')", "x = ['','']");

    test_same("x = 'abcde'.split(/ /)");
    test_same("x = 'abcde'.split(' ', -1)");

    // Template strings
    test_same("x = `abcdef`.split()");
    test_same("x = `abcdef ${abc}`.split()");

    // late = true;
    // test_same("x = 'a b c d e'.split(' ')");
}

#[test]
#[ignore = "TODO: Array.join edge case optimization not yet implemented"]
fn test_join_bug() {
    test("var x = [].join();", "var x = '';");
    test_same("var x = [x].join();");
    test_same("var x = [x,y].join();");
    test_same("var x = [x,y,z].join();");

    // test_same(
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
#[ignore = "TODO: Array.join with spread syntax optimization not yet implemented"]
fn test_join_spread1() {
    test_same("var x = [...foo].join('');");
    test_same("var x = [...someMap.keys()].join('');");
    test_same("var x = [foo, ...bar].join('');");
    test_same("var x = [...foo, bar].join('');");
    test_same("var x = [...foo, 'bar'].join('');");
    test_same("var x = ['1', ...'2', '3'].join('');");
    test_same("var x = ['1', ...['2'], '3'].join('');");
}

#[test]
#[ignore = "TODO: Array.join with spread syntax optimization not yet implemented"]
fn test_join_spread2() {
    test("var x = [...foo].join(',');", "var x = [...foo].join();");
    test("var x = [...someMap.keys()].join(',');", "var x = [...someMap.keys()].join();");
    test("var x = [foo, ...bar].join(',');", "var x = [foo, ...bar].join();");
    test("var x = [...foo, bar].join(',');", "var x = [...foo, bar].join();");
    test("var x = [...foo, 'bar'].join(',');", "var x = [...foo, 'bar'].join();");
    test("var x = ['1', ...'2', '3'].join(',');", "var x = ['1', ...'2', '3'].join();");
    test("var x = ['1', ...['2'], '3'].join(',');", "var x = ['1', ...['2'], '3'].join();");
}

#[test]
fn test_to_upper() {
    test("x = 'a'.toUpperCase()", "x = 'A'");
    test("x = 'A'.toUpperCase()", "x = 'A'");
    test("x = 'aBcDe'.toUpperCase()", "x = 'ABCDE'");

    test("x = `abc`.toUpperCase()", "x = 'ABC'");
    test_same("`a ${bc}`.toUpperCase()");

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
    test("x = '\u{0049}'.toUpperCase()", "x = '\u{0049}'");
    test("x = '\u{0069}'.toUpperCase()", "x = '\u{0049}'");
    test("x = '\u{0130}'.toUpperCase()", "x = '\u{0130}'");
    test("x = '\u{0131}'.toUpperCase()", "x = '\u{0049}'");
    test("x = '\u{0049}\u{0307}'.toUpperCase()", "x = '\u{0049}\u{0307}'");
    test("x = 'ß'.toUpperCase()", "x = 'SS'");
    test("x = 'SS'.toUpperCase()", "x = 'SS'");
    test("x = 'σ'.toUpperCase()", "x = 'Σ'");
    test("x = 'σς'.toUpperCase()", "x = 'ΣΣ'");
}

#[test]
fn test_to_lower() {
    test("x = 'A'.toLowerCase()", "x = 'a'");
    test("x = 'a'.toLowerCase()", "x = 'a'");
    test("x = 'aBcDe'.toLowerCase()", "x = 'abcde'");

    test("x = `ABC`.toLowerCase()", "x = 'abc'");
    test_same("`A ${BC}`.toLowerCase()");

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
    test("x = '\u{0049}'.toLowerCase()", "x = '\u{0069}'");
    test("x = '\u{0069}'.toLowerCase()", "x = '\u{0069}'");
    test("x = '\u{0130}'.toLowerCase()", "x = '\u{0069}\u{0307}'");
    test("x = '\u{0131}'.toLowerCase()", "x = '\u{0131}'");
    test("x = '\u{0049}\u{0307}'.toLowerCase()", "x = '\u{0069}\u{0307}'");
    test("x = 'ß'.toLowerCase()", "x = 'ß'");
    test("x = 'SS'.toLowerCase()", "x = 'ss'");
    test("x = 'Σ'.toLowerCase()", "x = 'σ'");
    test("x = 'ΣΣ'.toLowerCase()", "x = 'σς'");
}

#[test]
fn test_fold_string_trim() {
    test("x = '  abc  '.trim()", "x = 'abc'");
    test("x = 'abc'.trim()", "x = 'abc'");
    test_same("x = 'abc'.trim(1)");

    test("x = '  abc  '.trimStart()", "x = 'abc  '");
    test("x = 'abc'.trimStart()", "x = 'abc'");
    test_same("x = 'abc'.trimStart(1)");

    test("x = '  abc  '.trimEnd()", "x = '  abc'");
    test("x = 'abc'.trimEnd()", "x = 'abc'");
    test_same("x = 'abc'.trimEnd(1)");
}

#[test]
fn test_fold_math_functions_bug() {
    test_same("Math[0]()");
}

#[test]
fn test_fold_math_functions_abs() {
    test_same_value("Math.abs(Math.random())");

    test_value("Math.abs('-1')", "1");
    test_value("Math.abs(-2)", "2");
    test_value("Math.abs(null)", "0");
    test_value("Math.abs('')", "0");
    test_value("Math.abs(NaN)", "NaN");
    test_value("Math.abs(-0)", "0");
    test_value("Math.abs(-Infinity)", "Infinity");
    test_value("Math.abs([])", "0");
    test_value("Math.abs([2])", "2");
    test_value("Math.abs([1,2])", "NaN");
    test_value("Math.abs({})", "NaN");
    test_value("Math.abs('string');", "NaN");
}

#[test]
fn test_fold_math_functions_imul() {
    test_same_value("Math.imul(Math.random(),2)");
    test_value("Math.imul()", "0");
    test_value("Math.imul(-1,1)", "-1");
    test_value("Math.imul(2,2)", "4");
    test_value("Math.imul(2)", "0");
    test_value("Math.imul(2,3,5)", "6");
    test_value("Math.imul(0xfffffffe, 5)", "-10");
    test_value("Math.imul(0xffffffff, 5)", "-5");
    test_value("Math.imul(0xfffffffffffff34f, 0xfffffffffff342)", "13369344");
    test_value("Math.imul(0xfffffffffffff34f, -0xfffffffffff342)", "-13369344");
    test_value("Math.imul(NaN, 2)", "0");
}

#[test]
fn test_fold_math_functions_ceil() {
    test_same_value("Math.ceil(Math.random())");

    test_value("Math.ceil(1)", "1");
    test_value("Math.ceil(1.5)", "2");
    test_value("Math.ceil(1.3)", "2");
    test_value("Math.ceil(-1.3)", "-1");
}

#[test]
fn test_fold_math_functions_floor() {
    test_same_value("Math.floor(Math.random())");

    test_value("Math.floor(1)", "1");
    test_value("Math.floor(1.5)", "1");
    test_value("Math.floor(1.3)", "1");
    test_value("Math.floor(-1.3)", "-2");
}

#[test]
fn test_fold_math_functions_fround() {
    test_same_value("Math.fround(Math.random())");

    test_value("Math.fround(NaN)", "NaN");
    test_value("Math.fround(Infinity)", "Infinity");
    test_value("Math.fround(-Infinity)", "-Infinity");
    test_value("Math.fround(1)", "1");
    test_value("Math.fround(0)", "0");
    test_value("Math.fround(16777217)", "16777216");
    test_value("Math.fround(16777218)", "16777218");
}

#[test]
fn test_fold_math_functions_fround_j2cl() {
    test_same_value("Math.fround(1.2)");
}

#[test]
fn test_fold_math_functions_round() {
    test_same_value("Math.round(Math.random())");
    test_value("Math.round(NaN)", "NaN");
    test_value("Math.round(3)", "3");
    test_value("Math.round(3.5)", "4");
    test_value("Math.round(-3.5)", "-3");
}

#[test]
fn test_fold_math_functions_sign() {
    test_same_value("Math.sign(Math.random())");
    test_value("Math.sign(NaN)", "NaN");
    test_value("Math.sign(0.0)", "0");
    test_value("Math.sign(-0.0)", "-0");
    test_value("Math.sign(0.01)", "1");
    test_value("Math.sign(-0.01)", "-1");
    test_value("Math.sign(3.5)", "1");
    test_value("Math.sign(-3.5)", "-1");
}

#[test]
fn test_fold_math_functions_trunc() {
    test_same_value("Math.trunc(Math.random())");
    test_value("Math.sign(NaN)", "NaN");
    test_value("Math.trunc(3.5)", "3");
    test_value("Math.trunc(-3.5)", "-3");
    test_value("Math.trunc(0.5)", "0");
    test_value("Math.trunc(-0.5)", "-0");
}

#[test]
fn test_fold_math_functions_clz32() {
    test_value("Math.clz32(0)", "32");
    test_value("Math.clz32(0.0)", "32");
    test_value("Math.clz32(-0.0)", "32");
    let mut x = 1_i64;
    for i in (0..=31).rev() {
        test_value(&format!("Math.clz32({x})"), &i.to_string());
        test_value(&format!("Math.clz32({})", 2 * x - 1), &i.to_string());
        x *= 2;
    }
    test_value("Math.clz32('52')", "26");
    test_value("Math.clz32([52])", "26");
    test_value("Math.clz32([52, 53])", "32");

    // Overflow cases
    test_value("Math.clz32(0x100000000)", "32");
    test_value("Math.clz32(0x100000001)", "31");

    // Negative cases
    test_value("Math.clz32(-1)", "0");
    test_value("Math.clz32(-2147483647)", "0");
    test_value("Math.clz32(-2147483649)", "1");

    // NaN -> 0
    test_value("Math.clz32(NaN)", "32");
    test_value("Math.clz32('foo')", "32");
    test_value("Math.clz32(Infinity)", "32");
}

#[test]
fn test_fold_math_functions_max() {
    test_same_value("Math.max(Math.random(), 1)");

    test_value("Math.max()", "-Infinity");
    test_value("Math.max(0)", "0");
    test_value("Math.max(0, 1)", "1");
    test_value("Math.max(0, 1, -1, 200)", "200");
    test_value("Math.max(0, -1, -Infinity)", "0");
    test_value("Math.max(0, -1, -Infinity, NaN)", "NaN");
    test_value("Math.max(0, -0)", "0");
    test_value("Math.max(-0, 0)", "0");
    test_same_value("Math.max(...a, 1)");
}

#[test]
fn test_fold_math_functions_min() {
    test_same_value("Math.min(Math.random(), 1)");

    test_value("Math.min()", "Infinity");
    test_value("Math.min(3)", "3");
    test_value("Math.min(0, 1)", "0");
    test_value("Math.min(0, 1, -1, 200)", "-1");
    test_value("Math.min(0, -1, -Infinity)", "-Infinity");
    test_value("Math.min(0, -1, -Infinity, NaN)", "NaN");
    test_value("Math.min(0, -0)", "-0");
    test_value("Math.min(-0, 0)", "-0");
    test_same_value("Math.min(...a, 1)");
}

#[test]
#[ignore = "TODO: Math.pow optimization not yet implemented"]
fn test_fold_math_functions_pow() {
    test("Math.pow(1, 2)", "1");
    test("Math.pow(2, 0)", "1");
    test("Math.pow(2, 2)", "4");
    test("Math.pow(2, 32)", "4294967296");
    test("Math.pow(Infinity, 0)", "1");
    test("Math.pow(Infinity, 1)", "Infinity");
    test("Math.pow('a', 33)", "NaN");
}

#[test]
fn test_fold_number_functions_is_safe_integer() {
    test_value("Number.isSafeInteger(1)", "!0");
    test_value("Number.isSafeInteger(1.5)", "!1");
    test_value("Number.isSafeInteger(9007199254740991)", "!0");
    test_value("Number.isSafeInteger(9007199254740992)", "!1");
    test_value("Number.isSafeInteger(-9007199254740991)", "!0");
    test_value("Number.isSafeInteger(-9007199254740992)", "!1");
}

#[test]
fn test_fold_number_functions_is_finite() {
    test_value("Number.isFinite(1)", "!0");
    test_value("Number.isFinite(1.5)", "!0");
    test_value("Number.isFinite(NaN)", "!1");
    test_value("Number.isFinite(Infinity)", "!1");
    test_value("Number.isFinite(-Infinity)", "!1");
    test_same_value("Number.isFinite('a')");
}

#[test]
fn test_fold_number_functions_is_nan() {
    test_value("Number.isNaN(1)", "!1");
    test_value("Number.isNaN(1.5)", "!1");
    test_value("Number.isNaN(NaN)", "!0");
    test_same_value("Number.isNaN('a')");
    // unknown function may have side effects
    test_same_value("Number.isNaN(+(void unknown()))");
}

#[test]
fn test_fold_parse_numbers() {
    test("x = parseInt('123')", "x = 123");
    test("x = parseInt(`123`)", "x = 123");
    test("x = parseInt(` 123`)", "x = 123");
    test_same("x = parseInt(`12 ${a}`)");
    test("x = parseFloat('1.23')", "x = 1.23");
    test("x = parseFloat(`1.23`)", "x = 1.23");
    test_same("x = parseFloat(`1.${a}`)");

    test("x = parseInt('123')", "x = 123");
    test("x = parseInt(' 123')", "x = 123");
    test("x = parseInt('123', 10)", "x = 123");
    test("x = parseInt('0xA')", "x = 10");
    test("x = parseInt('0xA', 16)", "x = 10");
    test("x = parseInt('07', 8)", "x = 7");
    test("x = parseInt('08')", "x = 8");
    test("x = parseInt('0')", "x = 0");
    test("x = parseInt('-0')", "x = -0");
    test("x = parseFloat('0')", "x = 0");
    test("x = parseFloat('1.23')", "x = 1.23");
    test("x = parseFloat('-1.23')", "x = -1.23");
    test("x = parseFloat('1.2300')", "x = 1.23");
    test("x = parseFloat(' 0.3333')", "x = 0.3333");
    test("x = parseFloat('0100')", "x = 100");
    test("x = parseFloat('0100.000')", "x = 100");

    // Mozilla Dev Center test cases
    test("x = parseInt(' 0xF', 16)", "x = 15");
    test("x = parseInt(' F', 16)", "x = 15");
    test("x = parseInt('17', 8)", "x = 15");
    test("x = parseInt('015', 10)", "x = 15");
    test("x = parseInt('1111', 2)", "x = 15");
    test_same("x = parseInt('12', 13)");
    test("x = parseInt(15.99, 10)", "x = 15");
    test("x = parseInt(-15.99, 10)", "x = -15");
    test("x = parseInt('-15.99', 10)", "x = -15");
    test("x = parseFloat('3.14')", "x = 3.14");
    test("x = parseFloat(3.14)", "x = 3.14");
    test("x = parseFloat(-3.14)", "x = -3.14");
    test("x = parseFloat('-3.14')", "x = -3.14");
    test("x = parseFloat('-0')", "x = -0");

    test("x = parseInt('FXX123', 16)", "x = 15"); // Parses 'F' (15 in hex)
    test("x = parseInt('15*3', 10)", "x = 15"); // Parses '15', stops at '*'
    test("x = parseInt('15e2', 10)", "x = 15"); // Parses '15', stops at 'e'
    test("x = parseInt('15px', 10)", "x = 15"); // Parses '15', stops at 'p'
    test("x = parseInt('-0x08')", "x = -8");
    test("x = parseInt('1', -1)", "x = NaN");
    test("x = parseFloat('3.14more non-digit characters')", "x = 3.14"); // Parses '3.14', stops at 'm'
    test("x = parseFloat('314e-2')", "x = 3.14");
    test("x = parseFloat('0.0314E+2')", "x = 3.14");
    test("x = parseFloat('3.333333333333333333333333')", "x = 3.3333333333333335");

    test("x = parseInt('0xa', 10)", "x = 0"); // Parses '0' in base 10, stops at 'x'
    test("x = parseInt('')", "x = NaN");
}

#[test]
fn test_fold_parse_octal_numbers() {
    test("x = parseInt('021', 8)", "x = 17");
    test("x = parseInt('-021', 8)", "x = -17");
}

#[test]
fn test_fold_parse_numbers_additional() {
    test_value("parseInt('+1')", "1");
    test_value("parseFloat('+1')", "1");
    test_value("parseInt('10', 0)", "10");
    test_value("parseInt('0x10', 16)", "16");
    test_value("parseInt('')", "NaN");
    test_value("parseInt(' ')", "NaN");
    test_value("parseInt('abc')", "NaN");
    test_value("parseFloat('')", "NaN");
    test_value("parseFloat(' ')", "NaN");
    test_value("parseFloat('abc')", "NaN");
    test_value("parseFloat('Infinity')", "Infinity");
    test_value("parseFloat('-Infinity')", "-Infinity");
    test_value("parseFloat('+Infinity')", "Infinity");
    test_same_value("parseInt(unknown)");
    test_same_value("parseInt((foo, '0'))"); // foo may have side effects
    test_same_value("parseFloat(unknown)");
    test_same_value("parseFloat((foo, '0'))"); // foo may have side effects
}

#[test]
#[ignore = "TODO: String charAt replacement optimization not yet implemented"]
fn test_replace_with_char_at() {
    // enableTypeCheck();
    // replaceTypesWithColors();
    // disableCompareJsDoc();

    fold_string_typed("a.substring(0, 1)", "a.charAt(0)");
    test_same_string_typed("a.substring(-4, -3)");
    test_same_string_typed("a.substring(i, j + 1)");
    test_same_string_typed("a.substring(i, i + 1)");
    test_same_string_typed("a.substring(1, 2, 3)");
    test_same_string_typed("a.substring()");
    test_same_string_typed("a.substring(1)");
    test_same_string_typed("a.substring(1, 3, 4)");
    test_same_string_typed("a.substring(-1, 3)");
    test_same_string_typed("a.substring(2, 1)");
    test_same_string_typed("a.substring(3, 1)");

    fold_string_typed("a.slice(4, 5)", "a.charAt(4)");
    test_same_string_typed("a.slice(-2, -1)");
    fold_string_typed("var /** number */ i; a.slice(0, 1)", "var /** number */ i; a.charAt(0)");
    test_same_string_typed("a.slice(i, j + 1)");
    test_same_string_typed("a.slice(i, i + 1)");
    test_same_string_typed("a.slice(1, 2, 3)");
    test_same_string_typed("a.slice()");
    test_same_string_typed("a.slice(1)");
    test_same_string_typed("a.slice(1, 3, 4)");
    test_same_string_typed("a.slice(-1, 3)");
    test_same_string_typed("a.slice(2, 1)");
    test_same_string_typed("a.slice(3, 1)");

    // enableTypeCheck();

    test_same("function f(/** ? */ a) { a.substring(0, 1); }");
    // test_same(lines(
    //     "/** @constructor */ function A() {};",
    //     "A.prototype.substring = function(begin, end) {};",
    //     "function f(/** !A */ a) { a.substring(0, 1); }",
    // ));
    // test_same(lines(
    //     "/** @constructor */ function A() {};",
    //     "A.prototype.slice = function(begin, end) {};",
    //     "function f(/** !A */ a) { a.slice(0, 1); }",
    // ));

    // useTypes = false;
    test_same_string_typed("a.substring(0, 1)");
    test_same_string_typed("''.substring(i, i + 1)");
}

#[test]
fn test_fold_concat_chaining() {
    // array
    test("x = [1,2].concat(1).concat(2,['abc']).concat('abc')", "x = [1,2,1,2,'abc','abc']");
    test("x = [].concat(['abc']).concat(1).concat([2,3])", "x = ['abc',1,2,3]");
    test("x = [].concat(1).concat(2).join(',')", "x = [1,2].join(',')");

    test("var x, y; [1].concat(x).concat(y)", "var x, y; [1].concat(x, y)");
    test("var y; [1].concat(x).concat(y)", "var y; [1].concat(x, y)"); // x might have a getter that updates y, but that side effect is preserved correctly
    test("var x; [1].concat(x.a).concat(x)", "var x; [1].concat(x.a, x)"); // x.a might have a getter that updates x, but that side effect is preserved correctly
    test_same("x = [].map(a => a + 1).concat(1)");

    // string
    test("x = '1'.concat(1).concat(2,['abc']).concat('abc')", "x = '112abcabc'");
    test("x = ''.concat(['abc']).concat(1).concat([2,3])", "x = 'abc12,3'");
    test("x = ''.concat(1)", "x = '1'");
    test("x = ''.concat('a', ' ').concat('b').split(/[\\s\\n]+/)", "x = 'a b'.split(/[\\s\\n]+/)");
    test_same("x = ''.split().concat(1)");

    test("var x, y; v = ''.concat(x).concat(y)", "var x, y; v = `${x}${y}`");
    test("var y; v = ''.concat(x).concat(y)", "var y; v = `${x}${y}`"); // x might have a getter that updates y, but that side effect is preserved correctly
    test("var x; v = ''.concat(x.a).concat(x)", "var x; v = `${x.a}${x}`"); // x.a might have a getter that updates x, but that side effect is preserved correctly

    // other
    test("x = []['concat'](1)", "x = [1]");
    test("x = ''['concat'](1)", "x = '1'");
    test_same("x = obj.concat([1,2]).concat(1)");
}

#[test]
fn test_add_template_literal() {
    test("x = '$' + `{${x}}`", "x = `\\${${x}}`");
    test("x = `{${x}}` + '$'", "x = `{${x}}\\$`");
    test("x = `$` + `{${x}}`", "x = `\\${${x}}`");
    test("x = `{${x}}` + `$`", "x = `{${x}}\\$`");
}

#[test]
fn test_remove_array_literal_from_front_of_concat() {
    test("x = [].concat([1,2,3],1)", "x = [1,2,3,1]");

    test_same("[1,2,3].concat(foo())");
    // Call method with the same name as Array.prototype.concat
    test_same("obj.concat([1,2,3])");

    test("x = [].concat(1,[1,2,3])", "x = [1,1,2,3]");
    test("x = [].concat(1)", "x = [1]");
    test("x = [].concat([1])", "x = [1]");

    // Chained folding of empty array lit
    test("x = [].concat([], [1,2,3], [4])", "x = [1,2,3,4]");
    test("x = [].concat([]).concat([1]).concat([2,3])", "x = [1,2,3]");

    test("x = [].concat(1, x)", "x = [1].concat(x)"); // x might be an array or an object with `Symbol.isConcatSpreadable`
    test("x = [].concat(1, ...x)", "x = [1].concat(...x)");
    test_same("x = [].concat(x, 1)");
}

#[test]
fn test_array_of_spread() {
    // Here, since our tests are fully opened, the dce may automatically optimize it into a simple array, instead of simply substitute the function call.
    test("x = Array.of(...['a', 'b', 'c'])", "x = ['a', 'b', 'c']");
    test("x = Array.of(...['a', 'b', 'c',])", "x = ['a', 'b', 'c']");
    test("x = Array.of(...['a'], ...['b', 'c'])", "x = ['a', 'b', 'c']");
    test("x = Array.of('a', ...['b', 'c'])", "x = ['a', 'b', 'c']");
    test("x = Array.of('a', ...['b', 'c'])", "x = ['a', 'b', 'c']");
}

#[test]
fn test_array_of_no_spread() {
    test("x = Array.of('a', 'b', 'c')", "x = ['a', 'b', 'c']");
    test("x = Array.of('a', ['b', 'c'])", "x = ['a', ['b', 'c']]");
    test("x = Array.of('a', ['b', 'c'],)", "x = ['a', ['b', 'c']]");
}

#[test]
fn test_array_of_no_args() {
    test("x = Array.of()", "x = []");
}

#[test]
fn test_array_of_no_change() {
    test_same("x = Array.of.apply(window, ['a', 'b', 'c'])");
    test_same("x = ['a', 'b', 'c']");
    test_same("x = [Array.of, 'a', 'b', 'c']");
}

#[test]
fn test_fold_array_bug() {
    test_same("Array[123]()");
}

fn test_same_string_typed(js: &str) {
    fold_string_typed(js, js);
}

fn fold_string_typed(js: &str, expected: &str) {
    let left = "function f(/** string */ a) {".to_string() + js + "}";
    let right = "function f(/** string */ a) {".to_string() + expected + "}";
    test(left.as_str(), right.as_str());
}

#[test]
fn test_fold_string_from_char_code() {
    test("x = String.fromCharCode()", "x = ''");
    test("x = String.fromCharCode(0)", "x = '\\0'");
    test("x = String.fromCharCode(120)", "x = 'x'");
    test("x = String.fromCharCode(120, 121)", "x = 'xy'");
    test_same("x = String.fromCharCode(55358, 56768)");
    test("x = String.fromCharCode(0x10000)", "x = '\\0'");
    test("x = String.fromCharCode(0x10078, 0x10079)", "x = 'xy'");
    test("x = String.fromCharCode(0x1_0000_FFFF)", "x = '\u{ffff}'");
    test("x = String.fromCharCode(NaN)", "x = '\\0'");
    test("x = String.fromCharCode(-Infinity)", "x = '\\0'");
    test("x = String.fromCharCode(Infinity)", "x = '\\0'");
    test("x = String.fromCharCode(null)", "x = '\\0'");
    test("x = String.fromCharCode(undefined)", "x = '\\0'");
    test("x = String.fromCharCode('123')", "x = '{'");
    test_same("String.fromCharCode(x)");
    test("x = String.fromCharCode('x')", "x = '\\0'");
    test("x = String.fromCharCode('0.5')", "x = '\\0'");

    test_same("x = Unknown.fromCharCode('0.5')");
}

#[test]
fn test_fold_string_concat() {
    test_same("x = ''.concat()");
    test("x = ''.concat(a, b)", "x = `${a}${b}`");
    test("x = ''.concat(a, b, c)", "x = `${a}${b}${c}`");
    test("x = ''.concat(a, b, c, d)", "x = `${a}${b}${c}${d}`");
    test_same("x = ''.concat(a, b, c, d, e)");
    test("x = ''.concat('a')", "x = 'a'");
    test("x = ''.concat('a', 'b')", "x = 'ab'");
    test("x = ''.concat('a', 'b', 'c')", "x = 'abc'");
    test("x = ''.concat('a', 'b', 'c', 'd')", "x = 'abcd'");
    test("x = ''.concat('a', 'b', 'c', 'd', 'e')", "x = 'abcde'");
    test("x = ''.concat(a, 'b')", "x = `${a}b`");
    test("x = ''.concat('a', b)", "x = `a${b}`");
    test("x = ''.concat(a, 'b', c)", "x = `${a}b${c}`");
    test("x = ''.concat('a', b, 'c')", "x = `a${b}c`");
    test(
        "x = ''.concat('a', b, 'c', d, 'e', f, 'g', h, 'i', j, 'k', l, 'm', n, 'o', p, 'q', r, 's', t)",
        "x = `a${b}c${d}e${f}g${h}i${j}k${l}m${n}o${p}q${r}s${t}`",
    );
    test("x = ''.concat(a, 1)", "x = `${a}1`");

    test("x = '\\\\s'.concat(a)", "x = `\\\\s${a}`");
    test("x = '`'.concat(a)", "x = `\\`${a}`");
    test("x = '${'.concat(a)", "x = `\\${${a}`");
}

#[test]
fn test_to_string() {
    test("x = false['toString']()", "x = 'false';");
    test("x = false.toString()", "x = 'false';");
    test("x = true.toString()", "x = 'true';");
    test("x = (!0).toString()", "x = 'true';");
    test("x = (!1).toString()", "x = 'false';");
    test("x = 'xy'.toString()", "x = 'xy';");
    test("x = 0 .toString()", "x = '0';");
    test("x = 123 .toString()", "x = '123';");
    test("x = NaN.toString()", "x = 'NaN';");
    test("x = NaN.toString(2)", "x = 'NaN';");
    test("x = Infinity.toString()", "x = 'Infinity';");
    test("x = Infinity.toString(2)", "x = 'Infinity';");
    test("x = (-Infinity).toString(2)", "x = '-Infinity';");
    test("x = 1n.toString()", "x = '1'");
    test_same("254n.toString(16);"); // unimplemented
    // test("/a\\\\b/ig.toString()", "'/a\\\\\\\\b/ig';");
    test_same("null.toString()"); // type error

    test("x = 100 .toString(0)", "x = 100 .toString(0)");
    test("x = 100 .toString(1)", "x = 100 .toString(1)");
    test("x = 100 .toString(2)", "x = '1100100'");
    test("x = 100 .toString(5)", "x = '400'");
    test("x = 100 .toString(8)", "x = '144'");
    test("x = 100 .toString(13)", "x = '79'");
    test("x = 100 .toString(16)", "x = '64'");
    test("x = 10000 .toString(19)", "x = '18d6'");
    test("x = 10000 .toString(23)", "x = 'iki'");
    test("x = 1000000 .toString(29)", "x = '1c01m'");
    test("x = 1000000 .toString(31)", "x = '12hi2'");
    test("x = 1000000 .toString(36)", "x = 'lfls'");
    test("x = 0 .toString(36)", "x = '0'");
    test("x = 0.5.toString()", "x = '0.5'");

    test("false.toString(b)", "(!1).toString(b)");
    test("true.toString(b)", "(!0).toString(b)");
    test("'xy'.toString(b)", "'xy'.toString(b)");
    test("123 .toString(b)", "123 .toString(b)");
    test("1e99.toString(b)", "1e99.toString(b)");
    test("/./.toString(b)", "/./.toString(b)");
}

#[test]
fn test_fold_pow() {
    test("v = Math.pow(2, 3)", "v = 2 ** 3");
    test("v = Math.pow(a, 3)", "v = a ** 3");
    test("v = Math.pow(2, b)", "v = 2 ** b");
    test("v = Math.pow(a, b)", "v = a ** +b");
    test("v = Math.pow(2n, 3n)", "v = 2n ** +3n"); // errors both before and after
    test("v = Math.pow(a + b, c)", "v = (a + b) ** +c");
    test_same("v = Math.pow()");
    test_same("v = Math.pow(1)");
    test_same("v = Math.pow(...a, 1)");
    test_same("v = Math.pow(1, ...a)");
    test_same("v = Math.pow(1, 2, 3)");
    test_target("v = Math.pow(2, 3)", "v = Math.pow(2, 3)", "chrome51");
    test_same("v = Unknown.pow(1, 2)");
}

#[test]
fn test_fold_roots() {
    test_same("v = Math.sqrt()");
    test_same("v = Math.sqrt(1, 2)");
    test_same("v = Math.sqrt(...a)");
    test_same("v = Math.sqrt(a)"); // a maybe -0
    test_same("v = Math.sqrt(2n)");
    test("v = Math.sqrt(Infinity)", "v = Infinity");
    test("v = Math.sqrt(NaN)", "v = NaN");
    test("v = Math.sqrt(0)", "v = 0");
    test("v = Math.sqrt(-0)", "v = -0");
    test("v = Math.sqrt(-1)", "v = NaN");
    test("v = Math.sqrt(-Infinity)", "v = NaN");
    test("v = Math.sqrt(1)", "v = 1");
    test("v = Math.sqrt(4)", "v = 2");
    test_same("v = Math.sqrt(2)");
    test("v = Math.cbrt(1)", "v = 1");
    test("v = Math.cbrt(8)", "v = 2");
    test_same("v = Math.cbrt(2)");
    test_same("Unknown.sqrt(1)");
    test_same("Unknown.cbrt(1)");
}

#[test]
fn test_number_constants() {
    test("v = Number.POSITIVE_INFINITY", "v = Infinity");
    test("v = Number.NEGATIVE_INFINITY", "v = -Infinity");
    test("v = Number.NaN", "v = NaN");
    test("v = Number.MAX_SAFE_INTEGER", "v = 2**53-1");
    test("v = Number.MIN_SAFE_INTEGER", "v = -(2**53-1)");
    test("v = Number.EPSILON", "v = 2**-52");

    test_same("Number.POSITIVE_INFINITY = 1");
    test_same("Number.NEGATIVE_INFINITY = 1");
    test_same("Number.NaN = 1");
    test_same("Number.MAX_SAFE_INTEGER = 1");
    test_same("Number.MIN_SAFE_INTEGER = 1");
    test_same("Number.EPSILON = 1");

    test_target("v = Number.MAX_SAFE_INTEGER", "v = 9007199254740991", "chrome51");
    test_target("v = Number.MIN_SAFE_INTEGER", "v = -9007199254740991", "chrome51");
    test_target("v = Number.EPSILON", "v = Number.EPSILON", "chrome51");
}

#[test]
fn test_fold_integer_index_access() {
    test_same("v = ''[0]");
    test_same("v = 'a'[-1]");
    test_same("v = 'a'[0.3]");
    test("v = 'a'[0]", "v = 'a'");
    test_same("v = 'a'[1]");
    test("v = 'あ'[0]", "v = 'あ'");
    test_same("v = 'あ'[1]");
    test_same("v = '😀'[0]"); // surrogate pairs cannot be represented by rust string
    test_same("v = '😀'[1]"); // surrogate pairs cannot be represented by rust string
    test_same("v = '😀'[2]");
    test_same("v = (foo(), 'a')[1]"); // can be fold into `v = (foo(), 'a')`

    test_same("v = [][0]");
    test_same("v = [1][-1]");
    test_same("v = [1][0.3]");
    test("v = [1][0]", "v = 1");
    test_same("v = [1][1]");
    test("v = [,][0]", "v = void 0");
    // test("v = [...'a'][0]", "v = 'a'");
    // test_same("v = [...'a'][1]");
    // test("v = [...'😀'][0]", "v = '😀'");
    // test_same("v = [...'😀'][1]");
    test_same("v = [...a, 1][1]");
    test_same("v = [1, ...a][0]");
    test("v = [1, ...[1,2]][0]", "v = 1");

    // property access should be kept to keep `this` value
    test_same(
        "
        function f(){ console.log(this[0]) }
        ['PASS',f][1]()
    ",
    );
    test_same(
        "
        function f(){ console.log(this[0]) }
        ['PASS',f][1]``
    ",
    );
}

#[test]
fn test_fold_starts_with() {
    test_same("v = 'production'.startsWith('prod', 'bar')");
    test("v = 'production'.startsWith('prod')", "v = !0");
    test("v = 'production'.startsWith('dev')", "v = !1");
    test(
        "const node_env = 'production'; v = node_env.toLowerCase().startsWith('prod')",
        "const node_env = 'production'; v = !0",
    );
}

#[test]
fn test_fold_encode_uri() {
    test("x = encodeURI()", "x = 'undefined'");
    test("x = encodeURI('hello')", "x = 'hello'");
    test("x = encodeURI('hello world')", "x = 'hello%20world'");
    test(
        "x = encodeURI('http://example.com/path?a=1&b=2#hash')",
        "x = 'http://example.com/path?a=1&b=2#hash'",
    );
    test("x = encodeURI('a;b,c/d?e:f@g&h=i+j$k')", "x = 'a;b,c/d?e:f@g&h=i+j$k'");
    test("x = encodeURI('ABC-_abc.!~*()123')", "x = 'ABC-_abc.!~*()123'");
    test("x = encodeURI('hello<>\"')", "x = 'hello%3C%3E%22'");
    test("x = encodeURI('hello\\t\\n')", "x = 'hello%09%0A'");
    test("x = encodeURI('café')", "x = 'caf%C3%A9'"); // spellchecker:disable-line
    test("x = encodeURI('测试')", "x = '%E6%B5%8B%E8%AF%95'");

    test_same("x = encodeURI('a', 'b')");
    test_same("x = encodeURI(x)");
}

#[test]
fn test_fold_encode_uri_component() {
    test("x = encodeURIComponent()", "x = 'undefined'");
    test("x = encodeURIComponent('hello')", "x = 'hello'");
    test("x = encodeURIComponent('ABC-_abc.!~*()123')", "x = 'ABC-_abc.!~*()123'");
    test(
        "x = encodeURIComponent('a;b,c/d?e:f@g&h=i+j$k')",
        "x = 'a%3Bb%2Cc%2Fd%3Fe%3Af%40g%26h%3Di%2Bj%24k'",
    );
    test("x = encodeURIComponent('#')", "x = '%23'");
    test("x = encodeURIComponent('hello world')", "x = 'hello%20world'");
    test("x = encodeURIComponent('hello<>\"')", "x = 'hello%3C%3E%22'");
    test("x = encodeURIComponent('café')", "x = 'caf%C3%A9'"); // spellchecker:disable-line
    test("x = encodeURIComponent('测试')", "x = '%E6%B5%8B%E8%AF%95'");

    test_same("x = encodeURIComponent('a', 'b')");
    test_same("x = encodeURIComponent(x)");
}

#[test]
fn test_fold_decode_uri() {
    test("x = decodeURI()", "x = 'undefined'");
    test("x = decodeURI('hello%20world')", "x = 'hello world'");
    test("x = decodeURI('hello')", "x = 'hello'");
    test(
        "x = decodeURI('a%3Bb%2Cc%2Fd%3Fe%3Af%40g%26h%3Di%2Bj%24k')",
        "x = 'a%3Bb%2Cc%2Fd%3Fe%3Af%40g%26h%3Di%2Bj%24k'",
    );
    test("x = decodeURI('%2f')", "x = '%2f'"); // `/`, lower case
    test("x = decodeURI('%23')", "x = '%23'"); // `#`
    test("x = decodeURI('%23hash')", "x = '%23hash'");
    test("x = decodeURI('hello%3C%3E%22')", "x = 'hello<>\"'");
    test("x = decodeURI('hello%09%0A')", "x = 'hello\\t\\n'");
    test("x = decodeURI('caf%C3%A9')", "x = 'café'"); // spellchecker:disable-line
    test("x = decodeURI('%E6%B5%8B%E8%AF%95')", "x = '测试'");

    test_same("x = decodeURI('%ZZ')"); // URIError
    test_same("x = decodeURI('%A')"); // URIError

    test_same("x = decodeURI('a', 'b')");
    test_same("x = decodeURI(x)");
}

#[test]
fn test_fold_decode_uri_component() {
    test("x = decodeURIComponent()", "x = 'undefined'");
    test("x = decodeURIComponent('hello%20world')", "x = 'hello world'");
    test("x = decodeURIComponent('hello')", "x = 'hello'");
    test(
        "x = decodeURIComponent('a%3Bb%2Cc%2Fd%3Fe%3Af%40g%26h%3Di%2Bj%24k')",
        "x = 'a;b,c/d?e:f@g&h=i+j$k'",
    );
    test("x = decodeURIComponent('%23')", "x = '#'");
    test("x = decodeURIComponent('%23hash')", "x = '#hash'");
    test("x = decodeURIComponent('hello%3C%3E%22')", "x = 'hello<>\"'");
    test("x = decodeURIComponent('hello%09%0A')", "x = 'hello\\t\\n'");
    test("x = decodeURIComponent('caf%C3%A9')", "x = 'café'"); // spellchecker:disable-line
    test("x = decodeURIComponent('%E6%B5%8B%E8%AF%95')", "x = '测试'");

    test_same("x = decodeURIComponent('%ZZ')"); // URIError
    test_same("x = decodeURIComponent('%A')"); // URIError

    test_same("x = decodeURIComponent('a', 'b')");
    test_same("x = decodeURIComponent(x)");
}

#[test]
fn test_fold_uri_roundtrip() {
    test("x = decodeURI(encodeURI('hello world'))", "x = 'hello world'");
    test("x = decodeURIComponent(encodeURIComponent('hello world'))", "x = 'hello world'");
    test(
        "x = decodeURIComponent(encodeURIComponent('a;b,c/d?e:f@g&h=i+j$k'))",
        "x = 'a;b,c/d?e:f@g&h=i+j$k'",
    );
    test("x = decodeURI(encodeURI('café'))", "x = 'café'");
    test("x = decodeURIComponent(encodeURIComponent('测试'))", "x = '测试'");
}

#[test]
fn test_fold_global_is_nan() {
    test_value("isNaN()", "!0");
    test_value("isNaN(NaN)", "!0");
    test_value("isNaN(123)", "!1");
    test_value("isNaN('123')", "!1");
    test_value("isNaN('abc')", "!0");
    test_value("isNaN('')", "!1");
    test_value("isNaN(' ')", "!1");
    test_value("isNaN(null)", "!1");
    test_value("isNaN(Infinity)", "!1");
    test_value("isNaN(-Infinity)", "!1");

    test_same_value("isNaN(unknown)");
    test_same_value("isNaN((foo, 0))"); // foo may have sideeffect
}

#[test]
fn test_fold_global_is_finite() {
    test_value("isFinite()", "!1");
    test_value("isFinite(123)", "!0");
    test_value("isFinite(123.45)", "!0");
    test_value("isFinite('123')", "!0");
    test_value("isFinite('')", "!0");
    test_value("isFinite(' ')", "!0");
    test_value("isFinite(null)", "!0");
    test_value("isFinite(NaN)", "!1");
    test_value("isFinite(Infinity)", "!1");
    test_value("isFinite(-Infinity)", "!1");
    test_value("isFinite('abc')", "!1");
    test_same_value("isFinite(unknown)");
    test_same_value("isFinite((foo, 0))"); // foo may have sideeffect
}

#[test]
fn test_fold_regex_source() {
    test_value("/abc def/.source", "'abc def'");
    test_value("/\\d+/.source", "'\\\\d+'");
    test_value("/[a-z]/.source", "'[a-z]'");
    test_value("/a|b/.source", "'a|b'");
    test_value("/^test$/.source", "'^test$'");
    test_value("/./.source", "'.'");
    test_value("/.*/.source", "'.*'");

    test_value("/abc def/i.source", "'abc def'");
    test_same_value("/(/.source"); // this regex is invalid
    test_value("/\\u{}/.source", "'\\\\u{}'");
    test_same_value("/\\u{}/u.source"); // this regex is invalid, also u flag is not supported by ES2015
}
