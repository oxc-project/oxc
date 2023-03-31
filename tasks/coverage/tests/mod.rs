use oxc_coverage::{
    AppArgs, BabelCase, BabelSuite, PrinterTest262Case, Suite, Test262Case, Test262Suite,
    TypeScriptCase, TypeScriptSuite,
};

#[cfg(feature = "tarpaulin")]
#[test]
fn test() {
    let args = AppArgs { filter: None, detail: false, diff: false };
    Test262Suite::<Test262Case>::new().run("Test262", &args);
    BabelSuite::<BabelCase>::new().run("Babel", &args);
    TypeScriptSuite::<TypeScriptCase>::new().run("TypeScript", &args);
    Test262Suite::<PrinterTest262Case>::new().run("Printer", &args);
}
