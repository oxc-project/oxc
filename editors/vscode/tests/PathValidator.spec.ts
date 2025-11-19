import { strictEqual } from 'assert';
import { validateSafeBinaryPath } from '../client/PathValidator';

suite('validateSafeBinaryPath', () => {
  test('should return true for valid binary paths', () => {
    strictEqual(validateSafeBinaryPath('/usr/local/bin/oxc_language_server'), true);
    strictEqual(validateSafeBinaryPath('C:\\Program Files\\oxc_language_server.exe'), true);
    strictEqual(validateSafeBinaryPath('./oxc_language_server'), true);
    strictEqual(validateSafeBinaryPath('/opt/oxc_language_server'), true);
  });

  test('should accept case variations of oxc_language_server', () => {
    strictEqual(validateSafeBinaryPath('OXC_LANGUAGE_SERVER'), true);
    strictEqual(validateSafeBinaryPath('OXC_LANGUAGE_SERVER.exe'), true);
    strictEqual(validateSafeBinaryPath('/usr/local/bin/OXC_LANGUAGE_SERVER'), true);
    strictEqual(validateSafeBinaryPath('C:\\Program Files\\OXC_LANGUAGE_SERVER.exe'), true);
  });

  test('should reject paths with directory traversal', () => {
    strictEqual(validateSafeBinaryPath('../oxc_language_server'), false);
    strictEqual(validateSafeBinaryPath('../../oxc_language_server'), false);
    strictEqual(validateSafeBinaryPath('/usr/local/../bin/oxc_language_server'), false);
    strictEqual(validateSafeBinaryPath('..\\oxc_language_server'), false);
    strictEqual(validateSafeBinaryPath('.\\oxc_language_server'), false);
  });

  test('should reject paths with malicious characters', () => {
    strictEqual(validateSafeBinaryPath('oxc_language_server;rm -rf /'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server|cat /etc/passwd'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server$PATH'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server>output.txt'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server<input.txt'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server`whoami`'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server!'), false);

    // windows specific
    strictEqual(validateSafeBinaryPath('oxc_language_server^&pause'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server & del /f *'), false);
  });
});
