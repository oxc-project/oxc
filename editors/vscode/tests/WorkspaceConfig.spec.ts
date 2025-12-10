import { strictEqual } from 'assert';
import { ConfigurationTarget, workspace } from 'vscode';
import { FixKind, WorkspaceConfig } from '../client/WorkspaceConfig.js';
import { WORKSPACE_FOLDER } from './test-helpers.js';

const keys = [
  'lint.run',
  'configPath',
  'tsConfigPath',
  'unusedDisableDirectives',
  'typeAware',
  'disableNestedConfig',
  'fixKind',
  'fmt.experimental',
  'fmt.configPath',
  // deprecated
  'flags'
];

suite('WorkspaceConfig', () => {

  const updateConfiguration = async (key: string, value: unknown) => {
    const workspaceConfig = workspace.getConfiguration('oxc', WORKSPACE_FOLDER);
    const globalConfig = workspace.getConfiguration('oxc');

    await Promise.all([
      workspaceConfig.update(key, value, ConfigurationTarget.WorkspaceFolder),
      // VSCode will not save different workspace configuration inside a `.code-workspace` file.
      // Do not fail, we will make sure the global config is empty too.
      globalConfig.update(key, value)
    ]);
  };

  setup(async () => {
    await Promise.all(keys.map(key => updateConfiguration(key, undefined)));
  });

  teardown(async () => {
    await Promise.all(keys.map(key => updateConfiguration(key, undefined)));
  });

  test('default values on initialization', () => {
    const config = new WorkspaceConfig(WORKSPACE_FOLDER);
    strictEqual(config.runTrigger, 'onType');
    strictEqual(config.configPath, null);
    strictEqual(config.tsConfigPath, null);
    strictEqual(config.unusedDisableDirectives, 'allow');
    strictEqual(config.typeAware, false);
    strictEqual(config.disableNestedConfig, false);
    strictEqual(config.fixKind, "safe_fix");
    strictEqual(config.formattingExperimental, false);
    strictEqual(config.formattingConfigPath, null);
  });

  test('deprecated values are respected', async () => {
    await updateConfiguration('flags', {
      disable_nested_config: 'true',
      fix_kind: 'dangerous_fix'
    });

    const config = new WorkspaceConfig(WORKSPACE_FOLDER);
    strictEqual(config.disableNestedConfig, true);
    strictEqual(config.fixKind, "dangerous_fix");
  });

  test('updating values updates the workspace configuration', async () => {
    const config = new WorkspaceConfig(WORKSPACE_FOLDER);

    await Promise.all([
      config.updateRunTrigger('onSave'),
      config.updateConfigPath('./somewhere'),
      config.updateTsConfigPath('./tsconfig.json'),
      config.updateUnusedDisableDirectives('deny'),
      config.updateTypeAware(true),
      config.updateDisableNestedConfig(true),
      config.updateFixKind(FixKind.DangerousFix),
      config.updateFormattingExperimental(true),
      config.updateFormattingConfigPath('./oxfmt.json'),
    ]);

    const wsConfig = workspace.getConfiguration('oxc', WORKSPACE_FOLDER);

    strictEqual(wsConfig.get('lint.run'), 'onSave');
    strictEqual(wsConfig.get('configPath'), './somewhere');
    strictEqual(wsConfig.get('tsConfigPath'), './tsconfig.json');
    strictEqual(wsConfig.get('unusedDisableDirectives'), 'deny');
    strictEqual(wsConfig.get('typeAware'), true);
    strictEqual(wsConfig.get('disableNestedConfig'), true);
    strictEqual(wsConfig.get('fixKind'), 'dangerous_fix');
    strictEqual(wsConfig.get('fmt.experimental'), true);
    strictEqual(wsConfig.get('fmt.configPath'), './oxfmt.json');
  });

  test('toLanguageServerConfig method', async () => {
    const config = new WorkspaceConfig(WORKSPACE_FOLDER);

    await Promise.all([
      config.updateRunTrigger('onSave'),
      config.updateConfigPath('./somewhere'),
      config.updateTsConfigPath('./tsconfig.json'),
      config.updateUnusedDisableDirectives('deny'),
      config.updateTypeAware(true),
      config.updateDisableNestedConfig(true),
      config.updateFixKind(FixKind.DangerousFix),
      config.updateFormattingExperimental(true),
      config.updateFormattingConfigPath('./oxfmt.json'),
    ]);

    const lsConfig = config.toLanguageServerConfig();

    strictEqual(lsConfig.run, 'onSave');
    strictEqual(lsConfig.configPath, './somewhere');
    strictEqual(lsConfig.tsConfigPath, './tsconfig.json');
    strictEqual(lsConfig.unusedDisableDirectives, 'deny');
    strictEqual(lsConfig.typeAware, true);
    strictEqual(lsConfig.disableNestedConfig, true);
    strictEqual(lsConfig.fixKind, 'dangerous_fix');
    strictEqual(lsConfig['fmt.experimental'], true);
    strictEqual(lsConfig['fmt.configPath'], './oxfmt.json');
  });

  test('toOxlintConfig method', async () => {
    const config = new WorkspaceConfig(WORKSPACE_FOLDER);

    await Promise.all([
      config.updateRunTrigger('onSave'),
      config.updateConfigPath('./somewhere'),
      config.updateTsConfigPath('./tsconfig.json'),
      config.updateUnusedDisableDirectives('deny'),
      config.updateTypeAware(true),
      config.updateDisableNestedConfig(true),
      config.updateFixKind(FixKind.DangerousFix),
      config.updateFormattingExperimental(true),
      config.updateFormattingConfigPath('./oxfmt.json'),
    ]);

    const oxlintConfig = config.toOxlintConfig();

    strictEqual(oxlintConfig.run, 'onSave');
    strictEqual(oxlintConfig.configPath, './somewhere');
    strictEqual(oxlintConfig.tsConfigPath, './tsconfig.json');
    strictEqual(oxlintConfig.unusedDisableDirectives, 'deny');
    strictEqual(oxlintConfig.typeAware, true);
    strictEqual(oxlintConfig.disableNestedConfig, true);
    strictEqual(oxlintConfig.fixKind, 'dangerous_fix');
  });

  test('toOxfmtConfig method', async () => {
    const config = new WorkspaceConfig(WORKSPACE_FOLDER);

    await Promise.all([
      config.updateFormattingExperimental(true),
      config.updateFormattingConfigPath('./oxfmt.json'),
    ]);

    const oxfmtConfig = config.toOxfmtConfig();

    strictEqual(oxfmtConfig['fmt.experimental'], true);
    strictEqual(oxfmtConfig['fmt.configPath'], './oxfmt.json');
  });
});
