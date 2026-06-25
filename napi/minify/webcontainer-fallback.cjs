const fs = require("node:fs");
const childProcess = require("node:child_process");

const pkg = JSON.parse(fs.readFileSync(require.resolve("oxc-minify/package.json"), "utf-8"));
const { version } = pkg;
const baseDir = `/tmp/oxc-minify-${version}`;
const bindingEntry = `${baseDir}/node_modules/@oxc-minify/binding-wasm32-wasi/minify.wasi.cjs`;

if (!fs.existsSync(bindingEntry)) {
  fs.rmSync(baseDir, { recursive: true, force: true });
  fs.mkdirSync(baseDir, { recursive: true });
  const bindingPkg = `@oxc-minify/binding-wasm32-wasi@${version}`;
  // oxlint-disable-next-line no-console
  console.log(`[oxc-minify] Downloading ${bindingPkg} on WebContainer...`);
  childProcess.execFileSync("pnpm", ["i", bindingPkg], {
    cwd: baseDir,
    stdio: "inherit",
  });
}

module.exports = require(bindingEntry);
