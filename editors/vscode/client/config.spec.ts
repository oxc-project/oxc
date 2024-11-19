import { strictEqual } from 'assert';
import { ConfigService } from './config.js';

suite('default values on initialization', () => {
  const service = new ConfigService();

  strictEqual(service.runTrigger, 'onType');
  strictEqual(service.enable, true);
  strictEqual(service.trace, 'off');
  strictEqual(service.configPath, '.eslintrc');
  strictEqual(service.binPath, '');
});
