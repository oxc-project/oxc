import { dirname, join as pathJoin } from 'node:path';

import { describe, expect, it } from 'vitest';

import { execa } from 'execa';

const PACKAGE_ROOT_PATH = dirname(import.meta.dirname);
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
    // e.g. ` | at file:///path/to/oxc/apps/oxlint/test/fixtures/foor/bar.js:1:1`
    // e.g. ` | at whatever (file:///path/to/oxc/apps/oxlint/test/fixtures/foor/bar.js:1:1)`
    const match = line.match(/^(\s*\|\s+at (?:.+?\()?)(.+)$/);
    if (match) {
      const [, premable, at] = match;
      return at.startsWith(FIXTURES_URL) ? [`${premable}<root>/${at.slice(ROOT_URL.length)}`] : [];
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

  it('should receive data via `context`', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/context_properties');
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
});
