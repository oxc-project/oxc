const fs = require("node:fs");
const childProcess = require("node:child_process");

const pkg = JSON.parse(
  fs.readFileSync(require.resolve("oxc-transform-react/package.json"), "utf-8"),
);
const { version } = pkg;
const baseDir = `/tmp/oxc-transform-react-${version}`;
const bindingEntry = `${baseDir}/node_modules/@oxc-transform-react/binding-wasm32-wasi/transform-react.wasi.cjs`;

if (!fs.existsSync(bindingEntry)) {
  fs.rmSync(baseDir, { recursive: true, force: true });
  fs.mkdirSync(baseDir, { recursive: true });
  const bindingPkg = `@oxc-transform-react/binding-wasm32-wasi@${version}`;
  // oxlint-disable-next-line no-console
  console.log(`[oxc-transform-react] Downloading ${bindingPkg} on WebContainer...`);
  childProcess.execFileSync("pnpm", ["i", bindingPkg], {
    cwd: baseDir,
    stdio: "inherit",
  });
}

module.exports = require(bindingEntry);
