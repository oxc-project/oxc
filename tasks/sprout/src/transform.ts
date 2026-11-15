import { createRequire } from "node:module";
import { transformSync } from "@babel/core";

const require = createRequire(import.meta.url);
const presets = [
  [
    require.resolve("@babel/preset-typescript"),
    { allExtensions: true, isTSX: true, onlyRemoveTypeImports: true },
  ],
  [require.resolve("@babel/preset-react"), { throwIfNamespace: false }],
];
const plugins = [
  require.resolve("babel-plugin-fbt"),
  require.resolve("babel-plugin-fbt-runtime"),
  require.resolve("babel-plugin-idx"),
  require.resolve("@babel/plugin-transform-modules-commonjs"),
];

export function evaluatorCode(source: string, path: string): string {
  const result = transformSync(source, {
    filename: path,
    presets,
    plugins,
    configFile: false,
    babelrc: false,
  });
  if (result?.code == null) throw new Error("Evaluator transform emitted no code");
  return result.code;
}
