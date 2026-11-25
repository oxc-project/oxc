use std::path::{Path, PathBuf};

use serde_json::Value;

use oxc_transformer::Helper;

/// Every helper the transformer can emit must be published by `@oxc-project/runtime`
/// in both module formats, or `import x from "@oxc-project/runtime/helpers/<name>"` fails to resolve.
#[test]
fn runtime_package_provides_every_helper() {
    let runtime_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm/runtime");
    let package_json = std::fs::read_to_string(runtime_dir.join("package.json"))
        .expect("failed to read npm/runtime/package.json");
    let package_json: Value =
        serde_json::from_str(&package_json).expect("npm/runtime/package.json is not valid JSON");
    let exports = package_json["exports"].as_object().expect("`exports` must be an object");

    let mut missing = vec![];
    for helper in Helper::ALL {
        let name = helper.name();
        let key = format!("./helpers/{name}");
        let Some(entry) = exports.get(&key) else {
            missing.push(format!("{name}: no `{key}` entry in `exports`"));
            continue;
        };
        // An entry is either a conditions object or a fallback array whose first item is one.
        let conditions = match entry {
            Value::Array(items) => items.first(),
            entry => Some(entry),
        }
        .and_then(Value::as_object);
        let Some(conditions) = conditions else {
            missing.push(format!("{name}: `{key}` has no conditions object"));
            continue;
        };
        for (format, condition, expected) in [
            ("CJS", "default", format!("./src/helpers/{name}.js")),
            ("ESM", "import", format!("./src/helpers/esm/{name}.js")),
        ] {
            match conditions.get(condition).and_then(Value::as_str) {
                None => missing.push(format!("{name}: `{key}` has no `{condition}` ({format})")),
                Some(target) if target != expected => missing.push(format!(
                    "{name}: `{key}` `{condition}` ({format}) is `{target}`, expected `{expected}`"
                )),
                Some(target) => {
                    let file: PathBuf = runtime_dir.join(target);
                    if !file.is_file() {
                        missing.push(format!(
                            "{name}: {format} file `npm/runtime/{target}` is missing"
                        ));
                    }
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "`@oxc-project/runtime` (npm/runtime) is missing helpers listed in `Helper::ALL`:\n{}",
        missing.join("\n")
    );
}
