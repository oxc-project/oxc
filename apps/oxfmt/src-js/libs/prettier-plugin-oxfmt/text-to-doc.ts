import { jsTextToDoc } from "../../index";
import type { Parser, Doc } from "prettier";

export const textToDoc: Parser<Doc>["parse"] = async (embeddedSourceText, textToDocOptions) => {
  // `_oxfmtPluginOptionsJson` is a JSON string bundled by Rust (`oxfmtrc::finalize_external_options`),
  // containing format options + parent filepath for the Rust-side `oxc_formatter`.
  const { parser, parentParser, filepath, _oxfmtPluginOptionsJson } = textToDocOptions;

  // For (j|t)s-in-xxx, default `parser` is either `babel`, `babel-ts` or `typescript`
  // We need to infer `SourceType::from_extension(ext)` for `oxc_formatter`.
  // - JS: always enable JSX for js-in-xxx, it's safe
  // - TS: `typescript` (ts-in-vue|markdown|mdx) or `babel-ts` (ts-in-vue(script generic="..."))
  //   - In case of ts-in-md, `filepath` is overridden as `dummy.tx(x)` to distinguish TSX or TS
  //   - NOTE: tsx-in-vue is not supported since there is no signal from Prettier to detect it
  //     - Prettier is using `maybeJSXRe.test(sourceText)` to detect, but it's slow!
  const isTS = parser === "typescript" || parser === "babel-ts";
  const embeddedSourceExt = isTS ? (filepath?.endsWith(".tsx") ? "tsx" : "ts") : "jsx";

  // Detect context from Prettier's internal flags
  const parentContext = detectParentContext(parentParser!, textToDocOptions);

  const doc = await jsTextToDoc(
    embeddedSourceExt,
    embeddedSourceText,
    _oxfmtPluginOptionsJson as string,
    parentContext,
  );

  if (doc === null) {
    throw new Error("`oxfmt::textToDoc()` failed. Use `OXC_LOG` env var to see Rust-side logs.");
  }

  // SAFETY: Rust side returns Prettier's `Doc` JSON
  return JSON.parse(doc) as Doc;
};

/**
 * Detects Vue fragment mode from Prettier's internal flags.
 *
 * When Prettier formats Vue SFC templates, it calls textToDoc with special flags:
 * - `__isVueForBindingLeft`: v-for left-hand side (e.g., `(item, index)` in `v-for="(item, index) in items"`)
 * - `__isVueBindings`: v-slot bindings (e.g., `{ item }` in `#default="{ item }"`)
 * - `__isEmbeddedTypescriptGenericParameters`: `<script generic="...">` type parameters
 */
function detectParentContext(parentParser: string, options: Record<string, unknown>): string {
  if (parentParser === "vue") {
    if ("__isVueForBindingLeft" in options) return "vue-for-binding-left";
    if ("__isVueBindings" in options) return "vue-bindings";
    if ("__isEmbeddedTypescriptGenericParameters" in options) return "vue-script-generic";
    return "vue-script";
  }

  return parentParser;
}
