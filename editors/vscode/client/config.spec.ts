import { strictEqual } from 'assert';
import { Config } from './Config.js';

suite('default values on initialization', () => {
  const config = new Config();

  strictEqual(config.runTrigger, 'onType');
  strictEqual(config.enable, true);
  strictEqual(config.trace, 'off');
  strictEqual(config.configPath, '.eslintrc');
  strictEqual(config.binPath, '');
});
