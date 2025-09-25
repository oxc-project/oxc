import fs from 'node:fs';
import { dirname, join as pathJoin } from 'node:path';

import { describe, expect, it } from 'vitest';

import { execa } from 'execa';

const PACKAGE_ROOT_PATH = dirname(import.meta.dirname);
const ROOT_PATH = pathJoin(PACKAGE_ROOT_PATH, '../../');
const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, 'dist/cli.js');
const ROOT_URL = new URL('../../../', import.meta.url).href;
const FIXTURES_URL = new URL('./fixtures/', import.meta.url).href;

async function runOxlintWithoutPlugins(cwd: string, args: string[] = []) {
  return await execa('node', [CLI_PATH, ...args], {
    cwd: pathJoin(PACKAGE_ROOT_PATH, cwd),
    reject: false,
  });
}

async function runOxlint(cwd: string, args: string[] = []) {
  return await runOxlintWithoutPlugins(cwd, ['--js-plugins', ...args]);
}

async function runOxlintWithFixes(cwd: string, args: string[] = []) {
  return await runOxlintWithoutPlugins(cwd, ['--js-plugins', '--fix', ...args]);
}

function normalizeOutput(output: string): string {
  let lines = output.split('\n');

  // Remove timing and thread count info which can vary between runs
  lines[lines.length - 1] = lines[lines.length - 1].replace(
    /^Finished in \d+(?:\.\d+)?(?:s|ms|us|ns) on (\d+) file(s?) using \d+ threads.$/,
    'Finished in Xms on $1 file$2 using X threads.',
  );

  // Remove lines from stack traces which are outside `fixtures` directory.
  // Replace path to repo root in stack traces with `<root>`.
  lines = lines.flatMap((line) => {
    // e.g. ` | at file:///path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js:1:1`
    // e.g. ` | at whatever (file:///path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js:1:1)`
    let match = line.match(/^(\s*\|\s+at (?:.+?\()?)(.+)$/);
    if (match) {
      const [, premable, at] = match;
      return at.startsWith(FIXTURES_URL) ? [`${premable}<root>/${at.slice(ROOT_URL.length)}`] : [];
    } else {
      // e.g. ` | File path: /path/to/oxc/apps/oxlint/test/fixtures/foo/bar.js`
      match = line.match(/^(\s*\|\s+File path: )(.+)$/);
      if (match) {
        const [, premable, path] = match;
        if (path.startsWith(ROOT_PATH)) return [`${premable}<root>/${path.slice(ROOT_PATH.length)}`];
      }
    }
    return [line];
  });

  return lines.join('\n');
}

describe('oxlint CLI', () => {
  it('should lint a directory without errors without JS plugins enabled', async () => {
    const { stdout, exitCode } = await runOxlintWithoutPlugins('test/fixtures/built_in_no_errors');
    expect(exitCode).toBe(0);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should lint a directory with errors without JS plugins enabled', async () => {
    const { stdout, exitCode } = await runOxlintWithoutPlugins('test/fixtures/built_in_errors');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should lint a directory without errors with JS plugins enabled', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/built_in_no_errors');
    expect(exitCode).toBe(0);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should lint a directory with errors with JS plugins enabled', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/built_in_errors');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should load a custom plugin', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/basic_custom_plugin');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should load a custom plugin with various import styles', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/load_paths');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should load a custom plugin with multiple files', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/basic_custom_plugin_many_files');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should load a custom plugin when configured in overrides', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_via_overrides');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should report an error if a custom plugin in config but JS plugins are not enabled', async () => {
    const { stdout, exitCode } = await runOxlintWithoutPlugins('test/fixtures/basic_custom_plugin');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should report an error if a custom plugin is missing', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/missing_custom_plugin');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should report an error if a custom plugin throws an error during import', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_import_error');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should report an error if a rule is not found within a custom plugin', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_missing_rule');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should report an error if a a rule is not found within a custom plugin (via overrides)', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_via_overrides_missing_rule');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  describe('should report an error if a custom plugin throws an error during linting', () => {
    it('in `create` method', async () => {
      const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_lint_create_error');
      expect(exitCode).toBe(1);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
    });

    it('in `createOnce` method', async () => {
      const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_lint_createOnce_error');
      expect(exitCode).toBe(1);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
    });

    it('in visit function', async () => {
      const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_lint_visit_error');
      expect(exitCode).toBe(1);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
    });

    it('in `before` hook', async () => {
      const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_lint_before_hook_error');
      expect(exitCode).toBe(1);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
    });

    it('in `after` hook', async () => {
      const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_lint_after_hook_error');
      expect(exitCode).toBe(1);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
    });

    it('in `fix` function', async () => {
      const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_lint_fix_error');
      expect(exitCode).toBe(1);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
    });
  });

  it('should report the correct severity when using a custom plugin', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/basic_custom_plugin_warn_severity');
    expect(exitCode).toBe(0);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should work with multiple rules', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/basic_custom_plugin_multiple_rules');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should receive ESTree-compatible AST', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/estree');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should receive data via `context`', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/context_properties');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should support `createOnce`', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/createOnce');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should support `defineRule`', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/defineRule');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should support `definePlugin`', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/definePlugin');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should support `definePlugin` and `defineRule` together', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/definePlugin_and_defineRule');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should have UTF-16 spans in AST', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/utf16_offsets');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should respect disable directives for custom plugin rules', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/custom_plugin_disable_directives');
    expect(exitCode).toBe(1);
    expect(normalizeOutput(stdout)).toMatchSnapshot();
  });

  it('should not apply fixes when `--fix` is disabled', async () => {
    const fixtureFilePath = pathJoin(PACKAGE_ROOT_PATH, 'test/fixtures/fixes/files/index.js');
    const codeBefore = fs.readFileSync(fixtureFilePath, 'utf8');

    let error = true;
    try {
      const { stdout, exitCode } = await runOxlint('test/fixtures/fixes');
      expect(exitCode).toBe(1);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
      error = false;
    } finally {
      const codeAfter = fs.readFileSync(fixtureFilePath, 'utf8');
      if (codeAfter !== codeBefore) {
        fs.writeFileSync(fixtureFilePath, codeBefore);
        // oxlint-disable-next-line no-unsafe-finally
        if (!error) throw new Error('Test modified fixture file');
      }
    }
  });

  it('should apply fixes when `--fix` is enabled', async () => {
    const fixtureFilePath = pathJoin(PACKAGE_ROOT_PATH, 'test/fixtures/fixes/files/index.js');
    const codeBefore = fs.readFileSync(fixtureFilePath, 'utf8');

    let error = true;
    try {
      const { stdout, exitCode } = await runOxlintWithFixes('test/fixtures/fixes');
      expect(exitCode).toBe(0);
      expect(normalizeOutput(stdout)).toMatchSnapshot();
      error = false;
    } finally {
      const codeAfter = fs.readFileSync(fixtureFilePath, 'utf8');
      fs.writeFileSync(fixtureFilePath, codeBefore);
      if (!error) expect(codeAfter).toMatchSnapshot();
    }
  });
});
