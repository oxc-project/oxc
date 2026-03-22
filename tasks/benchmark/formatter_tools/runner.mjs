import { cp, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, join, relative } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import prettier from "prettier";

const [tool, inputDir, outputDir] = process.argv.slice(2);

if (!tool || !inputDir || !outputDir) {
  console.error("Usage: node runner.mjs <biome|prettier> <input_dir> <output_dir>");
  process.exit(1);
}

const files = await collectFiles(inputDir);
if (files.length === 0) {
  console.error(`No benchmark files found in ${inputDir}`);
  process.exit(1);
}

switch (tool) {
  case "prettier":
    await runPrettier(files, inputDir, outputDir);
    break;
  case "biome":
    await runBiome(files, inputDir, outputDir);
    break;
  default:
    console.error(`Unsupported formatter: ${tool}`);
    process.exit(1);
}

async function runPrettier(files, rootDir, destinationDir) {
  for (const file of files) {
    const source = await readFile(file, "utf8");
    const relativePath = relative(rootDir, file);
    const outputPath = join(destinationDir, relativePath);
    await mkdir(dirname(outputPath), { recursive: true });
    const formatted = await prettier.format(source, { filepath: file });
    await writeFile(outputPath, formatted);
  }
}

async function runBiome(files, rootDir, destinationDir) {
  void files;
  await cp(rootDir, destinationDir, { recursive: true });
  const configPath = join(dirname(destinationDir), "biome-benchmark.json");
  await writeFile(configPath, JSON.stringify({ files: { ignoreUnknown: true } }));
  const biomeBin = join(
    dirname(fileURLToPath(import.meta.url)),
    "node_modules",
    ".bin",
    "biome",
  );
  await spawnChecked(
    biomeBin,
    ["format", "--write", "--config-path", configPath, "."],
    { cwd: destinationDir },
  );
}

async function collectFiles(rootDir) {
  const entries = await readdir(rootDir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectFiles(fullPath)));
    } else if (entry.isFile()) {
      files.push(fullPath);
    }
  }
  return files.sort();
}

function spawnChecked(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: "inherit", ...options });
    child.on("exit", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${command} exited with code ${code}`));
      }
    });
    child.on("error", reject);
  });
}
