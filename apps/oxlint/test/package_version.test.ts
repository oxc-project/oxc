import { afterEach, describe, expect, it } from "vitest";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { tmpdir } from "node:os";
import { resolveNearestPackageVersion } from "../src-js/utils/package_version.ts";

const tempDirs: string[] = [];

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "oxlint-package-version-"));
  tempDirs.push(dir);
  return dir;
}

async function writePackageJson(dir: string, content: Record<string, unknown>): Promise<void> {
  await writeFile(join(dir, "package.json"), JSON.stringify(content, null, 2));
}

function toModuleUrl(path: string): string {
  return pathToFileURL(path).href;
}

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })));
});

describe("package version resolution", () => {
  it("finds the package version from the raw-source src-js/plugins layout", async () => {
    const rootDir = await createTempDir();
    await mkdir(join(rootDir, "src-js", "plugins"), { recursive: true });
    await writePackageJson(rootDir, {
      name: "oxlint-app",
      version: "1.2.3",
    });

    const version = resolveNearestPackageVersion(
      toModuleUrl(join(rootDir, "src-js", "plugins", "context.ts")),
    );

    expect(version).toBe("1.2.3");
  });

  it("finds the package version from the built dist-pkg-plugins layout", async () => {
    const rootDir = await createTempDir();
    await mkdir(join(rootDir, "dist-pkg-plugins"), { recursive: true });
    await writePackageJson(rootDir, {
      name: "oxlint-app",
      version: "4.5.6",
    });

    const version = resolveNearestPackageVersion(
      toModuleUrl(join(rootDir, "dist-pkg-plugins", "context.js")),
    );

    expect(version).toBe("4.5.6");
  });

  it("skips unrelated package.json files while walking upward", async () => {
    const rootDir = await createTempDir();
    await mkdir(join(rootDir, "src-js", "plugins"), { recursive: true });
    await writePackageJson(rootDir, {
      name: "oxlint-app",
      version: "7.8.9",
    });
    await writePackageJson(join(rootDir, "src-js"), {
      name: "not-oxlint",
      version: "0.0.1",
    });

    const version = resolveNearestPackageVersion(
      toModuleUrl(join(rootDir, "src-js", "plugins", "context.ts")),
    );

    expect(version).toBe("7.8.9");
  });

  it("falls back when no matching package.json exists", async () => {
    const rootDir = await createTempDir();
    await mkdir(join(rootDir, "src-js", "plugins"), { recursive: true });

    const version = resolveNearestPackageVersion(
      toModuleUrl(join(rootDir, "src-js", "plugins", "context.ts")),
    );

    expect(version).toBe("0.0.0");
  });
});
