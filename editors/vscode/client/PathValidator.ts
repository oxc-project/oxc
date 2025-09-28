/**
 * Validates the given project given path to ensure it is safe to use.
 *
 * Following checks are performed:
 * Check for path traversal (e.g., using `..` to go up directories).
 * Check for malicious characters or patterns (e.g., `$`, `&`, `;`, `|`).
 * Check if the filename contains `oxc_language_server` to ensure it's the expected binary.
 *
 * The check for malicious characters is not needed, but it's an additional layer of security.
 * When using `shell: true` in `LanguageClient.ServerOptions`, it can be vulnerable to command injection.
 * Even though we are not using `shell: true`, it's a good practice to validate the input.
 */
export function validateSafeBinaryPath(binary: string): boolean {
  // Check for path traversal (including Windows variants)
  if (binary.includes('..') || binary.includes('.\\')) {
    return false;
  }

  // Check for malicious characters or patterns
  // These characters are never expected in a binary path.
  // If any of these characters are present, we consider the path unsafe.
  const maliciousPatterns = [
    // linux/macOS
    '$',
    '&',
    ';',
    '|',
    '`',
    '>',
    '<',
    '!',
    // windows
    '%',
    '^',
  ];
  for (const pattern of maliciousPatterns) {
    if (binary.includes(pattern)) {
      return false;
    }
  }

  // Check if the filename contains `oxc_language_server`
  // Malicious projects might try to point to a different binary.
  if (!binary.replaceAll('\\', '/').toLowerCase().split('/').pop()?.includes('oxc_language_server')) {
    return false;
  }

  return true;
}
