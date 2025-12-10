import { strictEqual } from 'assert';
import { runExecutable } from '../client/tools/lsp_helper';

suite('runExecutable', () => {
  const originalPlatform = process.platform;
  const originalEnv = process.env;

  teardown(() => {
    Object.defineProperty(process, 'platform', { value: originalPlatform });
    process.env = originalEnv;
  });

  test('should create Node.js executable for .js files', () => {
    const result = runExecutable('/path/to/server.js');

    strictEqual(result.command, 'node');
    strictEqual(result.args?.[0], '/path/to/server.js');
    strictEqual(result.args?.[1], '--lsp');
  });

  test('should create Node.js executable for .cjs files', () => {
    const result = runExecutable('/path/to/server.cjs');

    strictEqual(result.command, 'node');
    strictEqual(result.args?.[0], '/path/to/server.cjs');
    strictEqual(result.args?.[1], '--lsp');
  });

  test('should create Node.js executable for .mjs files', () => {
    const result = runExecutable('/path/to/server.mjs');

    strictEqual(result.command, 'node');
    strictEqual(result.args?.[0], '/path/to/server.mjs');
    strictEqual(result.args?.[1], '--lsp');
  });

  test('should create binary executable for non-Node files', () => {
    const result = runExecutable('/path/to/oxc-language-server');

    strictEqual(result.command, '/path/to/oxc-language-server');
    strictEqual(result.args?.[0], '--lsp');
    strictEqual(result.options?.shell, false);
  });

  test('should use shell on Windows for binary executables', () => {
    Object.defineProperty(process, 'platform', { value: 'win32' });

    const result = runExecutable('/path/to/oxc-language-server');

    strictEqual(result.options?.shell, true);
  });

  test('should prepend nodePath to PATH', () => {
    Object.defineProperty(process, 'platform', { value: 'linux' });
    process.env.PATH = '/usr/bin:/bin';

    const result = runExecutable('/path/to/server', '/custom/node/path');

    strictEqual(result.options?.env?.PATH, '/custom/node/path:/usr/bin:/bin');
  });
});
