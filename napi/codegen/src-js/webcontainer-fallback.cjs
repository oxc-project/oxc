const fs = require("node:fs");
const childProcess = require("node:child_process");

const pkg = JSON.parse(fs.readFileSync(require.resolve("oxc-codegen/package.json"), "utf-8"));
const { version } = pkg;
const baseDir = `/tmp/oxc-codegen-${version}`;
const bindingEntry = `${baseDir}/node_modules/@oxc-codegen/binding-wasm32-wasi/codegen.wasi.cjs`;

if (!fs.existsSync(bindingEntry)) {
  fs.rmSync(baseDir, { recursive: true, force: true });
  fs.mkdirSync(baseDir, { recursive: true });
  const bindingPkg = `@oxc-codegen/binding-wasm32-wasi@${version}`;
  // oxlint-disable-next-line no-console
  console.log(`[oxc-codegen] Downloading ${bindingPkg} on WebContainer...`);
  childProcess.execFileSync("pnpm", ["i", bindingPkg], {
    cwd: baseDir,
    stdio: "inherit",
  });
}

module.exports = require(bindingEntry);
