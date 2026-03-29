use crate::{CompressOptions, test_options, test_same_options};

#[test]
fn r#const() {
    let options = CompressOptions::smallest();
    test_options("const foo = 1; log(foo)", "log(1)", &options);
    test_options("export const foo = 1; log(foo)", "export const foo = 1; log(1)", &options);

    test_options("let foo = 1; log(foo)", "log(1)", &options);
    test_options("export let foo = 1; log(foo)", "export let foo = 1; log(1)", &options);
}

#[test]
fn small_value() {
    let options = CompressOptions::smallest();
    test_options("const foo = 999; log(foo), log(foo)", "log(999), log(999)", &options);
    test_options("const foo = -99; log(foo), log(foo)", "log(-99), log(-99)", &options);
    test_same_options("const foo = 1000; log(foo), log(foo)", &options);
    test_same_options("const foo = -100; log(foo), log(foo)", &options);

    test_same_options("const foo = 0n; log(foo), log(foo)", &options);

    test_options("const foo = 'aaa'; log(foo), log(foo)", "log('aaa'), log('aaa')", &options);
    test_same_options("const foo = 'aaaa'; log(foo), log(foo)", &options);

    test_options("const foo = true; log(foo), log(foo)", "log(!0), log(!0)", &options);
    test_options("const foo = false; log(foo), log(foo)", "log(!1), log(!1)", &options);
    test_options("const foo = undefined; log(foo), log(foo)", "log(void 0), log(void 0)", &options);
    test_options("const foo = null; log(foo), log(foo)", "log(null), log(null)", &options);

    test_options(
        r#"
        const o = 'o';
        const d = 'd';
        const boolean = false;
        var frag = `<p autocapitalize="${`w${o}r${d}s`}" contenteditable="${boolean}"/>`;
        console.log(frag);
        "#,
        r#"console.log('<p autocapitalize="words" contenteditable="false"/>');"#,
        &options,
    );
}
