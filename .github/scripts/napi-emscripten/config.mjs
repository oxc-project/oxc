export const TARGET = "wasm32-unknown-emscripten";

const CONFIGS = {
  "oxc-parser": {
    directory: "parser",
    crate: "oxc_parser_napi",
    binary: "parser",
    bindingPackage: "@oxc-parser/binding-wasm32-emscripten",
    exports: [
      "Severity",
      "ParseResult",
      "ExportExportNameKind",
      "ExportImportNameKind",
      "ExportLocalNameKind",
      "ImportNameKind",
      "parse",
      "parseSync",
      "rawTransferSupported",
    ],
  },
  "oxc-minify": {
    directory: "minify",
    crate: "oxc_minify_napi",
    binary: "minify",
    bindingPackage: "@oxc-minify/binding-wasm32-emscripten",
    exports: ["LegalCommentsMode", "minify", "minifySync", "Severity"],
  },
  "oxc-transform": {
    directory: "transform",
    crate: "oxc_transform_napi",
    binary: "transform",
    bindingPackage: "@oxc-transform/binding-wasm32-emscripten",
    exports: [
      "Severity",
      "HelperMode",
      "isolatedDeclaration",
      "isolatedDeclarationSync",
      "moduleRunnerTransform",
      "moduleRunnerTransformSync",
      "transform",
      "transformSync",
    ],
  },
  "oxc-transform-react": {
    directory: "transform-react",
    crate: "oxc_transform_react_napi",
    binary: "transform-react",
    bindingPackage: "@oxc-transform-react/binding-wasm32-emscripten",
    exports: ["Severity", "transform", "transformSync"],
  },
};

export function getConfig(packageName) {
  const config = CONFIGS[packageName];
  if (!config) {
    throw new Error(`Unsupported Emscripten package: ${packageName}`);
  }
  return { packageName, ...config };
}
