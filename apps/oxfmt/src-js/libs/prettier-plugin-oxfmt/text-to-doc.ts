import { jsTextToDoc } from "../../index";
import type { Parser, Doc } from "prettier";

export const textToDoc: Parser<Doc>["parse"] = async (embeddedSourceText, textToDocOptions) => {
  // NOTE: For (j|t)s-in-xxx, default `parser` is either `babel`, `babel-ts`, or `typescript`
  // In case of ts-in-md, `filepath` is overridden to distinguish TSX or TS
  // We need to infer `SourceType::from_path(fileName)` for `oxc_formatter`.
  const { parser, parentParser, filepath, _oxfmtPluginOptionsJson } = textToDocOptions;
  const fileName =
    parser === "typescript"
      ? filepath.endsWith(".tsx")
        ? "dummy.tsx" // tsx-in-md
        : "dummy.ts" // ts-in-md / ts-in-xxx
      : "dummy.jsx"; // Otherwise, always enable JSX for js-in-xxx, it's safe

  // Detect Vue-specific flags and build parentContext with mode info
  const parentContext = detectVueMode(textToDocOptions) ?? [parentParser].join(":");

  const { doc: docJson, errors } = await jsTextToDoc(
    fileName,
    embeddedSourceText,
    _oxfmtPluginOptionsJson as string,
    parentContext,
  );

  if (0 < errors.length) throw new Error(errors[0].message);

  // Rust side now returns Prettier Doc JSON directly
  return JSON.parse(docJson);
};

/**
 * Detect Vue-specific embedding flags and return the appropriate parentContext string.
 * Returns undefined if no Vue flag is detected.
 */
function detectVueMode(textToDocOptions: Record<string, unknown>): string | undefined {
  if (textToDocOptions.__isVueForBindingLeft) return "vue:VueForBindingLeft";
  if (textToDocOptions.__isVueBindings) return "vue:VueBindings";
  if (textToDocOptions.__isEmbeddedTypescriptGenericParameters) return "vue:VueGeneric";
  return undefined;
}
