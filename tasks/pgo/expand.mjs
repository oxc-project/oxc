// oxlint-disable no-console, no-await-in-loop -- Keep preparation deterministic and bound memory use.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cp, mkdir, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { isMainThread, workerData } from "node:worker_threads";
import { CorpusWriter, readShard, sourceFiles, verifyCorpus } from "./corpus.mjs";
import { git, language, largeStackWorker, pnpm, root, sha256, sourceType } from "./utils.mjs";

const sourceExtension = /\.(?:[cm]?[jt]s|[jt]sx)$/i;
const svelteRevision = "df796708040f5200ec572b28ab7f88ecee5794dd";
const digest = (value) => createHash("sha256").update(value).digest("hex");
const outputMarker = ".oxc-pgo-expanded";

async function canonicalPath(path) {
  try {
    return await realpath(path);
  } catch (error) {
    if (error.code !== "ENOENT" || dirname(path) === path) throw error;
    return join(await canonicalPath(dirname(path)), basename(path));
  }
}

function contains(parent, child) {
  const path = relative(parent, child);
  return path === "" || (path !== ".." && !path.startsWith(`..${sep}`) && !isAbsolute(path));
}

export async function validateOutputDirectory(original, directory, work) {
  const [source, output, preparation] = await Promise.all(
    [original, directory, work].map((path) => canonicalPath(resolve(path))),
  );
  for (const [left, right] of [
    [source, output],
    [source, preparation],
    [output, preparation],
  ]) {
    assert.ok(
      !contains(left, right) && !contains(right, left),
      "Corpus input, output, and preparation directories must not overlap",
    );
  }
  let entries;
  try {
    entries = await readdir(directory);
  } catch (error) {
    if (error.code === "ENOENT") return;
    throw error;
  }
  if (!entries.length || entries.includes(outputMarker)) return;
  // Permit replacing a previously prepared corpus, but preserve unrelated directories.
  const previous = await readFile(join(directory, "manifest.json"), "utf8").catch((error) => {
    if (error.code === "ENOENT") return "null";
    throw error;
  });
  assert.ok(JSON.parse(previous)?.workloads, "Expanded output contains unrelated files");
}

export function recordKey(record) {
  return digest(JSON.stringify([record.code, record.options, record.parserOptions ?? {}]));
}

export async function* files(directory) {
  for (const entry of (await readdir(directory, { withFileTypes: true })).sort((a, b) =>
    a.name.localeCompare(b.name, "en"),
  )) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* files(path);
    else if (entry.isFile()) yield path;
  }
}

async function checkout(directory, repository, revision) {
  await rm(directory, { recursive: true, force: true });
  await mkdir(directory, { recursive: true });
  await git(directory, ["init", "--quiet"]);
  await git(directory, [
    "fetch",
    "--quiet",
    "--depth=1",
    `https://github.com/${repository}.git`,
    revision,
  ]);
  await git(directory, ["checkout", "--quiet", "--detach", "FETCH_HEAD"]);
  assert.equal(await git(directory, ["rev-parse", "HEAD"]), revision);
}

async function expand(original, directory) {
  const work = resolve(root, "target/pgo/expanded-workloads");
  await validateOutputDirectory(original, directory, work);
  const manifest = await verifyCorpus(original, {
    revision: await git(root, ["rev-parse", "HEAD"]),
  });
  assert.equal(manifest.workloads, undefined, "Expand the original corpus only");
  const tools = join(work, "tools");
  await mkdir(tools, { recursive: true });
  for (const file of ["package.json", "pnpm-lock.yaml"]) {
    await cp(join(root, "tasks/pgo/workloads", file), join(tools, file));
  }
  // Keep this tool installation independent of the repository workspace.
  await rm(join(tools, "node_modules"), { recursive: true, force: true });
  await pnpm([
    "install",
    "--dir",
    tools,
    "--ignore-workspace",
    "--frozen-lockfile",
    "--store-dir",
    join(work, "pnpm-store"),
    "--ignore-scripts",
  ]);
  const require = createRequire(join(tools, "package.json"));
  const esbuild = require("esbuild");
  const vue = require("@vue/compiler-sfc");
  const svelte = require("svelte/compiler");
  const helper = join(root, "target/pgo/prepare/transform.node");
  const { transformSync } = createRequire(import.meta.url)(helper);
  await rm(directory, { recursive: true, force: true });
  await cp(original, directory, { recursive: true });
  await writeFile(join(directory, outputMarker), "");
  await rm(join(directory, "manifest.json"));
  const writer = new CorpusWriter(directory);
  writer.shards = [...manifest.shards];
  writer.counts = { ...manifest.counts };
  writer.preparation = { ...manifest.preparation };
  const seen = new Set();
  for (const shard of manifest.shards) {
    for await (const record of readShard(original, shard)) seen.add(recordKey(record));
  }
  const workloads = {
    lockfileSha256: await sha256(join(tools, "pnpm-lock.yaml")),
    inputs: [],
    duplicates: 0,
    frameworkFailures: [],
    generated: {},
    npmPackages: [],
  };
  async function add(suite, filename, code, options = {}, provenance = {}) {
    if (!code.trim()) return;
    const record = {
      suite: `workload/${suite}`,
      filename: `workload/${suite}/${filename}`,
      code,
      options: { lang: language(filename), sourceType: sourceType(filename), ...options },
      expectedFailure: false,
    };
    // Keep default JSON parsing dominant and cover alternate AST settings separately.
    const optionSample = Number.parseInt(digest(code).slice(0, 2), 16);
    if (optionSample % 8 === 0) record.parserOptions = { preserveParens: false };
    if (optionSample % 16 === 0) {
      record.parserOptions.astType = ["ts", "tsx", "dts"].includes(record.options.lang)
        ? "js"
        : "ts";
    }
    const key = recordKey(record);
    if (seen.has(key)) {
      workloads.duplicates++;
      return;
    }
    seen.add(key);
    await writer.add(record, transformSync);
    workloads.generated[suite] = (workloads.generated[suite] ?? 0) + 1;
    workloads.inputs.push({
      filename: record.filename,
      sourceSha256: digest(code),
      sourceBytes: Buffer.byteLength(code),
      ...provenance,
    });
  }
  const [installation] = JSON.parse(
    await pnpm(["list", "--dir", tools, "--ignore-workspace", "--depth", "Infinity", "--json"], {
      capture: true,
    }),
  );
  const installedPackages = new Map();
  async function collectPackages(dependencies) {
    for (const [name, pkg] of Object.entries(dependencies ?? {})) {
      // pnpm list also reports lockfile entries for optional binaries omitted on
      // this host. Collect installed files only.
      const path = await realpath(pkg.path).catch((error) => {
        if (error.code === "ENOENT") return undefined;
        throw error;
      });
      if (!path) continue;
      if (installedPackages.has(path)) continue;
      installedPackages.set(path, { name, version: pkg.version });
      await collectPackages(pkg.dependencies);
      await collectPackages(pkg.optionalDependencies);
    }
  }
  await collectPackages(installation.dependencies);
  for (const [packageDirectory, pkg] of [...installedPackages].sort(([a], [b]) =>
    a.localeCompare(b, "en"),
  )) {
    const location = `${pkg.name}@${pkg.version}`;
    workloads.npmPackages.push({ name: pkg.name, version: pkg.version });
    for await (const path of files(packageDirectory)) {
      const name = relative(packageDirectory, path).replaceAll("\\", "/");
      if (name.startsWith("node_modules/") || !sourceExtension.test(name)) continue;
      const suite = /(?:@aws-sdk|@smithy|google-protobuf)/.test(pkg.name) ? "generated-sdk" : "npm";
      await add(
        suite,
        `${location}/${name}`,
        await readFile(path, "utf8"),
        {},
        { package: pkg.name, version: pkg.version },
      );
    }
  }
  const checkouts = {};
  for (const path of ["preact", "fastify-vite"]) {
    const entry = manifest.repositories.find((entry) => entry.path === path);
    assert.ok(entry, `Missing selected project ${path}`);
    const directory = join(work, path);
    await checkout(directory, entry.repository, entry.revision);
    checkouts[path] = directory;
    workloads.inputs.push({ repository: entry.repository, revision: entry.revision });
  }
  const svelteDirectory = join(work, "svelte-realworld");
  await checkout(svelteDirectory, "sveltejs/realworld", svelteRevision);
  workloads.inputs.push({ repository: "sveltejs/realworld", revision: svelteRevision });

  // Use actual components and application entries; compilation never executes them.
  for await (const { filename, code } of sourceFiles(
    checkouts["fastify-vite"],
    "HEAD",
    /\.vue$/i,
  )) {
    if (!filename.startsWith("starters/")) continue;
    const { descriptor, errors } = vue.parse(code, { filename });
    try {
      if (errors.length) throw new Error(errors.map(String).join("; "));
      for (const [kind, script] of [
        ["script", descriptor.script],
        ["setup", descriptor.scriptSetup],
      ]) {
        if (!script) continue;
        const extension = script.lang ?? "js";
        if (!["js", "jsx", "ts", "tsx"].includes(extension)) {
          throw new Error(`Unsupported script language ${extension}`);
        }
        await add("vue-script", `${filename}.${kind}.${extension}`, script.content);
      }
      if (descriptor.script || descriptor.scriptSetup) {
        const script = vue.compileScript(descriptor, { id: digest(code).slice(0, 12) });
        await add("vue-generated", `${filename}.script.${script.lang ?? "js"}`, script.content);
      }
      if (descriptor.template) {
        for (const ssr of [false, true]) {
          const result = vue.compileTemplate({
            source: descriptor.template.content,
            filename,
            id: digest(code).slice(0, 12),
            ssr,
            ssrCssVars: descriptor.cssVars,
            compilerOptions: { expressionPlugins: ["typescript"] },
          });
          if (result.errors.length) throw new Error(result.errors.map(String).join("; "));
          const lowered = await esbuild.transform(result.code, { loader: "ts", target: "esnext" });
          await add("vue-generated", `${filename}.${ssr ? "server" : "client"}.js`, lowered.code);
        }
      }
    } catch (error) {
      workloads.frameworkFailures.push({ filename, error: String(error) });
    }
  }
  for await (const { filename, code } of sourceFiles(svelteDirectory, "HEAD", /\.svelte$/i)) {
    if (!filename.startsWith("src/")) continue;
    for (const generate of ["client", "server"]) {
      try {
        const result = svelte.compile(code, { filename, generate, dev: false });
        await add("svelte-generated", `${filename}.${generate}.js`, result.js.code);
      } catch (error) {
        workloads.frameworkFailures.push({ filename, generate, error: String(error) });
      }
    }
  }
  const reactDirectory = join(checkouts["fastify-vite"], "e2e/react-vanilla-spa-ts/src/client");
  const bundles = [
    {
      name: "react-client",
      entryPoints: [join(reactDirectory, "mount.tsx")],
      platform: "browser",
      jsx: "automatic",
    },
    {
      name: "react-server",
      stdin: {
        contents:
          'import { App } from "./App.tsx"; import { renderToString } from "react-dom/server"; export function render() { return renderToString(<App />); }',
        resolveDir: reactDirectory,
        loader: "tsx",
      },
      platform: "node",
      jsx: "automatic",
    },
    {
      name: "preact-todo",
      entryPoints: [join(checkouts.preact, "demo/todo.jsx")],
      platform: "browser",
      jsxFactory: "createElement",
    },
    {
      name: "preact-pythagoras",
      entryPoints: [join(checkouts.preact, "demo/pythagoras/index.jsx")],
      platform: "browser",
      jsxFactory: "createElement",
    },
  ];
  for (const { name, ...configuration } of bundles) {
    for (const development of [true, false]) {
      for (const minify of [false, true]) {
        const result = await esbuild.build({
          ...configuration,
          bundle: true,
          write: false,
          format: "esm",
          target: "es2020",
          nodePaths: [join(tools, "node_modules")],
          minify,
          sourcemap: "external",
          outfile: "output.js",
          define: {
            "process.env.NODE_ENV": JSON.stringify(development ? "development" : "production"),
          },
          logLevel: "silent",
        });
        const output = result.outputFiles.find((file) => file.path.endsWith("output.js"));
        assert.ok(output);
        await add(
          "bundles",
          `${name}.${development ? "development" : "production"}.${minify ? "min" : "pretty"}.mjs`,
          output.text,
          { sourceType: "module" },
        );
      }
    }
  }
  for (const name of [
    "npm",
    "generated-sdk",
    "vue-script",
    "vue-generated",
    "svelte-generated",
    "bundles",
  ]) {
    assert.ok(workloads.generated[name] > 0, `No expanded inputs for ${name}`);
  }
  assert.equal(
    workloads.frameworkFailures.length,
    0,
    `Framework compilation failed: ${JSON.stringify(workloads.frameworkFailures)}`,
  );
  await writer.closeShard();
  await writeFile(
    join(directory, "manifest.json"),
    JSON.stringify(
      {
        ...manifest,
        counts: writer.counts,
        preparation: writer.preparation,
        shards: writer.shards,
        originalManifestSha256: await sha256(join(original, "manifest.json")),
        workloads,
      },
      null,
      2,
    ) + "\n",
  );
  await verifyCorpus(directory);
  console.log(
    JSON.stringify({
      original: Object.values(manifest.counts).reduce((a, b) => a + b, 0),
      added: workloads.generated,
      duplicates: workloads.duplicates,
      frameworkFailures: workloads.frameworkFailures,
    }),
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (isMainThread) {
    const { values } = parseArgs({
      options: {
        original: { type: "string", default: "target/pgo/corpus" },
        output: { type: "string", default: "target/pgo/corpus-expanded" },
      },
    });
    await largeStackWorker(new URL(import.meta.url), {
      workerData: { original: resolve(values.original), directory: resolve(values.output) },
    });
  } else await expand(workerData.original, workerData.directory);
}
