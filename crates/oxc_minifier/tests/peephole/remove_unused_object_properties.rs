use crate::{CompressOptions, test_options};

#[test]
fn remove_unused_object_methods() {
    let options = CompressOptions::smallest();

    test_options(
        "
        const sharedConfig = {
            context: void 0,
            getContextId() {
                return getContextId(this.context.count);
            },
            getNextContextId() {
                return getContextId(this.context.count++);
            }
        };
        function getContextId(count) {
            return sharedConfig.context.id;
        }
        export function isHydrating(node) {
            return sharedConfig.context;
        }
        ",
        "
        const sharedConfig = {
            context: void 0
        };
        export function isHydrating(node) {
            return sharedConfig.context;
        }
        ",
        &options,
    );
}

#[test]
fn keep_object_properties_for_dynamic_access_or_escape() {
    let options = CompressOptions::smallest();

    test_options(
        "
        const obj = { foo: 1, bar() {} };
        consume(obj);
        ",
        "
        consume({ foo: 1, bar() {} });
        ",
        &options,
    );

    test_options(
        "
        const obj = { foo: 1, bar() {} };
        obj[key];
        ",
        "
        ({ foo: 1, bar() {} })[key];
        ",
        &options,
    );
}

#[test]
fn keep_property_values_with_side_effects() {
    let options = CompressOptions::smallest();

    test_options(
        "
        const obj = { foo: sideEffect(), bar() {} };
        obj.bar;
        ",
        "
        ({ foo: sideEffect(), bar() {} }).bar;
        ",
        &options,
    );
}
