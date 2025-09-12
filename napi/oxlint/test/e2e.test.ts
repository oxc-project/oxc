import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { execa } from 'execa';

const PACKAGE_ROOT_PATH = path.dirname(import.meta.dirname);
const ENTRY_POINT_PATH = path.join(PACKAGE_ROOT_PATH, 'dist/index.js');

async function runOxlintWithoutPlugins(cwd: string, args: string[] = []) {
  return await execa('node', [ENTRY_POINT_PATH, ...args], {
    cwd: path.join(PACKAGE_ROOT_PATH, cwd),
    reject: false,
  });
}

async function runOxlint(cwd: string, args: string[] = []) {
  return await runOxlintWithoutPlugins(cwd, ['--experimental-js-plugins', ...args]);
}

function normalizeOutput(output: string): string {
  return output
    .replace(/Finished in \d+(\.\d+)?(s|ms|us|ns)/, 'Finished in Xms')
    .replace(/using \d+ threads./, 'using X threads.');
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

  it('should report an error if a custom plugin cannot be loaded', async () => {
    const { stdout, exitCode } = await runOxlint('test/fixtures/missing_custom_plugin');
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
