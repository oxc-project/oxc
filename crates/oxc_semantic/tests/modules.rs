mod util;

use oxc_semantic::SymbolFlags;
use oxc_span::Atom;
use oxc_syntax::module_record::ExportLocalName;
pub use util::SemanticTester;

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
        ",
    );

    test.has_some_symbol("foo")
        .contains_flags(
            SymbolFlags::Export | SymbolFlags::Function | SymbolFlags::BlockScopedVariable,
        )
        // Should an export be considered a reference?
        .has_number_of_references(0)
        .is_exported()
        .test();

    // FIXME: failing
    test.has_some_symbol("defaultExport").is_exported().test();
}

#[test]
fn test_default_export() {
    let tester = SemanticTester::js(
        "
        const foo = 1;
        export default foo;
        ",
    );
    tester
        .has_root_symbol("foo")
        .is_exported()
        .contains_flags(
            SymbolFlags::BlockScopedVariable | SymbolFlags::Export | SymbolFlags::ConstVariable,
        )
        .test();
    let semantic = tester.build();
    let module_record = semantic.module_record();

    assert!(semantic.symbols().get_symbol_id_from_name(&Atom::from("foo")).is_some());

    assert!(module_record.export_default.is_some());
    assert!(module_record.local_export_entries.len() == 1);
    let local_exports = &module_record.local_export_entries[0];
    assert!(
        matches!(&local_exports.local_name, ExportLocalName::Name(namespan) if namespan.name() == &"foo")
    );
}
