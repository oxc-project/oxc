//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/MinimizedConditionTest.java>

use oxc_minifier::CompressOptions;

// TODO: handle negative cases
fn test(source_text: &str, positive: &str, _negative: &str) {
    let options = CompressOptions {
        remove_syntax: true,
        minimize_conditions: true,
        ..CompressOptions::all_false()
    };
    crate::test(source_text, positive, options);
}

#[test]
#[ignore]
fn try_minimize_cond_simple() {
    test("x", "x", "x");
    test("!x", "!x", "!x");
    test("!!x", "x", "x");
    test("!(x && y)", "!x || !y", "!(x && y)");
}

#[test]
#[ignore]
fn minimize_demorgan_simple() {
    test("!(x&&y)", "!x||!y", "!(x&&y)");
    test("!(x||y)", "!x&&!y", "!(x||y)");
    test("!x||!y", "!x||!y", "!(x&&y)");
    test("!x&&!y", "!x&&!y", "!(x||y)");
    test("!(x && y && z)", "!(x && y && z)", "!(x && y && z)");
    test("(!a||!b)&&c", "(!a||!b)&&c", "!(a&&b||!c)");
    test("(!a||!b)&&(c||d)", "!(a&&b||!c&&!d)", "!(a&&b||!c&&!d)");
}

#[test]
#[ignore]
fn minimize_bug8494751() {
    test(
        "x && (y===2 || !f()) && (y===3 || !h())",
        // TODO(tbreisacher): The 'positive' option could be better:
        // "x && !((y!==2 && f()) || (y!==3 && h()))",
        "!(!x || (y!==2 && f()) || (y!==3 && h()))",
        "!(!x || (y!==2 && f()) || (y!==3 && h()))",
    );

    test(
        "x && (y===2 || !f?.()) && (y===3 || !h?.())",
        "!(!x || (y!==2 && f?.()) || (y!==3 && h?.()))",
        "!(!x || (y!==2 && f?.()) || (y!==3 && h?.()))",
    );
}

#[test]
#[ignore]
fn minimize_complementable_operator() {
    test("0===c && (2===a || 1===a)", "0===c && (2===a || 1===a)", "!(0!==c || 2!==a && 1!==a)");
}

#[test]
#[ignore]
fn minimize_hook() {
    test("!(x ? y : z)", "(x ? !y : !z)", "!(x ? y : z)");
}

#[test]
#[ignore]
fn minimize_comma() {
    test("!(inc(), test())", "inc(), !test()", "!(inc(), test())");
    test("!(inc?.(), test?.())", "inc?.(), !test?.()", "!(inc?.(), test?.())");
    test("!((x,y)&&z)", "(x,!y)||!z", "!((x,y)&&z)");
}
