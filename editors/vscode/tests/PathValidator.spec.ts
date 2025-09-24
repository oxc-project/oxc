import { strictEqual } from 'assert';
import { validateSafeBinaryPath } from '../client/PathValidator';

suite('validateSafeBinaryPath', () => {
  test('should return true for valid binary paths', () => {
    strictEqual(validateSafeBinaryPath('/usr/local/bin/oxc_language_server'), true);
    strictEqual(validateSafeBinaryPath('C:\\Program Files\\oxc_language_server.exe'), true);
    strictEqual(validateSafeBinaryPath('./oxc_language_server'), true);
    strictEqual(validateSafeBinaryPath('/opt/oxc_language_server'), true);
  });

  test('should reject paths with directory traversal', () => {
    strictEqual(validateSafeBinaryPath('../oxc_language_server'), false);
    strictEqual(validateSafeBinaryPath('../../oxc_language_server'), false);
    strictEqual(validateSafeBinaryPath('/usr/local/../bin/oxc_language_server'), false);
    strictEqual(validateSafeBinaryPath('..\\oxc_language_server'), false);
  });

  test('should reject paths with malicious characters', () => {
    strictEqual(validateSafeBinaryPath('oxc_language_server;rm -rf /'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server|cat /etc/passwd'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server$PATH'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server>output.txt'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server<input.txt'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server`whoami`'), false);
    strictEqual(validateSafeBinaryPath('oxc_language_server!'), false);
  });

  test('should reject paths not containing oxc_language_server', () => {
    strictEqual(validateSafeBinaryPath('/usr/local/bin/malicious'), false);
    strictEqual(validateSafeBinaryPath('fake_server'), false);
    strictEqual(validateSafeBinaryPath(''), false);
    strictEqual(validateSafeBinaryPath('oxc_language'), false);
    strictEqual(validateSafeBinaryPath('language_server'), false);
  });
});
