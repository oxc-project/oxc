/**
 * Prettier plugin that uses `oxc_formatter` for (j|t)s-in-xxx part.
 *
 * When Prettier formats Vue/HTML (which can embed JS/TS code inside) files,
 * it calls the `embed()` function for each block.
 *
 * By default, it uses the `babel` or `typescript` parser and `estree` printer.
 * Therefore, by overriding these internally, we can use `oxc_formatter` instead.
 * e.g. Now it's possible to apply our builtin sort-imports for JS/TS code inside Vue `<script>`.
 */

import { doc } from "prettier";
import { format } from "../index";
import type { Parser, Printer, SupportOptions, Options, Doc } from "prettier";

const { hardline, join } = doc.builders;
const LINE_BREAK_RE = /\r?\n/;

/**
 * Unwrap helpers for special Prettier cases.
 *
 * Prettier wraps certain Vue/HTML embedded expressions before parsing:
 * - v-for left side: `function _(params) {}`
 * - v-slot bindings: `function _(params) {}`
 * - script generic: `type T<params> = any`
 *
 * After formatting, we need to unwrap them back.
 */
const FUNCTION_WRAPPER_RE = /^function _\(([\s\S]*)\) \{\}\n?$/;
const TYPE_WRAPPER_RE = /^type T<([\s\S]*)> = any;?\n?$/;

/**
 * Check if params string has multiple parameters (top-level comma).
 * Need to handle destructuring like `{ id, name }` which has comma but is single param.
 */
function hasMultipleParams(params: string): boolean {
  let depth = 0;
  for (const char of params) {
    if (char === "{" || char === "[" || char === "(") {
      depth++;
    } else if (char === "}" || char === "]" || char === ")") {
      depth--;
    } else if (char === "," && depth === 0) {
      return true;
    }
  }
  return false;
}

function unwrapVueForBindingLeft(code: string): string | null {
  const match = code.match(FUNCTION_WRAPPER_RE);
  if (!match) return null;
  const params = match[1].trim();
  // Prettier wraps multiple params with (), single param without
  return hasMultipleParams(params) ? `(${params})` : params;
}

function unwrapVueBindings(code: string): string | null {
  const match = code.match(FUNCTION_WRAPPER_RE);
  if (!match) return null;
  return match[1].trim();
}

function unwrapTypescriptGenericParameters(code: string): string | null {
  const match = code.match(TYPE_WRAPPER_RE);
  if (!match) return null;
  return match[1].trim();
}

const oxfmtParser: Parser<Doc> = {
  parse: async (embeddedSourceText: string, textToDocOptions: Options) => {
    // Handle special Prettier wrapper cases for Vue/HTML attributes
    // These need to be formatted and then unwrapped
    const isVueForBindingLeft = textToDocOptions.__isVueForBindingLeft as boolean;
    const isVueBindings = textToDocOptions.__isVueBindings as boolean;
    const isTypescriptGenericParams =
      textToDocOptions.__isEmbeddedTypescriptGenericParameters as boolean;

    if (isVueForBindingLeft || isVueBindings) {
      // Force singleQuote in HTML attributes to avoid HTML entity escaping
      const pluginOptions = JSON.parse(textToDocOptions._oxfmtPluginOptionsJson as string);
      if (textToDocOptions.__isInHtmlAttribute) {
        pluginOptions.singleQuote = true;
      }
      const { code, errors } = await format("dummy.js", embeddedSourceText, pluginOptions);
      if (0 < errors.length) throw new Error(errors[0].message);

      const unwrapped = isVueForBindingLeft
        ? unwrapVueForBindingLeft(code)
        : unwrapVueBindings(code);
      if (unwrapped !== null) {
        return join(hardline, unwrapped.split(LINE_BREAK_RE));
      }
      // Fallback: return as-is if unwrap failed
    }

    if (isTypescriptGenericParams) {
      // Force singleQuote in HTML attributes to avoid HTML entity escaping
      const pluginOptions = JSON.parse(textToDocOptions._oxfmtPluginOptionsJson as string);
      if (textToDocOptions.__isInHtmlAttribute) {
        pluginOptions.singleQuote = true;
      }
      const { code, errors } = await format("dummy.ts", embeddedSourceText, pluginOptions);
      if (0 < errors.length) throw new Error(errors[0].message);

      const unwrapped = unwrapTypescriptGenericParameters(code);
      if (unwrapped !== null) {
        return join(hardline, unwrapped.split(LINE_BREAK_RE));
      }
      // Fallback: return as-is if unwrap failed
    }

    // NOTE: For (j|t)s-in-xxx, default `parser` is either `babel` or `typescript`
    const parser = textToDocOptions.parser as string;
    // In case of ts-in-md, `filepath` is overridden to distinguish TSX or TS
    const filepath = textToDocOptions.filepath as string;
    // We need to infer `SourceType` for `oxc_formatter`
    const filename =
      parser === "typescript"
        ? filepath.endsWith(".tsx")
          ? "dummy.tsx" // tsx-in-md
          : "dummy.ts" // ts-in-md / ts-in-xxx
        : "dummy.jsx"; // Otherwise, always enable JSX for js-in-xxx, it's safe

    // NOTE: Ultimately, this should be `textToDoc()` like Prettier originally does
    const { code, errors } = await format(
      filename,
      embeddedSourceText,
      // SAFETY: This is generated by Rust side and only available if plugin is used
      JSON.parse(textToDocOptions._oxfmtPluginOptionsJson as string),
    );

    if (0 < errors.length) throw new Error(errors[0].message);

    // NOTE: This is required for the parent ((j|t)s-in-xxx) printer
    // to handle line breaks correctly,
    // not only for `options.vueIndentScriptAndStyle` but also for basic printing.
    return join(hardline, code.split(LINE_BREAK_RE));
  },
  astFormat: "OXFMT",
  // Not used but required
  locStart: () => -1,
  locEnd: () => -1,
};

// ---

export const options: SupportOptions = {
  _oxfmtPluginOptionsJson: {
    category: "JavaScript",
    type: "string",
    default: "{}",
    description: "Bundled JSON string for oxfmt-plugin options",
  },
};

export const parsers: Record<string, Parser> = {
  // Override default JS/TS parsers
  babel: oxfmtParser,
  "babel-ts": oxfmtParser,
  typescript: oxfmtParser,
};

export const printers: Record<string, Printer<Doc>> = {
  OXFMT: {
    print: ({ node }) => node,
  },
};
