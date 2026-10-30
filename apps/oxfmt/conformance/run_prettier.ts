// oxlint-disable no-console, no-await-in-loop

import { createTwoFilesPatch, diffLines } from "diff";
import { globSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, extname, join, relative } from "node:path";
import { parseArgs } from "node:util";
import { format, type Language } from "../dist/index.js";

// Compares oxfmt with the committed outputs of Prettier's own test suite (`tests/format/**/__snapshots__`),
// so Prettier itself never runs here.
// `--filter <substring>` runs only the matching paths and prints their diffs instead of writing the report.

const SUITE_DIR = join(import.meta.dirname, "fixtures", "externals", "prettier");
const SNAPSHOTS_DIR = join(import.meta.dirname, "snapshots");

/** Extension of the filename oxfmt is given → its language (the `associations` name), in the report's order */
const LANGUAGES: Record<string, Language> = {
  ".js": "javascript",
  ".mjs": "javascript",
  ".cjs": "javascript",
  ".jsx": "jsx",
  ".ts": "typescript",
  ".mts": "typescript",
  ".cts": "typescript",
  ".tsx": "tsx",
  ".json": "json",
  ".importmap": "json",
  ".json5": "json5",
  ".jsonc": "jsonc",
  ".css": "css",
  ".scss": "scss",
  ".less": "less",
  ".graphql": "graphql",
  ".yaml": "yaml",
  ".md": "markdown",
  ".html": "html",
};

const withExt = (ext: string) => (name: string) => name.replace(/\.[^. ]*$/, "") + ext;
// A JS/TS file keeps its own extension, as oxfmt parses it by that (e.g. a `.js` listing `typescript` stays JS);
// only snippets, which have none, get `ext`
const jsLike = (ext: string) => (name: string) =>
  /\.[cm]?[jt]sx?$/.test(name) ? name : name + ext;

/**
 * Prettier parser → the filename oxfmt routes to that language.
 * A section runs once per distinct filename of its listed `parsers`, unlisted parsers are not run.
 */
const PARSERS: Record<string, (name: string) => string> = {
  babel: jsLike(".js"),
  typescript: jsLike(".ts"),
  // A spec may list only these (e.g. plain JSX under `flow`), Flow syntax itself fails to parse as JS
  flow: jsLike(".js"),
  "babel-ts": jsLike(".ts"),
  json: withExt(".json"),
  json5: withExt(".json5"),
  jsonc: withExt(".jsonc"),
  // Not `package.json`, which oxfmt also sorts
  "json-stringify": withExt(".importmap"),
  css: withExt(".css"),
  scss: withExt(".scss"),
  less: withExt(".less"),
  graphql: withExt(".graphql"),
  yaml: withExt(".yaml"),
  markdown: withExt(".md"),
  html: withExt(".html"),
};

/**
 * Suite-relative path prefixes not run,
 * mostly carried over from the crates' conformance minus the ones needing the dispatcher.
 * "Experimental syntax" is a proposal before stage 3, which oxc does not parse.
 */
const IGNORE = [
  // Bogus nodes
  "typescript/conformance/classes/constructorDeclarations/constructorParameters/readonlyReadonly.ts",
  // Expression syntax: `a?.b = c`
  "js/optional-chaining-assignment/",
  "js/babel-plugins/optional-chaining-assignment.js",
  // Experimental syntax: `do {}`
  "js/async-do-expressions/",
  "js/do",
  "jsx/do/",
  "js/babel-plugins/async-do-expressions.js",
  "js/babel-plugins/do-expressions.js",
  // Experimental syntax: `export X from "mod"`
  "js/export-default/export-default-from",
  "js/export-default/escaped",
  "js/babel-plugins/export-default-from.js",
  // Experimental syntax: `module <id> {}`
  "js/module-blocks",
  "js/babel-plugins/module-blocks.js",
  "js/explicit-resource-management/valid-module-block-",
  // Experimental syntax: pipeline operator `|>`
  "js/comments-pipeline-own-line",
  "js/partial-application",
  "js/pipeline-operator",
  "js/babel-plugins/partial-application.js",
  "js/babel-plugins/pipeline-operator-",
  // Experimental syntax: `::`
  "js/arrows-bind",
  "js/bind-expressions",
  "js/objects/expression.js",
  "js/no-semi-babylon-extensions/no-semi.js",
  "js/babel-plugins/function-bind.js",
  // Experimental syntax: `let { #x: x } = ...`
  "js/destructuring-private-fields",
  "js/babel-plugins/destructuring-private.js",
  // Experimental syntax: `throw` expressions
  "js/throw_expressions",
  "js/babel-plugins/throw-expressions.js",
  // Experimental syntax: discard bindings `void`
  "js/discard-binding/",
  "js/babel-plugins/discard-binding.js",
  // Experimental syntax: `function.sent`
  "js/babel-plugins/function-sent.js",
  // Syntax recovery
  "typescript/error-recovery/",
  // Trailing comma after rest element
  "typescript/trailing-comma/invalid.ts",
  // `<<` right after a heritage clause's or JSX element's type argument `<`
  "typescript/type-arguments-bit-shift-left-like/3.ts",
  "typescript/type-arguments-bit-shift-left-like/5.tsx",
  // Invalid modifier combos (`readonly accessor`, `private abstract`, ambient initializer, ...)
  "typescript/decorator-auto-accessors/decorator-auto-accessors-abstract-class.ts",
  "typescript/decorator-auto-accessors/decorator-auto-accessors-declare-class.ts",
  "typescript/decorator-auto-accessors/decorator-auto-accessors-mixed-modifiers.ts",
  "typescript/class/declare-readonly-field-initializer-w-annotation.ts",
  "typescript/conformance/classes/classDeclarations/classAbstractKeyword/classAbstractMixedWithModifiers.ts",
  "typescript/conformance/classes/classDeclarations/classAbstractKeyword/classAbstractProperties.ts",
  "typescript/conformance/classes/constructorDeclarations/constructorParameters/readonlyInConstructorParameters.ts",
  "typescript/custom/abstract/abstractProperties.ts",
  "typescript/declare/declare-class-fields.ts",
  "typescript/declare/declare-module.ts",
  "typescript/declare/declare-namespace.ts",
  "typescript/interface2/module.ts",
  "typescript/type-parameters-arguments/const.ts",
  // Other early errors (duplicate import attributes, `new.target` outside functions, bad escapes, ...)
  "js/import-assertions/multi-types.js",
  "js/import-attributes/multi-types.js",
  "js/multiparser-invalid/text.js",
  "js/new-target/outside-functions.js",
  "typescript/conformance/types/functions/functionOverloadErrorsSyntax.ts",
  "typescript/module/keyword.ts",
  "typescript/namespace/invalid-await.ts",
  "typescript/definite/definite.ts",
  "typescript/definite/without-annotation.ts",
  // TS syntax in a JS file, which Prettier parses with `typescript` but oxfmt as JS
  "jsx/expression-with-types/expression.js",
  "typescript/tsx/comma/snippet: test.js",
  "typescript/tsx/comma/snippet: test.cjs",
  "typescript/tsx/comma/snippet: test.mjs",
  "js/babel-plugins/typescript.js",
  // Ambiguous await
  "js/top-level-await",
  "jsx/top-level-await",
  "typescript/top-level-await",
  "js/ternaries/parenthesis/await-expression.js",
  "js/explicit-resource-management/for-await-using-of-comments.js",
  "js/explicit-resource-management/valid-using-as-identifier-for-await-of.js",
  // Top-level `await (1)` with no import/export: Prettier always parses `.js` as ESM (await expression),
  // while our unambiguous detection leans to script (call expression), whose output is valid under both
  "js/await/like-call.js",
  // ES5 vs ES6+ identifier: Prettier uses ES5 validation, OXC uses ES6+
  // Characters outside BMP (like U+102A7) are valid ES6+ identifiers but not ES5
  "js/quotes/objects.js",
  // Flow is not supported (`typescript/**/consistent-with-flow/` are TS)
  "flow/",
  "misc/babel-redirect-to-babel-flow/",
  "misc/filename-matters/flow-ambient/",
  "js/babel-plugins/flow.js",

  // postcss-conditionals (archived: https://github.com/andyjansson/postcss-conditionals)
  "css/atrule/if-else.css",

  // Depends on Prettier's test-only plugins (`tests/config/prettier-plugins/`)
  "misc/front-matter/with-plugins/",
  "misc/plugins/",
];

/** Sections not run, by their options */
const skipSection = (options: Options) =>
  // Prettier's runner replaces the `<|>` placeholder in the input (range sections are not even parsed)
  "cursorOffset" in options
  // Rejected by oxfmt's config, not supported yet
  || options.experimentalTernaries === true
  // Beyond what oxfmt's config holds (a `u8`), like `printWidth: Infinity` in `parseSection()`
  || (typeof options.tabWidth === "number" && options.tabWidth > 255)
  // No pragma options in oxfmt
  || options.insertPragma === true
  || options.requirePragma === true
  || options.checkIgnorePragma === true;

// Same as `printSeparator()` in Prettier's `tests/config/format-test/create-snapshot.js`
const separator = (label = "") => {
  const left = Math.floor((80 - label.length) / 2);
  return "=".repeat(left) + label + "=".repeat(80 - left - label.length);
};
const OPTIONS_HEADER = `${separator("options")}\n`;
const INPUT_SEPARATOR = `\n${separator("input")}\n`;
const OUTPUT_SEPARATOR = `\n${separator("output")}\n`;
const END_SEPARATOR = `\n${separator()}`;

const BOM = "\uFEFF";
const EOL_BY_NAME: Record<string, string> = { LF: "\n", CRLF: "\r\n", CR: "\r" };
const NAME_BY_EOL: Record<string, string> = { "\n": "LF", "\r\n": "CRLF", "\r": "CR" };

type Options = Record<string, unknown>;
type Row = {
  total: number;
  passed: number;
  failed: number;
  /** Line similarity summed over the runs, a passed one counts as 1 */
  ratio: number;
  /** The first error oxfmt reported */
  error: string | null;
  /** `null` when idempotent, else whether the second pass errored */
  nonIdempotent: boolean | null;
  variants: Set<string>;
};

const { filter } = parseArgs({ options: { filter: { type: "string" } } }).values;
const rowsByLanguage = new Map<Language, Map<string, Row>>();

for (const snapPath of globSync("**/__snapshots__/format.test.js.snap", {
  cwd: SUITE_DIR,
}).sort()) {
  const dir = dirname(dirname(snapPath));
  const entries = [
    ...readFileSync(join(SUITE_DIR, snapPath), "utf8").matchAll(
      /^exports\[`((?:[^`\\]|\\.)*)`\] = `((?:[^`\\]|\\.)*)`;$/gms,
    ),
  ].map(([, key, body]) => [unescape(key), body] as const);
  // Keys without the jest counter, `<title> format[<parser>]` holds the error of a parser expected to throw
  const keys = new Set(entries.map(([key]) => key.replace(/ \d+$/, "")));

  for (const [key, body] of entries) {
    const title = /^(.*) format \d+$/s.exec(key)?.[1];
    if (title === undefined) continue;
    const section = parseSection(unescape(body));
    if (section === null || skipSection(section.options)) continue;

    const name = title.replace(/ - \{.*\}$/, "");
    const path = `${dir}/${name}`;
    if (IGNORE.some((ignore) => path.startsWith(ignore))) continue;
    if (filter && !path.includes(filter)) continue;

    const { parsers, options, expected } = section;
    const input = name.startsWith("snippet: ")
      ? "endOfLine" in options
        ? section.input.replace(/<(CRLF|CR|LF)>\n/g, (_, eol: string) => EOL_BY_NAME[eol])
        : section.input
      : readFileSync(join(SUITE_DIR, path), "utf8");

    // JS/TS runs once by the first parser listed, as Prettier asserts they all print the same:
    // a JSX snippet is not also run as `.ts`, and one parser expected to throw still leaves another's output
    const ran = new Set<string>();
    for (const parser of parsers) {
      const filename = PARSERS[parser]?.(name);
      if (filename === undefined) continue;
      const key = /\.[cm]?[jt]sx?$/.test(filename) ? "js" : filename;
      if (ran.has(key) || (key !== "js" && keys.has(`${title} format[${parser}]`))) continue;
      ran.add(key);

      const result = await runCase(`${dir}/${filename}`, input, options, expected);
      if (filter) printResult(`${path} [${parser}]`, options, expected, result);

      const language = LANGUAGES[extname(filename)];
      const rows = rowsByLanguage.get(language) ?? new Map<string, Row>();
      rowsByLanguage.set(language, rows);
      const row = rows.get(path) ?? {
        total: 0,
        passed: 0,
        failed: 0,
        ratio: 0,
        error: null,
        nonIdempotent: null,
        variants: new Set(),
      };
      rows.set(path, row);

      row.total++;
      if (result.error !== undefined) {
        row.error ??= result.error;
        continue;
      }
      if (result.passed) row.passed++;
      else row.failed++;
      row.ratio += result.ratio;
      if (result.nonIdempotent !== null) row.nonIdempotent ||= result.nonIdempotent;
      for (const variant of result.variants) row.variants.add(variant);
    }
  }
}

if (!filter) writeReport();

// ---

/** Inverse of the template-literal escaping jest applies (`` ` ``, `\` and `${` get a leading `\`). */
function unescape(text: string) {
  return text.replace(/\\([\s\S])/g, "$1");
}

/** `null` for a section without an options block (error snapshots, range-visualized inputs) or with a non-JSON option. */
function parseSection(body: string) {
  // jest's raw serializer wraps the body in a leading and a trailing newline
  const content = body.slice(1, -1);
  if (!content.startsWith(OPTIONS_HEADER) || !content.endsWith(END_SEPARATOR)) return null;
  const inputStart = content.indexOf(INPUT_SEPARATOR);
  const outputStart = content.indexOf(OUTPUT_SEPARATOR, inputStart);
  if (inputStart === -1 || outputStart === -1) return null;

  const block: Options = {};
  for (const line of content.slice(OPTIONS_HEADER.length, inputStart).split("\n")) {
    // The indented line is the `printWidth` indicator
    if (line === "" || line.startsWith(" ")) continue;
    const [key, value] = line.split(/: (.*)/s);
    try {
      block[key] = JSON.parse(value);
    } catch {
      // Non-JSON values (`printWidth: Infinity`) cannot be given to oxfmt
      return null;
    }
  }
  const { parsers = [], ...options } = block as Options & { parsers?: string[] };

  return {
    parsers,
    options,
    input: content.slice(inputStart + INPUT_SEPARATOR.length, outputStart),
    expected: content.slice(outputStart + OUTPUT_SEPARATOR.length, -END_SEPARATOR.length),
  };
}

/** `code` is `null` when oxfmt reports an error (parse, unsupported file, invalid option). */
async function formatWithOxfmt(filename: string, input: string, options: Options) {
  // Prettier's default, oxfmt's is 100
  const res = await format(filename, input, { printWidth: 80, ...options });
  return res.errors.length === 0
    ? { code: res.code, error: "" }
    : { code: null, error: res.errors[0].message.split("\n")[0] };
}

async function runCase(filename: string, input: string, options: Options, expected: string) {
  const first = await formatWithOxfmt(filename, input, options);
  if (first.code === null) return { error: first.error };
  const actual = first.code;

  // Prettier visualizes the line endings of both input and output when `endOfLine` is set
  const compared =
    "endOfLine" in options
      ? actual.replace(/\r\n?|\n/g, (eol) => `<${NAME_BY_EOL[eol]}>\n`)
      : actual;
  const passed = compared === expected;

  // Line endings are normalized and a BOM is kept, so these must produce the same output.
  // Inputs already containing `\r` or a BOM are skipped, their variants would not be equivalent.
  const variants: [name: string, input: string, output: string][] = [];
  if (!input.includes("\r")) {
    variants.push(
      ["CRLF", input.replaceAll("\n", "\r\n"), actual],
      ["CR", input.replaceAll("\n", "\r"), actual],
    );
  }
  if (!input.startsWith(BOM)) variants.push(["BOM", BOM + input, BOM + actual]);

  // The second pass (idempotency) and the variants are independent of each other
  const [second, ...outputs] = await Promise.all(
    [actual, ...variants.map(([, text]) => text)].map((text) =>
      formatWithOxfmt(filename, text, options),
    ),
  );

  return {
    compared,
    passed,
    ratio: passed ? 1 : similarity(expected, compared),
    nonIdempotent: second.code === actual ? null : second.code === null,
    variants: variants.filter(([, , output], i) => outputs[i].code !== output).map(([n]) => n),
  };
}

/** Line similarity in `0..=1`, like `similar`'s `TextDiff::ratio()` the crates' conformance used. */
function similarity(a: string, b: string) {
  let common = 0;
  let total = 0;
  for (const change of diffLines(a, b)) {
    const count = change.count ?? 0;
    if (change.added || change.removed) {
      total += count;
    } else {
      common += count;
      total += count * 2;
    }
  }
  return total === 0 ? 1 : (common * 2) / total;
}

function printResult(
  label: string,
  options: Options,
  expected: string,
  result: Awaited<ReturnType<typeof runCase>>,
) {
  console.log(`${label} ${JSON.stringify(options)}`);
  if (result.error !== undefined) {
    console.log(`  => Error: ${result.error}\n`);
    return;
  }
  if (result.passed) console.log("  => Passed");
  else console.log(createTwoFilesPatch("prettier", "oxfmt", expected, result.compared));
  if (result.nonIdempotent !== null) {
    console.log(`  => Not idempotent${result.nonIdempotent ? " (second pass errored)" : ""}`);
  }
  if (result.variants.length > 0) {
    console.log(`  => Input variants differ: ${result.variants.join(", ")}`);
  }
  console.log();
}

function writeReport() {
  const lines: string[] = [];
  for (const language of new Set(Object.values(LANGUAGES))) {
    const rows = rowsByLanguage.get(language);
    if (!rows) continue;
    const sorted = [...rows].sort(([a], [b]) => a.localeCompare(b));
    let total = 0;
    let passed = 0;
    for (const [, row] of sorted) {
      total += row.total;
      passed += row.passed;
    }
    const summary = `${passed}/${total} (${((passed / total) * 100).toFixed(2)}%)`;
    console.log(`[${language}] ${summary}`);
    lines.push(`## ${language}`, "", summary, "");

    const failed = sorted.filter(([, row]) => row.failed > 0);
    if (failed.length > 0) {
      lines.push("### Failed", "");
      lines.push("| Spec path | Failed or Passed | Match ratio |");
      lines.push("| :-------- | :--------------: | :---------: |");
      for (const [path, row] of failed) {
        const marks = "💥".repeat(row.failed) + "✨".repeat(row.passed);
        lines.push(`| ${path} | ${marks} | ${((row.ratio / row.total) * 100).toFixed(2)}% |`);
      }
      lines.push("");
    }

    const lists: [string, (row: Row) => string | null][] = [
      [
        "Not idempotent",
        (r) => (r.nonIdempotent === null ? null : r.nonIdempotent ? " (second pass errored)" : ""),
      ],
      [
        "Input variants differ",
        (r) => (r.variants.size > 0 ? ` (${[...r.variants].join(", ")})` : null),
      ],
      ["Error", (r) => (r.error === null ? null : `: ${r.error}`)],
    ];
    for (const [heading, note] of lists) {
      const entries = sorted.flatMap(([path, row]) => {
        const text = note(row);
        return text === null ? [] : [`- ${path}${text}`];
      });
      if (entries.length > 0) lines.push(`### ${heading}`, "", ...entries, "");
    }
  }

  mkdirSync(SNAPSHOTS_DIR, { recursive: true });
  const outPath = join(SNAPSHOTS_DIR, "prettier.snap.md");
  writeFileSync(outPath, lines.join("\n"));
  console.log(`Report written to ${relative(process.cwd(), outPath)}`);
}
