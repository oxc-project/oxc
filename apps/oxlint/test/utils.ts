import fs from "node:fs/promises";
import { readdirSync, readFileSync } from "node:fs";
import { join as pathJoin, sep as pathSep } from "node:path";

import { execa } from "execa";
import { expect } from "vitest";

// Replace backslashes with forward slashes on Windows. Do nothing on Mac/Linux.
const normalizeSlashes =
  pathSep === "\\" ? (path: string) => path.replaceAll("\\", "/") : (path: string) => path;

export const PACKAGE_ROOT_PATH = pathJoin(import.meta.dirname, ".."); // `/path/to/oxc/apps/oxlint`
const FIXTURES_DIR_PATH = pathJoin(import.meta.dirname, "fixtures"); // `/path/to/oxc/apps/oxlint/test/fixtures`

const REPO_ROOT_PATH = pathJoin(PACKAGE_ROOT_PATH, "../.."); // `/path/to/oxc`
const FIXTURES_SUBPATH = normalizeSlashes(FIXTURES_DIR_PATH.slice(REPO_ROOT_PATH.length)); // `/apps/oxlint/test/fixtures`

const FIXTURES_URL = new URL("./fixtures/", import.meta.url).href; // `file:///path/to/oxc/apps/oxlint/test/fixtures/`

// Details of a test fixture.
export interface Fixture {
  name: string;
  dirPath: string;
  options: {
    // Run Oxlint. Default: `true`.
    oxlint: boolean;
    // Run ESLint. Default: `false`.
    eslint: boolean;
    // Run Oxlint with fixes. Default: `false`.
    fix: boolean;
  };
}

const DEFAULT_OPTIONS: Fixture["options"] = { oxlint: true, eslint: false, fix: false };

/**
 * Get all fixtures in `test/fixtures`, and their options.
 * @returns Array of fixtures
 */
export function getFixtures(): Fixture[] {
  const fixtures: Fixture[] = [];

  const fileObjs = readdirSync(FIXTURES_DIR_PATH, { withFileTypes: true });
  for (const fileObj of fileObjs) {
    if (!fileObj.isDirectory()) continue;

    const { name } = fileObj;

    // Read `options.json` file
    const dirPath = pathJoin(FIXTURES_DIR_PATH, name);

    let options: Fixture["options"];
    try {
      options = JSON.parse(readFileSync(pathJoin(dirPath, "options.json"), "utf8"));
    } catch (err) {
      if (err?.code !== "ENOENT") throw err;
      options = DEFAULT_OPTIONS;
    }

    if (typeof options !== "object" || options === null)
      throw new TypeError("`options.json` must be an object");
    options = { ...DEFAULT_OPTIONS, ...options };
    if (
      typeof options.oxlint !== "boolean" ||
      typeof options.eslint !== "boolean" ||
      typeof options.fix !== "boolean"
    ) {
      throw new TypeError(
        "`oxlint`, `eslint`, and `fix` properties in `options.json` must be booleans",
      );
    }

    fixtures.push({ name, dirPath, options });
  }

  return fixtures;
}

// Options to pass to `testFixtureWithCommand`.
interface TestFixtureOptions {
  // Command
  command: string;
  // Arguments to execute command with
  args: string[];
  // Fixture details
  fixture: Fixture;
  // Name of the snapshot file
  snapshotName: string;
  // `true` if the command is ESLint
  isESLint: boolean;
}

/**
 * Run a test fixture.
 * @param options - Options for running the test
 */
export async function testFixtureWithCommand(options: TestFixtureOptions): Promise<void> {
  const { name: fixtureName, dirPath } = options.fixture,
    pathPrefixLen = dirPath.length + 1;

  // Read all the files in fixture's directory
  const fileObjs = await fs.readdir(dirPath, { withFileTypes: true, recursive: true });

  const files: { filename: string; code: string }[] = [];
  await Promise.all(
    fileObjs.map(async (fileObj) => {
      if (fileObj.isFile()) {
        const path = pathJoin(fileObj.parentPath, fileObj.name);
        files.push({
          filename: normalizeSlashes(path.slice(pathPrefixLen)),
          code: await fs.readFile(path, "utf8"),
        });
      }
    }),
  );

  // Run command
  let { stdout, stderr, exitCode } = await execa(options.command, options.args, {
    cwd: dirPath,
    reject: false,
  });

  // Build snapshot `.snap.md` file
  const snapshotPath = pathJoin(dirPath, `${options.snapshotName}.snap.md`);

  stdout = normalizeStdout(stdout, fixtureName, options.isESLint);
  stderr = normalizeStdout(stderr, fixtureName, false);
  let snapshot =
    `# Exit code\n${exitCode}\n\n` +
    `# stdout\n\`\`\`\n${stdout}\`\`\`\n\n` +
    `# stderr\n\`\`\`\n${stderr}\`\`\`\n`;

  // Check for any changes to files in `files` and add them to the snapshot.
  // Revert any changes to the files (useful for `--fix` tests).
  const changes: { filename: string; code: string }[] = [];
  await Promise.all(
    files.map(async ({ filename, code: codeBefore }) => {
      const path = pathJoin(dirPath, filename);
      const codeAfter = await fs.readFile(path, "utf8");
      if (codeAfter !== codeBefore) {
        await fs.writeFile(path, codeBefore);
        changes.push({ filename, code: codeAfter });
      }
    }),
  );

  if (changes.length > 0) {
    changes.sort((a, b) => (a.filename > b.filename ? 1 : -1));
    for (const { filename, code } of changes) {
      snapshot += `\n# File altered: ${filename}\n\`\`\`\n${code}\n\`\`\`\n`;
    }
  }

  // Assert snapshot is as expected
  await expect(snapshot).toMatchFileSnapshot(snapshotPath);
}

// Regexp to match paths in output.
// Matches `/path/to/oxc`, `/path/to/oxc/`, `/path/to/oxc/whatever`,
// when preceded by whitespace, `(`, or a quote, and followed by whitespace, `)`, or a quote.
const PATH_REGEXP = new RegExp(
  // @ts-expect-error `RegExp.escape` is new in NodeJS v24
  `(?<=^|[\\s\\('"\`])${RegExp.escape(REPO_ROOT_PATH)}(${RegExp.escape(pathSep)}[^\\s\\)'"\`]*)?(?=$|[\\s\\)'"\`])`,
  "g",
);

// Regexp to match lines of form `whatever      plugin/rule` in ESLint output.
const ESLINT_REGEXP = /^(.*?)\s+([A-Za-z0-9_-]+\/[A-Za-z0-9_-]+)$/;
// Column to align rule names to in ESLint output.
const ESLINT_RULE_NAME_COLUMN = 60;
// Minimum number of spaces between line content and rule name.
const ESLINT_SPACES_MIN = 4;

/**
 * Normalize output, so it's the same on every machine.
 *
 * - Remove timing + thread count info.
 * - Replace start of file paths with `<root>`.
 * - Remove irrelevant lines from stack traces.
 * - Normalize line breaks.
 *
 * @param stdout - Output from process
 * @param fixtureName - Name of the fixture
 * @param isESLint - `true` if the output is from ESLint
 * @returns Normalized output
 */
function normalizeStdout(stdout: string, fixtureName: string, isESLint: boolean): string {
  // Normalize line breaks, and trim line breaks from start and end
  stdout = stdout.replace(/\r\n?/g, "\n").replace(/^\n+/, "").replace(/\n+$/, "");
  if (stdout === "") return "";

  let lines = stdout.split("\n");

  // Remove timing and thread count info which can vary between runs
  lines[lines.length - 1] = lines[lines.length - 1].replace(
    /^Finished in \d+(?:\.\d+)?(?:s|ms|us|ns) on (\d+) file(s?) using \d+ threads.$/,
    "Finished in Xms on $1 file$2 using X threads.",
  );

  // Remove lines from stack traces which are outside `fixtures` directory.
  // Shorten paths in output with `<root>`, `<fixtures>`, or `<fixture>`.
  lines = lines.flatMap((line) => {
    // Handle stack trace lines.
    // e.g. ` | at file:///path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js:1:1`
    // e.g. ` | at whatever (file:///path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js:1:1)`
    const match = line.match(/^(\s*\|\s+at (?:.+?\()?)(.+)$/);
    if (match) {
      let [, preamble, at] = match;
      if (!at.startsWith(FIXTURES_URL)) return [];
      at = convertFixturesSubPath(at.slice(FIXTURES_URL.length - 1), fixtureName);
      return [`${preamble}${at}`];
    }

    // Handle paths anywhere else in the line
    line = line.replaceAll(PATH_REGEXP, (_, subPath) => {
      if (subPath === undefined) return "<root>";
      return convertSubPath(normalizeSlashes(subPath), fixtureName);
    });

    // Align rule names in ESLint output.
    // This avoids:
    // 1. Churn in snapshots when 1 line of output changes, because that changes how ESLint aligns rule names.
    // 2. Mismatch between local development and CI, because alignment differs for some reason (terminal width?).
    if (isESLint) {
      const match = line.match(ESLINT_REGEXP);
      if (match) {
        const [, content, ruleName] = match;
        const spaces = Math.max(ESLINT_RULE_NAME_COLUMN - content.length, ESLINT_SPACES_MIN);
        line = `${content}${" ".repeat(spaces)}${ruleName}`;
      }
    }

    return [line];
  });

  if (lines.length === 0) return "";
  return lines.join("\n") + "\n";
}

/**
 * Convert a sub path to a shorter form.
 *
 * `subPath` must be a path relative to the root of the repository.
 * It should start with `/`, and have Windows backslashes replaced with forward slashes,
 * before being passed to this function.
 *
 * Examples:
 * - `/apps/oxlint/test/fixtures/foo/bar.js` => `<fixture>/bar.js`
 * - `/apps/oxlint/test/fixtures/foo` => `<fixtures>/foo`
 * - `/apps/oxlint/something/else` => `<root>/apps/oxlint/something/else`
 */
function convertSubPath(subPath: string, fixtureName: string): string {
  if (subPath.startsWith(FIXTURES_SUBPATH)) {
    const relPath = subPath.slice(FIXTURES_SUBPATH.length);
    if (relPath === "") return "<fixtures>";
    if (relPath.startsWith("/")) return convertFixturesSubPath(relPath, fixtureName);
  }

  return `<root>${subPath}`;
}

/**
 * Convert a fixtures sub path to a shorter form.
 *
 * `subPath` must be a path relative to the fixtures dir.
 * It should start with `/`, and have Windows backslashes replaced with forward slashes,
 * before being passed to this function.
 *
 * Examples:
 * - `/foo/bar.js` => `<fixture>/bar.js`
 * - `/foo` => `<fixtures>/foo`
 */
function convertFixturesSubPath(subPath: string, fixtureName: string): string {
  subPath = subPath.slice(1);
  if (subPath.startsWith(fixtureName)) {
    const relPath = subPath.slice(fixtureName.length);
    if (relPath === "") return "<fixture>";
    if (relPath.startsWith("/")) return `<fixture>${relPath}`;
  }
  return `<fixtures>/${subPath}`;
}
