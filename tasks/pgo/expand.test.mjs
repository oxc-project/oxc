import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { files, recordKey, validateOutputDirectory } from "./expand.mjs";

await test("expanded output preserves input, preparation, and unrelated directories", async (t) => {
  const directory = await mkdtemp(join(tmpdir(), "oxc-pgo-output-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const original = join(directory, "original");
  const work = join(directory, "preparation");
  await mkdir(original);
  await mkdir(work);
  await writeFile(join(original, "source.txt"), "preserved");
  await Promise.all(
    [original, directory, join(original, "child"), work, join(work, "child")].map((output) =>
      assert.rejects(validateOutputDirectory(original, output, work), /must not overlap/),
    ),
  );
  const unrelated = join(directory, "unrelated");
  await mkdir(unrelated);
  await writeFile(join(unrelated, "source.txt"), "preserved");
  await assert.rejects(validateOutputDirectory(original, unrelated, work), /unrelated files/);
  assert.equal(await readFile(join(unrelated, "source.txt"), "utf8"), "preserved");
  const output = join(directory, "expanded");
  await assert.rejects(
    validateOutputDirectory(join(work, "input"), output, work),
    /must not overlap/,
  );
  await validateOutputDirectory(original, output, work);
  await mkdir(output);
  await writeFile(join(output, ".oxc-pgo-expanded"), "");
  await validateOutputDirectory(original, output, work);
  if (process.platform !== "win32") {
    const alias = join(directory, "alias");
    await symlink(original, alias);
    await assert.rejects(validateOutputDirectory(original, alias, work), /must not overlap/);
    await assert.rejects(
      validateOutputDirectory(original, join(alias, "child"), work),
      /must not overlap/,
    );
  }
  assert.equal(await readFile(join(original, "source.txt"), "utf8"), "preserved");
});

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
