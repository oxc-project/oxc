import { mkdir, readFile, writeFile } from "node:fs/promises";

const CACHE = new URL("../../../target/", import.meta.url);
const URLS = [
  "https://cdn.jsdelivr.net/npm/react@17.0.2/cjs/react.development.js",
  "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/binder.ts",
  "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@cd3bc3d431452b640f5dfcabbc22a8d8a388f393/kitchen-sink.tsx",
];

/** The usual real-world targets plus controlled comment densities. */
export async function commentFixtures() {
  await mkdir(CACHE, { recursive: true });
  const fixtures = await Promise.all(
    URLS.map(async (url) => {
      const filename = url.split("/").at(-1);
      const path = new URL(filename, CACHE);
      let source;
      try {
        source = await readFile(path, "utf8");
      } catch {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`Cannot load ${url}: ${response.status}`);
        source = await response.text();
        await writeFile(path, source);
      }
      return { filename, source };
    }),
  );
  const statement = "value = left + right;\n";
  for (const density of ["none", "sparse", "dense"]) {
    let source = "";
    for (let i = 0; i < 1024; i++) {
      if (density === "dense" || (density === "sparse" && i % 128 === 0)) {
        source += "/* statement */\n";
      }
      source += statement;
    }
    fixtures.push({ filename: `${density}.js`, source });
  }
  return fixtures;
}
