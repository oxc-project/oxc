use std::path::Path;

use oxc::{
    CompilerInterface,
    codegen::{CodegenOptions, CodegenReturn},
    minifier::{ManglePropertiesOptions, ManglePropertyCache},
    span::SourceType,
    transformer::TransformOptions,
};

struct PropertyCompiler {
    transform: TransformOptions,
    mangle_properties: ManglePropertiesOptions,
    code: String,
    cache: Option<ManglePropertyCache>,
}

impl CompilerInterface for PropertyCompiler {
    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.transform)
    }

    fn mangle_properties_options(&self) -> Option<ManglePropertiesOptions> {
        Some(self.mangle_properties.clone())
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(CodegenOptions::default())
    }

    fn after_property_mangle(&mut self, cache: ManglePropertyCache) {
        self.cache = Some(cache);
    }

    fn after_codegen(&mut self, ret: CodegenReturn<'_>) {
        self.code = ret.code;
    }
}

#[test]
fn compiler_mangles_transformed_property_strings_with_provenance() {
    let mut mangle_properties = ManglePropertiesOptions::from_pattern("^_").unwrap();
    mangle_properties.debug = true;
    let mut compiler = PropertyCompiler {
        transform: TransformOptions::from_target("es2015").unwrap(),
        mangle_properties,
        code: String::new(),
        cache: None,
    };

    compiler.compile(
        "class C { _field = 1; } obj._power **= 2; obj['_quoted'];",
        SourceType::mjs(),
        Path::new("test.js"),
    );

    let cache = compiler.cache.unwrap();
    let power_target = cache["_power"].as_deref().unwrap();
    let field_target = cache["_field"].as_deref().unwrap();
    assert!(!cache.contains_key("_quoted"));
    assert!(!compiler.code.contains("[\"_power\"]"), "{}", compiler.code);
    assert!(!compiler.code.contains("._power"), "{}", compiler.code);
    assert!(!compiler.code.contains("\"_field\""), "{}", compiler.code);
    assert!(compiler.code.contains("_quoted"), "{}", compiler.code);
    assert!(compiler.code.contains(&format!("[\"{power_target}\"]")), "{}", compiler.code);
    assert!(compiler.code.contains(&format!(".{power_target}")), "{}", compiler.code);
    assert!(compiler.code.contains(&format!("\"{field_target}\"")), "{}", compiler.code);
}
