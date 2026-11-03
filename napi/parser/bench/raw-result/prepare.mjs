import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import os from "node:os";

export const root = fileURLToPath(new URL("../../../../", import.meta.url));
export const output = path.join(root, "target/raw-result");
fs.mkdirSync(output, { recursive: true });
const base = process.env.OXC_BENCH_BASE ?? "62a8e7818e86abe19e86a7125a017f665ba4448c";
const original = execFileSync("git", ["show", `${base}:napi/parser/src-js/raw-transfer/eager.js`], {
  cwd: root,
  encoding: "utf8",
});
const start = original.indexOf("  // We cannot lazily deserialize in the getters");
const end = original.indexOf("\n// `ESTreeKind` discriminants", start);
assert.ok(start > 0 && end > start);
const wrapper = original.slice(start, end - 2);
const toLocal = (source) =>
  source
    .replaceAll(
      '"../generated/constants.js"',
      JSON.stringify(
        pathToFileURL(path.join(root, "napi/parser/src-js/generated/constants.js")).href,
      ),
    )
    .replaceAll(
      '"./common.js"',
      JSON.stringify(
        pathToFileURL(path.join(root, "napi/parser/src-js/raw-transfer/common.js")).href,
      ),
    )
    .replaceAll(
      "`../generated/deserialize/",
      "`" + path.join(root, "napi/parser/src-js/generated/deserialize/"),
    );
const fields = ["program", "module", "comments", "errors"];

const bodies = {
  baseline: wrapper,
  helper: "  return createResult(data, tokens);",
  define: "  return createResult(data, tokens);",
  bound: "  return createResult(data, tokens);",
  private: "  return Reflect.construct(RawResult, [data, tokens], Object);",
  plain: "  if (tokens !== null) data.tokens = tokens;\n  return data;",
};
const helpers = {
  helper: `function createResult(data, tokens) {\n${wrapper}\n}`,
  define: `function createResult(data, tokens) {\nreturn Object.defineProperties({}, { ${[...fields.slice(0, 3), "tokens", "errors"].map((key) => (key === "tokens" ? "...(tokens === null ? {} : { tokens: { get: function getTokens() { return tokens; }, enumerable: true, configurable: true } })" : `${key}: { get: function getValue() { return data.${key}; }, enumerable: true, configurable: true }`)).join(",\n")} });\n}`,
  bound: `const getters = { ${fields.map((key) => `get ${key}() { return this.${key}; }`).join(",")} };\nconst descriptors = Object.getOwnPropertyDescriptors(getters);\nfunction createResult(data, tokens) {\nconst result = {};\nfor (const key of ["program", "module", "comments", ...(tokens === null ? [] : ["tokens"]), "errors"]) {\nconst getter = key === "tokens" ? function getTokens() { return tokens; } : descriptors[key].get.bind(data);\nObject.defineProperty(result, key, { get: getter, enumerable: true, configurable: true });\n}\nreturn result;\n}`,
  private: `let descriptors, tokenDescriptors;\nclass RawResult {\n#data; #tokens;\nconstructor(data, tokens) { this.#data=data; this.#tokens=tokens; Object.defineProperties(this, tokens === null ? descriptors : tokenDescriptors); }\nstatic {\nconst all = Object.getOwnPropertyDescriptors({ ${fields
    .slice(0, 3)
    .map((key) => `get ${key}() { return this.#data.${key}; }`)
    .join(
      ",",
    )}, get tokens() { return this.#tokens; }, get errors() { return this.#data.errors; } });\ntokenDescriptors=all;\ndescriptors={program:all.program,module:all.module,comments:all.comments,errors:all.errors};\n}\n}`,
};
for (const [name, body] of Object.entries(bodies)) {
  const source =
    original.slice(0, start) + body + "\n}\n" + original.slice(end) + "\n" + (helpers[name] ?? "");
  fs.writeFileSync(path.join(output, `${name}.mjs`), toLocal(source));
}
const tracked = [
  "napi/parser/src-js/raw-transfer/eager.js",
  "napi/parser/src-js/raw-transfer/common.js",
  "napi/parser/src-js/generated/constants.js",
  ...fs
    .readdirSync(path.join(root, "napi/parser/src-js"))
    .filter((name) => name.endsWith(".node"))
    .map((name) => `napi/parser/src-js/${name}`),
  ...fs
    .readdirSync(path.join(root, "napi/parser/src-js/generated/deserialize"))
    .filter((name) => name.endsWith(".js"))
    .map((name) => `napi/parser/src-js/generated/deserialize/${name}`),
  ...fs
    .readdirSync(path.join(root, "packages/codegen/dist"))
    .filter((name) => name.endsWith(".js"))
    .map((name) => `packages/codegen/dist/${name}`),
];
const hashes = Object.fromEntries(
  tracked.map((filename) => [
    filename,
    createHash("sha256")
      .update(fs.readFileSync(path.join(root, filename)))
      .digest("hex"),
  ]),
);
fs.writeFileSync(
  path.join(output, "build.json"),
  JSON.stringify(
    {
      base,
      commit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
      node: process.version,
      v8: process.versions.v8,
      platform: process.platform,
      arch: process.arch,
      cpu: os.cpus()[0].model,
      baselineSourceSha256: createHash("sha256").update(original).digest("hex"),
      hashes,
    },
    null,
    2,
  ) + "\n",
);
