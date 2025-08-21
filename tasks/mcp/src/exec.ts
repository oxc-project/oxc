import { mkdtempSync, unlinkSync, writeFileSync } from 'fs';
import * as os from 'os';
import { join } from 'path';

import { spawnCommand } from './spawn.js';

// Create a unique temp directory for this process
const tempDir: string = mkdtempSync(join(os.tmpdir(), 'oxc-mcp-'));

function writeTempFile(sourceCode: string, filename: string): string {
  const tempPath = join(tempDir, `${Date.now()}-${Math.random().toString(36).substring(2)}-${filename}`);
  writeFileSync(tempPath, sourceCode, 'utf8');
  return tempPath;
}

/**
 * Execute a tool with temporary file cleanup
 */
export async function executeWithTempFile<T extends { sourceCode: string; filename?: string }>(
  command: string,
  options: T,
  args: string[],
): Promise<string> {
  const { sourceCode, filename = 'index.js' } = options;
  const tempPath = writeTempFile(sourceCode, filename);
  args.push(tempPath);
  try {
    return spawnCommand(command, args);
  } finally {
    try {
      unlinkSync(tempPath);
    } catch {
    }
  }
}
