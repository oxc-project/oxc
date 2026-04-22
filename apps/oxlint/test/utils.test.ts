import { describe, it, expect, beforeEach } from "vitest";
import {
  PATH_REGEXP,
  NORMALIZED_REPO_ROOT,
  FIXTURES_SUBPATH,
  normalizeStdout,
  convertSubPath,
  convertFixturesSubPath,
} from "./utils.ts";

describe("PATH_REGEXP", () => {
  // Reset lastIndex before each test since PATH_REGEXP has the 'g' flag
  beforeEach(() => {
    PATH_REGEXP.lastIndex = 0;
  });

  describe("matches repo root path", () => {
    it("matches bare repo root", () => {
      const input = `Error at ${NORMALIZED_REPO_ROOT} in file`;
      expect(input.match(PATH_REGEXP)).toEqual([NORMALIZED_REPO_ROOT]);
    });

    it("matches repo root with trailing slash", () => {
      const input = `Error at ${NORMALIZED_REPO_ROOT}/ in file`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/`]);
    });

    it("matches repo root with subpath", () => {
      const input = `Error at ${NORMALIZED_REPO_ROOT}/apps/oxlint/test in file`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/apps/oxlint/test`]);
    });

    it("matches repo root at start of string", () => {
      // Note: The regex matches until whitespace, so `:` is included in the match
      const input = `${NORMALIZED_REPO_ROOT}/file.js: error`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js:`]);
    });

    it("matches repo root at end of string", () => {
      const input = `File is at ${NORMALIZED_REPO_ROOT}`;
      expect(input.match(PATH_REGEXP)).toEqual([NORMALIZED_REPO_ROOT]);
    });
  });

  describe("matches paths after various delimiters", () => {
    it("matches after whitespace", () => {
      const input = `Error at ${NORMALIZED_REPO_ROOT}/file.js`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });

    it("matches after opening parenthesis", () => {
      const input = `at function (${NORMALIZED_REPO_ROOT}/file.js:1:1)`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js:1:1`]);
    });

    it("matches after single quote", () => {
      const input = `path is '${NORMALIZED_REPO_ROOT}/file.js'`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });

    it("matches after double quote", () => {
      const input = `path is "${NORMALIZED_REPO_ROOT}/file.js"`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });

    it("matches after backtick", () => {
      const input = `path is \`${NORMALIZED_REPO_ROOT}/file.js\``;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });
  });

  describe("stops at correct delimiters", () => {
    it("stops at whitespace", () => {
      const input = `${NORMALIZED_REPO_ROOT}/file.js is here`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });

    it("stops at closing parenthesis", () => {
      const input = `(${NORMALIZED_REPO_ROOT}/file.js) is here`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });

    it("stops at single quote", () => {
      const input = `'${NORMALIZED_REPO_ROOT}/file.js' is here`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });

    it("stops at double quote", () => {
      const input = `"${NORMALIZED_REPO_ROOT}/file.js" is here`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });

    it("stops at backtick", () => {
      const input = `\`${NORMALIZED_REPO_ROOT}/file.js\` is here`;
      expect(input.match(PATH_REGEXP)).toEqual([`${NORMALIZED_REPO_ROOT}/file.js`]);
    });
  });

  describe("matches multiple paths in one string", () => {
    it("matches two paths separated by space", () => {
      const input = `${NORMALIZED_REPO_ROOT}/a.js ${NORMALIZED_REPO_ROOT}/b.js`;
      expect(input.match(PATH_REGEXP)).toEqual([
        `${NORMALIZED_REPO_ROOT}/a.js`,
        `${NORMALIZED_REPO_ROOT}/b.js`,
      ]);
    });
  });

  describe("does not match invalid contexts", () => {
    it("does not match path preceded by alphanumeric", () => {
      const input = `abc${NORMALIZED_REPO_ROOT}/file.js`;
      expect(input.match(PATH_REGEXP)).toBeNull();
    });

    it("matches path with trailing characters until end of string", () => {
      // The regex matches until a delimiter or end of string
      // So `file.jsabc` is included because there's no delimiter before end of string
      const input = `${NORMALIZED_REPO_ROOT}/file.jsabc`;
      const match = input.match(PATH_REGEXP);
      expect(match).toEqual([`${NORMALIZED_REPO_ROOT}/file.jsabc`]);
    });
  });
});

describe("convertFixturesSubPath", () => {
  const fixtureName = "my-fixture";

  it("converts fixture file path to <fixture>/path", () => {
    expect(convertFixturesSubPath("/my-fixture/file.js", fixtureName)).toBe("<fixture>/file.js");
  });

  it("converts nested fixture file path", () => {
    expect(convertFixturesSubPath("/my-fixture/src/nested/file.js", fixtureName)).toBe(
      "<fixture>/src/nested/file.js",
    );
  });

  it("converts fixture root to <fixture>", () => {
    // When the path is exactly the fixture name (no subpath), it returns <fixture>
    expect(convertFixturesSubPath("/my-fixture", fixtureName)).toBe("<fixture>");
  });

  it("converts other fixture path to <fixtures>/path", () => {
    expect(convertFixturesSubPath("/other-fixture/file.js", fixtureName)).toBe(
      "<fixtures>/other-fixture/file.js",
    );
  });

  it("converts bare other fixture name to <fixtures>/name", () => {
    expect(convertFixturesSubPath("/other-fixture", fixtureName)).toBe("<fixtures>/other-fixture");
  });

  it("handles fixture name as prefix of other fixture", () => {
    // my-fixture-extra should not match my-fixture
    expect(convertFixturesSubPath("/my-fixture-extra/file.js", fixtureName)).toBe(
      "<fixtures>/my-fixture-extra/file.js",
    );
  });
});

describe("convertSubPath", () => {
  const fixtureName = "my-fixture";

  it("converts fixture file path to <fixture>/path", () => {
    const result = convertSubPath(`${FIXTURES_SUBPATH}/my-fixture/file.js`, fixtureName);
    expect(result).toBe("<fixture>/file.js");
  });

  it("converts fixtures directory to <fixtures>", () => {
    const result = convertSubPath(FIXTURES_SUBPATH, fixtureName);
    expect(result).toBe("<fixtures>");
  });

  it("converts other repo paths to <root>/path", () => {
    const result = convertSubPath("/crates/oxc_linter/src/lib.rs", fixtureName);
    expect(result).toBe("<root>/crates/oxc_linter/src/lib.rs");
  });

  it("converts root path to <root>/path", () => {
    const result = convertSubPath("/apps/oxlint/src/main.rs", fixtureName);
    expect(result).toBe("<root>/apps/oxlint/src/main.rs");
  });

  it("handles different fixture in fixtures directory", () => {
    const result = convertSubPath(`${FIXTURES_SUBPATH}/other-fixture/file.js`, fixtureName);
    expect(result).toBe("<fixtures>/other-fixture/file.js");
  });
});

describe("normalizeStdout", () => {
  const fixtureName = "test-fixture";

  describe("line break normalization", () => {
    it("normalizes CRLF to LF", () => {
      const result = normalizeStdout("line1\r\nline2\r\n", fixtureName, false);
      expect(result).toBe("line1\nline2\n");
    });

    it("normalizes CR to LF", () => {
      const result = normalizeStdout("line1\rline2\r", fixtureName, false);
      expect(result).toBe("line1\nline2\n");
    });

    it("trims leading and trailing newlines", () => {
      const result = normalizeStdout("\n\nline1\nline2\n\n", fixtureName, false);
      expect(result).toBe("line1\nline2\n");
    });

    it("returns empty string for empty input", () => {
      expect(normalizeStdout("", fixtureName, false)).toBe("");
    });

    it("returns empty string for whitespace-only input", () => {
      expect(normalizeStdout("\n\n\n", fixtureName, false)).toBe("");
    });
  });

  describe("timing line normalization", () => {
    it("normalizes timing with milliseconds", () => {
      const result = normalizeStdout(
        "Finished in 123ms on 4 files with 10 rules using 2 threads.",
        fixtureName,
        false,
      );
      expect(result).toBe("Finished in Xms on 4 files with 10 rules using X threads.\n");
    });

    it("normalizes timing with seconds", () => {
      const result = normalizeStdout(
        "Finished in 1.23s on 1 file with 5 rules using 8 threads.",
        fixtureName,
        false,
      );
      expect(result).toBe("Finished in Xms on 1 file with 5 rules using X threads.\n");
    });

    it("normalizes timing with microseconds", () => {
      const result = normalizeStdout(
        "Finished in 456us on 2 files with 3 rules using 4 threads.",
        fixtureName,
        false,
      );
      expect(result).toBe("Finished in Xms on 2 files with 3 rules using X threads.\n");
    });

    it("normalizes timing with nanoseconds", () => {
      const result = normalizeStdout(
        "Finished in 789ns on 1 file with 1 rules using 1 threads.",
        fixtureName,
        false,
      );
      expect(result).toBe("Finished in Xms on 1 file with 1 rules using X threads.\n");
    });

    it("normalizes timing without rules count", () => {
      const result = normalizeStdout(
        "Finished in 100ms on 3 files using 4 threads.",
        fixtureName,
        false,
      );
      expect(result).toBe("Finished in Xms on 3 files using X threads.\n");
    });

    it("handles singular file", () => {
      const result = normalizeStdout(
        "Finished in 50ms on 1 file with 2 rules using 1 threads.",
        fixtureName,
        false,
      );
      expect(result).toBe("Finished in Xms on 1 file with 2 rules using X threads.\n");
    });
  });

  describe("path normalization", () => {
    it("replaces repo root path with <root>", () => {
      const result = normalizeStdout(
        `Error at ${NORMALIZED_REPO_ROOT}/crates/file.rs`,
        fixtureName,
        false,
      );
      expect(result).toBe("Error at <root>/crates/file.rs\n");
    });

    it("replaces bare repo root with <root>", () => {
      const result = normalizeStdout(`Path is ${NORMALIZED_REPO_ROOT}`, fixtureName, false);
      expect(result).toBe("Path is <root>\n");
    });

    it("replaces fixture path with <fixture>", () => {
      const result = normalizeStdout(
        `Error in ${NORMALIZED_REPO_ROOT}${FIXTURES_SUBPATH}/test-fixture/file.js`,
        fixtureName,
        false,
      );
      expect(result).toBe("Error in <fixture>/file.js\n");
    });

    it("replaces fixtures directory with <fixtures>", () => {
      const result = normalizeStdout(
        `Directory is ${NORMALIZED_REPO_ROOT}${FIXTURES_SUBPATH}`,
        fixtureName,
        false,
      );
      expect(result).toBe("Directory is <fixtures>\n");
    });

    it("replaces other fixture path with <fixtures>/name", () => {
      const result = normalizeStdout(
        `Error in ${NORMALIZED_REPO_ROOT}${FIXTURES_SUBPATH}/other-fixture/file.js`,
        fixtureName,
        false,
      );
      expect(result).toBe("Error in <fixtures>/other-fixture/file.js\n");
    });

    it("handles paths in parentheses", () => {
      const result = normalizeStdout(
        `at function (${NORMALIZED_REPO_ROOT}/file.js:1:1)`,
        fixtureName,
        false,
      );
      expect(result).toBe("at function (<root>/file.js:1:1)\n");
    });

    it("handles paths in quotes", () => {
      const result = normalizeStdout(
        `path is "${NORMALIZED_REPO_ROOT}/file.js"`,
        fixtureName,
        false,
      );
      expect(result).toBe('path is "<root>/file.js"\n');
    });

    it("handles multiple paths in one line", () => {
      const result = normalizeStdout(
        `From ${NORMALIZED_REPO_ROOT}/a.js to ${NORMALIZED_REPO_ROOT}/b.js`,
        fixtureName,
        false,
      );
      expect(result).toBe("From <root>/a.js to <root>/b.js\n");
    });
  });

  describe("ESLint output formatting", () => {
    it("aligns rule names in ESLint output", () => {
      const result = normalizeStdout(
        "  1:5  error  Unexpected var  plugin/no-var",
        fixtureName,
        true,
      );
      // Rule name should be aligned to column 60
      const lines = result.split("\n");
      expect(lines[0]).toContain("plugin/no-var");
      // The content should be followed by spaces then the rule name
      expect(lines[0]).toMatch(/error\s+Unexpected var\s+plugin\/no-var/);
    });

    it("does not align when not in ESLint mode", () => {
      const input = "  1:5  error  Unexpected var  plugin/no-var";
      const result = normalizeStdout(input, fixtureName, false);
      // Should keep original spacing (not realigned)
      expect(result).toBe(input + "\n");
    });
  });
});
