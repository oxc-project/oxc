// Source map tests.
//
// The conformance checker compares the complete Source Map v3 object against Rust `oxc_codegen`.
// The remaining tests here check invariants which give a more local failure than a full mapping diff -
// positions stay in bounds and ordered, indentation is respected, identifier text agrees,
// and turning source maps on does not change the generated code.
//
// Mappings use Oxc `start` / `end` offsets.

import { existsSync, readFileSync } from "node:fs";
import { join as pathJoin } from "node:path";
import { parseSync } from "oxc-parser";
import { beforeAll, describe, expect, test } from "vitest";

import { printSync } from "../dist/index.js";
import { checkFixture, getEcmaScriptLineTable } from "./utils/common.ts";

import type { Program } from "oxc-parser";
import type { SourceMap } from "../dist/index.js";

// Same directory the benchmarks download their fixtures to. Whichever are already cached are used.
// The inline fixtures below always run.
const CACHE_DIR_PATH = pathJoin(import.meta.dirname, "../../../target");

// `preserveParens: false` because this printer deliberately does not support the redundant
// `ParenthesizedExpression` / `TSParenthesizedType` nodes. `experimentalRawTransfer` is how the
// printer is meant to be used, so the ASTs here are the shape it really sees.
const PARSE_OPTIONS = {
  preserveParens: false,
  experimentalRawTransfer: true,
};

const INLINE_JS = `
class Foo extends Bar {
	static #x = 1;
	get y() {
		if (this.z) {
			for (const k of [1, 2, 3]) {
				console.log(\`\${k}-\${this.z}\`);
			}
		}
		return { a: 1, b: 2, c: this.#x };
	}
}
function outer(a, b = 2, ...rest) {
	label: while (a) {
		try {
			a = b;
		} catch (e) {
			break label;
		} finally {
			b++;
		}
	}
	return outer;
}
export default outer;
`;

const INLINE_TS = `
namespace NS {
	export interface Shape<T extends object = {}> {
		readonly kind: "a" | "b";
		fn?(x: T): asserts x is T;
	}
	export abstract class Impl<T> implements Shape<object> {
		declare readonly kind: "a";
		constructor(private readonly x: number, protected y?: string) { super(); }
		abstract go<U>(v: U): U;
	}
	export enum E { A = 1, B }
	export type Mapped<T> = { [K in keyof T]-?: T[K] };
}
export default NS;
`;

const INLINE_UNICODE = 'const smile = "😀";\r\nconst café = `first\u2028second`;\nsmile + café;';

describe("Rust conformance", () => {
  test.each([
    { name: "inline.js", code: INLINE_JS, lang: "js" as const },
    { name: "inline.ts", code: INLINE_TS, lang: "ts" as const },
    {
      name: "unicode.js",
      code: INLINE_UNICODE,
      lang: "js" as const,
    },
    {
      name: "escaped-name.js",
      code: "const \\u0061 = 1; \\u0061;",
      lang: "js" as const,
    },
    { name: "astral-new.js", code: "new 𐐀;", lang: "js" as const },
    {
      name: "large-line-gap.js",
      code: `\`${"\n".repeat(200)}\`;\nvalue;`,
      lang: "js" as const,
    },
    {
      name: "sparse-uncommon-line.js",
      code: `value;${" ".repeat(20000)}\u2028`,
      lang: "js" as const,
    },
    {
      name: "many-crlf.js",
      code: Array.from({ length: 100 }, (_, index) => `x${index};`).join("\r\n"),
      lang: "js" as const,
    },
  ])("$name mappings match oxc_codegen", ({ name, code, lang }) => {
    expect(checkFixture(name, code, lang, "module")).toBe(true);
  });

  test("invalid source offsets fall back to the printed name", () => {
    const code = "const name = 0;";
    const program = parseProgram("invalid-offset.js", code);
    const statement = program.body[0];
    if (statement.type !== "VariableDeclaration") throw new Error("Expected variable declaration");
    const identifier = statement.declarations[0].id;
    if (identifier.type !== "Identifier") throw new Error("Expected identifier");

    // Simulate a transformed AST whose old offsets are no longer valid for the source text
    identifier.end = code.length + 1;

    const { map } = printSync(program, {
      sourcemap: true,
      sourceFilename: "invalid-offset.js",
      sourceText: code,
    });
    expect(decodeSourceMap(map!)).toContainEqual(expect.objectContaining({ name: "name" }));
  });

  test("in-bounds non-identifier offsets fall back to the printed name", () => {
    const code = "const name = 0;";
    const program = parseProgram("stale-offset.js", code);
    const statement = program.body[0];
    if (statement.type !== "VariableDeclaration") throw new Error("Expected variable declaration");
    const identifier = statement.declarations[0].id;
    if (identifier.type !== "Identifier") throw new Error("Expected identifier");

    // Simulate a transformed AST whose stale offsets are in bounds but start on whitespace
    identifier.start = code.indexOf(" ");
    identifier.end = identifier.start + 1;

    const { map } = printSync(program, {
      sourcemap: true,
      sourceFilename: "stale-offset.js",
      sourceText: code,
    });
    const mappings = decodeSourceMap(map!);
    expect(mappings).toContainEqual(expect.objectContaining({ name: "name" }));
    expect(mappings).not.toContainEqual(expect.objectContaining({ name: "" }));
  });

  test("stale identifier offsets preserve the original longer spelling", () => {
    const code = "const ab = 0;";
    const program = parseProgram("renamed.js", code);
    const statement = program.body[0];
    if (statement.type !== "VariableDeclaration") throw new Error("Expected variable declaration");
    const identifier = statement.declarations[0].id;
    if (identifier.type !== "Identifier") throw new Error("Expected identifier");
    identifier.name = "a";

    const { map } = printSync(program, {
      sourcemap: true,
      sourceFilename: "renamed.js",
      sourceText: code,
    });
    expect(map?.names).toEqual(["ab"]);
  });

  test("handles transformed ASTs whose source locations move backwards", () => {
    const code = `const first = 1;\n${"\n".repeat(5000)}const second = 2;`;
    const program = parseProgram("reordered.js", code);
    program.body.reverse();

    const { map } = printSync(program, {
      sourcemap: true,
      sourceFilename: "reordered.js",
      sourceText: code,
    });
    const mappings = decodeSourceMap(map!);
    expect(mappings.find((mapping) => mapping.generatedLine === 1)?.originalLine).toBe(5002);
    expect(mappings.find((mapping) => mapping.generatedLine === 2)?.originalLine).toBe(1);
  });

  test("returns a source map only when requested", () => {
    const code = "const value = 1;";
    const program = parseProgram("return-map.js", code);
    expect(printSync(program).map).toBeNull();

    const { map } = printSync(program, {
      sourcemap: true,
      sourceFilename: "return-map.js",
      sourceText: code,
    });
    expect(map).toMatchObject({
      version: 3,
      sources: ["return-map.js"],
      sourcesContent: [code],
    });
  });
});

describe("source map options", () => {
  const code = "const value = 1;";
  const program = parseProgram("options.js", code);

  test.each([
    ["missing", undefined],
    ["null", null],
    ["number", 1],
  ])("rejects %s sourceText", (_name, sourceText) => {
    expect(() =>
      printSync(program, { sourcemap: true, sourceText: sourceText as unknown as string }),
    ).toThrow(new TypeError("`sourceText` must be a string when `sourcemap` is true"));
  });

  test.each([
    ["null", null],
    ["number", 1],
  ])("rejects %s sourceFilename", (_name, sourceFilename) => {
    expect(() =>
      printSync(program, {
        sourcemap: true,
        sourceText: code,
        sourceFilename: sourceFilename as unknown as string,
      }),
    ).toThrow(new TypeError("`sourceFilename` must be a string when supplied"));
  });

  test("accepts valid source map options", () => {
    expect(
      printSync(program, {
        sourcemap: true,
        sourceText: code,
        sourceFilename: "options.js",
      }).map,
    ).toMatchObject({
      sources: ["options.js"],
      sourcesContent: [code],
    });
  });
});

interface Fixture {
  name: string;
  /** Source text, or `null` if the fixture is a cached file which has not been downloaded */
  code: string | null;
  ts: boolean;
  jsx: boolean;
}

/**
 * Read a fixture the benchmarks download, or `null` if it is not in the cache.
 *
 * @param name - Fixture filename
 * @returns Source text, or `null` if not cached
 */
function cached(name: string): string | null {
  const path = pathJoin(CACHE_DIR_PATH, name);
  return existsSync(path) ? readFileSync(path, "utf8") : null;
}

const FIXTURES: Fixture[] = [
  { name: "inline.js", code: INLINE_JS, ts: false, jsx: false },
  { name: "inline.ts", code: INLINE_TS, ts: true, jsx: false },
  { name: "unicode.js", code: INLINE_UNICODE, ts: false, jsx: false },
  { name: "react.development.js", code: cached("react.development.js"), ts: false, jsx: false },
  {
    name: "RadixUIAdoptionSection.jsx",
    code: cached("RadixUIAdoptionSection.jsx"),
    ts: false,
    jsx: true,
  },
  { name: "binder.ts", code: cached("binder.ts"), ts: true, jsx: false },
  { name: "kitchen-sink.tsx", code: cached("kitchen-sink.tsx"), ts: true, jsx: true },
];

// --- Helpers --------------------------------------------------------------------------------

/** One decoded mapping used by the local invariant checks. */
interface Recorded {
  generatedLine: number;
  generatedColumn: number;
  originalLine: number;
  originalColumn: number;
  name: string | undefined;
  source: string;
}

/**
 * Decode a Source Map v3 mapping string for the local invariant checks below.
 */
function decodeSourceMap(map: SourceMap): Recorded[] {
  const mappings: Recorded[] = [];
  let sourceId = 0;
  let originalLine = 0;
  let originalColumn = 0;
  let nameId = 0;

  const lines = map.mappings.split(";");
  for (let generatedLine = 0; generatedLine < lines.length; generatedLine++) {
    const line = lines[generatedLine];
    if (line === "") continue;

    let generatedColumn = 0;
    for (const segment of line.split(",")) {
      const values = decodeVlqSegment(segment);
      generatedColumn += values[0];
      sourceId += values[1];
      originalLine += values[2];
      originalColumn += values[3];
      if (values.length === 5) nameId += values[4];

      const source = map.sources[sourceId];
      if (source === undefined) throw new Error(`Invalid source ID ${sourceId}`);
      mappings.push({
        generatedLine: generatedLine + 1,
        generatedColumn,
        originalLine: originalLine + 1,
        originalColumn,
        name: values.length === 5 ? map.names[nameId] : undefined,
        source,
      });
    }
  }
  return mappings;
}

/**
 * Decode one comma-delimited source-map segment.
 */
function decodeVlqSegment(segment: string): number[] {
  const values: number[] = [];
  let value = 0;
  let factor = 1;
  for (const char of segment) {
    const digit = BASE64_CHARS.indexOf(char);
    if (digit === -1) throw new Error(`Invalid base64 VLQ digit ${JSON.stringify(char)}`);
    value += (digit % 32) * factor;
    if (digit < 32) {
      values.push(value % 2 === 1 ? -Math.floor(value / 2) : Math.floor(value / 2));
      value = 0;
      factor = 1;
    } else {
      factor *= 32;
    }
  }
  if (factor !== 1) throw new Error("Unterminated base64 VLQ value");
  return values;
}

const BASE64_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/**
 * Parse a fixture.
 *
 * @param name - Fixture filename
 * @param code - Source text
 * @returns The AST
 */
function parseProgram(name: string, code: string): Program {
  const { program, errors } = parseSync(name, code, PARSE_OPTIONS);
  if (errors.length > 0) throw new Error(`parse ${name}: ${errors[0].message}`);
  return program;
}

const IDENT_RE = /^[A-Za-z_$][A-Za-z0-9_$]*/;

// --- Per-fixture tests ----------------------------------------------------------------------

interface Printed {
  withMaps: string;
  withoutMaps: string;
  mappings: Recorded[];
  outLines: string[];
  srcLines: string[];
}

function addFixtureTests(fixture: Fixture) {
  let printed!: Printed;

  beforeAll(() => {
    const { code } = fixture;
    if (code === null) return;

    const program = parseProgram(fixture.name, code);
    const { jsx, ts } = fixture;

    // Both builds through the public API.
    // `printSync` picks the maps build when `sourcemap` is true, and the no-maps build when it is not.
    const { code: withMaps, map } = printSync(program, {
      jsx,
      ts,
      sourcemap: true,
      sourceFilename: fixture.name,
      sourceText: code,
    });
    const { code: withoutMaps } = printSync(program, { jsx, ts });

    printed = {
      withMaps,
      withoutMaps,
      mappings: decodeSourceMap(map!),
      outLines: getEcmaScriptLineTable(withMaps).lines,
      srcLines: getEcmaScriptLineTable(code).lines,
    };
  });

  test("source maps do not change the printed code", () => {
    expect(printed.withMaps).toBe(printed.withoutMaps);
  });

  test("emits mappings", () => {
    expect(printed.mappings.length).toBeGreaterThan(0);
  });

  test("generated positions never go backwards", () => {
    let prevLine = 0,
      prevCol = 0;
    let outOfOrder = null;
    for (const mapping of printed.mappings) {
      const { generatedLine: line, generatedColumn: col } = mapping;
      if (line < prevLine || (line === prevLine && col < prevCol)) {
        outOfOrder = `${line}:${col} follows ${prevLine}:${prevCol}`;
        break;
      }
      prevLine = line;
      prevCol = col;
    }
    expect(outOfOrder).toBeNull();
  });

  test("generated positions are within the printed output", () => {
    let outOfRange = null;
    for (const { generatedLine, generatedColumn } of printed.mappings) {
      const line = printed.outLines[generatedLine - 1];
      if (line === undefined || generatedColumn > line.length) {
        outOfRange = `${generatedLine}:${generatedColumn}`;
        break;
      }
    }
    expect(outOfRange).toBeNull();
  });

  test("original positions are within the source", () => {
    let outOfRange = null;
    for (const { originalLine, originalColumn } of printed.mappings) {
      const line = printed.srcLines[originalLine - 1];
      if (line === undefined || originalColumn > line.length) {
        outOfRange = `${originalLine}:${originalColumn}`;
        break;
      }
    }
    expect(outOfRange).toBeNull();
  });

  test("every mapping names the source file", () => {
    const wrong = printed.mappings.find((mapping) => mapping.source !== fixture.name);
    expect(wrong).toBeUndefined();
  });

  // Where a mapping lands on an identifier in the output, the source position it points at
  // should hold the same identifier. A handful disagree legitimately - a printed name can come
  // from a node whose span starts at a keyword - so this is a ratio rather than an absolute.
  test("identifier text agrees at mapped positions", () => {
    let checked = 0,
      mismatched = 0;
    for (const mapping of printed.mappings) {
      const outLine = printed.outLines[mapping.generatedLine - 1];
      const srcLine = printed.srcLines[mapping.originalLine - 1];
      if (outLine === undefined || srcLine === undefined) continue;
      const generatedIdent = IDENT_RE.exec(outLine.slice(mapping.generatedColumn));
      const originalIdent = IDENT_RE.exec(srcLine.slice(mapping.originalColumn));
      if (generatedIdent !== null && originalIdent !== null) {
        checked++;
        if (generatedIdent[0] !== originalIdent[0]) mismatched++;
      }
    }
    // Guard against the ratio passing because nothing was compared
    expect(checked).toBeGreaterThan(0);
    expect(1 - mismatched / checked).toBeGreaterThan(0.97);
  });
}

// A cached fixture which has not been downloaded is reported as skipped rather than passing quietly.
// `pnpm run bench` downloads them.
const CACHED_FIXTURES = FIXTURES.filter((fixture) => fixture.code !== null);
const MISSING_FIXTURES = FIXTURES.filter((fixture) => fixture.code === null);

describe.for(CACHED_FIXTURES)("$name", (fixture) => {
  addFixtureTests(fixture);
});
describe.for(MISSING_FIXTURES)("$name", { skip: true }, (fixture) => {
  addFixtureTests(fixture);
});

// --- Indentation ----------------------------------------------------------------------------

// `printIndent` goes through the same write path as everything else, so the `indent` option
// and the mappings have to agree: indentation is honoured, and no mapping ever points into the middle
// of an indent run.
const INDENT_CASES = [undefined, "\t", "  ", "    ", "\t\t", " \t", "\t "];

describe("indent option", () => {
  test.each(
    INDENT_CASES.map((indent) => ({ label: JSON.stringify(indent) ?? "undefined", indent })),
  )("indent=$label", ({ indent }) => {
    const program = parseProgram("indent.js", INLINE_JS);
    const { code: out, map } = printSync(program, {
      indent: indent as string | undefined,
      sourcemap: true,
      sourceFilename: "indent.js",
      sourceText: INLINE_JS,
    });

    const expectedIndent = typeof indent === "string" && /^[ \t]+$/.test(indent) ? indent : "\t";
    const { lines } = getEcmaScriptLineTable(out);
    const indented = lines.filter((line) => line.startsWith("\t") || line.startsWith(" "));
    expect(indented.length).toBeGreaterThan(0);

    // Every indented line's leading whitespace is a whole number of indent levels
    for (const line of indented) {
      const lead = /^[\t ]+/.exec(line)![0];
      expect(lead.length % expectedIndent.length).toBe(0);
      expect(lead).toBe(expectedIndent.repeat(lead.length / expectedIndent.length));
    }

    // No mapping points into an indent run
    const mappings = decodeSourceMap(map!);
    expect(mappings.length).toBeGreaterThan(0);
    let insideIndent = null;
    for (const mapping of mappings) {
      const lead = /^[\t ]*/.exec(lines[mapping.generatedLine - 1])![0].length;
      if (mapping.generatedColumn !== 0 && mapping.generatedColumn < lead) {
        insideIndent = `${mapping.generatedLine}:${mapping.generatedColumn}`;
        break;
      }
    }
    expect(insideIndent).toBeNull();
  });
});
