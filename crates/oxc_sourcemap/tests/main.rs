use std::fs;

use oxc_sourcemap::{SourceMap, SourcemapVisualizer};

#[test]
fn snapshot_sourcemap_visualizer() {
    insta::glob!("fixtures/**/*.js", |path| {
        let js = fs::read_to_string(path).unwrap();
        let js_map = fs::read_to_string(path.with_extension("js.map")).unwrap();
        let sourcemap = SourceMap::from_json_string(&js_map).unwrap();
        let visualizer = SourcemapVisualizer::new(&js, &sourcemap);
        let visualizer_text = visualizer.into_visualizer_text();
        insta::with_settings!({ snapshot_path => path.parent().unwrap(), prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!("visualizer", visualizer_text);
        });
    });
}
