mod util;

#[allow(clippy::wildcard_imports)]
use util::*;

#[test]
fn test_named_exports() {
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
        "
    ).with_module_record_builder(true);

    test.has_some_symbol("defaultExport")
}
