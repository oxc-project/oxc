import { jsTextToDoc } from "../../index";
import type { Parser, Doc } from "prettier";

export const textToDoc: Parser<Doc>["parse"] = async (embeddedSourceText, textToDocOptions) => {
  // `_oxfmtPluginOptionsJson` is a JSON string bundled by Rust (`to_prettier::inject_oxfmt_plugin_payload`),
  // carrying the typed `FormatConfig` + parent filepath for the Rust-side `oxc_formatter`.
  const { parser, parentParser, filepath, _oxfmtPluginOptionsJson, _oxfmtVueScriptLang } =
    textToDocOptions;

  // Detect context from Prettier's internal flags
  const parentContext = detectParentContext(parentParser!, textToDocOptions);

  // For (j|t)s-in-xxx, default `parser` is either `babel`(JS), `babel-ts` or `typescript`.
  // We need to infer the parse grammar (`source_ext`) for `oxc_formatter`.
  // - JS: always enable JSX for js-in-xxx, it's syntactically valid
  // - TS: `typescript` (ts-in-vue|markdown|mdx) or `babel-ts` (ts-in-vue(script generic="..."))
  //   - In case of ts-in-md, `filepath` is overridden as `dummy.ts(x)` to distinguish TSX or TS
  //   - For tsx-in-vue (`<script lang="tsx">`), there is no signal from Prettier itself:
  //     - both `lang="ts"` and `lang="tsx"` resolve to the `typescript` parser and `filepath` is the parent `.vue` file
  //     - Nor from parsing: a JSX-free tsx block parses fine as plain ts
  //     - So, our host pre-scans the SFC and passes `_oxfmtVueScriptLang: "tsx"` instead
  const isTS = parser === "typescript" || parser === "babel-ts";
  const isTSX =
    filepath.endsWith(".tsx") || (parentContext === "vue-script" && _oxfmtVueScriptLang === "tsx");
  const embeddedSourceExt = isTS ? (isTSX ? "tsx" : "ts") : "jsx";

  const docJSON = await jsTextToDoc(
    embeddedSourceExt,
    embeddedSourceText,
    _oxfmtPluginOptionsJson as string,
    parentContext,
  );

  if (docJSON === null) {
    throw new Error("`oxfmt::textToDoc()` failed. Use `OXC_LOG` env var to see Rust-side logs.");
  }

  // SAFETY: Rust side returns valid JSON
  const { doc, refs, hasRootDedent } = JSON.parse(docJSON) as {
    doc: unknown;
    refs: unknown[];
    hasRootDedent: boolean;
  };

  // Fast path for no refs (common when formatting small AST fragments):
  // restore in place instead of the rebuilding walk `resolveRefs` does.
  if (refs.length === 0) {
    if (hasRootDedent) restoreRootDedents(doc);
    return doc as Doc;
  }

  // Sparse array sized to ref count; index = ref id.
  // Faster than `Map` for dense numeric keys and avoids hashing overhead.
  const cache: unknown[] = Array.from({ length: refs.length });
  return resolveRefs(doc, refs, cache) as Doc;
};

// JSON cannot represent `-Infinity`, which Prettier expects for dedent-to-root
// (`makeAlign()` treats other falsy `n` as a no-op, losing the dedent).
// Rust emits `n: null` only for `DedentMode::Root` (signaled by `hasRootDedent`),
// so restore it.
// `resolveRefs` does the same during its rebuild.
function restoreRootDedents(node: unknown): void {
  if (node === null || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const child of node) restoreRootDedents(child);
    return;
  }
  const obj = node as Record<string, unknown>;
  if (obj.type === "align" && obj.n === null) obj.n = Number.NEGATIVE_INFINITY;
  for (const k in obj) restoreRootDedents(obj[k]);
}

/**
 * Rust emits `Interned` sub-trees once into `refs` and references them via `{ _REF: <id> }` placeholders,
 * preventing exponential JSON blowup when the same sub-tree is duplicated variants.
 *
 * Restore shared object references so Prettier sees the original (memory-shared) structure.
 * Identity does not affect output because Prettier identifies groups by their `id` field,
 * not by JS object identity.
 *
 * The `_REF` key (uppercase, prefixed) is chosen to never collide with valid Prettier Doc node keys,
 * so the `typeof obj._REF === "number"` check uniquely identifies placeholders.
 *
 * Along the rebuild it also restores what JSON transport cannot carry (`align n: -Infinity`, see `restoreRootDedents`).
 *
 * Refs are resolved on-demand with memoization.
 * A ref `i` may reference any other ref `j` (including `j < i`) because Rust caches `Interned` by pointer
 * and an earlier-encountered `Interned` (smaller id) can also appear inside a later one's content.
 * Topological / reverse-order resolution would observe `undefined` holes, so we recurse lazily.
 */
function resolveRefs(node: unknown, rawRefs: unknown[], cache: unknown[]): unknown {
  if (node === null || typeof node !== "object") return node;
  if (Array.isArray(node)) return node.map((n) => resolveRefs(n, rawRefs, cache));

  const obj = node as Record<string, unknown>;
  if (typeof obj._REF === "number") {
    const id = obj._REF;
    // Doc values are never `undefined`,
    // so `undefined` in `cache[id]` means "not yet cached" rather than "cached undefined".
    const cached = cache[id];
    if (cached !== undefined) return cached;
    const resolved = resolveRefs(rawRefs[id], rawRefs, cache);
    cache[id] = resolved;
    return resolved;
  }

  const out: Record<string, unknown> = {};
  for (const k in obj) out[k] = resolveRefs(obj[k], rawRefs, cache);
  // Restore `align n: -Infinity` during the rebuild (see `restoreRootDedents`)
  if (out.type === "align" && out.n === null) out.n = Number.NEGATIVE_INFINITY;
  return out;
}

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
  if (parentParser === "svelte") {
    return "svelte-script";
  }

  return parentParser;
}
