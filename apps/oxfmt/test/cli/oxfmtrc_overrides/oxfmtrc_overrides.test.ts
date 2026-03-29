import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("oxfmtrc overrides", () => {
  // .oxfmtrc.json:
  //   tabWidth: 2
  //   overrides: [{ files: ["*_test.js", "*_test.ts", ".*rc.js"], excludeFiles: ["*.min.js", "vendor/**"], options: { tabWidth: 4 } }]
  //
  // Expected:
  // - app.js: tabWidth=2 (base)
  // - app_test.js, app_test.ts: tabWidth=4 (multiple patterns in files array)
  // - app_test.min.js: tabWidth=2 (excluded by excludeFiles)
  // - vendor/lib.js: tabWidth=2 (excluded by excludeFiles with directory pattern)
  // - .testrc.js: tabWidth=4 (dotfile pattern match)
  it("basic file pattern override with excludeFiles", async () => {
    const cwd = join(fixturesDir, "basic");
    const snapshot = await runAndSnapshot(cwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig:
  //   [*] indent_size=8
  // .oxfmtrc.json:
  //   overrides: [{ files: ["*.js"], options: { tabWidth: 4 } }]
  //
  // Expected:
  // - src/nested/app.js: formatted with tabWidth=4 (oxfmtrc override takes precedence over editorconfig)
  //
  // NOTE: "*.js" pattern matches nested files because it's internally converted to "**/*.js"
  it("oxfmtrc overrides take precedence over editorconfig", async () => {
    const cwd = join(fixturesDir, "priority_over_editorconfig");
    const snapshot = await runAndSnapshot(cwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .editorconfig:
  //   [*] indent_style=tab
  // .oxfmtrc.json:
  //   overrides: [{ files: ["*.js"], options: { semi: false } }]
  //
  // Expected:
  // - test.js: useTabs=true (from editorconfig [*]), semi=false (from oxfmtrc overrides)
  //
  // This test verifies editorconfig [*] section is applied even when oxfmtrc overrides trigger slow path
  it("editorconfig root section applies with oxfmtrc overrides", async () => {
    const cwd = join(fixturesDir, "editorconfig_root_only");
    const snapshot = await runAndSnapshot(cwd, [["--check", "test.js"]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json:
  //   tabWidth: 2
  //   overrides: [{ files: ["src/*.js"], options: { tabWidth: 4 } }]
  //
  // Expected:
  // - src/app.js: formatted with tabWidth=4 (pattern with "/" is NOT prefixed with "**/"")
  // - lib/app.js: formatted with tabWidth=2 (pattern "src/*.js" does NOT match "lib/app.js")
  //
  // Note: Patterns containing "/" are used as-is, NOT prefixed with "**/"
  it("pattern with slash - matches only specified path", async () => {
    const cwd = join(fixturesDir, "path_with_slash");
    const snapshot = await runAndSnapshot(cwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json:
  //   tabWidth: 2, semi: true
  //   overrides: [
  //     { files: ["src/**/*.js"], options: { tabWidth: 4 } },
  //     { files: ["src/**/*_test.js"], options: { semi: false } },
  //     { files: ["src/deep/**/*.js"], options: { tabWidth: 6 } }
  //   ]
  //
  // Expected:
  // - root.js: formatted with tabWidth=2, semi=true (base config)
  // - src/app.js: formatted with tabWidth=4, semi=true (first override)
  // - src/app_test.js: formatted with tabWidth=4, semi=false (first and second overrides applied)
  // - src/deep/nested/app.js: formatted with tabWidth=6, semi=true (first and third overrides, later wins)
  it("multiple overrides - later overrides take precedence", async () => {
    const cwd = join(fixturesDir, "multiple_overrides");
    const snapshot = await runAndSnapshot(cwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json:
  //   experimentalSortPackageJson: false
  //   vueIndentScriptAndStyle: false
  //   overrides: [
  //     { files: ["sorted/package.json"], options: { experimentalSortPackageJson: true } },
  //     { files: ["indented.vue"], options: { vueIndentScriptAndStyle: true } }
  //   ]
  //
  // Expected:
  // - sorted/package.json: keys sorted (name, version, description)
  // - unsorted/package.json: keys NOT sorted (original order preserved)
  // - indented.vue: script/style content indented
  // - not-indented.vue: script/style content NOT indented
  //
  // This test verifies overrides work for external formatter path (Prettier)
  it("Prettier options override - only enabled for specific files", async () => {
    const cwd = join(fixturesDir, "prettier_overrides");
    const snapshot = await runAndSnapshot(cwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });

  // .oxfmtrc.json:
  //   experimentalTailwindcss: {}
  //   overrides: [
  //     { files: ["use-clsx.tsx"], options: { experimentalTailwindcss: { functions: ["clsx"] } } },
  //     { files: ["use-cn.tsx"], options: { experimentalTailwindcss: { functions: ["cn"] } } }
  //   ]
  //
  // Expected:
  // - use-clsx.tsx: only clsx() classes sorted
  // - use-cn.tsx: only cn() classes sorted
  //
  // This test verifies different Tailwind options per file work correctly (cache is not shared)
  it("Tailwind CSS override - different options per file (cache test)", async () => {
    const cwd = join(fixturesDir, "tailwindcss_options");
    const snapshot = await runAndSnapshot(cwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });
});
