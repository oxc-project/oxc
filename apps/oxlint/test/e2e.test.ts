import fs from 'node:fs/promises';
import { join as pathJoin } from 'node:path';
import { describe, it } from 'vitest';
import { FIXTURES_DIR_PATH, PACKAGE_ROOT_PATH, testFixtureWithCommand } from './utils.js';

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, 'dist/cli.js');

// Options to pass to `testFixture`.
interface TestOptions {
  // Arguments to pass to the CLI.
  args?: string[];
  // Name of the snapshot file.
  // Defaults to `output`.
  // Supply a different name when there are multiple tests for a single fixture.
  snapshotName?: string;
  // Function to get extra data to include in the snapshot
  getExtraSnapshotData?: (dirPath: string) => Promise<{ [key: string]: string }>;

  /**
   * Override the `files` directory within the fixture to lint.
   * This is useful when the fixture has a different structure, e.g. when testing nested configs.
   * Defaults to `files`.
   */
  overrideFiles?: string;
}

/**
 * Run a test fixture.
 * @param fixtureName - Name of the fixture directory within `test/fixtures`
 * @param options - Options to customize the test (optional)
 */
async function testFixture(fixtureName: string, options?: TestOptions): Promise<void> {
  const args = options?.args ?? [];

  await testFixtureWithCommand({
    // Use current NodeJS executable, rather than `node`, to avoid problems with a Node version manager
    // installed on system resulting in using wrong NodeJS version
    command: process.execPath,
    args: [CLI_PATH, ...args, options?.overrideFiles ?? 'files'],
    fixtureName,
    snapshotName: options?.snapshotName ?? 'output',
    getExtraSnapshotData: options?.getExtraSnapshotData,
  });
}

describe('oxlint CLI', () => {
  it('should lint a directory without errors', async () => {
    await testFixture('built_in_no_errors', { args: [] });
  });

  it('should lint a directory with errors', async () => {
    await testFixture('built_in_errors', { args: [] });
  });

  it('should load a custom plugin', async () => {
    await testFixture('basic_custom_plugin');
  });

  it('should support message placeholder interpolation', async () => {
    await testFixture('message_interpolation');
  });

  it('should support messageId', async () => {
    await testFixture('message_id_plugin');
  });

  it('should support messageId placeholder interpolation', async () => {
    await testFixture('message_id_interpolation');
  });

  it('should report an error for unknown messageId', async () => {
    await testFixture('message_id_error');
  });

  it('should load a custom plugin with various import styles', async () => {
    await testFixture('load_paths');
  });

  it('should load a custom plugin with multiple files', async () => {
    await testFixture('basic_custom_plugin_many_files');
  });

  it('should load a custom plugin correctly when extending in a nested config', async () => {
    await testFixture('custom_plugin_nested_config', {
      overrideFiles: '.',
    });
  });
  it('should do something', async () => {
    await testFixture('custom_plugin_nested_config_duplicate', {
      overrideFiles: '.',
    });
  });

  it('should load a custom plugin when configured in overrides', async () => {
    await testFixture('custom_plugin_via_overrides');
  });

  it('should report an error if a custom plugin is missing', async () => {
    await testFixture('missing_custom_plugin');
  });

  it('should report an error if a custom plugin has a reserved name', async () => {
    await testFixture('reserved_name');
  });

  it('should report an error if a custom plugin throws an error during import', async () => {
    await testFixture('custom_plugin_import_error');
  });

  it('should report an error if a rule is not found within a custom plugin', async () => {
    await testFixture('custom_plugin_missing_rule');
  });

  it('should report an error if a a rule is not found within a custom plugin (via overrides)', async () => {
    await testFixture('custom_plugin_via_overrides_missing_rule');
  });

  describe('should report an error if a custom plugin throws an error during linting', () => {
    it('in `create` method', async () => {
      await testFixture('custom_plugin_lint_create_error');
    });

    it('in `createOnce` method', async () => {
      await testFixture('custom_plugin_lint_createOnce_error');
    });

    it('in visit function', async () => {
      await testFixture('custom_plugin_lint_visit_error');
    });

    it('in `before` hook', async () => {
      await testFixture('custom_plugin_lint_before_hook_error');
    });

    it('in `after` hook', async () => {
      await testFixture('custom_plugin_lint_after_hook_error');
    });

    it('in `fix` function', async () => {
      await testFixture('custom_plugin_lint_fix_error');
    });
  });

  it('should report the correct severity when using a custom plugin', async () => {
    await testFixture('basic_custom_plugin_warn_severity');
  });

  it('should work with multiple rules', async () => {
    await testFixture('basic_custom_plugin_multiple_rules');
  });

  it('should support reporting diagnostic with `loc`', async () => {
    await testFixture('diagnostic_loc');
  });

  it('should receive ESTree-compatible AST', async () => {
    await testFixture('estree');
  });

  it('should receive AST with all nodes having `parent` property', async () => {
    await testFixture('parent');
  });

  it('should receive data via `context`', async () => {
    await testFixture('context_properties');
  });

  it('should give access to source code via `context.sourceCode`', async () => {
    await testFixture('sourceCode');
  });

  it('should get source text and AST from `context.sourceCode` when accessed late', async () => {
    await testFixture('sourceCode_late_access');
  });

  it('should get source text and AST from `context.sourceCode` when accessed in `after` hook only', async () => {
    await testFixture('sourceCode_late_access_after_only');
  });

  it('should support selectors', async () => {
    await testFixture('selector');
  });

  it('should support `createOnce`', async () => {
    await testFixture('createOnce');
  });

  it('should support `definePlugin`', async () => {
    await testFixture('definePlugin');
  });

  it('should support `defineRule`', async () => {
    await testFixture('defineRule');
  });

  it('should support `definePlugin` and `defineRule` together', async () => {
    await testFixture('definePlugin_and_defineRule');
  });

  it('should have UTF-16 spans in AST', async () => {
    await testFixture('utf16_offsets');
  });

  it('should respect disable directives for custom plugin rules', async () => {
    await testFixture('custom_plugin_disable_directives');
  });

  it('should not apply fixes when `--fix` is disabled', async () => {
    await testFixture('fixes', {
      snapshotName: 'fixes_disabled',
      async getExtraSnapshotData(fixtureDirPath) {
        const fixtureFilePath = pathJoin(fixtureDirPath, 'files/index.js');
        const codeAfter = await fs.readFile(fixtureFilePath, 'utf8');
        return { 'Code after': codeAfter };
      },
    });
  });

  it('should apply fixes when `--fix` is enabled', async () => {
    const fixtureFilePath = pathJoin(FIXTURES_DIR_PATH, 'fixes/files/index.js');
    const codeBefore = await fs.readFile(fixtureFilePath, 'utf8');

    try {
      await testFixture('fixes', {
        args: ['--fix'],
        snapshotName: 'fixes_enabled',
        async getExtraSnapshotData() {
          const codeAfter = await fs.readFile(fixtureFilePath, 'utf8');
          return { 'Code after': codeAfter };
        },
      });
    } finally {
      // Revert fixture file code changes
      await fs.writeFile(fixtureFilePath, codeBefore);
    }
  });

  it('SourceCode.getAllComments() should return all comments', async () => {
    await testFixture('getAllComments');
  });
});
