import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pgoRustflags, run } from "./utils.mjs";

await test("pinned compiler generates valid PGO counters and consumes its own profile", async (t) => {
  const directory = await mkdtemp(join(tmpdir(), "oxc-pgo-compiler-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const rustc = await run("rustc", ["-vV"], { capture: true });
  const host = /^host: (.+)$/m.exec(rustc)[1];
  const sysroot = await run("rustc", ["--print", "sysroot"], { capture: true });
  const extension = process.platform === "win32" ? ".exe" : "";
  const llvm = join(sysroot, "lib/rustlib", host, "bin", `llvm-profdata${extension}`);
  const source = join(directory, "repro.rs");
  const binary = join(directory, `repro${extension}`);
  const raw = join(directory, "raw");
  const profile = join(directory, "merged.profdata");
  // Regression for rust-lang/rust#163616: select instrumentation used to add
  // the payload of an inactive enum variant to a branch counter.
  await writeFile(
    source,
    `
use std::hint::black_box;
enum Value {
    #[expect(dead_code)] Optional(Option<u64>),
    Words([u64; 2]),
}
#[inline(never)]
fn select(value: &Value) -> u64 {
    match value {
        Value::Optional(Some(_)) => 42,
        Value::Optional(None) => 7,
        Value::Words(words) => words[0],
    }
}
fn main() {
    let value = black_box(Value::Words([1 << 60, 0]));
    assert_eq!(select(&value), 1 << 60);
}
`,
  );
  await run("rustc", [
    "-O",
    "--target",
    host,
    ...pgoRustflags,
    `-Cprofile-generate=${raw}`,
    source,
    "-o",
    binary,
  ]);
  await run(binary, [], { env: { ...process.env, LLVM_PROFILE_FILE: join(raw, "%m-%p.profraw") } });
  await run(llvm, ["merge", "--failure-mode=any", "-o", profile, raw]);
  const summary = await run(llvm, ["show", profile], { capture: true });
  const maximum = BigInt(/Maximum internal block count:\s*(\d+)/.exec(summary)[1]);
  assert.ok(maximum < 1000n, summary);
  assert.match(summary, /Maximum function count:\s*[1-9]/);
  await run("rustc", [
    "-O",
    "--target",
    host,
    ...pgoRustflags,
    `-Cprofile-use=${profile}`,
    source,
    "-o",
    binary,
  ]);
  await run(binary, []);
});
