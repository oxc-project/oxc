/**
 * Prettier plugin that uses oxc_formatter for JavaScript/TypeScript.
 *
 * This plugin overrides the built-in `babel` and `typescript` parsers,
 * delegating the formatting to oxc_formatter via NAPI.
 *
 * When Prettier formats Vue/HTML files, it calls the `embed` function
 * for `<script>` tags, which uses the `babel` or `typescript` parser.
 * By overriding these parsers, oxc_formatter is automatically used
 * for embedded JavaScript in Vue/HTML files.
 */

import type { Parser, Printer, Plugin, Options, Doc } from "prettier";
import { doc } from "prettier";
import { formatToDoc } from "../bindings";
import { formatEmbeddedCode, sortTailwindClasses } from "./prettier";

const { hardline, join } = doc.builders;

type SourceType = "js" | "jsx" | "ts" | "tsx";

/**
 * Try to parse a JSON string option and restore it to the result object.
 */
function restoreJsonOption(
  source: Record<string, unknown>,
  target: Record<string, unknown>,
  jsonKey: string,
  resultKey: string,
): void {
  const jsonValue = source[jsonKey];
  if (typeof jsonValue !== "string" || jsonValue === "") return;

  try {
    target[resultKey] = JSON.parse(jsonValue);
  } catch {
    // Ignore parse errors
  }
}

/**
 * Extract only serializable options from Prettier options.
 * Filters out functions, plugins, and other non-primitive values.
 */
function extractSerializableOptions(options: Options): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  const anyOptions = options as Record<string, unknown>;

  const serializableKeys = [
    "useTabs",
    "tabWidth",
    "printWidth",
    "singleQuote",
    "jsxSingleQuote",
    "semi",
    "endOfLine",
    "trailingComma",
    "bracketSpacing",
    "bracketSameLine",
    "arrowParens",
    "quoteProps",
    "_tailwindPluginEnabled",
  ];

  for (const key of serializableKeys) {
    if (key in options) {
      result[key] = anyOptions[key];
    }
  }

  // Restore object options from JSON strings
  // These were serialized in formatFile() because Prettier doesn't preserve object options
  restoreJsonOption(anyOptions, result, "_experimentalSortImportsJson", "experimentalSortImports");
  restoreJsonOption(anyOptions, result, "_experimentalTailwindcssJson", "experimentalTailwindcss");

  return result;
}

function createOxcParser(sourceType: SourceType): Parser {
  return {
    parse: async (text: string, options: Options) => {
      const filepath = (options.filepath as string) ?? "unknown";

      // Extract only serializable options to pass to Rust
      const serializableOptions = extractSerializableOptions(options);

      // Call oxc_formatter via NAPI with Prettier options
      // Options are converted to FormatOptions on the Rust side
      const formatted = await formatToDoc(
        text,
        sourceType,
        filepath,
        serializableOptions,
        // Embedded formatter callback (CSS-in-JS, etc.)
        async (_opts, parserName, code) => formatEmbeddedCode({ code, parserName, options }),
        // Tailwind class sorter callback
        async (fp, opts, classes) => sortTailwindClasses({ filepath: fp, classes, options: opts }),
      );

      return formatted;
    },
    astFormat: "oxfmt-doc",
    locStart: () => -1,
    locEnd: () => -1,
  };
}

/**
 * Printer that converts the formatted string to a Prettier Doc.
 *
 * When the string contains newlines, we need to use hardline to preserve
 * Prettier's automatic indentation for embedded code (e.g., in Vue files
 * with vueIndentScriptAndStyle: true).
 */
const oxfmtPrinter: Printer = {
  print: ({ node }): Doc => {
    const str = node as string;
    const lines = str.split("\n");

    // Single line: return as-is
    if (lines.length === 1) {
      return str;
    }

    // Multiple lines: join with hardline for proper indentation
    return join(hardline, lines);
  },
};

/**
 * Prettier plugin for oxc_formatter.
 *
 * We define custom options here so that Prettier preserves them
 * when passing options to embed functions (for Vue/HTML <script> tags).
 *
 * Note: Prettier's option system doesn't support object types, so we use
 * JSON strings for the actual configuration and boolean flags for detection.
 */
const plugin: Plugin = {
  options: {
    // JSON string containing the experimentalSortImports options
    _experimentalSortImportsJson: {
      type: "string",
      category: "JavaScript",
      description: "JSON string for sort imports options (internal)",
      default: "",
    },
    // JSON string containing the experimentalTailwindcss options
    _experimentalTailwindcssJson: {
      type: "string",
      category: "JavaScript",
      description: "JSON string for Tailwind CSS options (internal)",
      default: "",
    },
  },
  parsers: {
    // Override babel parser for JavaScript
    babel: createOxcParser("jsx"),
    "babel-flow": createOxcParser("jsx"),
    // Override typescript parser for TypeScript
    typescript: createOxcParser("tsx"),
    "babel-ts": createOxcParser("tsx"),
  },
  printers: {
    "oxfmt-doc": oxfmtPrinter,
  },
};

export default plugin;
