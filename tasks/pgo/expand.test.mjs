import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { files, recordKey } from "./expand.mjs";

await test("deduplication preserves different parser configurations and ignores provenance", () => {
  const record = { code: "(foo)", options: { lang: "js", sourceType: "module" } };
  assert.equal(recordKey(record), recordKey({ ...record, filename: "other.js" }));
  assert.notEqual(
    recordKey(record),
    recordKey({ ...record, parserOptions: { preserveParens: false } }),
  );
  assert.notEqual(
    recordKey(record),
    recordKey({ ...record, options: { ...record.options, lang: "ts" } }),
  );
});

await test("artifact enumeration is deterministic and does not follow symlinks outside the package", async (t) => {
  const directory = await mkdtemp(join(tmpdir(), "oxc-pgo-artifacts-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await mkdir(join(directory, "nested"));
  await writeFile(join(directory, "z.js"), "z");
  await writeFile(join(directory, "nested/a.js"), "a");
  if (process.platform !== "win32") await symlink("..", join(directory, "nested/outside"));
  assert.deepEqual(await Array.fromAsync(files(directory)), [
    join(directory, "nested/a.js"),
    join(directory, "z.js"),
  ]);
});
