mod util;

pub use util::SemanticTester;

#[test]
fn test_exports() {
    let test = SemanticTester::js(
        "
        function foo(a, b) {
            let c = a + b;
            return c / 2
        }

        export class ExportModifier {
            constructor(x) {
                this.x = x;
            }
        }

        const defaultExport = 1;

        export { foo };
        export default defaultExport;
        ",
    );

    test.has_some_symbol("foo").is_exported().test();

    // FIXME: failing
    // test.has_some_symbol("defaultExport").is_exported().test();
}
