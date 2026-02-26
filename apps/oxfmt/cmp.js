// oxlint-disable

import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import * as prettier from "prettier";
import { format as oxfmtFormat } from "./dist/index.js";

const FIXTURES_DIR = join(
  import.meta.dirname,
  "../../tasks/prettier_conformance/prettier/tests/format/js/multiparser-graphql",
);

const EXCLUDE = new Set([
  "format.test.js",
  "comment-tag.js", // /* GraphQL */ comment tag (not yet supported)
  "expressions.js", // graphql() function call pattern (not yet supported)
  "graphql.js", // graphql() function call pattern (not yet supported)
]);

const files = (await readdir(FIXTURES_DIR))
  .filter((f) => f.endsWith(".js") && !EXCLUDE.has(f))
  .sort();

let matchCount = 0;
let mismatchCount = 0;
let errorCount = 0;

for (const file of files) {
  const filePath = join(FIXTURES_DIR, file);
  const source = await readFile(filePath, "utf8");

  const prettierOutput = await prettier.format(source, {
    parser: "babel",
    printWidth: 80,
  });

  let oxfmtOutput;
  try {
    const oxfmtResult = await oxfmtFormat(file, source, { printWidth: 80 });
    oxfmtOutput = oxfmtResult.code;
  } catch (e) {
    console.log(`✗ ${file} (ERROR: ${e.message})`);
    errorCount++;
    continue;
  }

  if (prettierOutput === oxfmtOutput) {
    console.log(`✓ ${file}`);
    matchCount++;
  } else {
    console.log(`✗ ${file}`);
    mismatchCount++;
    printUnifiedDiff(prettierOutput, oxfmtOutput);
  }
}

console.log(`\n--- Summary ---`);
console.log(
  `Match: ${matchCount}, Mismatch: ${mismatchCount}, Error: ${errorCount}, Total: ${files.length}`,
);

function printUnifiedDiff(expected, actual) {
  const expectedLines = expected.split("\n");
  const actualLines = actual.split("\n");
  console.log("  --- prettier");
  console.log("  +++ oxfmt");
  const maxLen = Math.max(expectedLines.length, actualLines.length);
  for (let i = 0; i < maxLen; i++) {
    const e = expectedLines[i];
    const a = actualLines[i];
    if (e === a) {
      console.log(`   ${e ?? ""}`);
    } else {
      if (e !== undefined) console.log(`  -${e}`);
      if (a !== undefined) console.log(`  +${a}`);
    }
  }
  console.log();
}
