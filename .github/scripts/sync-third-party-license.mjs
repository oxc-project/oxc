#!/usr/bin/env node
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(scriptDir, "../..");
const sourcePath = join(rootDir, "THIRD-PARTY-LICENSE");
const separator = "-".repeat(80);

const packageSections = {
  oxlint: [
    "* TypeScript",
    "ESLint",
    "typescript-eslint",
    "eslint-plugin-import",
    "eslint-plugin-jest",
    "eslint-plugin-promise",
    "jsparagus",
    "acorn",
    "sindresorhus/globals",
    "index_vec",
    "Ajv",
    "@typescript-eslint/scope-manager",
  ],
  oxfmt: [
    "* TypeScript",
    "jsparagus",
    "Biome",
    "acorn",
    "prettier",
    "prettier-plugin-tailwindcss",
    "prettier-plugin-svelte",
  ],
};

const args = process.argv.slice(2);
const clean = args[0] === "--clean";
const packageArgs = clean ? args.slice(1) : args;
const packageDirs = packageArgs.length === 0 ? ["npm/oxlint", "npm/oxfmt"] : packageArgs;

for (const packageDirArg of packageDirs) {
  const packageDir = resolve(process.cwd(), packageDirArg);
  const packageJsonPath = join(packageDir, "package.json");
  if (!existsSync(packageJsonPath)) {
    throw new Error(`package.json not found in ${packageDir}`);
  }

  const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
  if (!packageJson.files?.includes("THIRD-PARTY-LICENSE")) continue;

  const targetPath = join(packageDir, "THIRD-PARTY-LICENSE");
  if (clean) {
    rmSync(targetPath, { force: true });
  } else {
    writeFileSync(targetPath, buildThirdPartyLicense(packageJson.name));
  }
}

function buildThirdPartyLicense(packageName) {
  const sections = packageSections[packageName];
  if (!sections) {
    throw new Error(`No THIRD-PARTY-LICENSE sections configured for ${packageName}`);
  }

  const sourceSections = parseSourceSections();
  const missingSections = sections.filter((sectionName) => !sourceSections.has(sectionName));
  if (missingSections.length > 0) {
    throw new Error(
      `Missing THIRD-PARTY-LICENSE sections for ${packageName}: ${missingSections.join(", ")}`,
    );
  }

  return `${sections.map((sectionName) => sourceSections.get(sectionName).trim()).join(`\n\n${separator}\n\n`)}\n`;
}

function parseSourceSections() {
  const source = readFileSync(sourcePath, "utf8");
  const sections = new Map();

  for (const section of source.split(`\n${separator}\n`)) {
    const heading = section
      .trimStart()
      .split(/\r?\n/)
      .find((line) => line.trim().length > 0);

    if (heading) sections.set(heading.trim(), section);
  }

  return sections;
}
