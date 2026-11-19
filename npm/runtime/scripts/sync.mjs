import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync, mkdirSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const runtimeDir = fileURLToPath(new URL("../", import.meta.url));
const sourceDir = process.argv[2];
if (!sourceDir) {
  throw new Error("Usage: node scripts/sync.mjs <extracted-babel-runtime-directory>");
}
const upstreamPackage = JSON.parse(readFileSync(join(sourceDir, "package.json"), "utf8"));
if (upstreamPackage.name !== "@babel/runtime") {
  throw new Error("Expected an extracted @babel/runtime package");
}

const files = [];
function visit(dir, prefix = "") {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const relative = prefix + entry.name;
    if (entry.isDirectory()) {
      visit(join(dir, entry.name), `${relative}/`);
    } else if (entry.isFile()) {
      files.push(relative);
    } else {
      throw new Error(`Unexpected upstream entry: ${relative}`);
    }
  }
}
visit(join(sourceDir, "helpers"));
files.sort((a, b) => (a === b ? 0 : a < b ? -1 : 1));

const helpersDir = join(runtimeDir, "src/helpers");
const upstreamHelpers = files.filter((name) => !name.includes("/") && name.endsWith(".js"));
const customHelpers = ["decorate.js", "decorateMetadata.js", "decorateParam.js"];
const legacyHelpers = readdirSync(helpersDir)
  .filter(
    (name) =>
      name.endsWith(".js") && !upstreamHelpers.includes(name) && !customHelpers.includes(name),
  )
  .sort();
const hashes = Object.fromEntries(
  files.map((relative) => [
    relative,
    createHash("sha256")
      .update(readFileSync(join(sourceDir, "helpers", relative)))
      .digest("hex"),
  ]),
);
const previous = JSON.parse(readFileSync(join(runtimeDir, "upstream.json"), "utf8"));
const { patches } = previous;
for (const [relative, patch] of Object.entries(patches)) {
  if (hashes[relative] !== previous.files[relative]) {
    throw new Error(`Review the local patch for changed upstream helper ${relative}`);
  }
  const contents = readFileSync(join(helpersDir, relative));
  if (createHash("sha256").update(contents).digest("hex") !== patch.sha256) {
    throw new Error(`Unrecorded local changes to patched helper ${relative}`);
  }
}
for (const relative of files) {
  if (Object.hasOwn(patches, relative)) continue;
  const target = join(helpersDir, relative);
  mkdirSync(dirname(target), { recursive: true });
  copyFileSync(join(sourceDir, "helpers", relative), target);
}
copyFileSync(join(sourceDir, "LICENSE"), join(runtimeDir, "LICENSE"));

const packagePath = join(runtimeDir, "package.json");
const localPackage = JSON.parse(readFileSync(packagePath, "utf8"));
const mapTargets = (value, localValue) => {
  if (typeof value === "string") return value.replace(/^\.\/helpers\//, "./src/helpers/");
  if (Array.isArray(value)) {
    return value.map((target, index) =>
      mapTargets(target, Array.isArray(localValue) ? localValue[index] : localValue),
    );
  }
  // Older exports wrap their conditions in a fallback array. Keep Oxc's
  // condition precedence even when upstream removes that wrapper.
  if (Array.isArray(localValue)) localValue = localValue[0];
  const keys = Object.keys(value);
  const localKeys = localValue && typeof localValue === "object" ? Object.keys(localValue) : [];
  const orderedKeys = [
    ...localKeys.filter((key) => key !== "default" && Object.hasOwn(value, key)),
    ...keys.filter((key) => key !== "default" && !localKeys.includes(key)),
    ...keys.filter((key) => key === "default"),
  ];
  return Object.fromEntries(
    orderedKeys.map((key) => [key, mapTargets(value[key], localValue?.[key])]),
  );
};
const exports = mapTargets(upstreamPackage.exports, localPackage.exports);
for (const [key, value] of Object.entries(localPackage.exports)) {
  if (!(key in exports)) exports[key] = value;
}
for (const filename of upstreamHelpers) {
  const name = filename.slice(0, -3);
  exports[`./helpers/esm/${name}`] = `./src/helpers/esm/${filename}`;
}
localPackage.sideEffects = upstreamPackage.sideEffects;
localPackage.exports = Object.fromEntries(
  Object.entries(exports).sort(([a], [b]) => a.localeCompare(b, "en")),
);
writeFileSync(packagePath, `${JSON.stringify(localPackage, null, 2)}\n`);

writeFileSync(
  join(runtimeDir, "upstream.json"),
  `${JSON.stringify(
    {
      name: upstreamPackage.name,
      version: upstreamPackage.version,
      files: hashes,
      internalHelpers: upstreamHelpers.filter(
        (name) => !("./helpers/" + name.slice(0, -3) in upstreamPackage.exports),
      ),
      patches,
      customHelpers,
      legacyHelpers,
    },
    null,
    2,
  )}\n`,
);
// oxlint-disable-next-line no-console
console.log(
  `Copied @babel/runtime@${upstreamPackage.version}: ${upstreamHelpers.length} helper pairs; retained ${legacyHelpers.length} legacy and ${customHelpers.length} TypeScript helpers.`,
);
