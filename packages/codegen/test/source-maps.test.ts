// Source map tests.
//
// Source maps are the one part of the printer whose output is not the printed code, so the
// conformance suites say nothing about them. These check the mappings a print emits are
// self-consistent and point where they claim to, and that turning source maps on does not change
// the code that comes out.
//
// `oxc-parser` emits no `loc`, so it is synthesized here from `start` / `end` offsets.

import { existsSync, readFileSync } from "node:fs";
import { join as pathJoin } from "node:path";
import { parseSync } from "oxc-parser";
import { beforeAll, describe, expect, test } from "vitest";

import { printSync } from "../dist/index.js";

import type { Program } from "oxc-parser";
import type { Mapping, Position, SourceMapGenerator } from "../dist/index.js";

// Same directory the benchmarks download their fixtures to. Whichever are already cached are used;
// the inline fixtures below always run.
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

/**
 * Build a function converting a source offset to a line and column.
 *
 * @param source - Source text
 * @returns Function taking an offset and returning a 1-based line and 0-based column
 */
function lineColTable(source: string): (offset: number) => Position {
  const lineStarts = [0];
  for (let i = 0; i < source.length; i++) {
    if (source.charCodeAt(i) === 10) lineStarts.push(i + 1);
  }
  return (offset) => {
    let lo = 0,
      hi = lineStarts.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (lineStarts[mid] <= offset) lo = mid;
      else hi = mid - 1;
    }
    return { line: lo + 1, column: offset - lineStarts[lo] };
  };
}

/**
 * Add a `loc` to every node in the AST, computed from its `start` / `end` offsets.
 *
 * @param root - AST to walk
 * @param posOf - Function converting an offset to a line and column
 */
function addLocs(root: object, posOf: (offset: number) => Position): void {
  const seen = new Set<object>();
  const stack: unknown[] = [root];
  while (stack.length > 0) {
    const node = stack.pop();
    if (node === null || typeof node !== "object") continue;
    if (seen.has(node)) continue;
    seen.add(node);
    if (Array.isArray(node)) {
      for (const child of node) stack.push(child);
      continue;
    }
    const record = node as Record<string, unknown>;
    if (typeof record.type === "string" && typeof record.start === "number") {
      record.loc = { start: posOf(record.start), end: posOf(record.end as number) };
    }
    for (const key in record) {
      if (key === "loc" || key === "parent") continue;
      const child = record[key];
      if (child !== null && typeof child === "object") stack.push(child);
    }
  }
}

/** A mapping, copied out of the `Mapping` object the printer reuses across calls. */
interface Recorded {
  generatedLine: number;
  generatedColumn: number;
  originalLine: number;
  originalColumn: number;
  name: string | undefined;
  source: string;
}

/** Collects the mappings a print emits. This is the `sourceMap` option's whole interface. */
class Collector implements SourceMapGenerator {
  file: string;
  mappings: Recorded[] = [];

  constructor(file: string) {
    this.file = file;
  }

  addMapping(mapping: Mapping): void {
    // `Mapping` objects are reused across calls - must copy
    this.mappings.push({
      generatedLine: mapping.generated.line,
      generatedColumn: mapping.generated.column,
      originalLine: mapping.original.line,
      originalColumn: mapping.original.column,
      name: mapping.name,
      source: mapping.source,
    });
  }
}

/**
 * Parse a fixture and add a `loc` to every node.
 *
 * @param name - Fixture filename
 * @param code - Source text
 * @returns The AST
 */
function parseWithLocs(name: string, code: string): Program {
  const { program, errors } = parseSync(name, code, PARSE_OPTIONS);
  if (errors.length > 0) throw new Error(`parse ${name}: ${errors[0].message}`);
  addLocs(program, lineColTable(code));
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

for (const fixture of FIXTURES) {
  // A cached fixture which has not been downloaded is reported as skipped rather than passing
  // quietly. `pnpm run bench` downloads them.
  const describeFixture = fixture.code === null ? describe.skip : describe;

  describeFixture(fixture.name, () => {
    let printed!: Printed;

    beforeAll(() => {
      const code = fixture.code as string;
      const program = parseWithLocs(fixture.name, code);
      const collector = new Collector(fixture.name);
      const { jsx, ts } = fixture;

      // Both builds through the public API - `printSync` picks the maps build when given a
      // `sourceMap`, and the no-maps build when not
      const withMaps = printSync(program, { jsx, ts, sourceMap: collector }).code;
      const withoutMaps = printSync(program, { jsx, ts }).code;

      printed = {
        withMaps,
        withoutMaps,
        mappings: collector.mappings,
        outLines: withMaps.split("\n"),
        srcLines: code.split("\n"),
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
      const wrong = printed.mappings.find(
        (mapping) => mapping.source !== fixture.name && mapping.source !== undefined,
      );
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
  });
}

// --- Indentation ----------------------------------------------------------------------------

// `printIndent` goes through the same write path as everything else, so the `indent` option and
// the mappings have to agree: indentation is honoured (or falls back to a tab), and no mapping
// ever points into the middle of an indent run.
//
// The invalid values are typed `unknown` because they are what an untyped JS caller can pass -
// that they fall back to a tab rather than corrupting the output is the point of testing them.
const INDENT_CASES: unknown[] = [
  // Valid - used as given
  undefined,
  "\t",
  "  ",
  "    ",
  "\t\t",
  " \t",
  "\t ",
  // Invalid - must fall back to a tab
  "",
  "x",
  "\n",
  "\r\n",
  " x ",
  "   ",
  4,
  null,
  {},
];

describe("indent option", () => {
  test.each(
    INDENT_CASES.map((indent) => ({ label: JSON.stringify(indent) ?? "undefined", indent })),
  )("indent=$label", ({ indent }) => {
    const program = parseWithLocs("indent.js", INLINE_JS);
    const collector = new Collector("indent.js");
    const out = printSync(program, {
      indent: indent as string | undefined,
      sourceMap: collector,
    }).code;

    const expectedIndent = typeof indent === "string" && /^[ \t]+$/.test(indent) ? indent : "\t";
    const lines = out.split("\n");
    const indented = lines.filter((line) => line.startsWith("\t") || line.startsWith(" "));
    expect(indented.length).toBeGreaterThan(0);

    // Every indented line's leading whitespace is a whole number of indent levels
    for (const line of indented) {
      const lead = /^[\t ]+/.exec(line)![0];
      expect(lead.length % expectedIndent.length).toBe(0);
      expect(lead).toBe(expectedIndent.repeat(lead.length / expectedIndent.length));
    }

    // No mapping points into an indent run
    let insideIndent = null;
    for (const mapping of collector.mappings) {
      const lead = /^[\t ]*/.exec(lines[mapping.generatedLine - 1])![0].length;
      if (mapping.generatedColumn !== 0 && mapping.generatedColumn < lead) {
        insideIndent = `${mapping.generatedLine}:${mapping.generatedColumn}`;
        break;
      }
    }
    expect(insideIndent).toBeNull();
  });
});
